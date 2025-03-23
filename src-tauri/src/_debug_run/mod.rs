use crate::library_manager::*;


// This gets called on 'cargo run debug-backend'
pub(super) fn debug_main() {
    match update_all_libraries_with_tmdb(None) {
        Ok(_) => (),
        Err(e) => {println!("TMDB update failed: {}", e)},
    }
}