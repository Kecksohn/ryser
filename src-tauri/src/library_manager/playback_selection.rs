//! Automatic audio/subtitle track selection.
//!
//! Picks the audio track matching the film's original language (from TMDB) and,
//! when that audio is not English, a subtitle track in the user's preferred
//! language (English for now; configurable later).

use serde::{Deserialize, Serialize};

use super::lang_map::{tmdb_matches, to_iso_639_1};
use super::VideoElement;

/// Preferred subtitle language (ISO 639-1). Hard-coded for now; will move to
/// config. The whole feature keys off this single constant.
const PREFERRED_SUBTITLE_LANG: &str = "en";

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum SubtitleStatus {
    /// A subtitle track was chosen (see `subtitle_index`/`subtitle_lang`).
    Selected,
    /// Chosen audio is already in the preferred language; no subtitles wanted.
    #[default]
    NotNeeded,
    /// No subtitle tracks exist in the file.
    Unavailable,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct PlaybackSelection {
    /// Type-relative ordinal of the chosen audio track (VLC `--audio-track`).
    pub audio_index: Option<usize>,
    /// Chosen audio language, ISO 639-1 when known, else the raw tag.
    pub audio_lang: Option<String>,
    pub subtitle_index: Option<usize>,
    pub subtitle_lang: Option<String>,
    pub subtitle_status: SubtitleStatus,
}

fn is_commentary(title: &Option<String>) -> bool {
    title
        .as_ref()
        .map(|t| t.to_ascii_lowercase().contains("comment"))
        .unwrap_or(false)
}

/// Display label for a track tag: prefer ISO 639-1, fall back to the raw tag.
fn display_lang(tag: &str) -> String {
    to_iso_639_1(tag).map(|s| s.to_string()).unwrap_or_else(|| tag.to_string())
}

pub fn compute_selection(ve: &VideoElement) -> PlaybackSelection {
    let audio = ve.audio_languages.clone().unwrap_or_default();
    let audio_titles = ve.audio_titles.clone().unwrap_or_default();
    let subs = ve.subtitle_languages.clone().unwrap_or_default();

    let mut sel = PlaybackSelection::default();

    // --- Audio pick (tiered) ---
    let title_of = |i: usize| audio_titles.get(i).cloned().flatten();
    let is_comment = |i: usize| is_commentary(&title_of(i));

    let normalised: Vec<Option<&'static str>> =
        audio.iter().map(|t| to_iso_639_1(t)).collect();

    let audio_index = if audio.is_empty() {
        None
    } else {
        // 1. Match TMDB original language (skipping commentary tracks).
        let tmdb_pick = ve.tmdb_language.as_ref().and_then(|tl| {
            normalised.iter().enumerate().find_map(|(i, n)| {
                match n {
                    Some(code) if !is_comment(i) && tmdb_matches(tl, code) => Some(i),
                    _ => None,
                }
            })
        });
        // 2. First English track. 3. First non-commentary track. 4. Index 0.
        tmdb_pick
            .or_else(|| {
                normalised
                    .iter()
                    .enumerate()
                    .find_map(|(i, n)| (*n == Some("en") && !is_comment(i)).then_some(i))
            })
            .or_else(|| (0..audio.len()).find(|&i| !is_comment(i)))
            .or(Some(0))
    };

    sel.audio_index = audio_index;
    sel.audio_lang = audio_index.map(|i| display_lang(&audio[i]));

    let audio_is_preferred = sel
        .audio_lang
        .as_deref()
        .map(|l| l == PREFERRED_SUBTITLE_LANG)
        .unwrap_or(false);

    // --- Subtitle pick (depends on chosen audio) ---
    if audio_is_preferred {
        sel.subtitle_status = SubtitleStatus::NotNeeded;
    } else if subs.is_empty() {
        sel.subtitle_status = SubtitleStatus::Unavailable;
    } else {
        // Prefer a subtitle in PREFERRED_SUBTITLE_LANG, else first available.
        let pref = subs
            .iter()
            .position(|t| to_iso_639_1(t) == Some(PREFERRED_SUBTITLE_LANG));
        let idx = pref.unwrap_or(0);
        sel.subtitle_index = Some(idx);
        sel.subtitle_lang = Some(display_lang(&subs[idx]));
        sel.subtitle_status = SubtitleStatus::Selected;
    }

    sel
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ve(tmdb: Option<&str>, audio: &[&str], titles: &[Option<&str>], subs: &[&str]) -> VideoElement {
        VideoElement {
            tmdb_language: tmdb.map(|s| s.to_string()),
            audio_languages: Some(audio.iter().map(|s| s.to_string()).collect()),
            audio_titles: Some(titles.iter().map(|t| t.map(|s| s.to_string())).collect()),
            subtitle_languages: Some(subs.iter().map(|s| s.to_string()).collect()),
            ..Default::default()
        }
    }

    #[test]
    fn single_audio_picks_index_zero() {
        let s = compute_selection(&ve(Some("fr"), &["fre"], &[None], &["eng"]));
        assert_eq!(s.audio_index, Some(0));
        assert_eq!(s.audio_lang.as_deref(), Some("fr"));
        // non-English audio + English sub available
        assert_eq!(s.subtitle_status, SubtitleStatus::Selected);
        assert_eq!(s.subtitle_index, Some(0));
        assert_eq!(s.subtitle_lang.as_deref(), Some("en"));
    }

    #[test]
    fn tmdb_match_over_first_track() {
        // [eng, fre]; original = fr -> pick fre at index 1
        let s = compute_selection(&ve(Some("fr"), &["eng", "fre"], &[None, None], &["eng"]));
        assert_eq!(s.audio_index, Some(1));
        assert_eq!(s.audio_lang.as_deref(), Some("fr"));
    }

    #[test]
    fn english_audio_needs_no_subtitles() {
        let s = compute_selection(&ve(Some("en"), &["eng"], &[None], &["eng", "fre"]));
        assert_eq!(s.audio_lang.as_deref(), Some("en"));
        assert_eq!(s.subtitle_status, SubtitleStatus::NotNeeded);
        assert!(s.subtitle_index.is_none());
    }

    #[test]
    fn non_english_audio_no_subs_is_unavailable() {
        let s = compute_selection(&ve(Some("ja"), &["jpn"], &[None], &[]));
        assert_eq!(s.subtitle_status, SubtitleStatus::Unavailable);
    }

    #[test]
    fn commentary_track_is_skipped() {
        // English original; first eng track is commentary -> pick the second eng
        let s = compute_selection(&ve(
            Some("en"),
            &["eng", "eng"],
            &[Some("English Commentary by Director"), None],
            &[],
        ));
        assert_eq!(s.audio_index, Some(1));
    }

    #[test]
    fn tmdb_cn_matches_chinese_track() {
        // TMDB uses 'cn' for Cantonese; ffmpeg tags 'chi' -> zh
        let s = compute_selection(&ve(Some("cn"), &["chi", "eng"], &[None, None], &["eng"]));
        assert_eq!(s.audio_index, Some(0));
        assert_eq!(s.audio_lang.as_deref(), Some("zh"));
    }

    #[test]
    fn all_untagged_falls_back_to_first() {
        let s = compute_selection(&ve(Some("fr"), &["und", "und"], &[None, None], &[]));
        assert_eq!(s.audio_index, Some(0));
        assert_eq!(s.subtitle_status, SubtitleStatus::Unavailable);
    }

    #[test]
    fn first_subtitle_fallback_when_no_preferred() {
        // Japanese audio, subs exist but none English -> first sub used
        let s = compute_selection(&ve(Some("ja"), &["jpn"], &[None], &["fre", "ger"]));
        assert_eq!(s.subtitle_status, SubtitleStatus::Selected);
        assert_eq!(s.subtitle_index, Some(0));
        assert_eq!(s.subtitle_lang.as_deref(), Some("fr"));
    }
}
