
pub(crate) mod file_reader;
use file_reader::video_file;
use serde::Deserialize;

mod json_parser;
use json_parser::*;


#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct library {
    identifier:     String,
    library_paths:  Vec<String>,
    video_files:    Vec<video_file>,
}

use std::sync::Mutex;
static LIBRARIES: Mutex<Vec<library>> = Mutex::new(Vec::new());


#[tauri::command(rename_all = "snake_case")]
pub(crate) fn get_libraries_gui() -> Vec<library> {
    return LIBRARIES.lock().unwrap().clone();
}

pub(crate) fn set_libraries(libraries: Vec<library>) {
    *LIBRARIES.lock().unwrap() = libraries;
} 



#[tauri::command(rename_all = "snake_case")]
pub(crate) fn call_public() {
    
    let libraries: Vec<library> = get_all_libraries();
    
    if (libraries.len() > 0) {
        println!("{}", libraries[0].identifier);
    }

    /*
    let lib = library {
        identifier: "movies".to_owned(),
        library_paths: vec!["bitches stay".to_owned()],
        video_files: vec![]
    };
    write_library(lib);
    */
}