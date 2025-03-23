use serde::Deserialize;
use chrono::{serde::ts_milliseconds, DateTime, Utc};

use std::fmt;


#[derive(Default, Clone, serde::Serialize, Deserialize, Debug)]
pub struct VideoElement {
    pub filepath: String,
    pub watched: bool,
    pub tmdb_id: Option<usize>,
    pub poster_path: Option<String>,
    pub thumbnail_path: Option<String>,

    pub title: Option<String>,
    pub year: Option<i16>,
    pub director: Option<String>,
    pub countries: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,

    pub season: Option<i32>,
    pub episode: Option<i32>,

    pub index_priority: i32,
    pub length_in_seconds: i32,
    #[serde(with = "ts_milliseconds")]
    pub timestamp_modified: DateTime<Utc>,
}


impl fmt::Display for VideoElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Build the title display string (title with year if available)
        let title_display = match (&self.title, &self.year) {
            (Some(title), Some(year)) => format!("{} ({})", title, year),
            (Some(title), None) => title.clone(),
            (None, _) => String::from("Untitled"),
        };

        // Show episode info if available
        let episode_info = match (self.season, self.episode) {
            (Some(s), Some(e)) => format!(" - S{:02}E{:02}", s, e),
            _ => String::new(),
        };

        // Format countries and languages if available
        let countries = self.countries
            .as_ref()
            .map_or(String::new(), |c| format!(" | Countries: {}", c.join(", ")));

        let languages = self.languages
            .as_ref()
            .map_or(String::new(), |l| format!(" | Languages: {}", l.join(", ")));

        // Format director if available
        let director = self.director
            .as_ref()
            .map_or(String::new(), |d| format!(" | Director: {}", d));

        // Format watched status
        let watched_status = if self.watched { "✓ Watched" } else { "□ Unwatched" };

        // Format length as hours:minutes:seconds
        let hours = self.length_in_seconds / 3600;
        let minutes = (self.length_in_seconds % 3600) / 60;
        let seconds = self.length_in_seconds % 60;
        let length = if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        };

        // Write the formatted output
        write!(
            f,
            "{}{} [{}] {} | Modified: {}{}{}{}\nPath: {}{}",
            title_display,
            episode_info,
            length,
            watched_status,
            self.timestamp_modified.format("%Y-%m-%d %H:%M:%S"),
            director,
            countries,
            languages,
            self.filepath,
            self.tmdb_id.map_or(String::new(), |id| format!(" | TMDB ID: {}", id))
        )
    }
}