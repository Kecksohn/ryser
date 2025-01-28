use std::{fs::{self, DirEntry}, path::PathBuf};

use serde::Deserialize;

use crate::tmdb_api::*;

use super::LIBRARIES;


#[derive(Default, Clone, serde::Serialize, Deserialize, Debug)]
pub struct video_element {
    pub filepath: String,
    title: Option<String>,
    year: Option<i16>,
    poster_path: Option<String>,
    thumbnail_path: Option<String>,
    director: Option<String>,
    countries: Option<Vec<String>>,
    languages: Option<Vec<String>>,
    watched: bool,
}

pub fn create_video_element_from_file(filepath: &str) -> video_element {
    let ve = video_element {
        filepath: filepath.to_owned(),
        ..Default::default()
    };
    ve
}


pub fn get_video_files(folder_path: &str) -> Vec<video_element> {
    let mut video_files: Vec<video_element> = vec![];

    let files = fs::read_dir(folder_path).unwrap();
    for file in files {
        if let Ok(valid_file) = file {
            if valid_file.metadata().unwrap().is_file() && is_video_file(&valid_file.path()) {
                if let Some(filepath_str) = valid_file.path().to_str() {
                    let vf = video_element {
                        filepath: filepath_str.to_owned(),
                        ..Default::default()
                    };
                    video_files.push(vf)
                }
            }
        }
    }

    video_files
}

pub fn is_video_file(filepath: &PathBuf) -> bool {
    if let Some(ext) = filepath.extension() {
        if let Some(ext_str) = ext.to_str() {
            return matches!(ext_str.to_lowercase().as_str(),
                    "mkv" | "mp4" | "avi" | "mov" | "m2ts"); // TODO: Read from config and allow user additions
        }
    }
    else {
        println!("Couldn't get extension type for {:?}", filepath);
    }
    return false;
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_library_videos_old(library_id: &str) -> Vec<video_element> {
    
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