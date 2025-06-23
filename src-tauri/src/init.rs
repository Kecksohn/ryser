use anyhow::{anyhow, Error};

use super::library_manager::{
    load_all_libraries, rescan_all_libraries, update_all_libraries_with_tmdb,
};
use super::read_config;
use crate::notifications::*;

use tauri::async_runtime;

pub(super) fn init() -> Result<(), Error> {
    read_config();

    // Synced
    async_runtime::block_on(async {
        load_all_libraries()
            .await
            .map_err(|e| anyhow!("Failure while loading libraries: {}", e))
    })?;

    // Async
    async_runtime::spawn(async {
        rescan_all_libraries().await;
        update_all_libraries_with_tmdb(Some(false))
            .await
            .map_err(|e| anyhow!("TMDB update failed: {}", e))
    });

    Ok(())
}

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
