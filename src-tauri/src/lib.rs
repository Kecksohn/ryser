use tauri_plugin_log::{Target, TargetKind};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod file_reader;
use crate::file_reader::*;

mod video_player;
use crate::video_player::*;

use tauri::{Manager, Window};
// This command must be async so that it doesn't run on the main thread.
#[tauri::command]
async fn open_window(window: Window) {
    // Show main window
    window
        .get_webview_window("main")
        .expect("no window labeled 'ryser' found")
        .show()
        .unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            open_window,
            greet,
            get_video_files,
            start_video_in_mpc
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
