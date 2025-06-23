use serde::Deserialize;
use chrono::{serde::ts_milliseconds, DateTime, Utc};

use std::fmt;


#[derive(Default, Clone, serde::Serialize, Deserialize, Debug)]
pub struct VideoElement {
    pub id: i32,
    pub watched: bool,
    pub index_priority: i32, // Perhaps a better name? Should specify custom sort
    
    // Local file
    pub filepath: String,
    pub length_in_seconds: Option<i32>,
    #[serde(with = "ts_milliseconds")]
    pub timestamp_modified: DateTime<Utc>, // To see if file on disk changed

    pub audio_tracks: Option<Vec<String>>,
    pub audio_track_selected: i32,
    pub subtitle_tracks: Option<Vec<String>>,
    pub subtitles_external: Option<Vec<String>>,
    pub subtitle_track_selected: i32,

    // TMDB stuff
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
    pub tmdb_length_in_seconds: Option<String>,

    // TV Series Only
    pub season: Option<i32>,
    pub episode: Option<i32>,
}
