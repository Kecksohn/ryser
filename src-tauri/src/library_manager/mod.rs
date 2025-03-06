use directories::ProjectDirs;
use std::{fs, vec};

mod file_reader;
pub(crate) mod gui_functions;
mod json_parser;
mod tmdb_api;

use file_reader::*;
use json_parser::*;
use serde::Deserialize;
use tauri_plugin_http::reqwest::Error;
use tmdb_api::*;

use chrono::serde::ts_milliseconds;
use chrono::DateTime;
use chrono::Utc;

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct library {
    id: String,
    name: String,
    library_paths: Vec<String>,
    video_files: Vec<video_element>,
    child_libraries: Vec<library>,
}

#[derive(Default, Clone, serde::Serialize, Deserialize, Debug)]
pub struct video_element {
    pub filepath: String,
    watched: bool,
    parsed: bool,
    poster_path: Option<String>,
    thumbnail_path: Option<String>,

    title: Option<String>,
    year: Option<i16>,
    director: Option<String>,
    countries: Option<Vec<String>>,
    languages: Option<Vec<String>>,

    season: Option<i32>,
    episode: Option<i32>,

    index_priority: i32,
    length_in_seconds: i32,
    #[serde(with = "ts_milliseconds")]
    timestamp_modified: DateTime<Utc>,
}

use std::sync::Mutex;
static LIBRARIES: Mutex<Vec<library>> = Mutex::new(Vec::new());

pub(crate) fn load_all_libraries() {
    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        let libraries_folder = proj_dir.data_local_dir();
        for file_or_folder in fs::read_dir(libraries_folder).unwrap() {
            match file_or_folder {
                Ok(f) => {
                    if f.path().is_dir() {
                        match get_library(f.file_name().to_str().unwrap()) {
                            Ok(lib) => LIBRARIES.lock().unwrap().push(lib),
                            Err(error) => println!(
                                "Could not parse library at {}: {}",
                                f.path().to_str().unwrap(),
                                error
                            ),
                        }
                    }
                }
                Err(error) => println!("Error while reading libraries folder: {}", error),
            }
        }
    }
}

pub(crate) fn set_libraries(libraries: Vec<library>) {
    *LIBRARIES.lock().unwrap() = libraries;
}

//  Compares Files present in library paths with data in json
//  Tries to match files whose filenames have simply changed (!TODO: MD5 sum or simply length?)
pub(crate) fn check_for_library_changes() {
    for library in LIBRARIES.lock().unwrap().iter_mut() {
        // Since we want to optimize matching we first get all files and then compare both sorted lists simultaneously
        let mut video_files_in_library_paths: Vec<String> = vec![];

        // Get all video files in all library_paths
        for folder_path in library.library_paths.iter() {
            let files = fs::read_dir(folder_path).unwrap();
            for file in files {
                let Ok(valid_file) = file else {
                    println!("File is not valid");
                    continue;
                };
                let Ok(metadata) = valid_file.metadata() else {
                    println!("Could not read Metadata");
                    continue;
                };

                let filepath = valid_file.path();
                if metadata.is_file() && is_video_file(&filepath) {
                    let Some(filepath_str) = filepath.to_str() else {
                        println!("Could not extract string value from path");
                        continue;
                    };

                    video_files_in_library_paths.push(filepath_str.to_string());
                }
            }
        }

        video_files_in_library_paths.sort_by(|d1, d2| d1.cmp(&d2));

        let mut new_filepath_indices: Vec<usize> = vec![];
        let mut missing_video_files_indices: Vec<usize> = vec![];

        let mut current_library_match_start_index: usize = 0;

        //  Walk through video_files stored in library.json currently
        for filepath_index in 0..video_files_in_library_paths.len() {
            let filepath_str: &String = &video_files_in_library_paths[filepath_index];

            if current_library_match_start_index >= library.video_files.len() {
                //  We are out of elements in the json, all remaining files are new
                new_filepath_indices.push(filepath_index);
            } else {
                for index in current_library_match_start_index..library.video_files.len() {
                    if &library.video_files[index].filepath == filepath_str {
                        // Match, do nothing
                        current_library_match_start_index = index + 1;
                        break;
                    } else if &library.video_files[index].filepath > filepath_str {
                        // Compared File should come before json entry but isn't in it -> New Entry
                        new_filepath_indices.push(filepath_index);
                        break;
                    } else {
                        // Compared Entry in json is missing -> Removed Entry, but keep checking next json entry for file match
                        missing_video_files_indices.push(index);
                        current_library_match_start_index = index + 1;
                    }
                }
            }
        }
        // We are out of files in the paths, all remaining json entries have been removed
        for library_index in current_library_match_start_index..library.video_files.len() {
            missing_video_files_indices.push(library_index);
        }

        if missing_video_files_indices.len() > 0 || new_filepath_indices.len() > 0 {
            println!("Library: {}", library.id);
            println!("Removed Files: ({}):", missing_video_files_indices.len());
            for index in missing_video_files_indices.iter().rev() {
                println!("{}", library.video_files[*index].filepath);
                library.video_files.remove(*index);
            }
            println!("New Files ({}): ", new_filepath_indices.len());
            for filepath_index in new_filepath_indices.iter() {
                println!("{}", &video_files_in_library_paths[*filepath_index]);
                let video_file =
                    create_video_element_from_file(&video_files_in_library_paths[*filepath_index]);
                library.video_files.push(video_file);
            }

            library
                .video_files
                .sort_by(|d1, d2| d1.filepath.cmp(&d2.filepath));
            println!(
                "Before: {}, Now: {}",
                library.video_files.len() - new_filepath_indices.len()
                    + missing_video_files_indices.len(),
                library.video_files.len()
            );
            write_library(&library);
        }
    }
}

pub(crate) fn update_library_entry(
    library: &mut library,
    updated_element: video_element,
) -> Result<(), String> {
    let Some(proj_dir) = ProjectDirs::from("", "", "ryser") else {
        return Result::Err("Could not get project dirs".to_owned());
    };
    let library_folder = proj_dir.data_local_dir().join(&library.id);
    if !library_folder.exists() {
        return Result::Err("Could not find library".to_owned());
    }
    for old_element in library.video_files.iter_mut() {
        if old_element.filepath == updated_element.filepath {
            *old_element = updated_element;
            return Ok(());
        }
    }
    return Result::Err(format!(
        "Could not find '{}' in '{}'",
        updated_element.filepath, library.id
    ));
}

pub(crate) fn update_library_entry_by_index(
    library: &mut library,
    updated_element: video_element,
    index: usize,
) -> Result<(), String> {
    if index >= library.video_files.len() {
        return Result::Err(format!(
            "Index {} out of range of library elements ({})",
            index,
            library.video_files.len()
        ));
    }
    library.video_files[index] = updated_element;
    Ok(())
}
