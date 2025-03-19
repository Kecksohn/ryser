use directories::ProjectDirs;
use std::{default, fs};

use super::json_parser::write_library;
use super::tmdb_api::get_movie_information_tmdb;
use super::{library, library_path, video_element, 
            add_library, update_library_entry, 
            LIBRARIES};

use super::tmdb_api::json_structs::*;

use super::file_manager::file_utils::create_valid_filename;
use super::utils::*;


#[tauri::command(rename_all = "snake_case")]
pub fn get_available_libraries() -> Vec<(String, String)> {
    let mut available_libraries: Vec<(String, String)> = vec![];
    for library in LIBRARIES.lock().unwrap().iter() {
        available_libraries.push((library.id.clone(), library.name.clone()));
    }
    return available_libraries;
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_library(name: &str, paths: Vec<library_path>, allow_duplicate_name: bool) -> Result<(), String> {
        
    let mut new_library_id = create_valid_filename(name, Some(true), Some(true));

    // Check if name already exists and if, ask user for confirmation
    if !allow_duplicate_name {
        let current_library_names = get_all_library_names();
        for library_name in current_library_names
        {
            if library_name == name {
                return Err("duplicate_name".to_owned());
            }
        }
    }

    // If ID is already taken, add incremented numbers until a new unique id is found
    let current_library_ids = get_all_library_ids();
    let mut i = 2;

    loop
    {
        if !current_library_ids.contains(&new_library_id) {
            break;
        }

        // Remove the last added i, if any
        if i > 2 {
            for j in 0..(i/10)+1 {
                new_library_id.pop();
            }
        }

        let i_str = i.to_string();
        
        // Check if we would be over the max foldername chars 
        if new_library_id.chars().count() + i_str.chars().count() > 255 {
            for j in 0..(new_library_id.chars().count() + i_str.chars().count() - 255) { new_library_id.pop(); } // Yes this is dumb code i dont care this will never happen
        }

        new_library_id += &i_str;
        i+=1;
    }

    let new_lib = library {
        id: new_library_id,
        name: name.to_owned(),
        library_paths: paths,
        video_files: vec![],
        child_libraries: vec![],
    };
    add_library(new_lib);
    Ok(())
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_library_videos(library_id: &str) -> Vec<video_element> {
    if let Some(library) = LIBRARIES
        .lock()
        .unwrap()
        .iter_mut()
        .find(|library| library.id.to_string() == library_id)
    {
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
                }
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
                Some(identifier) => {
                    Some("https://image.tmdb.org/t/p/original/".to_owned() + &identifier)
                }
                None => None,
            },
            title: query_result.title.clone(),
            ..Default::default()
        };
        query_result_elements.push(result_element);
    }

    println!(
        "{} elements found for {}",
        query_result_elements.len(),
        search_title
    );

    return Ok(query_result_elements);
}
