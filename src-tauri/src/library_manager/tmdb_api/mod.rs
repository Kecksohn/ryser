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
    if api_token.len() == 0 {
        return Err(String::from("There is no API Read Access Token! Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"));
    }

    let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::https("http://proxy.fcp-intranet.at:8080")
                    .map_err(|e| format!("Failed to build proxy"))?)
                .build()
                .map_err(|e| format!("Failed to build reqwest client: {}", e))?;
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
    let response = call_tmdb_api(&client, test_authentification_url, api_token).await;

    match response {
        Ok(res) => Ok(res.json::<test_authentification>().await?.success),
        Err(err) => Err(err),
    }
}


pub(crate) async fn search_tmdb
(
    client: &Client,
    api_token: &str,
    movietitle: &str,
    year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>
)
    -> Result<search_movie_res, String>
{
    let include_adult = include_adult.unwrap_or(false);
    let page = page.unwrap_or(1);

    let search_movie_url = {
        let mut url = String::from("https://api.themoviedb.org/3/search/movie?query=");
        url.push_str(&movietitle);
        url.push_str("&include_adult=");
        url.push_str(match include_adult {true => "true", false => "false"} );
        url.push_str("&page=");
        url.push_str(page.to_string().as_str());

        if year.is_some() {
            url.push_str("&year=");
            url.push_str(year.unwrap().to_string().as_str());
        }

        if language.is_some() {
            url.push_str("&language=");
            url.push_str(language.unwrap().to_string().as_str());
        }

        url
    };

    let response = call_tmdb_api(&client, &search_movie_url, api_token).await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;

    response.json().await
        .map_err(|e| format!("Could not parse Movie Search JSON: {}", e))
}

pub(crate) async fn create_client_and_search_tmdb
(
    movietitle: &str,
    year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>
)
    -> Result<search_movie_res, String>
{
    let (client, api_token) = create_client().await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;
    search_tmdb(&client, &api_token, movietitle, year, language, include_adult, page).await
}

pub(crate) async fn get_movie_details(client: &Client, movie_id: usize, api_token: &str) -> Result<MovieDetails, String> {
    let get_movie_details_url = "https://api.themoviedb.org/3/movie/".to_owned() + movie_id.to_string().as_str() + "?language=en-US";

    let response = call_tmdb_api(&client, &get_movie_details_url, api_token).await
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

                print!("{:#?}", movie_details);

                fill_video_element_with_tmdb_result(video_element, best_match, None);
                println!("{}", video_element);
                return Ok(()); // TODO: Remove
            }
            Err(str) => {return Err(format!("Error when calling TMDB: {}", str))}
        }
    }

    Ok(())
}


pub(crate) fn fill_video_element_with_tmdb_result
(
    video_element: &mut VideoElement,
    tmdb_result: &search_movie_result_element,
    overwrite: Option<bool>,
)
{
    let overwrite = overwrite.unwrap_or(false);

    video_element.tmdb_id = tmdb_result.id;
    if video_element.title.is_none() || overwrite { video_element.title = tmdb_result.title.clone(); }
    if video_element.poster_path.is_none() || overwrite {
        video_element.poster_path = match &tmdb_result.poster_path {
            Some(path) => Some("https://image.tmdb.org/t/p/original/".to_owned() + path),
            None => None,
        }
    }

    /*
    pub(crate) adult: Option<bool>,
    pub(crate) backdrop_path: Option<String>,
    pub(crate) genre_ids: Option<Vec<usize>>,
    pub(crate) id: Option<usize>,
    pub(crate) original_language: Option<String>,
    pub(crate) original_title: Option<String>,


     */
}