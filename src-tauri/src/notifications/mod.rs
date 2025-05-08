// region: --- Modules

mod send;
mod error;

// -- Flatten
pub use send::*;
pub use error::*;

// -- Public Modules

pub(crate) mod notification_manager;

// endregion: --- Modules