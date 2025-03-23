#![allow(dead_code)]

mod api_token;
pub(super) mod json_structs;
mod search_name_creator;

use serde::Deserialize;
use tauri_plugin_http::reqwest::header::USER_AGENT;
use tauri_plugin_http::reqwest::{self, Client, Error, Response};

use super::library;
use api_token::get_api_token;
use json_structs::*;
use search_name_creator::get_movie_title_and_year_from_filename;


async fn create_client(api_token: &str) -> Result<Client, String>
{
    let api_token = get_api_token();
    if api_token.len() == 0 {
        return Err(String::from("There is no API Read Access Token! Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"));
    }

    let client = reqwest::Client::new();
    match is_tmdb_api_read_access_token_valid(&client, api_token).await {
        Ok(valid) => {
            if valid
                { Ok(client) }
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

pub(crate) async fn search_tmdb(movietitle: &str, ) -> Result<search_movie_res, String> {

    let api_token = get_api_token();
    let client = create_client(get_api_token()).await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;

    let search_movie_url =
        format!("https://api.themoviedb.org/3/search/movie?query={query}&include_adult={include_adult}&language={language}&page={page}",
                query = movietitle,
                include_adult = "false",
                language = "en-US",
                page = "1"
                //year = "1982",
            );

    let response = call_tmdb_api(&client, &search_movie_url, api_token).await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;

    response.json().await
        .map_err(|e| format!("Could not parse json: {}", e))
}



pub(super) async fn parse_library_tmdb(library: &mut library, reparse_all: Option<bool>) -> Result<(), String>
{
    let reparse_all: bool = reparse_all.unwrap_or(false);

    let api_token = get_api_token();
    let client = create_client(get_api_token()).await
        .map_err(|e| format!("Could not connect to TMDB: {}", e))?;

    for video_element in library.video_files.iter_mut() {
        let filename = &video_element.filepath;

        // TODO: Check if filename hints at this being a TV episode

        let (possible_title, year) = get_movie_title_and_year_from_filename(filename);

        // TODO: Call TMDB
    }

    Ok(())
}