use directories::ProjectDirs;
use std::{fs, vec};

pub(crate) mod file_reader;
mod json_parser;



use tauri_plugin_http::reqwest::Error;
use crate::tmdb_api::*;
use json_parser::*;
use file_reader::*;
use serde::Deserialize;


#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct library {
    id: String,
    library_paths: Vec<String>,
    video_files: Vec<video_element>,
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

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn get_libraries_gui() -> Vec<library> {
    print!("Please write a better function!");
    return LIBRARIES.lock().unwrap().clone();
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
            }
            else {
                for index in current_library_match_start_index..library.video_files.len() {
                    if &library.video_files[index].filepath == filepath_str {
                        // Match, do nothing
                        current_library_match_start_index = index+1;
                        break;
                    }
                    else if &library.video_files[index].filepath > filepath_str {
                        // Compared File should come before json entry but isn't in it -> New Entry
                        new_filepath_indices.push(filepath_index);
                        break;
                    }
                    else {
                        // Compared Entry in json is missing -> Removed Entry, but keep checking next json entry for file match
                        missing_video_files_indices.push(index);
                        current_library_match_start_index = index+1;
                    }
                }
            }
        }
        // We are out of files in the paths, all remaining json entries have been removed
        for library_index in current_library_match_start_index..library.video_files.len() {
            missing_video_files_indices.push(library_index);
        }

        println!("Library: {}", library.id);
        println!("Removed Files: ({}):", missing_video_files_indices.len());
        for index in missing_video_files_indices.iter().rev() {
            println!("{}", library.video_files[*index].filepath);
            library.video_files.remove(*index);
        }
        println!("New Files ({}): ", new_filepath_indices.len());
        for filepath_index in new_filepath_indices.iter() {
            println!("{}", &video_files_in_library_paths[*filepath_index]);
            let video_file = create_video_element_from_file(&video_files_in_library_paths[*filepath_index]);
            library.video_files.push(video_file);
        }

        library.video_files.sort_by(|d1, d2| d1.filepath.cmp(&d2.filepath));
        println!("Before: {}, Now: {}", library.video_files.len() - new_filepath_indices.len() + missing_video_files_indices.len() , library.video_files.len());
        write_library(&library);
    } 
}



#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn call_public() {

    match get_movie_information_tmdb("das weiße band").await {
        Ok(()) => (),
        Err(_) => return,
    }

    if LIBRARIES.lock().unwrap().len() > 0 {
        println!("{}", LIBRARIES.lock().unwrap()[0].id);
    }

    /*
    
    let lib = library {
        id: "tvshows".to_owned(),
        library_paths: vec!["F:/tv/".to_owned()],
        video_files: vec![]
    };
    write_library(&lib);
     */
    
}
