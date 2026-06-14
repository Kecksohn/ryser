use tauri::State;
use super::process_manager::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

const VLC_PATH: &str = "C:/Program Files/VideoLAN/VLC/vlc.exe";
const MPC_PATH: &str = "C:/Program Files (x86)/K-Lite Codec Pack/MPC-HC64/mpc-hc64.exe";

// MPC-HC web interface (must be enabled in its settings; we enable it via the
// registry before launch). No authentication. See WebClientSocket.cpp.
const MPC_WEB_ADDR: &str = "127.0.0.1:13579";
const MPC_WEB_PORT: u32 = 13579;
// MPC-HC command IDs (src/mpc-hc/resource.h). Selecting a track directly =
// menu-subitem base + a fixed menu offset + the stream's 0-based ordinal
// (MainFrm.cpp OnPlayAudio/OnPlaySubtitles):
//   audio menu: index 0 is an "Options" entry, tracks follow -> offset 1
//   subtitle menu (internal renderer): 6 control entries precede the list -> offset 6
// These offsets also keep us clear of the Options/Styles dialog entries. They
// assume MPC-HC's default filters (internal audio switcher + subtitle renderer),
// which is the standard K-Lite setup.
const MPC_ID_AUDIO_SUBITEM_START: usize = 2200;
const MPC_AUDIO_MENU_OFFSET: usize = 1;
const MPC_ID_SUBTITLES_SUBITEM_START: usize = 2300;
const MPC_SUBTITLE_MENU_OFFSET: usize = 6;
const MPC_ID_STREAM_SUB_ONOFF: u32 = 956; // toggles subtitle visibility
// After the initial select, verify a few times and re-send ONLY if MPC's
// load-time auto-selection clobbered our choice. MPC has no per-file track
// memory, so normally the first select sticks and we exit after one check -
// re-sending an already-correct track needlessly re-inits it (visible hitch).
const MPC_VERIFY_ROUNDS: usize = 3;

// region: --- VLC

/// VLC selects tracks by type-relative ordinal: `--audio-track=N` / `--sub-track=N`.
/// `--no-spu` disables subtitles when none is wanted.
fn vlc_args(filepath: &str, audio_track: Option<usize>, subtitle_track: Option<usize>) -> Vec<String> {
    let mut args = vec![filepath.to_owned()];
    if let Some(a) = audio_track {
        args.push(format!("--audio-track={}", a));
    }
    match subtitle_track {
        Some(s) => args.push(format!("--sub-track={}", s)),
        None => args.push("--no-spu".to_owned()),
    }
    args
}

#[tauri::command(rename_all = "snake_case")]
pub fn start_video_in_vlc(
    filepath: &str,
    audio_track: Option<usize>,
    subtitle_track: Option<usize>,
    state: State<Arc<ProcessManager>>,
) -> Option<u32> {
    let args = vlc_args(filepath, audio_track, subtitle_track);
    start_process(VLC_PATH, &args, state)
}

// endregion: --- VLC

// region: --- MPC-HC

/// MPC-HC has no command-line flag to pick an internal track by index, so we
/// drive its web interface instead: launch the file, wait until it is loaded,
/// then send commands to select the exact audio/subtitle track. The web
/// interface is enabled via the registry beforehand (takes effect on the next
/// MPC-HC start). Whole flow is best-effort; failures leave MPC's defaults.
#[tauri::command(rename_all = "snake_case")]
pub fn start_video_in_mpc(
    filepath: &str,
    audio_track: Option<usize>,
    subtitle_track: Option<usize>,
    state: State<Arc<ProcessManager>>,
) -> Option<u32> {
    enable_mpc_web_interface();
    // MPC-HC wants a leading space before the path (legacy quirk).
    let args = vec![format!(" {}", filepath)];
    let pid = start_process(MPC_PATH, &args, state)?;

    // Used to detect that MPC (single-instance) has switched to another file,
    // so a stale selection thread stops touching the now-current movie.
    let file_name = filepath
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filepath)
        .to_owned();
    std::thread::spawn(move || apply_mpc_track_selection(&file_name, audio_track, subtitle_track));

    Some(pid)
}

fn enable_mpc_web_interface() {
    use std::process::Command;
    const KEY: &str = r"HKCU\Software\MPC-HC\MPC-HC\Settings";
    let set_dword = |name: &str, value: u32| {
        let _ = Command::new("reg")
            .args(["add", KEY, "/v", name, "/t", "REG_DWORD", "/d", &value.to_string(), "/f"])
            .output();
    };
    set_dword("EnableWebServer", 1);
    set_dword("WebServerPort", MPC_WEB_PORT);
}

fn apply_mpc_track_selection(file_name: &str, audio_track: Option<usize>, subtitle_track: Option<usize>) {
    // Wait until our file is loaded (duration known AND the loaded path matches
    // the file we launched - guards against acting on a different movie if MPC's
    // single instance switched files).
    let mut loaded = false;
    for _ in 0..40 {
        if let Some(body) = mpc_get("/variables.html") {
            if body_is_our_file(&body, file_name)
                && extract_var(&body, "duration").map_or(false, |d| d > 0)
            {
                loaded = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    if !loaded {
        println!("MPC: '{}' not loaded / web interface unreachable; using player defaults", file_name);
        return;
    }
    // Small settle delay so the stream lists are populated before we select.
    std::thread::sleep(Duration::from_millis(300));

    // Pre-compute the command IDs that directly select the wanted streams.
    let audio_cmd =
        audio_track.map(|i| (MPC_ID_AUDIO_SUBITEM_START + MPC_AUDIO_MENU_OFFSET + i) as u32);
    let subtitle_cmd =
        subtitle_track.map(|i| (MPC_ID_SUBTITLES_SUBITEM_START + MPC_SUBTITLE_MENU_OFFSET + i) as u32);

    // No subtitle wanted: MPC enables one by default, so toggle it off (once).
    if subtitle_track.is_none() {
        mpc_command(MPC_ID_STREAM_SUB_ONOFF);
    }

    // Select the wanted tracks once.
    if let Some(cmd) = audio_cmd {
        mpc_command(cmd);
    }
    if let Some(cmd) = subtitle_cmd {
        mpc_command(cmd);
    }

    // Record what we landed on, then verify a few times: re-send only if MPC's
    // load-time auto-select later overrode us (avoids re-initialising an already
    // correct track, which causes a visible hitch). Exit as soon as it's stable.
    std::thread::sleep(Duration::from_millis(300));
    let (mut expected_audio, mut expected_subtitle) = match mpc_get("/variables.html") {
        Some(body) => (
            extract_var_str(&body, "audiotrack"),
            extract_var_str(&body, "subtitletrack"),
        ),
        None => (None, None),
    };
    for _ in 0..MPC_VERIFY_ROUNDS {
        std::thread::sleep(Duration::from_millis(400));
        let Some(body) = mpc_get("/variables.html") else {
            break;
        };
        if !body_is_our_file(&body, file_name) {
            return; // MPC's single instance switched movies; leave it alone.
        }
        let mut drifted = false;
        if let Some(cmd) = audio_cmd {
            if extract_var_str(&body, "audiotrack") != expected_audio {
                mpc_command(cmd);
                drifted = true;
            }
        }
        if let Some(cmd) = subtitle_cmd {
            if extract_var_str(&body, "subtitletrack") != expected_subtitle {
                mpc_command(cmd);
                drifted = true;
            }
        }
        if !drifted {
            break; // selection held; nothing more to do
        }
        // Re-read the names we just re-asserted to compare against next round.
        if let Some(body) = mpc_get("/variables.html") {
            expected_audio = extract_var_str(&body, "audiotrack");
            expected_subtitle = extract_var_str(&body, "subtitletrack");
        }
    }
}

/// True if MPC's variables.html shows our file as the currently loaded one.
fn body_is_our_file(body: &str, file_name: &str) -> bool {
    body.contains(file_name)
}

fn mpc_command(id: u32) -> bool {
    mpc_get(&format!("/command.html?wm_command={}", id)).is_some()
}

/// Minimal blocking HTTP/1.0 GET against the local MPC-HC web server.
/// Returns the response body on success.
fn mpc_get(path: &str) -> Option<String> {
    let mut stream = TcpStream::connect(MPC_WEB_ADDR).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let request = format!(
        "GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, MPC_WEB_ADDR
    );
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    // Strip headers; keep the body.
    response.split_once("\r\n\r\n").map(|(_, body)| body.to_string())
}

/// Extract the text content of a `variables.html` field, e.g. the body of
/// `<p id="audiotrack">A: Original [por] ...</p>`.
fn extract_var_str(body: &str, id: &str) -> Option<String> {
    let needle = format!("id=\"{}\">", id);
    let start = body.find(&needle)? + needle.len();
    let rest = &body[start..];
    let end = rest.find('<')?;
    let value = rest[..end].trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// Extract an integer field from `variables.html`, e.g.
/// `<p id="duration">12345</p>`.
fn extract_var(body: &str, id: &str) -> Option<i64> {
    extract_var_str(body, id)?.parse::<i64>().ok()
}

// endregion: --- MPC-HC
