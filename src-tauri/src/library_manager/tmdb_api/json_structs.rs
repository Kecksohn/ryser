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

pub(super) async fn print_response_json(response: Response) -> Result<(), String> {
    let json_value: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // Print the JSON in a pretty format to see the structure
    println!("JSON Response:\n{}", serde_json::to_string_pretty(&json_value).unwrap());
    Ok(())
}