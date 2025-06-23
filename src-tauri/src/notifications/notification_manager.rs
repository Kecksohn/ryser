use anyhow::{anyhow, Error};

use crate::library_manager::rescan_all_libraries;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, Window};
use uuid::Uuid;

use once_cell::sync::OnceCell;

static WINDOW: OnceCell<Window> = OnceCell::new();

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct TimedMessage {
    pub header: String,
    pub message: String,
    pub id: String,
    pub duration_ms: u64,
}

pub(crate) fn start(window: Window) -> Result<(), Error> {
    let _ = WINDOW.set(window);
    Ok(())
}

pub(super) fn send_msg(
    header: &str,
    message: &str,
    message_id: Option<String>,
    duration_ms: Option<u64>,
) -> Result<String, Error> {
    let duration_ms = duration_ms.unwrap_or(3000);
    let message_id = message_id.unwrap_or(Uuid::new_v4().to_string());

    let timed_message = TimedMessage {
        header: header.to_owned(),
        message: message.to_owned(),
        id: message_id.clone(),
        duration_ms: duration_ms,
    };

    WINDOW
        .get()
        .unwrap()
        .emit("display-message", timed_message)
        .unwrap();

    Ok(message_id)
}
