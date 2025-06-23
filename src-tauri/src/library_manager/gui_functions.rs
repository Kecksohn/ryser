use directories::ProjectDirs;
use std::{default, fs};

use super::json_parser::write_library;
use super::tmdb_api::{
    get_additional_covers, get_movie_details_for_video_element, get_tmdb_search_as_video_elements,
};
use super::{
    add_library, delete_library, update_library_entry, Library, LibraryPath, VideoElement,
    LIBRARIES,
};

use super::tmdb_api::json_structs::*;

use super::file_manager::file_utils::create_valid_filename;
use super::utils::*;

use crate::notifications::show_msg_gui;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_available_libraries() -> Vec<(String, String)> {
    let mut available_libraries: Vec<(String, String)> = vec![];
    for library in LIBRARIES.lock().await.iter() {
        available_libraries.push((library.id.clone(), library.name.clone()));
    }

    available_libraries
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_library(
    name: &str,
    paths: Vec<LibraryPath>,
    allow_duplicate_name: bool,
) -> Result<(), String> {
    let mut new_library_id = create_valid_filename(name, Some(true), Some(true));

    // Check if name already exists and if, ask user for confirmation
    if !allow_duplicate_name {
        let current_library_names = get_all_library_names().await;
        for library_name in current_library_names {
            if library_name == name {
                return Err("duplicate_name".to_owned());
            }
        }
    }

    // If ID is already taken, add incremented numbers until a new unique id is found
    let current_library_ids = get_all_library_ids().await;
    let mut i = 2;

    loop {
        if !current_library_ids.contains(&new_library_id) {
            break;
        }

        // Remove the last added i, if any
        if i > 2 {
            for j in 0..(i / 10) + 1 {
                new_library_id.pop();
            }
        }

        let i_str = i.to_string();

        // Check if we would be over the max foldername chars
        if new_library_id.chars().count() + i_str.chars().count() > 255 {
            for j in 0..(new_library_id.chars().count() + i_str.chars().count() - 255) {
                new_library_id.pop();
            } // Yes this is dumb code i dont care this will never happen
        }

        new_library_id += &i_str;
        i += 1;
    }

    let new_lib = Library {
        id: new_library_id,
        name: name.to_owned(),
        library_paths: paths,
        video_files: vec![],
        child_libraries: vec![],
    };
    add_library(new_lib).await;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_library_gui(library_id: &str) -> Result<(), String> {
    delete_library(library_id)
        .await
        .map_err(|e| format!("Could not delete library: {}", e))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_library_name(library_id: &str) -> Result<String, String> {
    if let Some(library) = LIBRARIES
        .lock()
        .await
        .iter_mut()
        .find(|library| library.id == library_id)
    {
        Ok(library.name.clone())
    } else {
        Err(format!("Library {} not found!", library_id))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_library_videos(library_id: &str) -> Result<Vec<VideoElement>, String> {
    if let Some(library) = LIBRARIES
        .lock()
        .await
        .iter_mut()
        .find(|library| library.id == library_id)
    {
        Ok(library.video_files.clone())
    } else {
        Err(format!("Library {} not found!", library_id))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_library_entry_from_gui(
    library_id: &str,
    updated_element: VideoElement,
) -> Result<(), String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            match update_library_entry(library, updated_element) {
                Ok(()) => {
                    write_library(library);
                    return Ok(());
                }
                Err(str) => {
                    return Err(format!("Error when updating Library: {}", str));
                }
            }
        }
    }

    Err(format!("Library {} not found!", library_id))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_tmdb_from_gui(search_title: &str) -> Result<Vec<VideoElement>, String> {
    let mut query_result_elements: Vec<VideoElement> =
        get_tmdb_search_as_video_elements(search_title)
            .await
            .map_err(|e| format!("Could not get TMDB search results: {}", e))?;

    // TODO: Return in-between results already and make this async
    for element in query_result_elements.iter_mut() {
        get_movie_details_for_video_element(element, None)
            .await
            .map_err(|e| format!("Could not get TMDB movie details: {}", e))?;
    }

    Ok(query_result_elements)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_covers_from_tmdb(
    tmdb_id: usize,
    sort_by_languages_in_iso_639_1: Option<Vec<String>>,
    filter_other_languages: Option<bool>,
) -> Result<Vec<String>, String> {
    get_additional_covers(
        tmdb_id,
        sort_by_languages_in_iso_639_1,
        filter_other_languages,
    )
    .await
    .map_err(|e| format!("Could not get TMDB covers: {}", e))
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn rescan_all_libraries_gui() {
    super::rescan_all_libraries().await;
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn rescan_library_by_id_gui(lib_id: &str) -> Result<(), String> {
    return super::rescan_library_by_id(lib_id)
        .await
        .map_err(|e| format!("{}", e));
}
