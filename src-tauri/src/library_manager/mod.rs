mod file_manager;
pub(crate) mod gui_functions;
mod json_parser;
mod tmdb_api;
mod utils;
mod video_element;

use crate::Error;

use std::cmp::Ordering;
use std::{fs, path::*, vec};
use directories::ProjectDirs;

use tauri::async_runtime;
use serde::Deserialize;

use video_element::VideoElement;

use file_manager::*;
use file_manager::directory_utils::*;

use json_parser::*;
use tmdb_api::*;


#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct library_path {
    path: String,
    include_subdirectories: bool,
} 

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct library {
    id: String,
    name: String,
    library_paths: Vec<library_path>,
    video_files: Vec<VideoElement>,
    child_libraries: Vec<library>,
}

use std::sync::Mutex;
static LIBRARIES: Mutex<Vec<library>> = Mutex::new(Vec::new());

pub(crate) fn get_libraries_path() -> PathBuf {
    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        proj_dir.data_local_dir().to_path_buf()
    }
    else { panic!("Project Dirs failed!"); }
}

pub(crate) fn load_all_libraries() {
    let libraries_folder = get_libraries_path();
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
    LIBRARIES.lock().unwrap().sort_by_key(|lib| lib.name.clone());
}

pub(crate) fn set_libraries(libraries: Vec<library>) {
    *LIBRARIES.lock().unwrap() = libraries;
}

pub(crate) fn get_library_index_by_id(lib_id: &str) -> Result<usize, Error> {
    LIBRARIES.lock().unwrap().iter()
        .position(|lib| lib.id == lib_id)
        .ok_or_else(|| format!("Could not find library with id {}", lib_id).into())
}

pub(crate) fn add_library(mut lib: library) {
    // TODO THINK: Maybe this should be async after the library is added?
    let mut video_files_in_library_paths: Vec<String> = vec![];
    match get_all_video_filepaths(&lib, &mut video_files_in_library_paths) {
        Ok(()) => {},
        Err(msg) => println!("{}", msg), // TODO: Show this on GUI
    }

    for video_filepath in video_files_in_library_paths.iter() {
        println!("{}", video_filepath);
        let video_file = create_video_element_from_file(video_filepath);
        lib.video_files.push(video_file);
    }

    write_library(&lib);
    LIBRARIES.lock().unwrap().push(lib);
    LIBRARIES.lock().unwrap().sort_by_key(|lib| lib.name.clone());
}

pub(crate) fn delete_library(lib_id: &str) -> Result<(), Error> {
    
    let mut libraries = LIBRARIES.lock().unwrap();

    let original_length = libraries.len();
    libraries.retain(|lib| lib.id != lib_id);
    let elements_removed = original_length - libraries.len();

    if elements_removed == 0 {
        return Err(format!("Did not find Library ID '{}', no removal occurred!", lib_id).into());
    } 

    let libraries_folder = get_libraries_path();
    let library_folder = libraries_folder.join(lib_id);

    if !library_folder.exists() {
        return Err(format!("Did not find library folder at path '{}'", library_folder.to_str().unwrap_or("!Path Unwrap Failed!")).into());
    }

    fs::remove_dir_all(&library_folder)
        .map_err(|e| format!("Could not delete '{}': {}", library_folder.to_str().unwrap_or("!Path Unwrap Failed!"), e))?;

    Ok(())
}


fn get_all_video_filepaths(lib: &library, video_files_in_library_paths: &mut Vec<String>) -> Result<(), Error> {
    
    let mut error_message: String = "".to_owned();
    for library_path in lib.library_paths.iter() {

        let mut filepaths: Vec<PathBuf> = vec![];
        if library_path.include_subdirectories {
            match get_files_in_folder_and_subdirectories(Path::new(&library_path.path), &mut filepaths)
            {
                Ok(()) => (),
                Err(e) => error_message = format!("{}Could not parse {}: {}\n", error_message, library_path.path, e),
            }
        }
        else {
            match fs::read_dir(&library_path.path) {
                Ok(f) => filepaths = f.filter_map(|entry| entry.ok().map(|e| e.path()))
                                    .collect(),
                Err(e) => error_message = format!("{}Could not parse {}: {}\n", error_message, library_path.path, e),
            }
        }

        for filepath in filepaths {
            if filepath.metadata().unwrap().is_file() && is_video_file(&filepath) {
                let Some(filepath_str) = filepath.to_str() 
                else {
                    error_message += "Could not extract string value from path\n";
                    continue;
                };

                video_files_in_library_paths.push(filepath_str.to_string());
            }
        }
    }

    video_files_in_library_paths.sort();
    
    if error_message.is_empty() {
        Ok(())
    }
    else {
        Err(error_message.into())
    }
}


#[tauri::command(rename_all = "snake_case")]
pub(crate) fn rescan_all_libraries() {
    for lib in LIBRARIES.lock().unwrap().iter_mut() {
        rescan_library(lib);
    }
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn rescan_library_by_id(lib_id: &str) -> Result<(), String> {
    let index = get_library_index_by_id(lib_id)
        .map_err(|e| format!("Could not get library index: {}", e))?;

    rescan_library(&mut LIBRARIES.lock().unwrap()[index]);
    Ok(())
}

//  Compares Files present in library paths with data in json
//  Tries to match files whose filenames have simply changed (!TODO: MD5 sum or simply length?)
pub(crate) fn rescan_library(lib: &mut library) {
        
    // Since we want to optimize matching we first get all files and then compare both sorted lists simultaneously
    let mut video_files_in_library_paths: Vec<String> = vec![];

    // Get all video files in all library_paths
    match get_all_video_filepaths(lib, &mut video_files_in_library_paths) {
        Ok(()) => {},
        Err(msg) => println!("{}", msg), // TODO: Show this on GUI
    }

    let mut new_filepath_indices: Vec<usize> = vec![];
    let mut missing_video_files_indices: Vec<usize> = vec![];

    let mut current_library_match_start_index: usize = 0;

    //  Walk through video_files stored in library.json currently
    for (filepath_index, filepath_str) in video_files_in_library_paths.iter().enumerate() 
    {
        if current_library_match_start_index < lib.video_files.len() 
        {
            for index in current_library_match_start_index..lib.video_files.len() 
            {
                match &lib.video_files[index].filepath.cmp(filepath_str) {
                    Ordering::Equal => {
                        // Match, do nothing
                        current_library_match_start_index = index + 1;
                        break;
                    },
                    Ordering::Greater => {
                        // Compared File should come before json entry but isn't in it -> New Entry
                        new_filepath_indices.push(filepath_index);
                        current_library_match_start_index = index;
                        break;
                    },
                    Ordering::Less => {
                        // JSON Entry is not in directory anymore -> Removed Entry, but keep checking next json entry for file match (no break)
                        missing_video_files_indices.push(index);
                    },
                }
            }
        }
        else {
            //  We are out of elements in the json, all remaining files are new
            new_filepath_indices.push(filepath_index);
        }
    }
    // We are out of files in the paths, all remaining json entries have been removed
    for library_index in current_library_match_start_index..lib.video_files.len() {
        missing_video_files_indices.push(library_index);
    }

    if !missing_video_files_indices.is_empty() || !new_filepath_indices.is_empty() 
    {
        println!("Library: {}", lib.id);
        println!("Removed Files: ({}):", missing_video_files_indices.len());
        for index in missing_video_files_indices.iter().rev() {
            println!("{}", lib.video_files[*index].filepath);
            lib.video_files.remove(*index);
        }
        println!("New Files ({}): ", new_filepath_indices.len());
        for filepath_index in new_filepath_indices.iter() {
            println!("{}", &video_files_in_library_paths[*filepath_index]);
            let video_file =
                create_video_element_from_file(&video_files_in_library_paths[*filepath_index]);
                lib.video_files.push(video_file);
        }

        lib.video_files.sort_by(|d1, d2| d1.filepath.cmp(&d2.filepath));
        println!(
            "Before: {}, Now: {}",
            lib.video_files.len() - new_filepath_indices.len()
                + missing_video_files_indices.len(),
            lib.video_files.len()
        );
        write_library(lib);
    }
}

pub(crate) fn update_library_entry(
    library: &mut library,
    updated_element: VideoElement,
) -> Result<(), Error> {
    let Some(proj_dir) = ProjectDirs::from("", "", "ryser") else {
        return Err("Could not get project dirs".into());
    };
    let library_folder = proj_dir.data_local_dir().join(&library.id);
    if !library_folder.exists() {
        return Err("Could not find library".into());
    }
    for old_element in library.video_files.iter_mut() {
        if old_element.filepath == updated_element.filepath {
            *old_element = updated_element;
            return Ok(());
        }
    }

    Err(format!("Could not find '{}' in '{}'", updated_element.filepath, library.id).into())
}

pub(crate) fn update_library_entry_by_index(
    library: &mut library,
    updated_element: VideoElement,
    index: usize,
) -> Result<(), Error> {
    if index >= library.video_files.len() {
        return Err(format!(
            "Index {} out of range of library elements ({})",
            index,
            library.video_files.len()
        ).into());
    }
    library.video_files[index] = updated_element;
    Ok(())
}


#[tauri::command]
pub(crate) fn update_all_libraries_with_tmdb(reparse_all: Option<bool>) -> Result<(), String> {
    for lib in LIBRARIES.lock().unwrap().iter_mut() { // TODO
        async_runtime::block_on(async {
            parse_library_tmdb(lib, reparse_all).await
                .map_err(|e| format!("Could not parse Library with TMDB: {}", e))
        })?;
        write_library(lib);
    }

    Ok(())
}