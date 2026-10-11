#![allow(dead_code)]

use anyhow::{anyhow, Error};

use serde::Deserialize;
use tauri_plugin_http::reqwest::header::USER_AGENT;
use tauri_plugin_http::reqwest::{self, Client, Response};

use std::collections::{BTreeMap, HashSet};

use super::super::{compute_selection, EpisodeMeta, FilmMeta, Library, Media, TvSeason, TvShow};
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

pub(crate) async fn get_tmdb_search_as_films(search_title: &str) -> Result<Vec<FilmMeta>, Error> {
    let query_result_object = match create_client_and_search_tmdb(search_title, None, None, None, None).await {
        Ok(res) => res,
        Err(e) => return Err(anyhow!("Error trying to call tmdb database: {}", e))
    };

    let mut query_result_films: Vec<FilmMeta> = vec![];

    for query_result in query_result_object.results.iter() {
        let mut result_film = FilmMeta::default();
        fill_film_with_search_result(&mut result_film, query_result, false);
        query_result_films.push(result_film);
    }

    Ok(query_result_films)
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

    // Build URL via parse_with_params so the query value (and any other params)
    // are percent-encoded. Concatenating raw breaks on spaces/&/special chars.
    let mut params: Vec<(&str, String)> = vec![
        ("query", tv_show_name.to_string()),
        ("include_adult", include_adult.to_string()),
        ("page", page.to_string()),
    ];
    if let Some(year) = first_air_date_year {
        params.push(("first_air_date_year", year.to_string()));
    }
    if let Some(language) = language {
        params.push(("language", language.to_string()));
    }

    let search_tv_url = reqwest::Url::parse_with_params("https://api.themoviedb.org/3/search/tv", &params)
        .map_err(|e| anyhow!("Could not build TV search URL: {}", e))?;

    let response = call_tmdb_api(client, search_tv_url.as_str(), api_token).await?;

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

    let Library { video_files, tv_shows, .. } = library;
    // A reparse searches for an unmatched show again, but once per run and not once per episode
    let mut resolved_show_names: HashSet<String> = HashSet::new();

    for video_element in video_files.iter_mut() {
        if reparse_all || !video_element.tmdb_searched {
            let filename = &video_element.filepath;

            // Check if filename hints at this being a TV episode
            if is_tv_episode(filename) {
                println!("[TV SHOW] {}", filename);

                let (show_name, season, episode) = get_tv_show_info_from_filename(filename);

                // Need both season and episode to fetch episode details
                let (Some(season), Some(episode)) = (season, episode) else {
                    println!("  Could not extract season/episode from filename: {} (Season: {:?}, Episode: {:?})",
                        filename, season, episode);
                    continue;
                };

                let retry_unmatched = reparse_all && !resolved_show_names.contains(&show_name);
                let show_id = match find_or_create_show(&client, &api_token, tv_shows, &show_name, retry_unmatched).await {
                    Ok(show_id) => show_id,
                    Err(e) => {
                        println!("  Error searching for TV show '{}': {}", show_name, e);
                        continue;
                    }
                };
                resolved_show_names.insert(show_name);
                let show = &tv_shows[&show_id];

                // Keep what an earlier parse or the user filled in
                let mut episode_meta = match &video_element.media {
                    Media::Episode(existing) if existing.show_id == show_id => existing.clone(),
                    _ => EpisodeMeta { show_id, ..Default::default() },
                };
                let season = *episode_meta.season.get_or_insert(season);
                let episode = *episode_meta.episode.get_or_insert(episode);

                let mut definite_result = true;
                if let Some(show_tmdb_id) = show.tmdb_id {
                    match get_tv_episode_details(&client, show_tmdb_id, season, episode, &api_token).await {
                        // TMDB answers an unknown episode with an error object, which parses to no ID
                        Ok(episode_details) if episode_details.id.is_some() => {
                            fill_episode_with_details(&mut episode_meta, &episode_details, false);

                            println!("  Updated: {} S{:02}E{:02} - {}",
                                show_name_of(show),
                                season,
                                episode,
                                episode_meta.name.as_ref().unwrap_or(&"!MISSING!".to_string()));
                        }
                        Ok(_) => {
                            println!("  No episode details on TMDB for {} S{:02}E{:02}", show_name_of(show), season, episode);
                        }
                        Err(e) => {
                            println!("  Error getting episode details for {} S{:02}E{:02}: {}",
                                show_name_of(show), season, episode, e);
                            definite_result = false;
                        }
                    }
                }

                video_element.media = Media::Episode(episode_meta);
                video_element.tmdb_language = show.tmdb_language.clone();
                video_element.tmdb_searched = definite_result;
                video_element.playback_selection = Some(compute_selection(video_element));
                continue;
            }

            // Handle as movie
            println!("[MOVIE] {}", filename);
            let (possible_title, year) = get_movie_title_and_year_from_filename(filename);

            match search_tmdb( &client, &api_token, possible_title.as_str(), year, None, None, None).await {
                Ok(search_tmdb_result) => {
                    let Some(movie_id) = search_tmdb_result.results.first().and_then(|best_match| best_match.id) else {
                        // TODO: Adjust filename by cutting one word from the end
                        video_element.tmdb_searched = true;
                        continue;
                    };

                    let movie_details = get_movie_details(&client, movie_id, &api_token).await
                        .map_err(|e| anyhow!("Error when getting Movie Details: {}", e))?;

                    let mut film = match std::mem::take(&mut video_element.media) {
                        Media::Film(film) => film,
                        _ => {
                            // Was an episode before, its show's language does not apply
                            video_element.tmdb_language = None;
                            FilmMeta::default()
                        }
                    };
                    fill_film_with_movie_details(&mut film, &movie_details, false);
                    if film.release_date.is_none() && year.is_some() {
                        film.release_date = Some(year.unwrap().to_string());
                    }
                    println!("Updated: {} [{}] ({}) by {}", film.original_title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            film.title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            film.release_date.as_ref().unwrap_or(&"!MISSING!".to_string()),
                                                            film.director.as_ref().unwrap_or(&"!MISSING!".to_string()));

                    video_element.media = Media::Film(film);
                    fill(&mut video_element.tmdb_language, &movie_details.tmdb_movie.original_language, false);
                    video_element.tmdb_searched = true;
                    video_element.playback_selection = Some(compute_selection(video_element));
                }
                Err(str) => {return Err(anyhow!("Error when calling TMDB: {}", str))}
            }
        }
    }

    Ok(())
}

fn show_name_of(show: &TvShow) -> &str {
    show.name.as_deref().unwrap_or("!MISSING!")
}

/// ID of the show that `show_name` (as parsed from a filename) belongs to.
/// A name not seen before is searched on TMDB once and the show is added to
/// `tv_shows`, without a `tmdb_id` when TMDB has no result.
/// `retry_unmatched` searches again for a known show that has no `tmdb_id`.
/// Fails without changing `tv_shows` when TMDB cannot be reached.
async fn find_or_create_show(
    client: &Client,
    api_token: &str,
    tv_shows: &mut BTreeMap<u32, TvShow>,
    show_name: &str,
    retry_unmatched: bool,
) -> Result<u32, Error> {
    let known_id = tv_shows.values()
        .find(|show| show.parsed_names.iter().any(|name| name == show_name))
        .map(|show| show.id);
    if let Some(id) = known_id {
        if !retry_unmatched || tv_shows[&id].tmdb_id.is_some() {
            return Ok(id);
        }
    }

    let search_result = search_tv_show(client, api_token, show_name, None, None, None, None).await?;
    let tmdb_id = search_result.results.first().and_then(|best_match| best_match.id);
    let tv_show_details = match tmdb_id {
        Some(tmdb_id) => Some(get_tv_show_details(client, tmdb_id, api_token).await?),
        None => {
            println!("  No TV show found for: {}", show_name);
            None
        }
    };

    // The same show can be reached through differently spelled filenames
    let same_tmdb_show_id = tmdb_id.and_then(|tmdb_id| {
        tv_shows.values().find(|show| show.tmdb_id == Some(tmdb_id)).map(|show| show.id)
    });
    let id = known_id.or(same_tmdb_show_id)
        .unwrap_or_else(|| tv_shows.keys().next_back().map_or(1, |highest_id| highest_id + 1));

    let show = tv_shows.entry(id).or_insert_with(|| TvShow { id, ..Default::default() });
    if !show.parsed_names.iter().any(|name| name == show_name) {
        show.parsed_names.push(show_name.to_owned());
    }
    if let Some(tv_show_details) = &tv_show_details {
        show.tmdb_id = tmdb_id;
        fill_show_with_details(show, tv_show_details, false);
    }
    if show.name.is_none() {
        show.name = Some(show_name.to_owned());
    }

    Ok(id)
}

/// Reparse library TMDB data, updating all metadata but preserving existing covers
pub async fn reparse_library_tmdb_preserve_covers(library: &mut Library) -> Result<(), Error>
{
    let (client, api_token) = create_client().await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    let Library { video_files, tv_shows, .. } = library;

    for show in tv_shows.values_mut() {
        let Some(tmdb_id) = show.tmdb_id else { continue };

        let tv_show_details = get_tv_show_details(&client, tmdb_id, &api_token).await
            .map_err(|e| anyhow!("Error when getting TV Show Details: {}", e))?;
        if tv_show_details.tmdb_tv_show.id.is_none() {
            println!("Not on TMDB anymore, kept as is: {}", show_name_of(show));
            continue;
        }

        let existing_poster = show.poster_path.take();
        let existing_backdrop = show.backdrop_path.take();
        fill_show_with_details(show, &tv_show_details, true);
        keep_existing(&mut show.poster_path, existing_poster);
        keep_existing(&mut show.backdrop_path, existing_backdrop);

        println!("Reparsed: {}", show_name_of(show));
    }

    for video_element in video_files.iter_mut() {
        // Only reparse what is already matched
        let tmdb_language = match &mut video_element.media {
            Media::Unmatched => continue,
            Media::Film(film) => {
                let Some(tmdb_id) = film.tmdb_id else { continue };

                let movie_details = get_movie_details(&client, tmdb_id, &api_token).await
                    .map_err(|e| anyhow!("Error when getting Movie Details: {}", e))?;
                if movie_details.tmdb_movie.id.is_none() {
                    println!("Not on TMDB anymore, kept as is: {}", video_element.filepath);
                    continue;
                }

                let existing_poster = film.poster_path.take();
                let existing_backdrop = film.backdrop_path.take();
                fill_film_with_movie_details(film, &movie_details, true);
                keep_existing(&mut film.poster_path, existing_poster);
                keep_existing(&mut film.backdrop_path, existing_backdrop);

                println!("Reparsed: {} [{}] ({}) by {}",
                    film.original_title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                    film.title.as_ref().unwrap_or(&"!MISSING!".to_string()),
                    film.release_date.as_ref().unwrap_or(&"!MISSING!".to_string()),
                    film.director.as_ref().unwrap_or(&"!MISSING!".to_string()));

                movie_details.tmdb_movie.original_language
            }
            Media::Episode(episode_meta) => {
                let Some(show) = tv_shows.get(&episode_meta.show_id) else { continue };

                if let (Some(show_tmdb_id), Some(season), Some(episode)) = (show.tmdb_id, episode_meta.season, episode_meta.episode) {
                    match get_tv_episode_details(&client, show_tmdb_id, season, episode, &api_token).await {
                        Ok(episode_details) if episode_details.id.is_some() => {
                            let existing_still = episode_meta.still_path.take();
                            fill_episode_with_details(episode_meta, &episode_details, true);
                            keep_existing(&mut episode_meta.still_path, existing_still);

                            println!("Reparsed: {} S{:02}E{:02}", show_name_of(show), season, episode);
                        }
                        Ok(_) => {
                            println!("No episode details on TMDB for {} S{:02}E{:02}", show_name_of(show), season, episode);
                        }
                        Err(e) => {
                            println!("Error getting episode details for {} S{:02}E{:02}: {}", video_element.filepath, season, episode, e);
                        }
                    }
                }

                // The show may have been re-matched since the episode was parsed
                show.tmdb_language.clone()
            }
        };

        // tmdb_language may have changed during reparse; recompute audio/subtitle selection.
        video_element.tmdb_language = tmdb_language;
        video_element.playback_selection = Some(compute_selection(video_element));
    }

    Ok(())
}

pub async fn get_movie_details_for_film(
    film: &mut FilmMeta,
    overwrite: Option<bool>,
) -> Result<(), Error> {
    if film.tmdb_id.is_none() {
        return Err(anyhow!("No TMDB ID specified, Implement getting it!"));
    }

    let (client, api_token) = create_client()
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;
    let movie_details = get_movie_details(&client, film.tmdb_id.unwrap(), &api_token)
        .await
        .map_err(|e| anyhow!("Error when getting Movie Details: {}", e))?;
    fill_film_with_movie_details(film, &movie_details, overwrite.unwrap_or(false));

    Ok(())
}


// ==================== Fill Functions ====================
// All of them fill empty fields and replace filled ones only with `overwrite`.
// The caller owns `tmdb_language` and the playback selection of the element.

fn fill<T: Clone>(field: &mut Option<T>, value: &Option<T>, overwrite: bool) {
    if overwrite || field.is_none() {
        *field = value.clone();
    }
}

/// Like `fill`, but a value TMDB does not have never empties the field
fn fill_if_some<T: Clone>(field: &mut Option<T>, value: Option<T>, overwrite: bool) {
    if value.is_some() {
        fill(field, &value, overwrite);
    }
}

fn keep_existing(field: &mut Option<String>, existing: Option<String>) {
    if existing.is_some() {
        *field = existing;
    }
}

fn image_url(path: &Option<String>) -> Option<String> {
    path.as_ref().map(|path| format!("{}{}", TMDB_IMAGE_URL, path))
}

fn country_codes(production_countries: &Option<Vec<TMDBProductionCountries>>) -> Option<Vec<String>> {
    let country_codes: Vec<String> = production_countries.as_ref()?
        .iter()
        .filter_map(|country| country.iso_3166_1.clone())
        .collect();
    (!country_codes.is_empty()).then_some(country_codes)
}

/// The details endpoints list genres as objects, only search results have `genre_ids`
fn genre_ids(genres: &Option<Vec<TMDBGenre>>) -> Option<Vec<usize>> {
    Some(genres.as_ref()?.iter().filter_map(|genre| genre.id).collect())
}

fn director(crew: &Option<Vec<TMDBCrew>>) -> Option<String> {
    crew.as_ref()?
        .iter()
        .find(|crew_member| crew_member.job.as_deref() == Some("Director"))?
        .tmdb_person.name.clone()
}

fn fill_film_with_search_result(film: &mut FilmMeta, movie_search_result: &TMDBMovie, overwrite: bool)
{
    film.tmdb_id = movie_search_result.id;
    fill(&mut film.title, &movie_search_result.title, overwrite);
    fill(&mut film.original_title, &movie_search_result.original_title, overwrite);
    fill(&mut film.genre_ids, &movie_search_result.genre_ids, overwrite);
    fill(&mut film.overview, &movie_search_result.overview, overwrite);
    fill(&mut film.release_date, &movie_search_result.release_date, overwrite);
    fill(&mut film.poster_path, &image_url(&movie_search_result.poster_path), overwrite);
    fill(&mut film.backdrop_path, &image_url(&movie_search_result.backdrop_path), overwrite);

    /* Not used:
    pub adult: Option<bool>,
    pub popularity: Option<f32>,
    pub video: Option<bool>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<usize>,
     */
}

fn fill_film_with_movie_details(film: &mut FilmMeta, movie_details: &TMDBMovieDetails, overwrite: bool)
{
    fill_film_with_search_result(film, &movie_details.tmdb_movie, overwrite);

    fill(&mut film.tagline, &movie_details.tagline, overwrite);
    fill_if_some(&mut film.genre_ids, genre_ids(&movie_details.genres), overwrite);
    fill_if_some(&mut film.countries, country_codes(&movie_details.production_countries), overwrite);

    if let Some(credits) = &movie_details.credits {
        fill_if_some(&mut film.director, director(&credits.crew), true);
    }

    /* Not used:
    pub belongs_to_collection: Option<TMDBCollection>,
    pub budget: Option<usize>,
    pub homepage: Option<String>,
    pub imdb_id: Option<String>,
    pub production_companies: Option<Vec<TMDBProductionCompanies>>,
    pub revenue: Option<usize>,
    pub runtime: Option<usize>,
    pub spoken_languages: Option<Vec<TMDBSpokenLanguage>>,
    pub status: Option<String>,

    Rest of crew/cast
    */
}

/// Does not set `tmdb_id`: the details of a show that left TMDB parse to all-empty
fn fill_show_with_details(show: &mut TvShow, tv_show_details: &TMDBTVShowDetails, overwrite: bool)
{
    let tmdb_tv_show = &tv_show_details.tmdb_tv_show;

    fill(&mut show.name, &tmdb_tv_show.name, overwrite);
    fill(&mut show.original_name, &tmdb_tv_show.original_name, overwrite);
    fill(&mut show.overview, &tmdb_tv_show.overview, overwrite);
    fill(&mut show.first_air_date, &tmdb_tv_show.first_air_date, overwrite);
    fill(&mut show.tmdb_language, &tmdb_tv_show.original_language, overwrite);
    fill(&mut show.poster_path, &image_url(&tmdb_tv_show.poster_path), overwrite);
    fill(&mut show.backdrop_path, &image_url(&tmdb_tv_show.backdrop_path), overwrite);
    fill(&mut show.tagline, &tv_show_details.tagline, overwrite);
    fill(&mut show.status, &tv_show_details.status, overwrite);

    let genre_ids = genre_ids(&tv_show_details.genres).or_else(|| tmdb_tv_show.genre_ids.clone());
    fill_if_some(&mut show.genre_ids, genre_ids, overwrite);
    let countries = country_codes(&tv_show_details.production_countries).or_else(|| tmdb_tv_show.origin_country.clone());
    fill_if_some(&mut show.countries, countries, overwrite);
    let creators = tv_show_details.created_by.as_ref()
        .map(|creators| creators.iter().filter_map(|creator| creator.name.clone()).collect());
    fill_if_some(&mut show.creators, creators, overwrite);

    if let Some(seasons) = &tv_show_details.seasons {
        if overwrite || show.seasons.is_empty() {
            show.seasons = seasons.iter()
                .filter_map(|season| Some(TvSeason {
                    season_number: season.season_number?,
                    name: season.name.clone(),
                    overview: season.overview.clone(),
                    poster_path: image_url(&season.poster_path),
                    air_date: season.air_date.clone(),
                    episode_count: season.episode_count,
                }))
                .collect();
        }
    }
}

/// `show_id` and the parsed season/episode numbers are set by the caller
fn fill_episode_with_details(episode_meta: &mut EpisodeMeta, episode_details: &TMDBEpisodeDetails, overwrite: bool)
{
    episode_meta.tmdb_id = episode_details.id;
    fill_if_some(&mut episode_meta.season, episode_details.season_number, overwrite);
    fill_if_some(&mut episode_meta.episode, episode_details.episode_number, overwrite);
    fill(&mut episode_meta.name, &episode_details.name, overwrite);
    fill(&mut episode_meta.overview, &episode_details.overview, overwrite);
    fill(&mut episode_meta.air_date, &episode_details.air_date, overwrite);
    fill(&mut episode_meta.still_path, &image_url(&episode_details.still_path), overwrite);
    fill_if_some(&mut episode_meta.director, director(&episode_details.crew), true);
}


use std::collections::HashMap;

pub async fn get_additional_covers(
    tmdb_id: usize,
    is_tv: bool,
    sort_by_languages_in_iso_639_1: Option<Vec<String>>,
    filter_other_languages: Option<bool>
) -> Result<Vec<String>, Error> {

    let filter_other_languages = filter_other_languages.unwrap_or(false);

    let (client, api_token) = create_client()
        .await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    let api_url = if is_tv { TMDB_API_TV_URL } else { TMDB_API_MOVIE_URL };
    let mut get_covers_url = format!("{}{}{}", api_url, tmdb_id, "/images");

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
