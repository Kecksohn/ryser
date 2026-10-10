use chrono::{TimeZone, Utc};
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use super::super::{compute_selection, VideoElement, LIBRARIES};
use super::external_subtitles::{find_external_subtitles, DirCache};
use super::read_metadata::{get_audio_tracks, get_duration_in_s, get_subtitle_tracks};

pub fn get_modified_secs(filepath: &str) -> usize {
    // Get modification timestamp from file.
    let modified_date = fs::metadata(filepath).expect("Need metadata");
    let secs = modified_date
        .modified()
        .expect("Need modified date")
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Need duration")
        .as_secs();
    secs.try_into().unwrap()
}

pub fn create_video_element_from_file(filepath: &str) -> VideoElement {
    let mut ve = VideoElement {
        filepath: filepath.to_owned(),
        ..Default::default()
    };
    match get_duration_in_s(filepath) {
        Ok(length_in_s) => {
            ve.length_in_seconds = length_in_s as i32;
        }
        Err(error) => {
            println!("Get duration of video file failed with Error: {}", error);
        }
    }

    fill_track_metadata(&mut ve);

    let modified = get_modified_secs(filepath);
    ve.timestamp_modified = Utc.timestamp_opt(modified as i64, 0).unwrap();
    ve
}

/// Parse audio/subtitle track languages + titles from the file and (re)compute
/// the playback selection. `tmdb_language` may not be set yet at scan time; the
/// selection is recomputed again after TMDB enrich.
pub(crate) fn fill_track_metadata(ve: &mut VideoElement) {
    let audio = get_audio_tracks(&ve.filepath);
    ve.audio_languages = Some(audio.languages);
    ve.audio_titles = Some(audio.titles);
    let subs = get_subtitle_tracks(&ve.filepath);
    ve.subtitle_languages = Some(subs.languages);
    ve.subtitle_titles = Some(subs.titles);
    ve.external_subtitles = None;
    fill_external_subtitles(ve, &mut DirCache::new());
    ve.playback_selection = Some(compute_selection(ve));
}

/// (Re)discover external subtitle files and list them after the embedded
/// subtitle tracks. Returns whether anything changed (selection is then recomputed).
pub(crate) fn fill_external_subtitles(ve: &mut VideoElement, cache: &mut DirCache) -> bool {
    let found = find_external_subtitles(&ve.filepath, cache);
    let paths: Vec<String> = found.iter().map(|s| s.path.clone()).collect();
    let old = ve.external_subtitles.clone().unwrap_or_default();
    if old == paths {
        return false;
    }

    let languages = ve.subtitle_languages.get_or_insert_with(Vec::new);
    let titles = ve.subtitle_titles.get_or_insert_with(Vec::new);
    let embedded = languages.len().saturating_sub(old.len());
    languages.truncate(embedded);
    titles.resize(embedded, None);
    for sub in found {
        languages.push(sub.language);
        titles.push(Some(sub.title));
    }
    ve.external_subtitles = Some(paths);
    ve.playback_selection = Some(compute_selection(ve));
    true
}

pub fn get_video_files(folder_path: &str) -> Vec<VideoElement> {
    let mut video_files: Vec<VideoElement> = vec![];

    let files = fs::read_dir(folder_path).unwrap();
    for file in files.flatten() {
        if file.metadata().unwrap().is_file() && is_video_file(&file.path()) {
            if let Some(filepath_str) = file.path().to_str() {
                let vf = create_video_element_from_file(filepath_str);
                video_files.push(vf)
            }
        }
    }

    video_files
}

pub fn is_video_file(filepath: &PathBuf) -> bool {
    if let Some(ext) = filepath.extension() {
        if let Some(ext_str) = ext.to_str() {
            return matches!(
                ext_str.to_lowercase().as_str(),
                "mkv" | "mp4" | "avi" | "mov" | "m2ts"
            ); // TODO: Read from config and allow user additions
        }
    } else {
        println!("Couldn't get extension type for {:?}", filepath);
    }
    false
}
