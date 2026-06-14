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
// MPC-HC command IDs (src/mpc-hc/resource.h). We deliberately avoid the
// menu-subitem IDs (2200/2300+index): their offset depends on which internal
// filters are loaded (audio switcher prepends an "Options" entry, the subtitle
// renderer prepends 6 control items), so a static offset mis-selects and can
// pop up the Styles/Options dialog. Instead we step the enabled stream with the
// next-track commands and converge using the readback from variables.html.
const MPC_ID_STREAM_AUDIO_NEXT: u32 = 952;
const MPC_ID_STREAM_SUB_NEXT: u32 = 954;
const MPC_ID_STREAM_SUB_ONOFF: u32 = 956; // toggles subtitle visibility
// Upper bound on cycling steps; far above any real track count.
const MPC_MAX_CYCLE_STEPS: usize = 50;

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

    if let Some(target) = audio_track {
        converge_track(file_name, "audiotrack", MPC_ID_STREAM_AUDIO_NEXT, target);
    }
    match subtitle_track {
        Some(target) => converge_track(file_name, "subtitletrack", MPC_ID_STREAM_SUB_NEXT, target),
        // No subtitle wanted: MPC enables one by default, so toggle it off.
        None => mpc_command(MPC_ID_STREAM_SUB_ONOFF),
    }
}

/// Step the enabled audio/subtitle stream forward (which wraps) until the
/// readback variable equals the target ordinal. Robust to MPC's menu layout
/// because both the step command and the readback use the real stream index.
///
/// Stops as soon as a readback value repeats (a full cycle completed without a
/// match → target unreachable) so we never hammer MPC with rapid track
/// switches, and bails if MPC has switched to a different file.
fn converge_track(file_name: &str, var_id: &str, next_cmd: u32, target: usize) {
    let mut seen = std::collections::HashSet::new();
    for _ in 0..MPC_MAX_CYCLE_STEPS {
        let Some(body) = mpc_get("/variables.html") else {
            return;
        };
        if !body_is_our_file(&body, file_name) {
            println!("MPC: file changed; abandoning {} selection", var_id);
            return;
        }
        match extract_var(&body, var_id) {
            Some(current) if current == target as i64 => return, // reached target
            Some(current) => {
                if !seen.insert(current) {
                    // Value already seen: we've cycled through every track without
                    // a match. Stop rather than thrash the renderer.
                    println!("MPC: target {} for {} unreachable (cycled); leaving as-is", target, var_id);
                    return;
                }
            }
            None => return, // no readback; give up quietly
        }
        mpc_command(next_cmd);
        std::thread::sleep(Duration::from_millis(200));
    }
    println!("MPC: step limit reached converging {} to {}", var_id, target);
}

/// True if MPC's variables.html shows our file as the currently loaded one.
fn body_is_our_file(body: &str, file_name: &str) -> bool {
    body.contains(file_name)
}

fn mpc_command(id: u32) {
    let _ = mpc_get(&format!("/command.html?wm_command={}", id));
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

/// Extract an integer from MPC-HC's `variables.html`, e.g.
/// `<p id="duration">12345</p>`.
fn extract_var(body: &str, id: &str) -> Option<i64> {
    let needle = format!("id=\"{}\">", id);
    let start = body.find(&needle)? + needle.len();
    let rest = &body[start..];
    let end = rest.find('<')?;
    rest[..end].trim().parse::<i64>().ok()
}

// endregion: --- MPC-HC
