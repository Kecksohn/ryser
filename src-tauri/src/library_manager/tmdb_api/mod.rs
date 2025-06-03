// region: --- Modules

mod api_token;
mod search_name_creator;
mod tmdb_client;
mod error;

// -- Flatten

pub use tmdb_client::*;
pub use error::*;

// -- Public Modules

pub mod json_structs;

// endregion: --- Modules