use std::fs;
use directories::ProjectDirs;

use super::library;
use crate::{get_video_files, video_file};


pub(super) fn get_library(identifier: &str) -> Option<library> {

    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        let library_json_filepath = proj_dir.data_local_dir().join(identifier).join("library.json");
    
        if! library_json_filepath.exists() {
            print!("No good.");
        }

        match fs::File::open(&library_json_filepath) {
            Ok(json_file) => {
                match serde_json::from_reader(json_file) {
                    Ok(library) => return library,
                    Err(error) => {
                        println!("Error extracting {} , error: {}", &library_json_filepath.to_str().unwrap(), error);
                        return None;
                    }
                }
            },
            Err(error) => {
                println!("Problem opening {}: {}", &library_json_filepath.to_str().unwrap(), error);
                return None;
            }
        }
    }
    else {
        println!("Could not get project config dir paths");
        return None;
    }
}


pub(super) fn get_all_libraries() -> Vec<library> {
    
    let mut libraries = Vec::new();

    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        let libraries_folder = proj_dir.data_local_dir();
        for file_or_folder in fs::read_dir(libraries_folder).unwrap() {
            match file_or_folder {
                Ok(f) => {
                    if f.path().is_dir() {
                        match get_library(f.file_name().to_str().unwrap()) {
                            Some(lib) => libraries.push(lib),
                            None => println!("Could not parse library at {}", f.path().to_str().unwrap())
                        }
                    }
                },
                Err(error) => println!("Error while reading libraries folder: {}", error)
            }
        }
    }

    return libraries;
}


pub(super) fn write_library(library: library) {
    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        
        // Create library folder if it does not exist
        let library_folder = proj_dir.data_local_dir().join(&library.identifier);
        if! library_folder.exists() {
            match fs::create_dir_all(&library_folder) {
                Ok(()) => {},
                Err(error) => panic!("Problem creating folder: {error:?}"),
            }
        }

        // Write json to library.json
        let library_json_file = library_folder.join("library.json");
        let file = fs::File::create(&library_json_file);
        match file {
            Ok(file) => {
                let _ = serde_json::to_writer_pretty(file, &library);
                println!("Wrote to {}", &library_json_file.to_str().unwrap());
            },
            Err(error) => panic!("Problem creating config.json: {error:?}"),
        }
    }
}