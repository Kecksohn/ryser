use directories::ProjectDirs;
use std::fs;

pub(crate) mod file_reader;
mod json_parser;



use tauri_plugin_http::reqwest::Error;
use crate::tmdb_api::*;
use json_parser::*;

use file_reader::video_file;
use serde::Deserialize;


#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct library {
    id: String,
    library_paths: Vec<String>,
    video_files: Vec<video_file>,
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
