#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod config;
use config::read_config;

mod notifications;
use notifications::*;

mod library_manager;
use library_manager::{
    gui_functions::*,
    load_all_libraries, 
    update_all_libraries_with_tmdb, 
    rescan_all_libraries, 
    rescan_library_by_id,
};


mod video_player;
use video_player::*;
use std::{collections::HashMap, sync::{Mutex, Arc}};


mod _debug_run;


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

    let _ = notification_manager::start(window);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    read_config();
    load_all_libraries();
    // TODO: make this async
    rescan_all_libraries();
    match update_all_libraries_with_tmdb(Some(false)) {
        Ok(_) => (),
        Err(e) => println!("TMDB update failed: {}", e),
    }

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
                // Library Rescan
                rescan_all_libraries,
                rescan_library_by_id,
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
        use _debug_run::*;
        debug_main();
    }
}
