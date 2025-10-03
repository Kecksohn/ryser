// Set-Up
use super::init::*;

// GUI / User Actions
use super::library_manager::gui_functions::*;
use super::video_player::*;

// Process Manager
use std::{collections::HashMap, sync::{Mutex, Arc}};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = init();

    #[cfg(not(feature = "backend-only"))]
    {
        tauri::Builder::default()
            .plugin(tauri_plugin_window_state::Builder::new().build())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_http::init())
            .manage(Arc::new(ProcessManager {
                processes: Mutex::new(HashMap::new()),
            }))
            .invoke_handler(tauri::generate_handler![
                open_window,

                // UI Home
                get_available_libraries,
                // Library Management
                create_library,
                delete_library_gui,
                // Library View
                get_library_name,
                get_library_videos,
                get_library_sort_preference,
                set_library_sort_preference,
                get_library_filter_preferences,
                set_library_filter_preferences,
                // Library Rescan
                rescan_all_libraries_gui,
                rescan_library_by_id_gui,
                reparse_all_libraries_preserve_covers_gui,
                // Library Update
                update_library_entry_from_gui,
                search_tmdb_from_gui,
                get_covers_from_tmdb,
                // Video Start
                start_video_in_mpc,
                start_video_in_vlc,
                is_process_running,
            ])
            .setup(|app| {
                #[cfg(debug_assertions)]
                {
                    use tauri::Manager;
                    let main_window = app.get_webview_window("main").unwrap();
                    main_window.open_devtools();
                }
                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }

    #[cfg(feature = "debug-backend")]
    {
        // Back-end Only Debug Run (see Readme)
        super::_debug_run::debug_main();
    }
}
