#![allow(dead_code)]

mod api_token;
pub(super) mod json_structs;
mod search_name_creator;

use serde::Deserialize;
use tauri_plugin_http::reqwest::header::USER_AGENT;
use tauri_plugin_http::reqwest::{self, Client, Error, Response};

use super::{library, VideoElement};
use api_token::get_api_token;
use json_structs::*;
use search_name_creator::get_movie_title_and_year_from_filename;


async fn create_client() -> Result<(Client, String), String>
{
    let api_token = get_api_token();
    if api_token.is_empty() {
        return Err(String::from("There is no API Read Access Token! Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"));
    }

    let client = reqwest::Client::new();
    match is_tmdb_api_read_access_token_valid(&client, api_token).await {
        Ok(valid) => {
            if valid
                { Ok((client, api_token.to_string())) }
            else
                { Err(String::from("API Read Access Token not valid. Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"))}
        }
        Err(str) => { Err(format!("Could not connect to TMDB: {}", str)) }
    }

}

async fn call_tmdb_api(client: &Client, api_url: &str, api_token: &str) -> Result<Response, Error> {
    client
        .get(api_url)
        .header(USER_AGENT, "rust-web-api-client") // ? api requires a user-agent header
        .header("accept", "application/json")
        .header("Authorization", "Bearer ".to_owned() + api_token)
        .send()
        .await
}

pub(crate) async fn is_tmdb_api_read_access_token_valid(client: &Client, api_token: &str, ) -> Result<bool, Error> {
    let test_authentification_url = "https://api.themoviedb.org/3/authentication";
    let response = call_tmdb_api(client, test_authentification_url, api_token).await;

    match response {
        Ok(res) => Ok(res.json::<TMDBTestAuthentification>().await?.success),
        Err(err) => Err(err),
    }
}


async fn search_tmdb
(
    client: &Client,
    api_token: &str,
    movietitle: &str,
    year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>
)
    -> Result<TMDBSearchMovieResult, String>
{
    let include_adult = include_adult.unwrap_or(false);
    let page = page.unwrap_or(1);

    let search_movie_url = {
        let mut url = String::from("https://api.themoviedb.org/3/search/movie?query=");
        url.push_str(movietitle);
        url.push_str("&include_adult=");
        url.push_str(match include_adult {true => "true", false => "false"} );
        url.push_str("&page=");
        url.push_str(page.to_string().as_str());

        if let Some(year) = year {
            url.push_str("&year=");
            url.push_str(year.to_string().as_str());
        }

        if let Some(language) = language {
            url.push_str("&language=");
            url.push_str(language.to_string().as_str());
        }

        url
    };

    let response = call_tmdb_api(client, &search_movie_url, api_token).await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;

    response.json().await
        .map_err(|e| format!("Could not parse Movie Search JSON: {}", e))
}

async fn create_client_and_search_tmdb
(
    movietitle: &str,
    year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>
)
    -> Result<TMDBSearchMovieResult, String>
{
    let (client, api_token) = create_client().await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;
    search_tmdb(&client, &api_token, movietitle, year, language, include_adult, page).await
}

pub(crate) async fn get_tmdb_search_as_video_elements(search_title: &str) -> Result<Vec<VideoElement>, String> {
    let query_result_object = match create_client_and_search_tmdb(search_title, None, None, None, None).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Error trying to call tmdb database: {}", e))
    };

    let mut query_result_elements: Vec<VideoElement> = vec![];

    for query_result in query_result_object.results.iter() {
        
        let mut result_element = VideoElement {
            filepath: "".to_owned(),
            watched: false,
            ..Default::default()
        };

        fill_video_element_with_search_result(&mut result_element, query_result, None);
        query_result_elements.push(result_element);
    }

    Ok(query_result_elements)
}

async fn get_movie_details(client: &Client, movie_id: usize, api_token: &str) -> Result<TMDBMovieDetails, String> {
    let get_movie_details_url = "https://api.themoviedb.org/3/movie/".to_owned() + movie_id.to_string().as_str() + "?append_to_response=credits";

    let response = call_tmdb_api(client, &get_movie_details_url, api_token).await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;

    response.json().await
        .map_err(|e| format!("Could not parse Movie Details JSON: {}", e))
}



pub(super) async fn parse_library_tmdb(library: &mut library, reparse_all: Option<bool>) -> Result<(), String>
{
    let reparse_all: bool = reparse_all.unwrap_or(false);

    let (client, api_token) = create_client().await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;

    for video_element in library.video_files.iter_mut() {
        if video_element.tmdb_id.is_none() {
            let filename = &video_element.filepath;

            // TODO: Check if filename hints at this being a TV episode

            let (possible_title, year) = get_movie_title_and_year_from_filename(filename);

            match search_tmdb( &client, &api_token, possible_title.as_str(), year, None, None, None).await {
                Ok(search_tmdb_result) => {
                    if search_tmdb_result.results.is_empty() {
                        // TODO: Adjust filename by cutting one word from the end
                        continue;
                    }

                    let best_match = &search_tmdb_result.results[0];
                    if best_match.id.is_none() {
                        continue;
                    }

                    let movie_details = get_movie_details(&client, best_match.id.unwrap(), &api_token).await
                        .map_err(|e| format!("Error when getting Movie Details: {}", e))?;

                    fill_video_element_with_movie_details(video_element, &movie_details, None);
                    if video_element.release_date.is_none() && year.is_some() {
                        video_element.release_date = Some(year.unwrap().to_string());
                    }
                    println!("Updated: {} [{}] ({}) by {}", video_element.original_title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            &video_element.title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            video_element.release_date.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            &video_element.director.as_ref().unwrap_or(&"!MISSING!".to_string()))
                }
                Err(str) => {return Err(format!("Error when calling TMDB: {}", str))}
            }
        }
    }

    Ok(())
}


fn fill_video_element_with_search_result(video_element: &mut VideoElement, movie_search_result: &TMDBMovie, overwrite: Option<bool>)
{
    let overwrite = overwrite.unwrap_or(false);

    video_element.tmdb_id = movie_search_result.id;
    if overwrite || video_element.title.is_none() 
        { video_element.title = movie_search_result.title.clone(); }
    if overwrite || video_element.original_title.is_none()
        { video_element.original_title = movie_search_result.original_title.clone(); }
    if overwrite || video_element.tmdb_language.is_none() 
        { video_element.tmdb_language = movie_search_result.original_language.clone(); }
    if overwrite || video_element.genre_ids.is_none()
        { video_element.genre_ids = movie_search_result.genre_ids.clone(); }
    if overwrite || video_element.overview.is_none()
        { video_element.overview = movie_search_result.overview.clone(); }
    if overwrite || video_element.release_date.is_none()
        { video_element.release_date = movie_search_result.release_date.clone(); }

    if overwrite || video_element.poster_path.is_none() {
        video_element.poster_path = movie_search_result.poster_path.as_ref().map(|path| "https://image.tmdb.org/t/p/original/".to_owned() + path);
    }
    if overwrite || video_element.backdrop_path.is_none() {
        video_element.backdrop_path = movie_search_result.backdrop_path.as_ref().map(|path| "https://image.tmdb.org/t/p/original/".to_owned() + path);
    }

    /* Not used:
    pub adult: Option<bool>,
    pub popularity: Option<f32>,
    pub video: Option<bool>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<usize>,
     */
}

fn fill_video_element_with_movie_details(video_element: &mut VideoElement, movie_details: &TMDBMovieDetails, overwrite: Option<bool>)
{
    fill_video_element_with_search_result(video_element, &movie_details.tmdb_movie, overwrite);
    let overwrite = overwrite.unwrap_or(false);

    if overwrite || video_element.tagline.is_none() 
        { video_element.tagline = movie_details.tagline.clone(); }
        
    if let Some(credits) = &movie_details.credits {
        if let Some(crew) = &credits.crew {
            for crew_member in crew {
                if let Some(job) = &crew_member.job {
                    if job == "Director" {
                        video_element.director = crew_member.tmdb_person.name.clone();
                    }
                }
            }
        }
    }
    
    /* Not used:
    pub belongs_to_collection: Option<TMDBCollection>,
    pub budget: Option<usize>,
    pub genres: Option<Vec<TMDBGenre>>,
    pub homepage: Option<String>,
    pub imdb_id: Option<String>,
    pub origin_country: Option<Vec<String>>,
    pub production_companies: Option<Vec<TMDBProductionCompanies>>,
    pub production_countries: Option<Vec<TMDBProductionCountries>>,
    pub revenue: Option<usize>,
    pub runtime: Option<usize>,
    pub spoken_languages: Option<Vec<TMDBSpokenLanguage>>,
    pub status: Option<String>,
    
    Rest of crew/cast
    */

}