use tauri::async_runtime;

use crate::library_manager::*;

use crate::library_manager::gui_functions::*;

// This gets called on 'cargo run debug-backend'
pub(super) fn debug_main() {

    match update_all_libraries_with_tmdb(None) {
        Ok(_) => (),
        Err(e) => {println!("TMDB update failed: {}", e)},
    }

    let _ = async_runtime::block_on(async {
        debug_async().await.map_err(|e| print!("{}", e))
    });
}

async fn debug_async() -> Result<(), String> {
    //let _ = function("here").await.map_err(|e| format!("{}", e))?;
    Ok(())
} 