use super::process_manager::*;

use std::sync::{Arc, Mutex};
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub fn start_video_in_vlc(filepath: &str, state: State<Arc<ProcessManager>>) -> Option<u32> {
    start_process("C:/Program Files/VideoLAN/VLC/vlc.exe", filepath, state)
}

// TODO: Do we really need the " " in mpc but not in vlc??
#[tauri::command(rename_all = "snake_case")]
pub fn start_video_in_mpc(filepath: &str, state: State<Arc<ProcessManager>>) -> Option<u32> {
    start_process(
        "C:/Program Files (x86)/K-Lite Codec Pack/MPC-HC64/mpc-hc64.exe",
        &(" ".to_owned() + filepath),
        state,
    )
}
