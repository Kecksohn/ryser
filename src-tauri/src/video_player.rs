use std::process::Command;

#[tauri::command(rename_all = "snake_case")]
pub fn start_video_in_mpc(filepath: &str) -> () {
    let _ = Command::new("C:/Program Files (x86)/K-Lite Codec Pack/MPC-HC64/mpc-hc64.exe")
        .arg(" ".to_owned() + filepath)
        .spawn();
}
