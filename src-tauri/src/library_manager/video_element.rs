use serde::Deserialize;
use chrono::{serde::ts_milliseconds, DateTime, Utc};

use std::fmt;

use super::playback_selection::PlaybackSelection;


#[derive(Default, Clone, serde::Serialize, Deserialize, Debug)]
pub struct VideoElement {
    pub filepath: String,
    pub watched: bool,
    pub tmdb_id: Option<usize>,
    pub poster_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub backdrop_path: Option<String>,

    pub title: Option<String>,
    pub original_title: Option<String>,
    pub release_date: Option<String>,
    pub director: Option<String>,
    pub countries: Option<Vec<String>>,
    pub genre_ids: Option<Vec<usize>>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    
    pub tmdb_language: Option<String>,
    pub audio_languages: Option<Vec<String>>,
    pub subtitle_languages: Option<Vec<String>>,
    #[serde(default)]
    pub audio_titles: Option<Vec<Option<String>>>,
    #[serde(default)]
    pub subtitle_titles: Option<Vec<Option<String>>>,
    #[serde(default)]
    pub playback_selection: Option<PlaybackSelection>,

    pub season: Option<i32>,
    pub episode: Option<i32>,

    pub index_priority: i32,
    pub length_in_seconds: i32,
    #[serde(with = "ts_milliseconds")]
    pub timestamp_modified: DateTime<Utc>,
}
