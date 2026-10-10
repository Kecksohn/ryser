// region: --- Modules

mod create_video_element;
mod external_subtitles;
mod read_metadata;
mod error;

// -- Flatten

pub(super) use create_video_element::*;
pub(super) use external_subtitles::DirCache;
pub use error::*;

// -- Public Modules

pub(super) mod directory_utils;
pub(super) mod file_utils;

// endregion: --- Modules