use reqwest::{
    ClientBuilder, Method,
    header::{self, HeaderMap, HeaderValue},
};
use serde::Deserialize;

#[derive(Clone)]
pub struct TmdbClient {
    client: reqwest::Client,
}
#[derive(Deserialize, Debug, Clone)]
pub struct MovieSearchResult {
    pub id: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MovieSearchResponse {
    pub results: Vec<MovieSearchResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MovieDetailsResponse {
    pub overview: Option<String>,
    pub runtime: u64,
    pub original_title: String,
    pub title: String,
    pub poster_path: Option<String>,
    pub release_date: Option<String>,
    pub tagline: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MovieCastMember {
    pub name: String,
    pub profile_path: Option<String>,
    pub character: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MovieCreditsResponse {
    pub cast: Vec<MovieCastMember>,
}

pub const ENGLISH: &str = "en-US";
pub const GERMAN: &str = "de-DE";

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
            .query(&[("query", search_text)])
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

    pub async fn movie_details(&self, id: u64, language: &str) -> MovieDetailsResponse {
        let request = self
            .client
            .request(
                Method::GET,
                format!("https://api.themoviedb.org/3/movie/{id}"),
            )
            .query(&[("language", language)])
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

    pub async fn movie_credits(&self, id: u64) -> MovieCreditsResponse {
        self.client
            .get(format!("https://api.themoviedb.org/3/movie/{id}/credits"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}
