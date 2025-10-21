#![allow(dead_code)]

use anyhow::{anyhow, Error};

use serde::Deserialize;
use tauri_plugin_http::reqwest::header::USER_AGENT;
use tauri_plugin_http::reqwest::{self, Client, Response};

use super::super::{Library, VideoElement};
use super::api_token::get_api_token;
use super::json_structs::*;
use super::search_name_creator::{get_movie_title_and_year_from_filename, get_tv_show_info_from_filename, is_tv_episode};

static TMDB_API_MOVIE_URL: &str = "https://api.themoviedb.org/3/movie/";
static TMDB_API_TV_URL: &str = "https://api.themoviedb.org/3/tv/";
static TMDB_IMAGE_URL: &str = "https://image.tmdb.org/t/p/original";


async fn create_client() -> Result<(Client, String), Error>
{
    let api_token = get_api_token();
    if api_token.is_empty() {
        return Err(anyhow!("There is no API Read Access Token! Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"));
    }

    let client = reqwest::Client::new();
    match is_tmdb_api_read_access_token_valid(&client, api_token).await {
        Ok(valid) => {
            if valid
                { Ok((client, api_token.to_string())) }
            else
                { Err(anyhow!("API Read Access Token not valid. Go to https://www.themoviedb.org/settings/api and insert in src/tmdb_api/api_token.rs"))}
        }
        Err(str) => { Err(anyhow!("Could not connect to TMDB: {}", str)) }
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

pub async fn is_tmdb_api_read_access_token_valid(client: &Client, api_token: &str) -> Result<bool, Error> {
    let test_authentification_url = "https://api.themoviedb.org/3/authentication";
    let response = call_tmdb_api(client, test_authentification_url, api_token).await?;

    response.json::<TMDBTestAuthentification>().await
        .map(|res| res.success)
        .map_err(|e| anyhow!("Could not parse authentication response: {}", e))
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
    -> Result<TMDBSearchMovieResult, Error>
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

    let response = call_tmdb_api(client, &search_movie_url, api_token).await?;

    response.json().await
        .map_err(|e| anyhow!("Could not parse Movie Search JSON: {}", e))
}

async fn create_client_and_search_tmdb
(
    movietitle: &str,
    year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>
)
    -> Result<TMDBSearchMovieResult, Error>
{
    let (client, api_token) = create_client().await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;
    search_tmdb(&client, &api_token, movietitle, year, language, include_adult, page).await
}

pub(crate) async fn get_tmdb_search_as_video_elements(search_title: &str) -> Result<Vec<VideoElement>, Error> {
    let query_result_object = match create_client_and_search_tmdb(search_title, None, None, None, None).await {
        Ok(res) => res,
        Err(e) => return Err(anyhow!("Error trying to call tmdb database: {}", e))
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

async fn get_movie_details(client: &Client, movie_id: usize, api_token: &str) -> Result<TMDBMovieDetails, Error> {
    let get_movie_details_url = format!("{}{}{}", TMDB_API_MOVIE_URL, movie_id, "?append_to_response=credits");

    let response = call_tmdb_api(client, &get_movie_details_url, api_token).await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    response.json().await
        .map_err(|e| anyhow!("Could not parse Movie Details JSON: {}", e))
}

// ==================== TV Show API Functions ====================

async fn search_tv_show
(
    client: &Client,
    api_token: &str,
    tv_show_name: &str,
    first_air_date_year: Option<i32>,
    language: Option<&str>,
    include_adult: Option<bool>,
    page: Option<i32>
)
    -> Result<TMDBSearchTVResult, Error>
{
    let include_adult = include_adult.unwrap_or(false);
    let page = page.unwrap_or(1);

    let search_tv_url = {
        let mut url = String::from("https://api.themoviedb.org/3/search/tv?query=");
        url.push_str(tv_show_name);
        url.push_str("&include_adult=");
        url.push_str(match include_adult {true => "true", false => "false"} );
        url.push_str("&page=");
        url.push_str(page.to_string().as_str());

        if let Some(year) = first_air_date_year {
            url.push_str("&first_air_date_year=");
            url.push_str(year.to_string().as_str());
        }

        if let Some(language) = language {
            url.push_str("&language=");
            url.push_str(language.to_string().as_str());
        }

        url
    };

    let response = call_tmdb_api(client, &search_tv_url, api_token).await?;

    response.json().await
        .map_err(|e| anyhow!("Could not parse TV Search JSON: {}", e))
}

async fn get_tv_show_details(client: &Client, tv_id: usize, api_token: &str) -> Result<TMDBTVShowDetails, Error> {
    let get_tv_details_url = format!("{}{}{}", TMDB_API_TV_URL, tv_id, "?append_to_response=credits");

    let response = call_tmdb_api(client, &get_tv_details_url, api_token).await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    response.json().await
        .map_err(|e| anyhow!("Could not parse TV Show Details JSON: {}", e))
}

async fn get_tv_episode_details(
    client: &Client,
    tv_id: usize,
    season_number: i32,
    episode_number: i32,
    api_token: &str
) -> Result<TMDBEpisodeDetails, Error> {
    let get_episode_details_url = format!(
        "{}{}{}{}{}{}{}",
        TMDB_API_TV_URL,
        tv_id,
        "/season/",
        season_number,
        "/episode/",
        episode_number,
        "?append_to_response=credits"
    );

    let response = call_tmdb_api(client, &get_episode_details_url, api_token).await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    response.json().await
        .map_err(|e| anyhow!("Could not parse TV Episode Details JSON: {}", e))
}



pub async fn parse_library_tmdb(library: &mut Library, reparse_all: Option<bool>) -> Result<(), Error>
{
    let reparse_all: bool = reparse_all.unwrap_or(false);

    let (client, api_token) = create_client().await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    for video_element in library.video_files.iter_mut() {
        if reparse_all || video_element.tmdb_id.is_none() {
            let filename = &video_element.filepath;

            // Check if filename hints at this being a TV episode
            if is_tv_episode(filename) {
                println!("[TV SHOW] {}", filename);

                let (show_name, season, episode) = get_tv_show_info_from_filename(filename);

                // Need both season and episode to fetch episode details
                if season.is_none() || episode.is_none() {
                    println!("  Could not extract season/episode from filename: {} (Season: {:?}, Episode: {:?})",
                        filename, season, episode);
                    continue;
                }

                // Search for the TV show
                match search_tv_show(&client, &api_token, &show_name, None, None, None, None).await {
                    Ok(search_result) => {
                        if search_result.results.is_empty() {
                            println!("  No TV show found for: {}", show_name);
                            continue;
                        }

                        let best_match = &search_result.results[0];
                        if best_match.id.is_none() {
                            println!("  TV show search result has no ID");
                            continue;
                        }

                        let tv_id = best_match.id.unwrap();

                        // Get TV show details
                        let tv_show_details = get_tv_show_details(&client, tv_id, &api_token).await
                            .map_err(|e| anyhow!("Error when getting TV Show Details: {}", e))?;

                        // Get episode details
                        match get_tv_episode_details(
                            &client,
                            tv_id,
                            season.unwrap(),
                            episode.unwrap(),
                            &api_token
                        ).await {
                            Ok(episode_details) => {
                                fill_video_element_with_tv_episode(
                                    video_element,
                                    &tv_show_details,
                                    &episode_details,
                                    season.unwrap(),
                                    episode.unwrap(),
                                    None
                                );

                                println!("  Updated: {} S{:02}E{:02} - {}",
                                    tv_show_details.tmdb_tv_show.name.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                    season.unwrap(),
                                    episode.unwrap(),
                                    episode_details.name.as_ref().unwrap_or(&"!MISSING!".to_string()));
                            }
                            Err(e) => {
                                println!("  Error getting episode details for {} S{:02}E{:02}: {}",
                                    show_name, season.unwrap(), episode.unwrap(), e);

                                // Fallback: Fill with partial data (show info + parsed season/episode numbers)
                                fill_video_element_with_tv_show_only(
                                    video_element,
                                    &tv_show_details,
                                    season.unwrap(),
                                    episode.unwrap()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Error searching for TV show '{}': {}", show_name, e);
                    }
                }
                continue;
            }

            // Handle as movie
            println!("[MOVIE] {}", filename);
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
                        .map_err(|e| anyhow!("Error when getting Movie Details: {}", e))?;

                    fill_video_element_with_movie_details(video_element, &movie_details, None);
                    if video_element.release_date.is_none() && year.is_some() {
                        video_element.release_date = Some(year.unwrap().to_string());
                    }
                    println!("Updated: {} [{}] ({}) by {}", video_element.original_title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            &video_element.title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            video_element.release_date.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            &video_element.director.as_ref().unwrap_or(&"!MISSING!".to_string()))
                }
                Err(str) => {return Err(anyhow!("Error when calling TMDB: {}", str))}
            }
        }
    }

    Ok(())
}

/// Reparse library TMDB data, updating all metadata but preserving existing covers
pub async fn reparse_library_tmdb_preserve_covers(library: &mut Library) -> Result<(), Error>
{
    let (client, api_token) = create_client().await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    for video_element in library.video_files.iter_mut() {
        // Only reparse movies that already have TMDB IDs
        if let Some(tmdb_id) = video_element.tmdb_id {
            let movie_details = get_movie_details(&client, tmdb_id, &api_token).await
                .map_err(|e| anyhow!("Error when getting Movie Details: {}", e))?;

            // Update all metadata but preserve existing covers
            fill_video_element_with_movie_details_preserve_covers(video_element, &movie_details);

            println!("Reparsed: {} [{}] ({}) by {}",
                video_element.original_title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                video_element.title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                video_element.release_date.as_ref().unwrap_or(&"!MISSING!".to_string()),
                video_element.director.as_ref().unwrap_or(&"!MISSING!".to_string()));
        }
    }

    Ok(())
}

/// Fill video element with movie details, updating all data but preserving existing covers
fn fill_video_element_with_movie_details_preserve_covers(video_element: &mut VideoElement, movie_details: &TMDBMovieDetails)
{
    // Store existing cover paths to preserve them
    let existing_poster = video_element.poster_path.clone();
    let existing_backdrop = video_element.backdrop_path.clone();
    let existing_thumbnail = video_element.thumbnail_path.clone();

    // Update all metadata from search result (this includes titles, language, genre, overview, release_date)
    fill_video_element_with_search_result(video_element, &movie_details.tmdb_movie, Some(true));

    // Restore existing covers if they were set, otherwise keep the new ones from TMDB
    if existing_poster.is_some() {
        video_element.poster_path = existing_poster;
    }
    if existing_backdrop.is_some() {
        video_element.backdrop_path = existing_backdrop;
    }
    if existing_thumbnail.is_some() {
        video_element.thumbnail_path = existing_thumbnail;
    }

    // Update tagline (from movie details)
    video_element.tagline = movie_details.tagline.clone();

    // Update countries using production_countries (the corrected field)
    if let Some(production_countries) = &movie_details.production_countries {
        let country_codes: Vec<String> = production_countries
            .iter()
            .filter_map(|country| country.iso_3166_1.clone())
            .collect();

        if !country_codes.is_empty() {
            video_element.countries = Some(country_codes);
        }
    }

    // Update director from credits
    if let Some(credits) = &movie_details.credits {
        if let Some(crew) = &credits.crew {
            for crew_member in crew {
                if let Some(job) = &crew_member.job {
                    if job == "Director" {
                        video_element.director = crew_member.tmdb_person.name.clone();
                        break;
                    }
                }
            }
        }
    }
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
        video_element.poster_path = movie_search_result.poster_path.as_ref().map(|path| format!("{}{}", TMDB_IMAGE_URL, path));
    }
    if overwrite || video_element.backdrop_path.is_none() {
        video_element.backdrop_path = movie_search_result.backdrop_path.as_ref().map(|path| format!("{}{}", TMDB_IMAGE_URL, path));
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
    let overwrite = overwrite.unwrap_or(false);
    fill_video_element_with_search_result(video_element, &movie_details.tmdb_movie, Some(overwrite));

    if overwrite || video_element.tagline.is_none()
        { video_element.tagline = movie_details.tagline.clone(); }
    if overwrite || video_element.countries.is_none() {
        // Extract ISO country codes from production_countries
        if let Some(production_countries) = &movie_details.production_countries {
            let country_codes: Vec<String> = production_countries
                .iter()
                .filter_map(|country| country.iso_3166_1.clone())
                .collect();

            if !country_codes.is_empty() {
                video_element.countries = Some(country_codes);
            }
        }
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

// ==================== TV Show Fill Functions ====================

/// Fill video element with only TV show details (no episode data)
/// Used as fallback when episode details aren't available from TMDB
fn fill_video_element_with_tv_show_only(
    video_element: &mut VideoElement,
    tv_show_details: &TMDBTVShowDetails,
    season_number: i32,
    episode_number: i32
)
{
    // Set TMDB ID to the show ID
    video_element.tmdb_id = tv_show_details.tmdb_tv_show.id;

    // Set season and episode numbers from filename parsing
    video_element.season = Some(season_number);
    video_element.episode = Some(episode_number);

    // Build title with show name and episode numbers (no episode name)
    let combined_title = {
        let show_name = tv_show_details.tmdb_tv_show.name.as_deref().unwrap_or("Unknown Show");
        format!("{} S{:02}E{:02}", show_name, season_number, episode_number)
    };

    video_element.title = Some(combined_title.clone());
    video_element.original_title = Some(combined_title);

    // Set show-level metadata
    video_element.tmdb_language = tv_show_details.tmdb_tv_show.original_language.clone();
    video_element.genre_ids = tv_show_details.tmdb_tv_show.genre_ids.clone();
    video_element.tagline = tv_show_details.tagline.clone();

    // Set countries from the show
    if let Some(production_countries) = &tv_show_details.production_countries {
        let country_codes: Vec<String> = production_countries
            .iter()
            .filter_map(|country| country.iso_3166_1.clone())
            .collect();

        if !country_codes.is_empty() {
            video_element.countries = Some(country_codes);
        }
    }

    // Set poster and backdrop from show
    if let Some(poster_path) = &tv_show_details.tmdb_tv_show.poster_path {
        video_element.poster_path = Some(format!("{}{}", TMDB_IMAGE_URL, poster_path));
    }
    if let Some(backdrop_path) = &tv_show_details.tmdb_tv_show.backdrop_path {
        video_element.backdrop_path = Some(format!("{}{}", TMDB_IMAGE_URL, backdrop_path));
    }

    println!("  Partial update (no episode details): {} S{:02}E{:02}",
        tv_show_details.tmdb_tv_show.name.as_ref().unwrap_or(&"!MISSING!".to_string()),
        season_number,
        episode_number);
}

/// Fill video element with TV episode details
/// Title format: "(Series Name) SxxExx (Episode Name)"
///
/// parsed_season and parsed_episode are the values extracted from the filename,
/// used as fallback when TMDB returns None values
fn fill_video_element_with_tv_episode(
    video_element: &mut VideoElement,
    tv_show_details: &TMDBTVShowDetails,
    episode_details: &TMDBEpisodeDetails,
    parsed_season: i32,
    parsed_episode: i32,
    overwrite: Option<bool>
)
{
    let overwrite = overwrite.unwrap_or(false);

    // Set TMDB ID to the show ID (not episode ID)
    video_element.tmdb_id = tv_show_details.tmdb_tv_show.id;

    // Set season and episode numbers, preferring TMDB data but falling back to parsed values
    if overwrite || video_element.season.is_none() {
        video_element.season = Some(episode_details.season_number.unwrap_or(parsed_season));
    }
    if overwrite || video_element.episode.is_none() {
        video_element.episode = Some(episode_details.episode_number.unwrap_or(parsed_episode));
    }

    // Build combined title: "(Series Name) SxxExx (Episode Name)"
    let combined_title = {
        let show_name = tv_show_details.tmdb_tv_show.name.as_deref().unwrap_or("Unknown Show");
        let season_num = episode_details.season_number.unwrap_or(parsed_season);
        let episode_num = episode_details.episode_number.unwrap_or(parsed_episode);
        let episode_name = episode_details.name.as_deref().unwrap_or("Unknown Episode");

        format!("{} S{:02}E{:02} {}", show_name, season_num, episode_num, episode_name)
    };

    if overwrite || video_element.title.is_none() {
        video_element.title = Some(combined_title.clone());
    }

    // Set original_title to the same combined format
    if overwrite || video_element.original_title.is_none() {
        video_element.original_title = Some(combined_title);
    }

    // Set release_date to episode air_date
    if overwrite || video_element.release_date.is_none() {
        video_element.release_date = episode_details.air_date.clone();
    }

    // Set language
    if overwrite || video_element.tmdb_language.is_none() {
        video_element.tmdb_language = tv_show_details.tmdb_tv_show.original_language.clone();
    }

    // Set genre_ids from the show
    if overwrite || video_element.genre_ids.is_none() {
        video_element.genre_ids = tv_show_details.tmdb_tv_show.genre_ids.clone();
    }

    // Set overview to episode overview
    if overwrite || video_element.overview.is_none() {
        video_element.overview = episode_details.overview.clone();
    }

    // Set tagline from the show
    if overwrite || video_element.tagline.is_none() {
        video_element.tagline = tv_show_details.tagline.clone();
    }

    // Set countries from the show
    if overwrite || video_element.countries.is_none() {
        if let Some(production_countries) = &tv_show_details.production_countries {
            let country_codes: Vec<String> = production_countries
                .iter()
                .filter_map(|country| country.iso_3166_1.clone())
                .collect();

            if !country_codes.is_empty() {
                video_element.countries = Some(country_codes);
            }
        }
    }

    // Set poster_path - use episode still if available, otherwise show poster
    if overwrite || video_element.poster_path.is_none() {
        if let Some(still_path) = &episode_details.still_path {
            video_element.poster_path = Some(format!("{}{}", TMDB_IMAGE_URL, still_path));
        } else if let Some(poster_path) = &tv_show_details.tmdb_tv_show.poster_path {
            video_element.poster_path = Some(format!("{}{}", TMDB_IMAGE_URL, poster_path));
        }
    }

    // Set backdrop_path from show
    if overwrite || video_element.backdrop_path.is_none() {
        video_element.backdrop_path = tv_show_details.tmdb_tv_show.backdrop_path.as_ref()
            .map(|path| format!("{}{}", TMDB_IMAGE_URL, path));
    }

    // Set director from episode crew (look for "Director" job)
    if let Some(crew) = &episode_details.crew {
        for crew_member in crew {
            if let Some(job) = &crew_member.job {
                if job == "Director" {
                    video_element.director = crew_member.tmdb_person.name.clone();
                    break;
                }
            }
        }
    }
}


use std::collections::HashMap;

pub async fn get_additional_covers(
    tmdb_id: usize, 
    sort_by_languages_in_iso_639_1: Option<Vec<String>>, 
    filter_other_languages: Option<bool> 
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

    let mut posters: Vec<TMDBImage> = response.json::<TMDBImages>()
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
                let a_pos = a.iso_639_1.as_ref()
                    .and_then(|code| position_map.get(code).copied())
                    .unwrap_or(usize::MAX);
                    
                // Get position of b's language code or MAX if not found
                let b_pos = b.iso_639_1.as_ref()
                    .and_then(|code| position_map.get(code).copied())
                    .unwrap_or(usize::MAX);
                    
                // Compare positions for sorting
                a_pos.cmp(&b_pos)
            });
        }
    }

    let poster_paths = posters
        .into_iter()
        .filter_map(|poster| poster.file_path
            .map(|path| format!("{}{}", TMDB_IMAGE_URL, path)))
        .collect();


    Ok(poster_paths)
}
