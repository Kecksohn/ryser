use super::read_config;
use super::library_manager::{load_all_libraries, rescan_all_libraries, update_all_libraries_with_tmdb};

pub(super) fn init() {
    read_config();
    load_all_libraries();
    // TODO: make this async
    rescan_all_libraries();
    match update_all_libraries_with_tmdb(Some(false)) {
        Ok(_) => (),
        Err(e) => println!("TMDB update failed: {}", e),
    }
}


use super::notifications::*;

use tauri::{Manager, Window};
// This command must be async so that it doesn't run on the main thread.
#[tauri::command]
pub(super) async fn open_window(window: Window) {
    // Show main window
    window
        .get_webview_window("main")
        .expect("no window labeled 'ryser' found")
        .show()
        .unwrap();

    let _ = notification_manager::start(window);
}
