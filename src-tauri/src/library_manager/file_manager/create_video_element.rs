use chrono::{TimeZone, Utc};
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use super::super::{compute_selection, VideoElement, LIBRARIES};
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
    ve.playback_selection = Some(compute_selection(ve));
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
