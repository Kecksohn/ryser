use serde::{Deserialize, Serialize};
use chrono::{serde::ts_milliseconds, DateTime, Utc};

use super::playback_selection::PlaybackSelection;


/// One video file. File facts and user state live here, catalog metadata in `media`.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(from = "VideoElementRaw")]
pub struct VideoElement {
    pub filepath: String,
    pub watched: bool,
    pub thumbnail_path: Option<String>,

    /// Original language of the film or show, cached as input for `compute_selection`.
    pub tmdb_language: Option<String>,
    /// Automatic matching ran to a definite result (a match, or nothing found).
    pub tmdb_searched: bool,
    pub media: Media,

    pub audio_languages: Option<Vec<String>>,
    pub subtitle_languages: Option<Vec<String>>,
    pub audio_titles: Option<Vec<Option<String>>>,
    pub subtitle_titles: Option<Vec<Option<String>>>,
    /// Paths of external subtitle files. Their languages/titles are the last
    /// entries of `subtitle_languages`/`subtitle_titles`, after the embedded tracks.
    pub external_subtitles: Option<Vec<String>>,
    pub playback_selection: Option<PlaybackSelection>,

    pub index_priority: i32,
    pub length_in_seconds: i32,
    #[serde(with = "ts_milliseconds")]
    pub timestamp_modified: DateTime<Utc>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Media {
    #[default]
    Unmatched,
    Film(FilmMeta),
    Episode(EpisodeMeta),
    /// Bonus material of a show, not looked up on TMDB
    Extra(ExtraMeta),
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct ExtraMeta {
    /// Key into `Library.tv_shows`
    pub show_id: u32,
    /// None = belongs to the show, not to one season
    pub season: Option<i32>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct FilmMeta {
    /// None = titled by the user, not on TMDB
    pub tmdb_id: Option<usize>,
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub release_date: Option<String>,
    pub director: Option<String>,
    pub countries: Option<Vec<String>>,
    pub genre_ids: Option<Vec<usize>>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct EpisodeMeta {
    /// Key into `Library.tv_shows`
    pub show_id: u32,
    /// 0 = specials
    pub season: Option<i32>,
    pub episode: Option<i32>,
    /// The episode's own TMDB ID. None = no details from TMDB
    pub tmdb_id: Option<usize>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub air_date: Option<String>,
    pub still_path: Option<String>,
    pub director: Option<String>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct TvShow {
    /// Local ID, also the key in `Library.tv_shows`
    pub id: u32,
    /// None = not found on TMDB
    pub tmdb_id: Option<usize>,
    /// Show names parsed from filenames/folders that map to this show
    pub parsed_names: Vec<String>,
    pub name: Option<String>,
    pub original_name: Option<String>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub first_air_date: Option<String>,
    pub genre_ids: Option<Vec<usize>>,
    pub countries: Option<Vec<String>>,
    pub tmdb_language: Option<String>,
    pub creators: Option<Vec<String>>,
    pub status: Option<String>,
    pub seasons: Vec<TvSeason>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct TvSeason {
    pub season_number: i32,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub air_date: Option<String>,
    /// Number of episodes according to TMDB, not the number of files owned
    pub episode_count: Option<i32>,
}


/// On-disk form of `VideoElement`: the current fields plus the flat film
/// fields of schema 1, which `From` folds into `media`.
#[derive(Deserialize)]
struct VideoElementRaw {
    filepath: String,
    watched: bool,
    thumbnail_path: Option<String>,
    tmdb_language: Option<String>,
    #[serde(default)]
    tmdb_searched: bool,
    media: Option<Media>,

    audio_languages: Option<Vec<String>>,
    subtitle_languages: Option<Vec<String>>,
    audio_titles: Option<Vec<Option<String>>>,
    subtitle_titles: Option<Vec<Option<String>>>,
    external_subtitles: Option<Vec<String>>,
    playback_selection: Option<PlaybackSelection>,

    index_priority: i32,
    length_in_seconds: i32,
    #[serde(with = "ts_milliseconds")]
    timestamp_modified: DateTime<Utc>,

    // Schema 1 only
    season: Option<i32>,
    #[serde(flatten)]
    film: FilmMeta,
}

impl From<VideoElementRaw> for VideoElement {
    fn from(raw: VideoElementRaw) -> Self {
        let (media, tmdb_searched) = match raw.media {
            Some(media) => (media, raw.tmdb_searched),
            // Schema 1 episodes held the show's ID and a baked title: rebuilt by the next parse
            None if raw.season.is_some() => (Media::Unmatched, false),
            None if raw.film == FilmMeta::default() => (Media::Unmatched, false),
            None => {
                let tmdb_searched = raw.film.tmdb_id.is_some();
                (Media::Film(raw.film), tmdb_searched)
            }
        };

        VideoElement {
            filepath: raw.filepath,
            watched: raw.watched,
            thumbnail_path: raw.thumbnail_path,
            tmdb_language: raw.tmdb_language,
            tmdb_searched,
            media,
            audio_languages: raw.audio_languages,
            subtitle_languages: raw.subtitle_languages,
            audio_titles: raw.audio_titles,
            subtitle_titles: raw.subtitle_titles,
            external_subtitles: raw.external_subtitles,
            playback_selection: raw.playback_selection,
            index_priority: raw.index_priority,
            length_in_seconds: raw.length_in_seconds,
            timestamp_modified: raw.timestamp_modified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_FIELDS: &str = r#""filepath": "a.mkv", "watched": true, "thumbnail_path": null,
        "tmdb_language": "de", "audio_languages": ["ger"], "subtitle_languages": [],
        "index_priority": 0, "length_in_seconds": 60, "timestamp_modified": 1000"#;

    fn parse(extra_fields: &str) -> VideoElement {
        serde_json::from_str(&format!("{{{}, {}}}", FILE_FIELDS, extra_fields)).unwrap()
    }

    #[test]
    fn schema_1_film_becomes_film() {
        let ve = parse(r#""tmdb_id": 5, "title": "T", "poster_path": "p.jpg", "season": null, "episode": null"#);
        assert!(ve.watched && ve.tmdb_searched);
        let Media::Film(film) = ve.media else { panic!("expected a film") };
        assert_eq!(film.tmdb_id, Some(5));
        assert_eq!(film.title.as_deref(), Some("T"));
        assert_eq!(film.poster_path.as_deref(), Some("p.jpg"));
    }

    #[test]
    fn schema_1_user_titled_film_is_still_searched() {
        let ve = parse(r#""tmdb_id": null, "title": "T""#);
        assert!(matches!(ve.media, Media::Film(_)));
        assert!(!ve.tmdb_searched);
    }

    #[test]
    fn schema_1_episode_and_empty_entry_become_unmatched() {
        let episode = parse(r#""tmdb_id": 7, "title": "Show S01E02 Name", "season": 1, "episode": 2"#);
        assert_eq!(episode.media, Media::Unmatched);
        assert!(!episode.tmdb_searched);

        let empty = parse(r#""tmdb_id": null, "title": null"#);
        assert_eq!(empty.media, Media::Unmatched);
    }

    #[test]
    fn current_format_round_trips() {
        let mut ve = parse(r#""tmdb_id": 5, "title": "T""#);
        ve.media = Media::Episode(EpisodeMeta { show_id: 3, season: Some(0), name: Some("N".to_owned()), ..Default::default() });
        ve.tmdb_searched = true;

        let json = serde_json::to_string(&ve).unwrap();
        assert!(json.contains(r#""kind":"episode""#));
        let reloaded: VideoElement = serde_json::from_str(&json).unwrap();
        assert_eq!(reloaded.media, ve.media);
        assert!(reloaded.tmdb_searched && reloaded.watched);
        assert_eq!(reloaded.timestamp_modified, ve.timestamp_modified);
    }

    /// A `media` object as the edit view sends it to `set_media`.
    /// `buildMedia.test.js` asserts the frontend produces exactly these files.
    macro_rules! sent_media {
        ($name:literal) => {
            sent_media(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../test-fixtures/media/", $name)))
        };
    }

    fn sent_media(json: &str) -> Media {
        let sent: serde_json::Value = serde_json::from_str(json).unwrap();
        let media: Media = serde_json::from_value(sent.clone()).unwrap();
        // Unknown fields are dropped silently, so check that every sent field arrived
        let stored = serde_json::to_value(&media).unwrap();
        for (field, value) in sent.as_object().unwrap() {
            assert_eq!(stored.get(field), Some(value), "field '{}'", field);
        }
        media
    }

    #[test]
    fn media_sent_by_the_edit_view_deserializes() {
        let Media::Film(film) = sent_media!("film.json") else { panic!("expected a film") };
        assert_eq!(film.tmdb_id, Some(348));
        assert_eq!(film.countries, Some(vec!["US".to_owned(), "GB".to_owned()]));

        let Media::Film(user_film) = sent_media!("user_film.json") else { panic!("expected a film") };
        assert_eq!(user_film.tmdb_id, None);
        assert_eq!(user_film.title.as_deref(), Some("Holiday 2019"));

        let Media::Episode(episode) = sent_media!("episode.json") else { panic!("expected an episode") };
        assert_eq!((episode.show_id, episode.season, episode.episode), (3, Some(1), Some(2)));
        assert_eq!(episode.name.as_deref(), Some("Die Mitte der Welt"));

        // Moved or switched: no TMDB ID, so set_media looks the episode up
        let Media::Episode(moved) = sent_media!("episode_moved.json") else { panic!("expected an episode") };
        assert_eq!((moved.show_id, moved.season, moved.episode, moved.tmdb_id), (4, Some(0), Some(5), None));
        assert_eq!(moved.name.as_deref(), Some("Christmas Special"));
        let Media::Episode(from_film) = sent_media!("episode_from_film.json") else { panic!("expected an episode") };
        assert_eq!((from_film.show_id, from_film.tmdb_id, from_film.name), (3, None, None));

        let Media::Film(from_episode) = sent_media!("film_from_episode.json") else { panic!("expected a film") };
        assert_eq!((from_episode.tmdb_id, from_episode.title.as_deref()), (None, Some("Die Mitte der Welt")));

        assert_eq!(sent_media!("extra.json"), Media::Extra(ExtraMeta { show_id: 3, season: Some(1) }));
    }
}
