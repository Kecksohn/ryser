#![allow(dead_code)]

use anyhow::{anyhow, Error};

use serde::Deserialize;
use tauri_plugin_http::reqwest::header::USER_AGENT;
use tauri_plugin_http::reqwest::{self, Client, Response};

use super::super::{Library, VideoElement};
use super::api_token::get_api_token;
use super::json_structs::*;
use super::search_name_creator::get_movie_title_and_year_from_filename;

static TMDB_API_MOVIE_URL: &str = "https://api.themoviedb.org/3/movie/";
static TMDB_IMAGE_URL: &str = "https://image.tmdb.org/t/p/original";

async fn create_client() -> Result<(Client, String), Error> {
    let api_token = get_api_token();
    if api_token.is_empty() {
        return Err(anyhow!("There is no API Read Access Token! Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"));
    }

    let client = reqwest::Client::new();
    match is_tmdb_api_read_access_token_valid(&client, api_token).await {
        Ok(valid) => {
            if valid {
                Ok((client, api_token.to_string()))
            } else {
                Err(anyhow!("API Read Access Token not valid. Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"))
            }
        }
        Err(str) => Err(anyhow!("Could not connect to TMDB: {}", str)),
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
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))
}

pub async fn is_tmdb_api_read_access_token_valid(
    client: &Client,
    api_token: &str,
) -> Result<bool, Error> {
    let test_authentification_url = "https://api.themoviedb.org/3/authentication";
    let response = call_tmdb_api(client, test_authentification_url, api_token).await?;

    response
        .json::<TMDBTestAuthentification>()
        .await
        .map(|res| res.success)
        .map_err(|e| anyhow!("Could not parse authentication response: {}", e))
}

async fn search_tmdb(
    client: &Client,
    api_token: &str,
    movietitle: &str,
    year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>,
) -> Result<TMDBSearchMovieResult, Error> {
    let include_adult = include_adult.unwrap_or(false);
    let page = page.unwrap_or(1);

    let search_movie_url = {
        let mut url = String::from("https://api.themoviedb.org/3/search/movie?query=");
        url.push_str(movietitle);
        url.push_str("&include_adult=");
        url.push_str(match include_adult {
            true => "true",
            false => "false",
        });
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

    let response = call_tmdb_api(client, &search_movie_url, api_token).await?;

    response
        .json()
        .await
        .map_err(|e| anyhow!("Could not parse Movie Search JSON: {}", e))
}

async fn create_client_and_search_tmdb(
    movietitle: &str,
    year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>,
) -> Result<TMDBSearchMovieResult, Error> {
    let (client, api_token) = create_client()
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;
    search_tmdb(
        &client,
        &api_token,
        movietitle,
        year,
        language,
        include_adult,
        page,
    )
    .await
}

pub(crate) async fn get_tmdb_search_as_video_elements(
    search_title: &str,
) -> Result<Vec<VideoElement>, Error> {
    let query_result_object =
        match create_client_and_search_tmdb(search_title, None, None, None, None).await {
            Ok(res) => res,
            Err(e) => return Err(anyhow!("Error trying to call tmdb database: {}", e)),
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

async fn get_movie_details(
    client: &Client,
    movie_id: usize,
    api_token: &str,
) -> Result<TMDBMovieDetails, Error> {
    let get_movie_details_url = format!(
        "{}{}{}",
        TMDB_API_MOVIE_URL, movie_id, "?append_to_response=credits"
    );

    let response = call_tmdb_api(client, &get_movie_details_url, api_token)
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    response
        .json()
        .await
        .map_err(|e| anyhow!("Could not parse Movie Details JSON: {}", e))
}

pub async fn parse_library_tmdb(
    library: &mut Library,
    reparse_all: Option<bool>,
) -> Result<(), Error> {
    let reparse_all: bool = reparse_all.unwrap_or(false);

    let (client, api_token) = create_client()
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    for video_element in library.video_files.iter_mut() {
        if reparse_all || video_element.tmdb_id.is_none() {
            let filename = &video_element.filepath;

            // TODO: Check if filename hints at this being a TV episode

            let (possible_title, year) = get_movie_title_and_year_from_filename(filename);

            match search_tmdb(
                &client,
                &api_token,
                possible_title.as_str(),
                year,
                None,
                None,
                None,
            )
            .await
            {
                Ok(search_tmdb_result) => {
                    if search_tmdb_result.results.is_empty() {
                        // TODO: Adjust filename by cutting one word from the end
                        continue;
                    }

                    let best_match = &search_tmdb_result.results[0];
                    if best_match.id.is_none() {
                        continue;
                    }

                    let movie_details =
                        get_movie_details(&client, best_match.id.unwrap(), &api_token)
                            .await
                            .map_err(|e| anyhow!("Error when getting Movie Details: {}", e))?;

                    fill_video_element_with_movie_details(video_element, &movie_details, None);
                    if video_element.release_date.is_none() && year.is_some() {
                        video_element.release_date = Some(year.unwrap().to_string());
                    }
                    println!(
                        "Updated: {} [{}] ({}) by {}",
                        video_element
                            .original_title
                            .as_ref()
                            .unwrap_or(&"!MISSING!".to_string()),
                        &video_element
                            .title
                            .as_ref()
                            .unwrap_or(&"!MISSING!".to_string()),
                        video_element
                            .release_date
                            .as_ref()
                            .unwrap_or(&"!MISSING!".to_string()),
                        &video_element
                            .director
                            .as_ref()
                            .unwrap_or(&"!MISSING!".to_string())
                    )
                }
                Err(str) => return Err(anyhow!("Error when calling TMDB: {}", str)),
            }
        }
    }

    Ok(())
}

pub async fn get_movie_details_for_video_element(
    video_element: &mut VideoElement,
    overwrite: Option<bool>,
) -> Result<(), Error> {
    if video_element.tmdb_id.is_none() {
        return Err(anyhow!("No TMDB ID specified, Implement getting it!"));
    }

    let (client, api_token) = create_client()
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;
    let movie_details = get_movie_details(&client, video_element.tmdb_id.unwrap(), &api_token)
        .await
        .map_err(|e| anyhow!("Error when getting Movie Details: {}", e))?;
    fill_video_element_with_movie_details(video_element, &movie_details, overwrite);

    Ok(())
}

fn fill_video_element_with_search_result(
    video_element: &mut VideoElement,
    movie_search_result: &TMDBMovie,
    overwrite: Option<bool>,
) {
    let overwrite = overwrite.unwrap_or(false);

    video_element.tmdb_id = movie_search_result.id;
    if overwrite || video_element.title.is_none() {
        video_element.title = movie_search_result.title.clone();
    }
    if overwrite || video_element.original_title.is_none() {
        video_element.original_title = movie_search_result.original_title.clone();
    }
    if overwrite || video_element.tmdb_language.is_none() {
        video_element.tmdb_language = movie_search_result.original_language.clone();
    }
    if overwrite || video_element.genre_ids.is_none() {
        video_element.genre_ids = movie_search_result.genre_ids.clone();
    }
    if overwrite || video_element.overview.is_none() {
        video_element.overview = movie_search_result.overview.clone();
    }
    if overwrite || video_element.release_date.is_none() {
        video_element.release_date = movie_search_result.release_date.clone();
    }

    if overwrite || video_element.poster_path.is_none() {
        video_element.poster_path = movie_search_result
            .poster_path
            .as_ref()
            .map(|path| format!("{}{}", TMDB_IMAGE_URL, path));
    }
    if overwrite || video_element.backdrop_path.is_none() {
        video_element.backdrop_path = movie_search_result
            .backdrop_path
            .as_ref()
            .map(|path| format!("{}{}", TMDB_IMAGE_URL, path));
    }

    /* Not used:
    pub adult: Option<bool>,
    pub popularity: Option<f32>,
    pub video: Option<bool>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<usize>,
     */
}

fn fill_video_element_with_movie_details(
    video_element: &mut VideoElement,
    movie_details: &TMDBMovieDetails,
    overwrite: Option<bool>,
) {
    let overwrite = overwrite.unwrap_or(false);
    fill_video_element_with_search_result(
        video_element,
        &movie_details.tmdb_movie,
        Some(overwrite),
    );

    if overwrite || video_element.tagline.is_none() {
        video_element.tagline = movie_details.tagline.clone();
    }
    if overwrite || video_element.countries.is_none() {
        video_element.countries = movie_details.origin_country.clone()
    }

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
    pub production_companies: Option<Vec<TMDBProductionCompanies>>,
    pub production_countries: Option<Vec<TMDBProductionCountries>>,
    pub revenue: Option<usize>,
    pub runtime: Option<usize>,
    pub spoken_languages: Option<Vec<TMDBSpokenLanguage>>,
    pub status: Option<String>,

    Rest of crew/cast
    */
}

use std::collections::HashMap;

pub async fn get_additional_covers(
    tmdb_id: usize,
    sort_by_languages_in_iso_639_1: Option<Vec<String>>,
    filter_other_languages: Option<bool>,
) -> Result<Vec<String>, Error> {
    let filter_other_languages = filter_other_languages.unwrap_or(false);

    let (client, api_token) = create_client()
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    let mut get_covers_url = format!("{}{}{}", TMDB_API_MOVIE_URL, tmdb_id, "/images");

    if filter_other_languages {
        if let Some(languages) = &sort_by_languages_in_iso_639_1 {
            if !languages.is_empty() {
                get_covers_url.push_str("?include_image_language=");
                get_covers_url.push_str(&languages.join(","));
            }
        }
    }

    let response = call_tmdb_api(&client, &get_covers_url, &api_token)
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    let mut posters: Vec<TMDBImage> = response
        .json::<TMDBImages>()
        .await
        .map_err(|e| anyhow!("Could not parse TMDB Images JSON: {}", e))?
        .posters
        .ok_or_else(|| anyhow!("Movie has no posters!"))?;

    if let Some(sort_languages) = sort_by_languages_in_iso_639_1 {
        if !filter_other_languages || sort_languages.len() > 1 {
            // Create HashMap of languages to sort by for O(1)
            let position_map: HashMap<&String, usize> = sort_languages
                .iter()
                .enumerate()
                .map(|(index, code)| (code, index))
                .collect();

            // Then sort posters based on their position in the reference vector
            posters.sort_by(|a, b| {
                // Get position of a's language code or MAX if not found
                let a_pos = a
                    .iso_639_1
                    .as_ref()
                    .and_then(|code| position_map.get(code).copied())
                    .unwrap_or(usize::MAX);

                // Get position of b's language code or MAX if not found
                let b_pos = b
                    .iso_639_1
                    .as_ref()
                    .and_then(|code| position_map.get(code).copied())
                    .unwrap_or(usize::MAX);

                // Compare positions for sorting
                a_pos.cmp(&b_pos)
            });
        }
    }

    let poster_paths = posters
        .into_iter()
        .filter_map(|poster| {
            poster
                .file_path
                .map(|path| format!("{}{}", TMDB_IMAGE_URL, path))
        })
        .collect();

    Ok(poster_paths)
}
