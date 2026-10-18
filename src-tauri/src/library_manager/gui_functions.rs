use directories::ProjectDirs;
use std::{default, fs};

use super::json_parser::write_library;
use std::collections::BTreeMap;

use super::tmdb_api::{get_tmdb_search_as_films, get_tmdb_search_as_shows, get_movie_details_for_film, get_additional_covers,
                      add_tmdb_show, rematch_show as rematch_show_on_tmdb, set_media_with_lookup};
use super::{Library, LibraryPath, VideoElement, FilterPreferences,
            FilmMeta, Media, TvShow,
            add_library, delete_library, drop_shows_without_episodes,
            LIBRARIES, LIBRARY_SCHEMA_VERSION};

use super::tmdb_api::json_structs::*;

use super::file_manager::file_utils::{create_valid_filename, reveal_file_in_file_manager};
use super::file_manager::refresh_file_metadata;
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
pub async fn create_library(name: &str, paths: Vec<LibraryPath>, allow_duplicate_name: bool) -> Result<(), String> {
        
    let mut new_library_id = create_valid_filename(name, Some(true), Some(true));

    // Check if name already exists and if, ask user for confirmation
    if !allow_duplicate_name {
        let current_library_names = get_all_library_names().await;
        for library_name in current_library_names
        {
            if library_name == name {
                return Err("duplicate_name".to_owned());
            }
        }
    }

    // If ID is already taken, add incremented numbers until a new unique id is found
    let current_library_ids = get_all_library_ids().await;
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

    let new_lib = Library {
        id: new_library_id,
        name: name.to_owned(),
        library_paths: paths,
        video_files: vec![],
        tv_shows: BTreeMap::new(),
        schema_version: LIBRARY_SCHEMA_VERSION,
        sort_preference: "title".to_string(),
        filter_preferences: FilterPreferences::default(),
    };
    add_library(new_lib).await;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_library_gui(library_id: &str) -> Result<(), String> {
    delete_library(library_id).await
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
    }
    else {
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
    }
    else {
        Err(format!("Library {} not found!", library_id))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_library_shows(library_id: &str) -> Result<BTreeMap<u32, TvShow>, String> {
    if let Some(library) = LIBRARIES
        .lock()
        .await
        .iter()
        .find(|library| library.id == library_id)
    {
        Ok(library.tv_shows.clone())
    }
    else {
        Err(format!("Library {} not found!", library_id))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_watched(library_id: &str, filepaths: Vec<String>, watched: bool) -> Result<(), String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            for element in library.video_files.iter_mut() {
                if filepaths.contains(&element.filepath) {
                    element.watched = watched;
                }
            }
            write_library(library);
            return Ok(());
        }
    }

    Err(format!("Library {} not found!", library_id))
}

/// Replace the catalog metadata of one file with what the user chose.
#[tauri::command(rename_all = "snake_case")]
pub async fn set_media(library_id: &str, filepath: &str, media: Media) -> Result<(), String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            let Library { video_files, tv_shows, .. } = &mut *library;
            let Some(element) = video_files.iter_mut().find(|ve| ve.filepath == filepath)
            else {
                return Err(format!("Could not find '{}' in '{}'", filepath, library_id));
            };
            set_media_with_lookup(element, tv_shows, media).await
                .map_err(|e| format!("{} in '{}'", e, library_id))?;
            drop_shows_without_episodes(library);
            write_library(library);
            return Ok(());
        }
    }

    Err(format!("Library {} not found!", library_id))
}

fn edit_show(tv_shows: &mut BTreeMap<u32, TvShow>, show_id: u32, name: Option<String>, poster_path: Option<String>) -> Result<(), String> {
    let show = tv_shows.get_mut(&show_id).ok_or(format!("Show {} not found", show_id))?;
    show.name = name;
    show.poster_path = poster_path;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_show(library_id: &str, show_id: u32, name: Option<String>, poster_path: Option<String>) -> Result<(), String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            edit_show(&mut library.tv_shows, show_id, name, poster_path)?;
            write_library(library);
            return Ok(());
        }
    }
    Err(format!("Library {} not found!", library_id))
}

/// The library's show with this TMDB ID, created from TMDB when there is none. Returns its ID.
#[tauri::command(rename_all = "snake_case")]
pub async fn add_show_from_tmdb(library_id: &str, tmdb_id: usize) -> Result<u32, String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            let show_id = add_tmdb_show(&mut library.tv_shows, tmdb_id).await
                .map_err(|e| format!("Could not add TV show {}: {}", tmdb_id, e))?;
            write_library(library);
            return Ok(show_id);
        }
    }
    Err(format!("Library {} not found!", library_id))
}

/// Returns the ID of the show the episodes belong to afterwards (another one when merged).
#[tauri::command(rename_all = "snake_case")]
pub async fn rematch_show(library_id: &str, show_id: u32, tmdb_id: usize) -> Result<u32, String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            let Library { video_files, tv_shows, .. } = &mut *library;
            let new_show_id = rematch_show_on_tmdb(video_files, tv_shows, show_id, tmdb_id).await
                .map_err(|e| format!("Could not rematch show {}: {}", show_id, e))?;
            write_library(library);
            return Ok(new_show_id);
        }
    }
    Err(format!("Library {} not found!", library_id))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_tv_from_gui(search_title: &str) -> Result<Vec<TvShow>, String> {
    get_tmdb_search_as_shows(search_title).await
        .map_err(|e| format!("Could not get TMDB search results: {}", e))
}

/// Re-read what is derived from the file's content (duration, tracks) and
/// leave everything else (TMDB data, user edits) alone.
#[tauri::command(rename_all = "snake_case")]
pub async fn refresh_library_entry(library_id: &str, filepath: &str) -> Result<VideoElement, String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            let Some(element) = library.video_files.iter_mut().find(|ve| ve.filepath == filepath)
            else {
                return Err(format!("Could not find '{}' in '{}'", filepath, library_id));
            };
            if let Err(msg) = refresh_file_metadata(element) {
                let _ = show_msg_gui("Refresh failed", &msg, None, None);
                return Err(msg);
            }
            let refreshed_element = element.clone();
            write_library(library);
            return Ok(refreshed_element);
        }
    }

    Err(format!("Library {} not found!", library_id))
}


#[tauri::command(rename_all = "snake_case")]
pub async fn search_tmdb_from_gui(search_title: &str) -> Result<Vec<FilmMeta>, String> {
    let mut query_result_films: Vec<FilmMeta> =
        get_tmdb_search_as_films(search_title)
            .await
            .map_err(|e| format!("Could not get TMDB search results: {}", e))?;

    // TODO: Return in-between results already and make this async
    for film in query_result_films.iter_mut() {
        get_movie_details_for_film(film, None).await
            .map_err(|e| format!("Could not get TMDB movie details: {}", e))?;
    }

    Ok(query_result_films)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_covers_from_tmdb(
    tmdb_id: usize,
    is_tv: Option<bool>,
    sort_by_languages_in_iso_639_1: Option<Vec<String>>,
    filter_other_languages: Option<bool>
) -> Result<Vec<String>, String> {
    get_additional_covers(tmdb_id, is_tv.unwrap_or(false), sort_by_languages_in_iso_639_1, filter_other_languages).await
        .map_err(|e| format!("Could not get TMDB covers: {}", e))
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn rescan_all_libraries_gui() {
    super::rescan_all_libraries().await;
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn reparse_all_libraries_preserve_covers_gui() -> Result<(), String> {
    super::reparse_all_libraries_preserve_covers().await
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn rescan_library_by_id_gui(lib_id: &str) -> Result<(), String> {
    return super::rescan_library_by_id(lib_id).await
        .map_err(|e| format!("{}", e));
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_library_sort_preference(library_id: &str) -> Result<String, String> {
    if let Some(library) = LIBRARIES
        .lock()
        .await
        .iter()
        .find(|library| library.id == library_id)
    {
        Ok(library.sort_preference.clone())
    }
    else {
        Err(format!("Library {} not found!", library_id))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_library_sort_preference(library_id: &str, sort_preference: &str) -> Result<(), String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            library.sort_preference = sort_preference.to_string();
            write_library(library);
            return Ok(());
        }
    }
    Err(format!("Library {} not found!", library_id))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_library_filter_preferences(library_id: &str) -> Result<FilterPreferences, String> {
    if let Some(library) = LIBRARIES
        .lock()
        .await
        .iter()
        .find(|library| library.id == library_id)
    {
        Ok(library.filter_preferences.clone())
    }
    else {
        Err(format!("Library {} not found!", library_id))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_library_filter_preferences(library_id: &str, filter_preferences: FilterPreferences) -> Result<(), String> {
    for library in LIBRARIES.lock().await.iter_mut() {
        if library.id == library_id {
            library.filter_preferences = filter_preferences;
            write_library(library);
            return Ok(());
        }
    }
    Err(format!("Library {} not found!", library_id))
}

#[tauri::command(rename_all = "snake_case")]
pub fn show_in_explorer(filepath: &str) -> Result<(), String> {
    reveal_file_in_file_manager(filepath)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_show_replaces_name_and_poster_only() {
        let mut tv_shows = BTreeMap::from([
            (1, TvShow { id: 1, tmdb_id: Some(43879), name: Some("Braunschlag".to_owned()), overview: Some("O".to_owned()), ..Default::default() }),
            (2, TvShow { id: 2, name: Some("Other".to_owned()), ..Default::default() }),
        ]);

        edit_show(&mut tv_shows, 1, Some("Braunschlag (2012)".to_owned()), Some("/p.jpg".to_owned())).unwrap();
        assert_eq!(tv_shows[&1].name.as_deref(), Some("Braunschlag (2012)"));
        assert_eq!(tv_shows[&1].poster_path.as_deref(), Some("/p.jpg"));
        assert_eq!((tv_shows[&1].tmdb_id, tv_shows[&1].overview.as_deref()), (Some(43879), Some("O")));
        assert_eq!(tv_shows[&2].name.as_deref(), Some("Other"));

        assert!(edit_show(&mut tv_shows, 3, None, None).is_err());
    }
}
