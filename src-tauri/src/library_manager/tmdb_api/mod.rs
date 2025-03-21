#![allow(dead_code)]

mod api_token;
pub(super) mod json_structs;

use serde::Deserialize;
use tauri_plugin_http::reqwest::header::USER_AGENT;
use tauri_plugin_http::reqwest::{self, Client, Error, Response};

use api_token::get_api_token;
use json_structs::*;

async fn call_tmdb_api(client: &Client, api_url: &str, api_token: &str) -> Result<Response, Error> {
    client
        .get(api_url)
        .header(USER_AGENT, "rust-web-api-client") // ? api requires a user-agent header
        .header("accept", "application/json")
        .header("Authorization", "Bearer ".to_owned() + api_token)
        .send()
        .await
}

pub(crate) async fn is_tmdb_api_read_access_token_valid(
    client: &Client,
    api_token: &str,
) -> Result<bool, Error> {
    let test_authentification_url = "https://api.themoviedb.org/3/authentication";
    let response = call_tmdb_api(&client, test_authentification_url, api_token).await;

    match response {
        Ok(res) => Ok(res.json::<test_authentification>().await?.success),
        Err(err) => Err(err),
    }
}

pub(crate) async fn search_tmdb(
    movietitle: &str,
) -> Result<search_movie_res, Error> {
    
    let api_token = get_api_token();
    if api_token.len() == 0 {
        panic!(
            "There is no API Read Access Token! Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"
        );
    }
    
    let client = reqwest::Client::new();

    if is_tmdb_api_read_access_token_valid(&client, api_token).await? 
    {
        let search_movie_url = 
            format!("https://api.themoviedb.org/3/search/movie?query={query}&include_adult={include_adult}&language={language}&page={page}",
                    query = movietitle,
                    include_adult = "false",
                    language = "en-US",
                    page = "1"
                    //year = "1982",
                );

        let response = call_tmdb_api(&client, &search_movie_url, api_token).await?;
        return response.json().await;
    } 
    else {
        // TODO: Send to GUI
        panic!(
            "API Read Access Token not valid. Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"
        );
    }
}


