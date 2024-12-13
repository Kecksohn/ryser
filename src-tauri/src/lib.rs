mod file_reader;
use crate::file_reader::*;

mod video_player;
use crate::video_player::*;

mod app_start;
use crate::app_start::*;

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

    read_config();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_window,
            get_video_files,
            start_video_in_mpc
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
