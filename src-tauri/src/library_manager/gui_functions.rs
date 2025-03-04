use directories::ProjectDirs;
use std::{default, fs};


use super::{library, video_element, LIBRARIES, update_library_entry};
use super::tmdb_api::get_movie_information_tmdb;
use super::json_parser::write_library;

use super::tmdb_api::json_structs::*;


#[tauri::command(rename_all = "snake_case")]
pub fn get_available_libraries() -> Vec<(String, String)> {
    let mut available_libraries : Vec<(String, String)> = vec![];
    for library in LIBRARIES.lock().unwrap().iter() {
        available_libraries.push((library.id.clone(), library.name.clone()));
    }
    return available_libraries;
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
pub async fn search_tmdb_from_gui(search_title: &str) -> Result<Vec<video_element>, String> {
    let Ok(query_result_object) = get_movie_information_tmdb(search_title).await else {
        return Err("Error trying to call tmdb database!".to_owned());
    };
    
    println!("{}", query_result_object.results[0].title.clone().unwrap());

    let mut query_result_elements: Vec<video_element> = vec![];

    for query_result in query_result_object.results.iter() {
        let result_element = video_element {
            filepath: "".to_owned(),
            watched: false,
            parsed: true,
            poster_path: match query_result.poster_path.as_ref() {
                Some(identifier) => Some("https://image.tmdb.org/t/p/original/".to_owned() + &identifier),
                None => None,
            },
            title: query_result.title.clone(),
            ..Default::default()
        };
        query_result_elements.push(result_element);
    }

    println!("{} elements found for {}", query_result_elements.len(), search_title);

    return Ok(query_result_elements);
}
