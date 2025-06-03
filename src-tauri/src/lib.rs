// Disable Compiler Warnings
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]


// region: --- Modules

mod run;
mod init;

mod config;
mod notifications;
mod library_manager;
mod video_player;

mod _debug_run;

// -- Flatten

pub use run::*;
pub(crate) use config::*;

// -- Public Modules

// endregion: --- Modules