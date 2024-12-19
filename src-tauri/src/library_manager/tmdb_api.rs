#![allow(dead_code)]

use serde::Deserialize;
use tauri_plugin_http::reqwest::{self, Client, Response, Error};
use tauri_plugin_http::reqwest::header::USER_AGENT;

use super::tmdb_api_token::get_api_token;

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

#[derive(Deserialize, Debug)]
struct test_authentification {
    success: bool
}


async fn call_tmdb_api(client: &Client, api_url: &str, api_token: &str) -> Result<Response, Error> {
    client
    .get(api_url)
    .header(USER_AGENT, "rust-web-api-client") // gh api requires a user-agent header
    .header("accept", "application/json")
    .header("Authorization", "Bearer ".to_owned() + api_token)
    .send()
    .await
}


pub(super) async fn is_tmdb_api_read_access_token_valid(client: &Client, api_token: &str) -> Result<bool, Error> {
    let test_authentification_url = "https://api.themoviedb.org/3/authentication";
    let response = call_tmdb_api(&client, test_authentification_url, api_token).await;

    match response {
        Ok(res) => Ok(res.json::<test_authentification>().await?.success),
        Err(err) => Err(err),
    }
}

pub(super) async fn get_movie_information_tmdb(movietitle: &str) -> Result<(), Error> {
    
    let api_token = get_api_token();
    let client = reqwest::Client::new();

    
    if is_tmdb_api_read_access_token_valid(&client, api_token).await? {
        let search_movie_url = format!("https://api.themoviedb.org/3/search/movie?query={query}&include_adult={include_adult}&language={language}&page={page}",
                              query = movietitle,
                              include_adult = "false",
                              language = "en-US",
                              page = "1"
                              //year = "1982",
                              );

        let response = call_tmdb_api(&client, &search_movie_url, api_token).await?;
        println!("{}", response.text().await?);
    }
    else {
        panic!("API token not valid. Go to api.themoviedb.org and insert in src/library_manager/tmdb_api_token.rs");
    }
    
    Ok(())
}