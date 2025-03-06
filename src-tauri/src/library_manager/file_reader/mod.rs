mod read_metadata;

use chrono::{TimeZone, Utc};
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use super::tmdb_api::*;
use super::{video_element, LIBRARIES};
use read_metadata::get_duration_in_s;

pub fn get_modified_secs(file: &str) -> usize {
    // Get modification timestamp from file.
    let modified_date = fs::metadata(file).expect("Need metadata");
    let secs = modified_date
        .modified()
        .expect("Need modified date")
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Need duration")
        .as_secs();
    secs.try_into().unwrap()
}

pub(super) fn create_video_element_from_file(filepath: &str) -> video_element {
    let mut ve = video_element {
        filepath: filepath.to_owned(),
        ..Default::default()
    };
    match get_duration_in_s(filepath) {
        Ok(length_in_s) => {
            ve.length_in_seconds = length_in_s as i32;
        }
        Err(error) => {
            println!(
                "Get duration of video file failed with Error: {}",
                error.to_string()
            )
        }
    }
    let modified = get_modified_secs(filepath);
    ve.timestamp_modified = Utc.timestamp_opt(modified as i64, 0).unwrap();
    ve
}

pub(super) fn get_video_files(folder_path: &str) -> Vec<video_element> {
    let mut video_files: Vec<video_element> = vec![];

    let files = fs::read_dir(folder_path).unwrap();
    for file in files {
        if let Ok(valid_file) = file {
            if valid_file.metadata().unwrap().is_file() && is_video_file(&valid_file.path()) {
                if let Some(filepath_str) = valid_file.path().to_str() {
                    let vf = create_video_element_from_file(filepath_str);
                    video_files.push(vf)
                }
            }
        }
    }

    video_files
}

pub(super) fn is_video_file(filepath: &PathBuf) -> bool {
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

fn get_library_videos_old(library_id: &str) -> Vec<video_element> {
    let mut library_videos: Vec<video_element> = vec![];

    for library in LIBRARIES.lock().unwrap().iter() {
        if library.id == library_id {
            for folder_path in library.library_paths.iter() {
                library_videos.append(&mut get_video_files(&folder_path));
            }
        }
    }

    library_videos
}
