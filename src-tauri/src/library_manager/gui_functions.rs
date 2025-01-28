use directories::ProjectDirs;
use std::fs;

use super::{library, video_element, LIBRARIES, update_library_entry};
use super::tmdb_api::get_movie_information_tmdb;
use super::json_parser::write_library;

#[tauri::command(rename_all = "snake_case")]
pub fn get_libraries_gui() -> Vec<library> {
    print!("Please write a better function!");
    return LIBRARIES.lock().unwrap().clone();
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_library_videos(library_id: &str) -> Vec<video_element> {
    if let Some(library) = LIBRARIES.lock().unwrap()
        .iter_mut()
        .find(|library| library.id.to_string() == library_id) {
            return library.video_files.clone();
        }
    println!("Library {} not found!", library_id);
    vec![]
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_library_entry_from_gui(library_id: &str, updated_element: video_element) {
    for library in LIBRARIES.lock().unwrap().iter_mut() {
        if library.id.to_string() == library_id {
            match update_library_entry(library, updated_element) {
                Ok(()) => {
                    write_library(library);
                    return;
                },
                Err(str) => {
                    println!("Error when updating Library: {}", str); 
                    return;
                }
            }
        }
    }
    println!("Library {} not found!", library_id);
}



#[tauri::command(rename_all = "snake_case")]
pub async fn call_public() {

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