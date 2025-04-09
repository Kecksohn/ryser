use super::default_config;

use directories::ProjectDirs;
use std::fs;
use std::fs::File;

pub(crate) fn read_config() {
    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        // Create config dir if it doesn't exist
        let config_dir = proj_dir.config_local_dir();
        if !config_dir.exists() {
            match fs::create_dir_all(config_dir) {
                Ok(()) => {}
                Err(error) => panic!("Problem creating folder: {error:?}"),
            }
        }

        // Create data dir if it doesn't exist
        let data_dir = proj_dir.data_local_dir();
        if !data_dir.exists() {
            match fs::create_dir_all(data_dir) {
                Ok(()) => {}
                Err(error) => panic!("Problem creating folder: {error:?}"),
            }
        }

        // Create config.json if it doesn't exist
        let config_filepath = config_dir.join("config.json");
        if !config_filepath.exists() {
            let file = File::create(config_filepath);
            match file {
                Ok(file) => {
                    let _ = serde_json::to_writer_pretty(file, &default_config::get_json());
                    println!("Wrote to config.json");
                }
                Err(error) => panic!("Problem creating config.json: {error:?}"),
            }
        }
    }
}


// TODO: Probably refactor this to a notification manager?
use tauri::{Manager, Window, Emitter};
use serde::{Serialize, Deserialize};
use crate::library_manager::rescan_all_libraries;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
struct TimedMessage {
    header: String,
    message: String,
    id: String,
    duration_ms: u64,
}

pub(crate) fn on_gui_available(window: Window) {
    
    // TODO: make this async
    rescan_all_libraries();

    let message_id = Uuid::new_v4().to_string();
    let timed_message = TimedMessage {
        header: "Hello from Rust!".to_owned(),
        message: "WHAT is UP my dude".to_owned(),
        id: message_id,
        duration_ms: 3000,
    };

    window.emit("display-message", timed_message).unwrap();
}
