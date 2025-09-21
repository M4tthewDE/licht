use reqwest::{
    ClientBuilder, Method,
    header::{self, HeaderMap, HeaderValue},
};
use serde::Deserialize;

#[derive(Clone)]
pub struct TmdbClient {
    client: reqwest::Client,
}
#[derive(Deserialize, Debug)]
pub struct MovieSearchResult {
    pub original_title: String,
    pub release_date: String,
    pub poster_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MovieSearchResponse {
    pub results: Vec<MovieSearchResult>,
}

impl TmdbClient {
    pub fn new(token: String) -> Self {
        let mut headers = HeaderMap::new();

        let auth_value = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();
        headers.insert(header::AUTHORIZATION, auth_value);

        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client }
    }

    pub async fn search_movies(&self, search_text: &str) -> MovieSearchResponse {
        let request = self
            .client
            .request(Method::GET, "https://api.themoviedb.org/3/search/movie")
            .query(&[("language", "en-US"), ("page", "1"), ("query", search_text)])
            .build()
            .unwrap();

        self.client
            .execute(request)
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}
