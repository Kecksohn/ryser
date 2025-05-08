use tauri::async_runtime;

use crate::Error;

use crate::library_manager::*;
use crate::library_manager::gui_functions::*;

// This gets called on 'cargo run debug-backend'
pub(super) fn debug_main() {

    // function()

    let _ = async_runtime::block_on(async {
        debug_async().await.map_err(|e| print!("{}", e))
    });
}

async fn debug_async() -> Result<(), Error> {
    
    //let _ = function().await.map_err(|e| format!("{}", e))?;
    
    Ok(())
} 