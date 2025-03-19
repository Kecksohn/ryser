#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod video_player;
use crate::video_player::*;
use std::{collections::HashMap, sync::{Mutex, Arc}};

mod app_start;
use crate::app_start::*;

mod library_manager;
use crate::library_manager::gui_functions::*;
use crate::library_manager::load_all_libraries;
use crate::library_manager::rescan_all_libraries;
use crate::library_manager::rescan_library_by_id;



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
    load_all_libraries();
    rescan_all_libraries();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .manage(Arc::new(ProcessManager {
            processes: Mutex::new(HashMap::new()),
        }))
        .invoke_handler(tauri::generate_handler![
            open_window,

            // UI Home
            get_available_libraries,
            // Library Creation
            create_library,

            // Library View
            get_library_videos,
            // Library Rescan
            rescan_all_libraries,
            rescan_library_by_id,
            // Library Update
            update_library_entry_from_gui,
            search_tmdb_from_gui,
            // Video Start
            start_video_in_mpc,
            start_video_in_vlc,
            is_process_running,
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
