// region: --- Modules

mod library_manager;
mod file_manager;
mod json_parser;
mod tmdb_api;
mod utils;
mod video_element;
mod lang_map;
mod playback_selection;

mod error;

// -- Flatten

pub use library_manager::*;
pub use video_element::*;
pub use playback_selection::*;
pub use error::*;

// -- Public Modules

pub(crate) mod gui_functions;

// endregion: --- Modules