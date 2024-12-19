use std::fs;

use serde::Deserialize;

use crate::tmdb_api::*;

#[derive(Default, Clone, serde::Serialize, Deserialize, Debug)]
pub struct video_file {
    filepath: String,
    title: Option<String>,
    year: Option<i16>,
    poster_path: Option<String>,
    director: Option<String>,
    countries: Option<Vec<String>>,
    languages: Option<Vec<String>>,
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_video_files(folder_path: &str) -> Vec<video_file> {
    let mut video_files: Vec<video_file> = vec![];

    let files = fs::read_dir(folder_path).unwrap();
    for file in files {
        if let Ok(valid_file) = file {
            match valid_file.path().to_str() {
                Some(v) => {
                    let vf = video_file {
                        filepath: v.to_owned(),
                        ..Default::default()
                    };
                    video_files.push(vf)
                }
                None => {}
            }
        }
    }

    video_files
}
