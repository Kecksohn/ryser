// region: --- Modules

mod send;
mod error;

// -- Flatten
pub use send::show_msg_gui;
pub use error::{Error, Result};

// -- Public Modules

pub(crate) mod notification_manager;

// endregion: --- Modules