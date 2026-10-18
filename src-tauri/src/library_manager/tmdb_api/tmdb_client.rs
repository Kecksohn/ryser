#![allow(dead_code)]

use anyhow::{anyhow, Error};

use serde::Deserialize;
use tauri_plugin_http::reqwest::header::USER_AGENT;
use tauri_plugin_http::reqwest::{self, Client, Response};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use super::super::{compute_selection, EpisodeMeta, FilmMeta, Library, Media, TvSeason, TvShow, VideoElement};
use super::api_token::get_api_token;
use super::json_structs::*;
use super::search_name_creator::{
    get_movie_title_and_year_from_filename, get_show_name_and_year, get_tv_show_info_from_filename, is_tv_episode,
    show_name_similarity,
};

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

async fn get_tv_season_details(client: &Client, tv_id: usize, season_number: i32, api_token: &str) -> Result<TMDBSeasonDetails, Error> {
    let get_season_details_url = format!("{}{}{}{}", TMDB_API_TV_URL, tv_id, "/season/", season_number);

    let response = call_tmdb_api(client, &get_season_details_url, api_token).await
        .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;

    // An unknown season is a 404 with an error object, which parses to a season without episodes.
    // Any other failure (rate limit, server error) must not look like one.
    let status = response.status();
    if !status.is_success() && status != reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow!("TMDB answered {} for season {} of show {}", status, season_number, tv_id));
    }

    response.json().await
        .map_err(|e| anyhow!("Could not parse TV Season Details JSON: {}", e))
}


/// The requests a parse run sends to TMDB. Tests replace it to count them and to run without network.
trait TmdbApi {
    async fn search_movie(&self, title: &str, year: Option<i32>) -> Result<TMDBSearchMovieResult, Error>;
    async fn get_movie_details(&self, movie_id: usize) -> Result<TMDBMovieDetails, Error>;
    async fn search_tv_show(&self, show_name: &str, first_air_date_year: Option<i32>) -> Result<TMDBSearchTVResult, Error>;
    async fn get_tv_show_details(&self, tv_id: usize) -> Result<TMDBTVShowDetails, Error>;
    async fn get_tv_season_details(&self, tv_id: usize, season_number: i32) -> Result<TMDBSeasonDetails, Error>;
}

struct TmdbClient {
    client: Client,
    api_token: String,
}

impl TmdbClient {
    async fn connect() -> Result<Self, Error> {
        let (client, api_token) = create_client().await
            .map_err(|e| anyhow!("Could not connect to TMDB: {}", e))?;
        Ok(TmdbClient { client, api_token })
    }
}

impl TmdbApi for TmdbClient {
    async fn search_movie(&self, title: &str, year: Option<i32>) -> Result<TMDBSearchMovieResult, Error> {
        search_tmdb(&self.client, &self.api_token, title, year, None, None, None).await
    }

    async fn get_movie_details(&self, movie_id: usize) -> Result<TMDBMovieDetails, Error> {
        get_movie_details(&self.client, movie_id, &self.api_token).await
    }

    async fn search_tv_show(&self, show_name: &str, first_air_date_year: Option<i32>) -> Result<TMDBSearchTVResult, Error> {
        search_tv_show(&self.client, &self.api_token, show_name, first_air_date_year, None, None, None).await
    }

    async fn get_tv_show_details(&self, tv_id: usize) -> Result<TMDBTVShowDetails, Error> {
        get_tv_show_details(&self.client, tv_id, &self.api_token).await
    }

    async fn get_tv_season_details(&self, tv_id: usize, season_number: i32) -> Result<TMDBSeasonDetails, Error> {
        get_tv_season_details(&self.client, tv_id, season_number, &self.api_token).await
    }
}

/// Seasons fetched in this run, by local show ID and season number
type SeasonCache = HashMap<(u32, i32), TMDBSeasonDetails>;

/// A failed request is not cached, the next episode of the season tries again.
async fn get_season<'a>(
    api: &impl TmdbApi,
    seasons: &'a mut SeasonCache,
    show_id: u32,
    show_tmdb_id: usize,
    season: i32,
) -> Result<&'a TMDBSeasonDetails, Error> {
    let key = (show_id, season);
    if !seasons.contains_key(&key) {
        seasons.insert(key, api.get_tv_season_details(show_tmdb_id, season).await?);
    }
    Ok(&seasons[&key])
}

fn find_episode(season_details: &TMDBSeasonDetails, episode: i32) -> Option<&TMDBEpisodeDetails> {
    season_details.episodes.iter().find(|episode_details| episode_details.episode_number == Some(episode))
}


pub async fn parse_library_tmdb(library: &mut Library, reparse_all: Option<bool>) -> Result<(), Error>
{
    let api = TmdbClient::connect().await?;
    let library_roots: Vec<&str> = library.library_paths.iter().map(|library_path| library_path.path.as_str()).collect();
    parse_video_files(&api, &library_roots, &mut library.video_files, &mut library.tv_shows, reparse_all.unwrap_or(false)).await
}

/// The episode files of one show that a run has to match
#[derive(Default)]
struct ShowFiles {
    /// Names to search the show with: name, year, number of files the name comes from
    names: Vec<(String, Option<i32>, usize)>,
    /// Index into the video files, season, episode
    episodes: Vec<(usize, Option<i32>, Option<i32>)>,
}

impl ShowFiles {
    fn add_name(&mut self, name: &str, year: Option<i32>, files: usize) {
        if name.is_empty() {
            return;
        }
        match self.names.iter_mut().find(|(known_name, known_year, _)| known_name.eq_ignore_ascii_case(name) && *known_year == year) {
            Some((_, _, known_files)) => *known_files = known_files.saturating_add(files),
            None => self.names.push((name.to_owned(), year, files)),
        }
    }

    /// The name used by the most files first
    fn search_names(&self) -> Vec<(String, Option<i32>)> {
        let mut names = self.names.clone();
        names.sort_by_key(|(_, _, files)| std::cmp::Reverse(*files));
        names.into_iter().map(|(name, year, _)| (name, year)).collect()
    }

    fn owned_episodes(&self) -> HashSet<(i32, i32)> {
        self.episodes.iter()
            .filter_map(|(_, season, episode)| Some(((*season)?, (*episode)?)))
            .collect()
    }
}

/// The folder directly below a library root that `filepath` is in. None for a file in the root itself.
fn show_folder<'a>(library_roots: &[&str], filepath: &'a str) -> Option<&'a str> {
    let below_root = library_roots.iter().find_map(|root| Path::new(filepath).strip_prefix(root).ok())?;
    below_root.parent()?.components().next()?.as_os_str().to_str()
}

async fn parse_video_files(
    api: &impl TmdbApi,
    library_roots: &[&str],
    video_files: &mut [VideoElement],
    tv_shows: &mut BTreeMap<u32, TvShow>,
    reparse_all: bool,
) -> Result<(), Error>
{
    let mut seasons = SeasonCache::new();
    // Episode files are collected first: a show is matched once, with all its files.
    // One folder below a library root is one show, whatever its files are called.
    let mut shows_to_match: BTreeMap<String, ShowFiles> = BTreeMap::new();

    for (index, video_element) in video_files.iter_mut().enumerate() {
        if reparse_all || !video_element.tmdb_searched {
            // Check if filename hints at this being a TV episode
            if is_tv_episode(&video_element.filepath) {
                let (parsed_name, season, episode) = get_tv_show_info_from_filename(&video_element.filepath);
                let (name, year) = get_show_name_and_year(&parsed_name);
                let folder = show_folder(library_roots, &video_element.filepath)
                    .map(get_show_name_and_year)
                    .filter(|(folder_name, _)| !folder_name.is_empty());

                // A file directly in the library root goes by its own name
                let show_key = folder.as_ref().map_or(&name, |(folder_name, _)| folder_name).clone();
                let show_files = shows_to_match.entry(show_key).or_default();
                if let Some((folder_name, folder_year)) = &folder {
                    // No fixed precedence over the filename names, but it is always among the searched ones
                    show_files.add_name(folder_name, *folder_year, usize::MAX);
                }
                show_files.add_name(&name, year, 1);
                show_files.episodes.push((index, season, episode));
                continue;
            }

            let filename = &video_element.filepath;

            // Handle as movie
            println!("[MOVIE] {}", filename);
            let (possible_title, year) = get_movie_title_and_year_from_filename(filename);

            match api.search_movie(possible_title.as_str(), year).await {
                Ok(search_tmdb_result) => {
                    let Some(movie_id) = search_tmdb_result.results.first().and_then(|best_match| best_match.id) else {
                        // TODO: Adjust filename by cutting one word from the end
                        video_element.tmdb_searched = true;
                        continue;
                    };

                    let movie_details = api.get_movie_details(movie_id).await
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

    for (show_name, show_files) in &shows_to_match {
        // A reparse matches an unmatched show again
        let show_id = match find_or_create_show(api, tv_shows, show_name, show_files, reparse_all).await {
            Ok(show_id) => show_id,
            Err(e) => {
                println!("  Error searching for TV show '{}': {}", show_name, e);
                continue;
            }
        };
        let Some(show) = tv_shows.get_mut(&show_id) else { continue };

        for &(index, season, episode) in &show_files.episodes {
            parse_episode(api, show, &mut seasons, &mut video_files[index], season, episode).await;
        }
    }

    Ok(())
}

/// Fills one episode file of `show`, `season` and `episode` are the numbers parsed from its path.
async fn parse_episode(
    api: &impl TmdbApi,
    show: &mut TvShow,
    seasons: &mut SeasonCache,
    video_element: &mut VideoElement,
    season: Option<i32>,
    episode: Option<i32>,
) {
    println!("[TV SHOW] {}", video_element.filepath);
    let show_id = show.id;

    // Keep what an earlier parse or the user filled in
    let mut episode_meta = match &video_element.media {
        Media::Episode(existing) if existing.show_id == show_id => existing.clone(),
        _ => EpisodeMeta { show_id, ..Default::default() },
    };
    episode_meta.season = episode_meta.season.or(season);
    episode_meta.episode = episode_meta.episode.or(episode);

    // Without both numbers there is nothing to ask TMDB: the file stays an episode of its show without details
    let mut definite_result = true;
    if let (Some(show_tmdb_id), Some(season), Some(episode)) = (show.tmdb_id, episode_meta.season, episode_meta.episode) {
        match get_season(api, seasons, show_id, show_tmdb_id, season).await {
            Ok(season_details) => {
                fill_show_season(show, season_details);

                match find_episode(season_details, episode) {
                    Some(episode_details) => {
                        fill_episode_with_details(&mut episode_meta, episode_details, false);

                        println!("  Updated: {} S{:02}E{:02} - {}",
                            show_name_of(show),
                            season,
                            episode,
                            episode_meta.name.as_ref().unwrap_or(&"!MISSING!".to_string()));
                    }
                    None => {
                        println!("  No episode details on TMDB for {} S{:02}E{:02}", show_name_of(show), season, episode);
                    }
                }
            }
            Err(e) => {
                println!("  Error getting season details for {} S{:02}E{:02}: {}",
                    show_name_of(show), season, episode, e);
                definite_result = false;
            }
        }
    }

    video_element.media = Media::Episode(episode_meta);
    video_element.tmdb_language = show.tmdb_language.clone();
    video_element.tmdb_searched = definite_result;
    video_element.playback_selection = Some(compute_selection(video_element));
}

fn show_name_of(show: &TvShow) -> &str {
    show.name.as_deref().unwrap_or("!MISSING!")
}

/// ID of the show that `show_name` (its folder, or the name parsed from the filename of a
/// file without show folder) belongs to.
/// A name not seen before is matched on TMDB once and the show is added to
/// `tv_shows`, without a `tmdb_id` when no search result fits.
/// `retry_unmatched` matches again for a known show that has no `tmdb_id`.
/// Fails without changing `tv_shows` when TMDB cannot be reached.
async fn find_or_create_show(
    api: &impl TmdbApi,
    tv_shows: &mut BTreeMap<u32, TvShow>,
    show_name: &str,
    show_files: &ShowFiles,
    retry_unmatched: bool,
) -> Result<u32, Error> {
    if let Some(id) = known_show_id(tv_shows, show_name) {
        if !retry_unmatched || tv_shows[&id].tmdb_id.is_some() {
            return Ok(id);
        }
    }

    let tv_show_details = match_show(api, &show_files.search_names(), &show_files.owned_episodes()).await?;
    if tv_show_details.is_none() {
        println!("  No TV show found for: {}", show_name);
    }

    Ok(upsert_show(tv_shows, show_name, tv_show_details.as_ref()))
}

/// Name similarity a search result needs to count as the searched show
const MIN_SHOW_NAME_SIMILARITY: f32 = 0.6;
const SHOW_YEAR_BONUS: f32 = 0.1;
/// Smaller than the distance between an equal name and one that only starts with the searched name
const EPISODE_FIT_BONUS: f32 = 0.15;
const MAX_SHOW_SEARCHES: usize = 3;
const MAX_RESULTS_PER_SEARCH: usize = 5;
const MAX_SHOW_DETAILS_REQUESTS: usize = 3;

/// Details of the show on TMDB that fits the names and the owned (season, episode) pairs best.
/// None when no search result passes the name check.
async fn match_show(
    api: &impl TmdbApi,
    names: &[(String, Option<i32>)],
    owned_episodes: &HashSet<(i32, i32)>,
) -> Result<Option<TMDBTVShowDetails>, Error> {
    let mut search_results = Vec::new();
    for (name, year) in names.iter().take(MAX_SHOW_SEARCHES) {
        let search_result = api.search_tv_show(name, *year).await?;
        search_results.extend(search_result.results.into_iter().take(MAX_RESULTS_PER_SEARCH));
    }

    let candidates = rank_show_candidates(names, &search_results);
    let Some(&(top_score, _)) = candidates.first() else { return Ok(None) };

    // The episode fit is a tie-break: details are only requested for candidates it can still bring to the top
    let mut best: Option<(f32, TMDBTVShowDetails)> = None;
    for &(score, show) in candidates.iter().take(MAX_SHOW_DETAILS_REQUESTS).filter(|(score, _)| top_score - score < EPISODE_FIT_BONUS) {
        let Some(tmdb_id) = show.id else { continue };
        let tv_show_details = api.get_tv_show_details(tmdb_id).await?;
        let score = score + EPISODE_FIT_BONUS * episode_fit(owned_episodes, &tv_show_details);
        if best.as_ref().map_or(true, |(best_score, _)| score > *best_score) {
            best = Some((score, tv_show_details));
        }
    }

    Ok(best.map(|(_, tv_show_details)| tv_show_details))
}

/// The search results that pass the name check with their score, best first. TMDB's order decides a tie.
/// The score is the best similarity of the show's names to one of `names`, plus a bonus when
/// one of `names` has the year the show first aired. A differing year does not count against
/// a show: the year in a filename is often the one of the episode.
fn rank_show_candidates<'a>(names: &[(String, Option<i32>)], search_results: &'a [TMDBTVShow]) -> Vec<(f32, &'a TMDBTVShow)> {
    let mut candidates: Vec<(f32, &TMDBTVShow)> = Vec::new();

    for show in search_results {
        if show.id.is_none() || candidates.iter().any(|(_, candidate)| candidate.id == show.id) {
            continue;
        }

        let name_score = names.iter()
            .flat_map(|(name, _)| [&show.name, &show.original_name].into_iter().flatten().map(move |found| show_name_similarity(name, found)))
            .fold(0.0, f32::max);
        if name_score < MIN_SHOW_NAME_SIMILARITY {
            continue;
        }

        let first_air_year: Option<i32> = show.first_air_date.as_ref().and_then(|date| date.get(..4)?.parse().ok());
        let has_year = first_air_year.is_some() && names.iter().any(|(_, year)| *year == first_air_year);
        candidates.push((name_score + if has_year { SHOW_YEAR_BONUS } else { 0.0 }, show));
    }

    candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
    candidates
}

/// Share of the owned (season, episode) pairs that the season list of the show holds.
/// Only ever added to a score: absolute numbering (episode 201 of season 1) fits no show.
fn episode_fit(owned_episodes: &HashSet<(i32, i32)>, tv_show_details: &TMDBTVShowDetails) -> f32 {
    if owned_episodes.is_empty() {
        return 0.0;
    }

    let seasons = tv_show_details.seasons.as_deref().unwrap_or_default();
    let fitting = owned_episodes.iter()
        .filter(|(owned_season, owned_episode)| seasons.iter().any(|season| {
            season.season_number == Some(*owned_season)
                && season.episode_count.is_some_and(|episode_count| (1..=episode_count).contains(owned_episode))
        }))
        .count();

    fitting as f32 / owned_episodes.len() as f32
}

fn known_show_id(tv_shows: &BTreeMap<u32, TvShow>, show_name: &str) -> Option<u32> {
    tv_shows.values()
        .find(|show| show.parsed_names.iter().any(|name| name == show_name))
        .map(|show| show.id)
}

/// Adds `show_name` to the show its TMDB search led to and returns the local ID of that show.
/// `tv_show_details` is None when the search found nothing: the show is then kept or
/// created without a `tmdb_id`.
fn upsert_show(tv_shows: &mut BTreeMap<u32, TvShow>, show_name: &str, tv_show_details: Option<&TMDBTVShowDetails>) -> u32 {
    // Details of a show that left TMDB parse to no ID, which counts as not found
    let tmdb_id = tv_show_details.and_then(|tv_show_details| tv_show_details.tmdb_tv_show.id);

    // The same show can be reached through differently spelled filenames
    let same_tmdb_show_id = tmdb_id.and_then(|tmdb_id| {
        tv_shows.values().find(|show| show.tmdb_id == Some(tmdb_id)).map(|show| show.id)
    });
    let id = known_show_id(tv_shows, show_name).or(same_tmdb_show_id)
        .unwrap_or_else(|| next_show_id(tv_shows));

    let show = tv_shows.entry(id).or_insert_with(|| TvShow { id, ..Default::default() });
    if !show.parsed_names.iter().any(|name| name == show_name) {
        show.parsed_names.push(show_name.to_owned());
    }
    if let (Some(tv_show_details), Some(_)) = (tv_show_details, tmdb_id) {
        show.tmdb_id = tmdb_id;
        fill_show_with_details(show, tv_show_details, false);
    }
    if show.name.is_none() {
        show.name = Some(show_name.to_owned());
    }

    id
}

fn next_show_id(tv_shows: &BTreeMap<u32, TvShow>) -> u32 {
    tv_shows.keys().next_back().map_or(1, |highest_id| highest_id + 1)
}

fn show_from_details(id: u32, tv_show_details: &TMDBTVShowDetails) -> TvShow {
    let mut show = TvShow { id, tmdb_id: tv_show_details.tmdb_tv_show.id, ..Default::default() };
    fill_show_with_details(&mut show, tv_show_details, true);
    show
}

async fn tv_show_details_of(api: &impl TmdbApi, tmdb_id: usize) -> Result<TMDBTVShowDetails, Error> {
    let tv_show_details = api.get_tv_show_details(tmdb_id).await?;
    if tv_show_details.tmdb_tv_show.id.is_none() {
        return Err(anyhow!("TV show {} is not on TMDB", tmdb_id));
    }
    Ok(tv_show_details)
}

/// ID of the library's show with `tmdb_id`, added from TMDB when there is none
async fn find_or_add_tmdb_show(api: &impl TmdbApi, tv_shows: &mut BTreeMap<u32, TvShow>, tmdb_id: usize) -> Result<u32, Error> {
    if let Some(show) = tv_shows.values().find(|show| show.tmdb_id == Some(tmdb_id)) {
        return Ok(show.id);
    }
    let tv_show_details = tv_show_details_of(api, tmdb_id).await?;
    let id = next_show_id(tv_shows);
    tv_shows.insert(id, show_from_details(id, &tv_show_details));
    Ok(id)
}

/// Puts show `show_id` on the TMDB show `tmdb_id` and refills its episodes from it, keeping
/// their numbers. When another show of the library already has `tmdb_id`, the episodes and
/// parsed names move to that show and `show_id` is dropped.
/// Returns the ID of the show the episodes belong to now.
async fn rematch_show_with(
    api: &impl TmdbApi,
    video_files: &mut [VideoElement],
    tv_shows: &mut BTreeMap<u32, TvShow>,
    show_id: u32,
    tmdb_id: usize,
) -> Result<u32, Error> {
    let Some(old_show) = tv_shows.get(&show_id) else { return Err(anyhow!("Show {} not found", show_id)) };
    let parsed_names = old_show.parsed_names.clone();

    let merge_into = tv_shows.values().find(|show| show.id != show_id && show.tmdb_id == Some(tmdb_id)).map(|show| show.id);
    let target_id = match merge_into {
        Some(target_id) => {
            tv_shows.remove(&show_id);
            target_id
        }
        None => {
            let tv_show_details = tv_show_details_of(api, tmdb_id).await?;
            tv_shows.insert(show_id, show_from_details(show_id, &tv_show_details));
            show_id
        }
    };

    let show = tv_shows.get_mut(&target_id).unwrap();
    for name in parsed_names {
        if !show.parsed_names.contains(&name) {
            show.parsed_names.push(name);
        }
    }

    let mut seasons = SeasonCache::new();
    for video_element in video_files.iter_mut() {
        let Media::Episode(episode_meta) = &video_element.media else { continue };
        if episode_meta.show_id != show_id {
            continue;
        }
        // What TMDB said about the episode belonged to the old show
        let (season, episode) = (episode_meta.season, episode_meta.episode);
        video_element.media = Media::Episode(EpisodeMeta { show_id: target_id, season, episode, ..Default::default() });
        parse_episode(api, show, &mut seasons, video_element, None, None).await;
    }

    Ok(target_id)
}

/// Stores the media the user chose for one file. An episode without TMDB details is looked
/// up with its show and numbers when `api` is given.
async fn apply_media(
    api: Option<&impl TmdbApi>,
    video_element: &mut VideoElement,
    tv_shows: &mut BTreeMap<u32, TvShow>,
    media: Media,
) -> Result<(), Error> {
    match &media {
        Media::Episode(episode_meta) => {
            let Some(show) = tv_shows.get_mut(&episode_meta.show_id) else {
                return Err(anyhow!("Show {} not found", episode_meta.show_id));
            };
            let lookup = episode_meta.tmdb_id.is_none();
            video_element.tmdb_language = show.tmdb_language.clone();
            video_element.media = media;
            if let (Some(api), true) = (api, lookup) {
                parse_episode(api, show, &mut SeasonCache::new(), video_element, None, None).await;
            }
        }
        _ => {
            // The show's language does not apply to a film, and a film's is only known from TMDB
            if matches!(video_element.media, Media::Episode(_)) {
                video_element.tmdb_language = None;
            }
            video_element.media = media;
        }
    }

    // The user decided, automatic matching must not fill in something else
    video_element.tmdb_searched = true;
    video_element.playback_selection = Some(compute_selection(video_element));
    Ok(())
}

/// Same condition as the season request in `parse_episode`
fn needs_episode_lookup(tv_shows: &BTreeMap<u32, TvShow>, media: &Media) -> bool {
    matches!(media, Media::Episode(episode_meta)
        if episode_meta.tmdb_id.is_none() && episode_meta.season.is_some() && episode_meta.episode.is_some()
            && tv_shows.get(&episode_meta.show_id).is_some_and(|show| show.tmdb_id.is_some()))
}

fn show_from_search_result(search_result: &TMDBTVShow) -> TvShow {
    TvShow {
        tmdb_id: search_result.id,
        name: search_result.name.clone(),
        original_name: search_result.original_name.clone(),
        overview: search_result.overview.clone(),
        first_air_date: search_result.first_air_date.clone(),
        poster_path: image_url(&search_result.poster_path),
        ..Default::default()
    }
}

async fn search_shows(api: &impl TmdbApi, search_title: &str) -> Result<Vec<TvShow>, Error> {
    Ok(api.search_tv_show(search_title, None).await?.results.iter().map(show_from_search_result).collect())
}

pub(crate) async fn get_tmdb_search_as_shows(search_title: &str) -> Result<Vec<TvShow>, Error> {
    search_shows(&TmdbClient::connect().await?, search_title).await
}

pub(crate) async fn add_tmdb_show(tv_shows: &mut BTreeMap<u32, TvShow>, tmdb_id: usize) -> Result<u32, Error> {
    find_or_add_tmdb_show(&TmdbClient::connect().await?, tv_shows, tmdb_id).await
}

pub(crate) async fn rematch_show(video_files: &mut [VideoElement], tv_shows: &mut BTreeMap<u32, TvShow>, show_id: u32, tmdb_id: usize) -> Result<u32, Error> {
    rematch_show_with(&TmdbClient::connect().await?, video_files, tv_shows, show_id, tmdb_id).await
}

/// Connects to TMDB only when the episode has to be looked up. Without a connection the
/// media is stored without details.
pub(crate) async fn set_media_with_lookup(video_element: &mut VideoElement, tv_shows: &mut BTreeMap<u32, TvShow>, media: Media) -> Result<(), Error> {
    let api = if needs_episode_lookup(tv_shows, &media) {
        TmdbClient::connect().await.map_err(|e| println!("Episode not looked up: {}", e)).ok()
    } else {
        None
    };
    apply_media(api.as_ref(), video_element, tv_shows, media).await
}

/// Reparse library TMDB data, updating all metadata but preserving existing covers
pub async fn reparse_library_tmdb_preserve_covers(library: &mut Library) -> Result<(), Error>
{
    let api = TmdbClient::connect().await?;

    let Library { video_files, tv_shows, .. } = library;
    let mut seasons = SeasonCache::new();

    for show in tv_shows.values_mut() {
        let Some(tmdb_id) = show.tmdb_id else { continue };

        let tv_show_details = api.get_tv_show_details(tmdb_id).await
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

                let movie_details = api.get_movie_details(tmdb_id).await
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
                    match get_season(&api, &mut seasons, show.id, show_tmdb_id, season).await {
                        Ok(season_details) => match find_episode(season_details, episode) {
                            Some(episode_details) => {
                                let existing_still = episode_meta.still_path.take();
                                fill_episode_with_details(episode_meta, episode_details, true);
                                keep_existing(&mut episode_meta.still_path, existing_still);

                                println!("Reparsed: {} S{:02}E{:02}", show_name_of(show), season, episode);
                            }
                            None => {
                                println!("No episode details on TMDB for {} S{:02}E{:02}", show_name_of(show), season, episode);
                            }
                        },
                        Err(e) => {
                            println!("Error getting season details for {} S{:02}E{:02}: {}", video_element.filepath, season, episode, e);
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

/// Show details are only fetched when a show is created. A season TMDB got since then is
/// added from its season reply, and the episode count of a known one is updated.
fn fill_show_season(show: &mut TvShow, season_details: &TMDBSeasonDetails)
{
    let tmdb_season = &season_details.tmdb_season;
    // The error object of an unknown season has no number
    let Some(season_number) = tmdb_season.season_number else { return };
    let episode_count = Some(season_details.episodes.len() as i32);

    match show.seasons.iter_mut().find(|season| season.season_number == season_number) {
        Some(season) => season.episode_count = episode_count,
        None => {
            show.seasons.push(TvSeason {
                season_number,
                name: tmdb_season.name.clone(),
                overview: tmdb_season.overview.clone(),
                poster_path: image_url(&tmdb_season.poster_path),
                air_date: tmdb_season.air_date.clone(),
                episode_count,
            });
            show.seasons.sort_by_key(|season| season.season_number);
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::Cell;

    macro_rules! fixture {
        ($name:literal) => {
            serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../test-fixtures/tmdb/", $name))).unwrap()
        };
    }

    fn show_details(tmdb_id: usize, name: &str) -> TMDBTVShowDetails {
        serde_json::from_value(json!({ "id": tmdb_id, "name": name, "original_language": "de" })).unwrap()
    }

    #[test]
    fn show_decision() {
        let mut tv_shows = BTreeMap::new();

        // Unmatched: created without a tmdb_id, named after the file
        assert_eq!(upsert_show(&mut tv_shows, "Unknown Show", None), 1);
        assert_eq!(tv_shows[&1].tmdb_id, None);
        assert_eq!(tv_shows[&1].name.as_deref(), Some("Unknown Show"));

        // New ID = highest + 1, also with a gap below
        tv_shows.insert(5, TvShow { id: 5, ..Default::default() });
        let braunschlag = show_details(43879, "Braunschlag");
        assert_eq!(upsert_show(&mut tv_shows, "Braunschlag", Some(&braunschlag)), 6);
        assert_eq!(tv_shows[&6].tmdb_id, Some(43879));
        assert_eq!(tv_shows[&6].tmdb_language.as_deref(), Some("de"));

        // Known name
        assert_eq!(known_show_id(&tv_shows, "Braunschlag"), Some(6));
        assert_eq!(known_show_id(&tv_shows, "braunschlag 2012"), None);

        // Same tmdb_id under another name
        assert_eq!(upsert_show(&mut tv_shows, "braunschlag 2012", Some(&braunschlag)), 6);
        assert_eq!(tv_shows[&6].parsed_names, ["Braunschlag", "braunschlag 2012"]);

        // A retried unmatched show keeps its ID and gets matched
        assert_eq!(upsert_show(&mut tv_shows, "Unknown Show", Some(&show_details(7, "Found"))), 1);
        assert_eq!(tv_shows[&1].tmdb_id, Some(7));
        // Its name was set before, so it is kept
        assert_eq!(tv_shows[&1].name.as_deref(), Some("Unknown Show"));
        assert_eq!(tv_shows.len(), 3);
    }

    #[test]
    fn episode_is_picked_from_a_real_season_reply() {
        let season: TMDBSeasonDetails = fixture!("tv_season.json");
        assert_eq!(season.tmdb_season.season_number, Some(1));
        assert_eq!(season.episodes.len(), 8);

        let mut episode_meta = EpisodeMeta::default();
        fill_episode_with_details(&mut episode_meta, find_episode(&season, 3).unwrap(), false);
        assert_eq!(episode_meta.tmdb_id, Some(890561));
        assert_eq!(episode_meta.name.as_deref(), Some("Der Fluch"));
        assert_eq!((episode_meta.season, episode_meta.episode), (Some(1), Some(3)));
        assert!(episode_meta.still_path.unwrap().starts_with(TMDB_IMAGE_URL));

        assert!(find_episode(&season, 9).is_none());

        let unknown_season: TMDBSeasonDetails = fixture!("tv_season_unknown.json");
        assert!(unknown_season.episodes.is_empty());

        // A season the show does not list yet is added, a listed one only gets its count updated
        let mut show = TvShow { seasons: vec![TvSeason { season_number: 2, ..Default::default() }], ..Default::default() };
        fill_show_season(&mut show, &unknown_season);
        assert_eq!(show.seasons.len(), 1);
        fill_show_season(&mut show, &season);
        assert_eq!(show.seasons.iter().map(|season| season.season_number).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(show.seasons[0].episode_count, Some(8));
        assert_eq!(show.seasons[0].air_date.as_deref(), Some("2012-09-18"));

        show.seasons[0].episode_count = Some(6);
        show.seasons[0].name = Some("Edited".to_owned());
        fill_show_season(&mut show, &season);
        assert_eq!(show.seasons[0].episode_count, Some(8));
        assert_eq!(show.seasons[0].name.as_deref(), Some("Edited"));
    }

    /// Knows the shows it is given and finds one only by its exact name, so a name that is off
    /// finds nothing. Every season has 24 episodes. Counts the requests.
    #[derive(Default)]
    struct FakeTmdb {
        shows: Vec<serde_json::Value>,
        searches: Cell<u32>,
        show_details: Cell<u32>,
        seasons: Cell<u32>,
    }

    impl FakeTmdb {
        fn with_shows(shows: Vec<serde_json::Value>) -> Self {
            FakeTmdb { shows, ..Default::default() }
        }

        fn braunschlag() -> Self {
            Self::with_shows(vec![json!({ "id": 43879, "name": "Braunschlag", "original_language": "de" })])
        }

        fn requests(&self) -> (u32, u32, u32) {
            (self.searches.get(), self.show_details.get(), self.seasons.get())
        }
    }

    impl TmdbApi for FakeTmdb {
        async fn search_movie(&self, title: &str, _year: Option<i32>) -> Result<TMDBSearchMovieResult, Error> {
            panic!("unexpected movie search for '{}'", title)
        }

        async fn get_movie_details(&self, movie_id: usize) -> Result<TMDBMovieDetails, Error> {
            panic!("unexpected movie details request for {}", movie_id)
        }

        async fn search_tv_show(&self, show_name: &str, first_air_date_year: Option<i32>) -> Result<TMDBSearchTVResult, Error> {
            self.searches.set(self.searches.get() + 1);
            let results: Vec<_> = self.shows.iter()
                .filter(|show| show["name"].as_str().unwrap().eq_ignore_ascii_case(show_name))
                .filter(|show| first_air_date_year.map_or(true, |year| {
                    show["first_air_date"].as_str().is_some_and(|date| date.starts_with(&year.to_string()))
                }))
                .collect();
            Ok(serde_json::from_value(json!({ "page": 1, "results": results, "total_pages": 1, "total_results": results.len() })).unwrap())
        }

        async fn get_tv_show_details(&self, tv_id: usize) -> Result<TMDBTVShowDetails, Error> {
            self.show_details.set(self.show_details.get() + 1);
            let show = self.shows.iter().find(|show| show["id"] == tv_id).unwrap();
            Ok(serde_json::from_value(show.clone()).unwrap())
        }

        async fn get_tv_season_details(&self, _tv_id: usize, season_number: i32) -> Result<TMDBSeasonDetails, Error> {
            self.seasons.set(self.seasons.get() + 1);
            let episodes: Vec<_> = (1..=24)
                .map(|episode| json!({ "id": 1000 + episode, "episode_number": episode, "season_number": season_number }))
                .collect();
            Ok(serde_json::from_value(json!({ "season_number": season_number, "episodes": episodes })).unwrap())
        }
    }

    const LIBRARY_ROOTS: &[&str] = &["F:/mov", "T:/tv"];

    /// A file below the TV root
    fn show_file(path: &str) -> VideoElement {
        VideoElement { filepath: format!("T:/tv/{}", path), ..Default::default() }
    }

    fn episode_file(episode: i32) -> VideoElement {
        show_file(&format!("Braunschlag/Braunschlag S01E{:02}.mkv", episode))
    }

    fn episode_meta(video_element: &VideoElement) -> &EpisodeMeta {
        let Media::Episode(episode_meta) = &video_element.media else { panic!("expected an episode: {}", video_element.filepath) };
        episode_meta
    }

    fn candidate_ids(names: &[(&str, Option<i32>)], search_reply: &TMDBSearchTVResult) -> Vec<usize> {
        let names: Vec<(String, Option<i32>)> = names.iter().map(|(name, year)| (name.to_string(), *year)).collect();
        rank_show_candidates(&names, &search_reply.results).iter().filter_map(|(_, show)| show.id).collect()
    }

    #[test]
    fn show_candidates_from_real_search_replies() {
        let braunschlag: TMDBSearchTVResult = fixture!("tv_search_braunschlag.json");
        // The 2012 series has the name of the folder, "Braunschlag 1986" only starts with it
        assert_eq!(candidate_ids(&[("Braunschlag", None), ("tv4a braunschlag", None)], &braunschlag), [43879, 316811]);

        let the_office: TMDBSearchTVResult = fixture!("tv_search_the_office.json");
        // Same-named shows stay in TMDB's order, a year decides between them
        assert_eq!(candidate_ids(&[("The Office", None)], &the_office)[..2], [2316, 2996]);
        assert_eq!(candidate_ids(&[("The Office", Some(2001))], &the_office)[0], 2996);
        // A year no show has changes nothing
        assert_eq!(candidate_ids(&[("The Office", Some(1950))], &the_office)[0], 2316);
        // "Phantom of the Office"
        assert!(!candidate_ids(&[("The Office", None)], &the_office).contains(&249145));

        // A weak name match is no candidate
        assert!(candidate_ids(&[("MDE tv", None)], &the_office).is_empty());
        assert!(candidate_ids(&[("tv4a", None)], &braunschlag).is_empty());
        assert!(candidate_ids(&[], &braunschlag).is_empty());
    }

    #[test]
    fn owned_episodes_decide_between_equal_names() {
        let api = FakeTmdb::with_shows(vec![
            json!({ "id": 1, "name": "Nirvanna the Band the Show", "seasons": [{ "season_number": 1, "episode_count": 10 }] }),
            json!({ "id": 2, "name": "Nirvanna the Band the Show", "seasons": [
                { "season_number": 1, "episode_count": 6 },
                { "season_number": 2, "episode_count": 6 },
            ] }),
        ]);
        let names = [("Nirvanna the Band the Show".to_owned(), None)];
        let matched = |owned_episodes: &[(i32, i32)]| {
            tauri::async_runtime::block_on(match_show(&api, &names, &owned_episodes.iter().copied().collect()))
                .unwrap()
                .and_then(|tv_show_details| tv_show_details.tmdb_tv_show.id)
        };

        assert_eq!(matched(&[(1, 1), (2, 1)]), Some(2));
        assert_eq!(matched(&[(1, 8)]), Some(1));
        // Both hold the episodes: TMDB's order
        assert_eq!(matched(&[(1, 1), (1, 2)]), Some(1));
        // Absolute numbering is in no season list, which does not reject the show
        assert_eq!(matched(&[(1, 201)]), Some(1));
        assert_eq!(matched(&[]), Some(1));
    }

    #[test]
    fn show_folder_is_the_first_folder_below_a_root() {
        assert_eq!(show_folder(LIBRARY_ROOTS, "T:/tv/Stromberg/Staffel 1/a.mkv"), Some("Stromberg"));
        assert_eq!(show_folder(LIBRARY_ROOTS, "T:/tv/Stromberg/a.mkv"), Some("Stromberg"));
        assert_eq!(show_folder(LIBRARY_ROOTS, "T:/tv/a.mkv"), None);
        assert_eq!(show_folder(LIBRARY_ROOTS, "D:/other/Stromberg/a.mkv"), None);
    }

    #[test]
    fn a_folder_is_one_show() {
        // Four release groups, four names parsed from the filenames
        let stromberg_files = || -> Vec<VideoElement> {
            ["cfd-strombergdxvid", "exp-strombergxvid", "itg-stromberg", "utopia-stromberg"].iter().enumerate()
                .flat_map(|(group, name)| (1..=2).map(move |episode| {
                    show_file(&format!("Stromberg/Staffel 1/{}.s01e{:02}.mkv", name, group * 2 + episode))
                }))
                .collect()
        };

        // Only the folder name finds the show: the folder and two of the filename names are searched
        let api = FakeTmdb::with_shows(vec![json!({ "id": 6202, "name": "Stromberg", "original_language": "de" })]);
        let mut video_files = stromberg_files();
        let mut tv_shows = BTreeMap::new();
        tauri::async_runtime::block_on(parse_video_files(&api, LIBRARY_ROOTS, &mut video_files, &mut tv_shows, false)).unwrap();

        assert_eq!(api.requests(), (3, 1, 1));
        assert_eq!(tv_shows.len(), 1);
        assert_eq!(tv_shows[&1].tmdb_id, Some(6202));
        assert_eq!(tv_shows[&1].parsed_names, ["Stromberg"]);
        for (video_element, episode) in video_files.iter().zip(1..) {
            let episode_meta = episode_meta(video_element);
            assert_eq!((episode_meta.show_id, episode_meta.episode), (1, Some(episode)));
            assert!(episode_meta.tmdb_id.is_some());
        }

        // Not on TMDB: still one show, named after the folder
        let api = FakeTmdb::default();
        let mut video_files = stromberg_files();
        let mut tv_shows = BTreeMap::new();
        tauri::async_runtime::block_on(parse_video_files(&api, LIBRARY_ROOTS, &mut video_files, &mut tv_shows, false)).unwrap();

        assert_eq!(api.requests(), (3, 0, 0));
        assert_eq!(tv_shows.len(), 1);
        assert_eq!((tv_shows[&1].tmdb_id, tv_shows[&1].name.as_deref()), (None, Some("Stromberg")));
        for video_element in &video_files {
            assert_eq!(episode_meta(video_element).show_id, 1);
            assert!(video_element.tmdb_searched);
        }

        // Nothing is asked again, except by a reparse, and that once per show
        tauri::async_runtime::block_on(parse_video_files(&api, LIBRARY_ROOTS, &mut video_files, &mut tv_shows, false)).unwrap();
        assert_eq!(api.requests(), (3, 0, 0));
        tauri::async_runtime::block_on(parse_video_files(&api, LIBRARY_ROOTS, &mut video_files, &mut tv_shows, true)).unwrap();
        assert_eq!(api.requests(), (6, 0, 0));
        assert_eq!(tv_shows.len(), 1);
    }

    #[test]
    fn show_names_come_from_folder_and_filename() {
        let api = FakeTmdb::with_shows(vec![
            json!({ "id": 43879, "name": "Braunschlag" }),
            json!({ "id": 69900, "name": "Nirvanna the Band the Show" }),
            json!({ "id": 156861, "name": "The Curse", "first_air_date": "2022-02-06" }),
            json!({ "id": 114655, "name": "The Curse", "first_air_date": "2023-11-12" }),
            json!({ "id": 1667, "name": "Saturday Night Live" }),
            json!({ "id": 1400, "name": "Seinfeld" }),
        ]);
        let mut video_files = vec![
            // Release group in the filename
            show_file("Braunschlag/tv4a-braunschlag.s01e01-1080p.mkv"),
            // Abbreviation in the filename
            show_file("Nirvanna the Band the Show/NTBTS S1 E1.mp4"),
            // The year is no part of the name, and picks the show
            show_file("The Curse/The.Curse.2023.S01E01.1080p.mkv"),
            // Abbreviation in the folder name
            show_file("SNL/Saturday.Night.Live.S01E01.mkv"),
            // No show folder
            show_file("Seinfeld.S01E01.mkv"),
            show_file("Unknown.Show.S01E01.mkv"),
        ];
        let mut tv_shows = BTreeMap::new();
        tauri::async_runtime::block_on(parse_video_files(&api, LIBRARY_ROOTS, &mut video_files, &mut tv_shows, false)).unwrap();

        let matched: Vec<(&str, Option<usize>)> = video_files.iter()
            .map(|video_element| &tv_shows[&episode_meta(video_element).show_id])
            .map(|show| (show.parsed_names[0].as_str(), show.tmdb_id))
            .collect();
        assert_eq!(matched, [
            ("Braunschlag", Some(43879)),
            ("Nirvanna the Band the Show", Some(69900)),
            ("The Curse", Some(114655)),
            ("SNL", Some(1667)),
            ("Seinfeld", Some(1400)),
            ("Unknown Show", None),
        ]);
    }

    #[test]
    fn a_season_costs_three_requests() {
        let api = FakeTmdb::braunschlag();
        let mut video_files: Vec<VideoElement> = (1..=24).map(episode_file).collect();
        // Not part of the season on TMDB
        video_files.push(episode_file(25));
        let mut tv_shows = BTreeMap::new();

        tauri::async_runtime::block_on(parse_video_files(&api, LIBRARY_ROOTS, &mut video_files, &mut tv_shows, false)).unwrap();

        assert_eq!(api.requests(), (1, 1, 1));
        assert_eq!(tv_shows.len(), 1);
        // The fake's show details list no seasons, the season reply does
        assert_eq!(tv_shows[&1].seasons.len(), 1);
        assert_eq!(tv_shows[&1].seasons[0].episode_count, Some(24));
        for (video_element, episode) in video_files.iter().zip(1..=24) {
            let Media::Episode(episode_meta) = &video_element.media else { panic!("expected an episode") };
            assert_eq!(episode_meta.tmdb_id, Some(1000 + episode as usize));
            assert!(video_element.tmdb_searched);
            assert_eq!(video_element.tmdb_language.as_deref(), Some("de"));
        }

        let Media::Episode(missing) = &video_files[24].media else { panic!("expected an episode") };
        assert_eq!((missing.season, missing.episode, missing.tmdb_id), (Some(1), Some(25), None));
        assert!(video_files[24].tmdb_searched);

        // Everything has a definite result, so the next run asks nothing
        tauri::async_runtime::block_on(parse_video_files(&api, LIBRARY_ROOTS, &mut video_files, &mut tv_shows, false)).unwrap();
        assert_eq!(api.requests(), (1, 1, 1));
    }

    fn episode_of(show_id: u32, episode: i32) -> VideoElement {
        VideoElement {
            filepath: format!("T:/tv/Show {}/S01E{:02}.mkv", show_id, episode),
            tmdb_language: Some("de".to_owned()),
            tmdb_searched: true,
            media: Media::Episode(EpisodeMeta {
                show_id, season: Some(1), episode: Some(episode), tmdb_id: Some(5),
                name: Some("Old".to_owned()), still_path: Some("/old.jpg".to_owned()), ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn film_file() -> VideoElement {
        VideoElement {
            filepath: "F:/mov/film.mkv".to_owned(),
            tmdb_language: Some("fr".to_owned()),
            media: Media::Film(FilmMeta { tmdb_id: Some(1), title: Some("F".to_owned()), ..Default::default() }),
            ..Default::default()
        }
    }

    fn matched_show(id: u32, tmdb_id: usize, name: &str) -> TvShow {
        TvShow { id, tmdb_id: Some(tmdb_id), parsed_names: vec![name.to_owned()], name: Some(name.to_owned()), tmdb_language: Some("de".to_owned()), ..Default::default() }
    }

    fn seinfeld() -> FakeTmdb {
        FakeTmdb::with_shows(vec![json!({ "id": 1400, "name": "Seinfeld", "original_language": "en" })])
    }

    #[test]
    fn rematch_show_replaces_the_show_and_refills_its_episodes() {
        let api = seinfeld();
        let mut tv_shows = BTreeMap::from([(1, matched_show(1, 43879, "Braunschlag")), (2, matched_show(2, 7, "Other"))]);
        let mut video_files = vec![episode_of(1, 1), episode_of(1, 2), film_file(), episode_of(2, 1)];
        let (film_before, other_before) = (video_files[2].clone(), video_files[3].clone());

        let show_id = tauri::async_runtime::block_on(rematch_show_with(&api, &mut video_files, &mut tv_shows, 1, 1400)).unwrap();

        assert_eq!(show_id, 1);
        assert_eq!(api.requests(), (0, 1, 1));
        let show = &tv_shows[&1];
        assert_eq!((show.tmdb_id, show.name.as_deref(), show.tmdb_language.as_deref()), (Some(1400), Some("Seinfeld"), Some("en")));
        assert_eq!(show.parsed_names, ["Braunschlag"]);
        for (video_element, episode) in video_files[..2].iter().zip(1..) {
            let episode_meta = episode_meta(video_element);
            assert_eq!((episode_meta.show_id, episode_meta.season, episode_meta.episode), (1, Some(1), Some(episode)));
            assert_eq!(episode_meta.tmdb_id, Some(1000 + episode as usize));
            // The fake's episodes have no name or still: the old ones must be gone
            assert_eq!((episode_meta.name.as_deref(), episode_meta.still_path.as_deref()), (None, None));
            assert_eq!(video_element.tmdb_language.as_deref(), Some("en"));
            assert!(video_element.playback_selection.is_some());
        }

        // Other shows and films are untouched
        assert_eq!(tv_shows[&2], matched_show(2, 7, "Other"));
        assert_eq!((video_files[2].media.clone(), video_files[2].tmdb_language.clone()), (film_before.media, film_before.tmdb_language));
        assert_eq!(video_files[3].media, other_before.media);

        assert!(tauri::async_runtime::block_on(rematch_show_with(&api, &mut video_files, &mut tv_shows, 9, 1400)).is_err());
    }

    #[test]
    fn rematch_onto_a_show_of_the_library_merges() {
        let api = seinfeld();
        let mut tv_shows = BTreeMap::from([(1, matched_show(1, 43879, "Braunschlag")), (2, matched_show(2, 1400, "Seinfeld"))]);
        let mut video_files = vec![episode_of(1, 1), episode_of(2, 2)];
        let kept_episode = video_files[1].clone();

        let show_id = tauri::async_runtime::block_on(rematch_show_with(&api, &mut video_files, &mut tv_shows, 1, 1400)).unwrap();

        assert_eq!(show_id, 2);
        // The library's show is kept as it is, no show details needed
        assert_eq!(api.requests(), (0, 0, 1));
        assert_eq!(tv_shows.keys().collect::<Vec<_>>(), [&2]);
        assert_eq!(tv_shows[&2].parsed_names, ["Seinfeld", "Braunschlag"]);
        assert_eq!(episode_meta(&video_files[0]).show_id, 2);
        assert_eq!(episode_meta(&video_files[0]).tmdb_id, Some(1001));
        assert_eq!(video_files[1].media, kept_episode.media);
    }

    #[test]
    fn a_show_is_added_from_tmdb_once() {
        let api = seinfeld();
        let mut tv_shows = BTreeMap::from([(4, matched_show(4, 43879, "Braunschlag"))]);
        let add = |tv_shows: &mut BTreeMap<u32, TvShow>, tmdb_id| tauri::async_runtime::block_on(find_or_add_tmdb_show(&api, tv_shows, tmdb_id)).unwrap();

        assert_eq!(add(&mut tv_shows, 43879), 4);
        assert_eq!(api.requests(), (0, 0, 0));
        assert_eq!(add(&mut tv_shows, 1400), 5);
        assert_eq!((tv_shows[&5].tmdb_id, tv_shows[&5].name.as_deref()), (Some(1400), Some("Seinfeld")));
        assert!(tv_shows[&5].parsed_names.is_empty());
        assert_eq!(add(&mut tv_shows, 1400), 5);
        assert_eq!(api.requests(), (0, 1, 0));
    }

    #[test]
    fn media_set_by_the_user() {
        let api = FakeTmdb::default();
        let mut tv_shows = BTreeMap::from([(1, matched_show(1, 43879, "Braunschlag")), (2, TvShow { id: 2, ..Default::default() })]);
        let apply = |video_element: &mut VideoElement, tv_shows: &mut BTreeMap<u32, TvShow>, media: Media| {
            tauri::async_runtime::block_on(apply_media(Some(&api), video_element, tv_shows, media))
        };
        let new_episode = |show_id, episode| Media::Episode(EpisodeMeta { show_id, season: Some(1), episode, ..Default::default() });

        // A film becomes an episode: looked up with its show and numbers, the show's language applies
        let mut video_element = film_file();
        video_element.tmdb_searched = false;
        assert!(needs_episode_lookup(&tv_shows, &new_episode(1, Some(3))));
        apply(&mut video_element, &mut tv_shows, new_episode(1, Some(3))).unwrap();
        assert_eq!(episode_meta(&video_element).tmdb_id, Some(1003));
        assert_eq!(video_element.tmdb_language.as_deref(), Some("de"));
        assert!(video_element.tmdb_searched && video_element.playback_selection.is_some());
        assert_eq!(api.requests(), (0, 0, 1));

        // An episode that has its details, one without episode number and one of an unmatched show are not looked up
        let edited = Media::Episode(EpisodeMeta { name: Some("Mine".to_owned()), ..episode_meta(&video_element).clone() });
        apply(&mut video_element, &mut tv_shows, edited.clone()).unwrap();
        assert_eq!(video_element.media, edited);
        for media in [new_episode(1, None), new_episode(2, Some(3))] {
            assert!(!needs_episode_lookup(&tv_shows, &media));
            apply(&mut video_element, &mut tv_shows, media).unwrap();
        }
        assert_eq!(api.requests(), (0, 0, 1));
        // An unmatched show has no language
        assert_eq!(video_element.tmdb_language, None);

        // Back to a film: the show's language goes
        video_element.tmdb_language = Some("de".to_owned());
        apply(&mut video_element, &mut tv_shows, Media::Film(FilmMeta::default())).unwrap();
        assert_eq!(video_element.tmdb_language, None);
        // A film stays a film: its language stays
        video_element.tmdb_language = Some("fr".to_owned());
        apply(&mut video_element, &mut tv_shows, Media::Film(FilmMeta { title: Some("T".to_owned()), ..Default::default() })).unwrap();
        assert_eq!(video_element.tmdb_language.as_deref(), Some("fr"));

        // Unknown show: nothing changes
        let before = video_element.clone();
        assert!(apply(&mut video_element, &mut tv_shows, new_episode(9, Some(1))).is_err());
        assert_eq!(video_element.media, before.media);
    }

    #[test]
    fn tv_search_returns_shows() {
        let reply: TMDBSearchTVResult = fixture!("tv_search_braunschlag.json");
        let shows: Vec<TvShow> = reply.results.iter().map(show_from_search_result).collect();
        assert_eq!(shows.iter().map(|show| show.tmdb_id).take(2).collect::<Vec<_>>(), [Some(43879), Some(316811)]);
        assert_eq!((shows[0].name.as_deref(), shows[0].first_air_date.as_deref()), (Some("Braunschlag"), Some("2012-09-18")));
        assert!(shows[0].poster_path.as_ref().unwrap().starts_with(TMDB_IMAGE_URL));
        // Not part of the library
        assert_eq!(shows[0].id, 0);

        let api = seinfeld();
        let found = tauri::async_runtime::block_on(search_shows(&api, "Seinfeld")).unwrap();
        assert_eq!(found.iter().map(|show| show.tmdb_id).collect::<Vec<_>>(), [Some(1400)]);
    }

    #[test]
    fn an_episode_without_numbers_is_kept() {
        let api = FakeTmdb::braunschlag();
        let mut show = TvShow { id: 1, tmdb_id: Some(43879), ..Default::default() };
        let mut parse = |filepath: &str| {
            let mut video_element = VideoElement { filepath: filepath.to_owned(), ..Default::default() };
            let (_, season, episode) = get_tv_show_info_from_filename(filepath);
            tauri::async_runtime::block_on(parse_episode(
                &api, &mut show, &mut SeasonCache::new(), &mut video_element, season, episode,
            ));
            assert!(video_element.tmdb_searched, "{}", filepath);
            let Media::Episode(episode_meta) = video_element.media else { panic!("expected an episode: {}", filepath) };
            episode_meta
        };

        let season_only = parse("T:/tv/Braunschlag/Season 2/Pilot.mkv");
        let no_number = parse("T:/tv/Braunschlag/Braunschlag Making Of.mkv");
        let special = parse("T:/tv/Braunschlag/Braunschlag Special.mkv");

        assert_eq!((season_only.season, season_only.episode, season_only.tmdb_id), (Some(2), None, None));
        assert_eq!((no_number.season, no_number.episode, no_number.tmdb_id), (None, None, None));
        assert_eq!((special.season, special.episode, special.tmdb_id), (Some(0), None, None));

        assert_eq!([season_only.show_id, no_number.show_id, special.show_id], [1, 1, 1]);
        // No season is fetched for a file without episode number
        assert_eq!(api.requests(), (0, 0, 0));
    }
}
