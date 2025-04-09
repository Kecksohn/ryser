use tauri::{Manager, Window, Emitter};
use serde::{Serialize, Deserialize};
use crate::library_manager::rescan_all_libraries;
use uuid::Uuid;

use super::Result;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct TimedMessage {
    pub header: String,
    pub message: String,
    pub id: String,
    pub duration_ms: u64,
}

pub(crate) fn start(window: Window) {
    let message_id = Uuid::new_v4().to_string();
    let timed_message = TimedMessage {
        header: "Hello from Rust!".to_owned(),
        message: "WHAT is UP my dude".to_owned(),
        id: message_id,
        duration_ms: 3000,
    };

    window.emit("display-message", timed_message).unwrap();
}
