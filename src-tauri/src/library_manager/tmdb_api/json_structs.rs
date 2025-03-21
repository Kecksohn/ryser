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