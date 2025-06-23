// Disable Compiler Warnings
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

// region: --- Modules

mod init;
mod run;

mod config;
mod library_manager;
mod notifications;
mod video_player;

mod _debug_run;

// -- Flatten

pub(crate) use config::*;
pub use run::*;

// -- Public Modules

// endregion: --- Modules
