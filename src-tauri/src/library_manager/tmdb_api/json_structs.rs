use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct test_authentification {
    pub(crate) success: bool,
}


#[derive(Deserialize, Debug)]
pub(crate) struct search_movie_res {
    pub(crate) page: usize,
    pub(crate) results: Vec<search_movie_result_element>,
    pub(crate) total_pages: usize,
    pub(crate) total_results: usize,
}


#[derive(Deserialize, Debug)]
pub(crate) struct search_movie_result_element {
    pub(crate) adult: Option<bool>,
    pub(crate) backdrop_path: Option<String>,
    pub(crate) genre_ids: Option<Vec<usize>>,
    pub(crate) id: Option<usize>,
    pub(crate) original_language: Option<String>,
    pub(crate) original_title: Option<String>,
    pub(crate) overview: Option<String>,
    pub(crate) popularity: Option<f32>,
    pub(crate) poster_path: Option<String>,
    pub(crate) release_date: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) video: Option<bool>,
    pub(crate) vote_average: Option<f32>,
    pub(crate) vote_count: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct MovieDetails {
    pub(crate) adult: Option<bool>,
    pub(crate) backdrop_path: Option<String>,
    pub(crate) belongs_to_collection: Option<Collection>,
    pub(crate) budget: Option<usize>,
    pub(crate) genres: Option<Vec<Genre>>,
    pub(crate) homepage: Option<String>,
    pub(crate) id: Option<usize>,
    pub(crate) imdb_id: Option<String>,
    pub(crate) origin_country: Option<Vec<String>>,
    pub(crate) original_language: Option<String>,
    pub(crate) original_title: Option<String>,
    pub(crate) overview: Option<String>,
    pub(crate) popularity: Option<f32>,
    pub(crate) poster_path: Option<String>,
    pub(crate) production_companies: Option<Vec<ProductionCompanies>>,
    pub(crate) production_countries: Option<Vec<ProductionCountries>>,
    pub(crate) release_date: Option<String>,
    pub(crate) revenue: Option<usize>,
    pub(crate) runtime: Option<usize>,
    pub(crate) spoken_languages: Option<Vec<SpokenLanguage>>,
    pub(crate) status: Option<String>,
    pub(crate) tagline: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) video: Option<bool>,
    pub(crate) vote_average: Option<f32>,
    pub(crate) vote_count: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Collection {
    pub(crate) backdrop_path: Option<String>,
    pub(crate) id: Option<usize>,
    pub(crate) name: Option<String>,
    pub(crate) poster_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Genre {
    pub(crate) id: Option<usize>,
    pub(crate) name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ProductionCompanies {
    pub(crate) id: Option<usize>,
    pub(crate) logo_path: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) origin_country: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ProductionCountries {
    pub(crate) iso_3166_1: Option<String>,
    pub(crate) name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct SpokenLanguage {
    pub(crate) english_name: Option<String>,
    pub(crate) iso_639_1: Option<String>,
    pub(crate) name: Option<String>,
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