use anyhow::{Error, anyhow};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(super) struct TMDBTestAuthentification {
    pub success: bool,
}


#[derive(Deserialize, Debug)]
pub(super) struct TMDBSearchMovieResult {
    pub page: usize,
    pub results: Vec<TMDBMovie>,
    pub total_pages: usize,
    pub total_results: usize,
}


#[derive(Deserialize, Debug)]
pub(super) struct TMDBMovie {
    pub adult: Option<bool>,
    pub backdrop_path: Option<String>,
    pub genre_ids: Option<Vec<usize>>,
    pub id: Option<usize>,
    pub original_language: Option<String>,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub popularity: Option<f32>,
    pub poster_path: Option<String>,
    pub release_date: Option<String>,
    pub title: Option<String>,
    pub video: Option<bool>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBMovieDetails {
    #[serde(flatten)]
    pub tmdb_movie: TMDBMovie,
    
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
    pub tagline: Option<String>,

    // Possible appends see https://developer.themoviedb.org/docs/append-to-response
    pub credits: Option<TMDBCredits>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBCollection {
    pub backdrop_path: Option<String>,
    pub id: Option<usize>,
    pub name: Option<String>,
    pub poster_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBGenre {
    pub id: Option<usize>,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBProductionCompanies {
    pub id: Option<usize>,
    pub logo_path: Option<String>,
    pub name: Option<String>,
    pub origin_country: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBProductionCountries {
    pub iso_3166_1: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBSpokenLanguage {
    pub english_name: Option<String>,
    pub iso_639_1: Option<String>,
    pub name: Option<String>,
}


#[derive(Deserialize, Debug)]
pub(super) struct TMDBCredits {
    pub cast: Option<Vec<TMDBCast>>,
    pub crew: Option<Vec<TMDBCrew>>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBPerson {
    pub adult: Option<bool>,
    pub gender: Option<usize>,
    pub id: Option<usize>,
    pub known_for_department: Option<String>,
    pub name: Option<String>,
    pub original_name: Option<String>,
    pub popularity: Option<f64>,
    pub profile_path: Option<String>,
    pub credit_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBCast {
    #[serde(flatten)]
    pub tmdb_person: TMDBPerson,
    
    pub cast_id: Option<usize>,
    pub character: Option<String>,
    pub order: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBCrew {
    #[serde(flatten)]
    pub tmdb_person: TMDBPerson,
    
    pub department: Option<String>,
    pub job: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBImages {
    pub id: Option<i32>,
    pub backdrops: Option<Vec<TMDBImage>>,
    pub logos: Option<Vec<TMDBImage>>,
    pub posters: Option<Vec<TMDBImage>>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBImage {
    pub aspect_ratio: Option<f64>,
    pub height: Option<i32>,
    pub iso_639_1: Option<String>,
    pub file_path: Option<String>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i32>,
    pub width: Option<i32>,
}



// Helpers
use tauri_plugin_http::reqwest::Response;

// TV Show Structs
#[derive(Deserialize, Debug)]
pub(super) struct TMDBSearchTVResult {
    pub page: usize,
    pub results: Vec<TMDBTVShow>,
    pub total_pages: usize,
    pub total_results: usize,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBTVShow {
    pub adult: Option<bool>,
    pub backdrop_path: Option<String>,
    pub genre_ids: Option<Vec<usize>>,
    pub id: Option<usize>,
    pub origin_country: Option<Vec<String>>,
    pub original_language: Option<String>,
    pub original_name: Option<String>,
    pub overview: Option<String>,
    pub popularity: Option<f32>,
    pub poster_path: Option<String>,
    pub first_air_date: Option<String>,
    pub name: Option<String>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBTVShowDetails {
    #[serde(flatten)]
    pub tmdb_tv_show: TMDBTVShow,

    pub created_by: Option<Vec<TMDBCreator>>,
    pub episode_run_time: Option<Vec<i32>>,
    pub genres: Option<Vec<TMDBGenre>>,
    pub homepage: Option<String>,
    pub in_production: Option<bool>,
    pub languages: Option<Vec<String>>,
    pub last_air_date: Option<String>,
    pub number_of_episodes: Option<i32>,
    pub number_of_seasons: Option<i32>,
    pub production_companies: Option<Vec<TMDBProductionCompanies>>,
    pub production_countries: Option<Vec<TMDBProductionCountries>>,
    pub seasons: Option<Vec<TMDBSeason>>,
    pub status: Option<String>,
    pub tagline: Option<String>,
    pub type_field: Option<String>,

    // Possible appends
    pub credits: Option<TMDBCredits>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBCreator {
    pub id: Option<usize>,
    pub credit_id: Option<String>,
    pub name: Option<String>,
    pub gender: Option<usize>,
    pub profile_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBSeason {
    pub air_date: Option<String>,
    pub episode_count: Option<i32>,
    pub id: Option<usize>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub season_number: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TMDBEpisodeDetails {
    pub air_date: Option<String>,
    pub episode_number: Option<i32>,
    pub id: Option<usize>,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub production_code: Option<String>,
    pub runtime: Option<i32>,
    pub season_number: Option<i32>,
    pub show_id: Option<usize>,
    pub still_path: Option<String>,
    pub vote_average: Option<f32>,
    pub vote_count: Option<usize>,
    pub crew: Option<Vec<TMDBCrew>>,
    pub guest_stars: Option<Vec<TMDBCast>>,
}


pub(super) async fn print_response_json(response: Response) -> Result<(), Error> {
    let json_value: serde_json::Value = response.json().await
        .map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

    // Print the JSON in a pretty format to see the structure
    println!("JSON Response:\n{}", serde_json::to_string_pretty(&json_value).unwrap());
    Ok(())
}