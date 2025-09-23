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
    pub original_title: String,
    pub release_date: Option<String>,
    pub poster_path: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MovieSearchResponse {
    pub results: Vec<MovieSearchResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MovieDetailsResponse {
    pub id: u64,
    pub overview: Option<String>,
    pub runtime: u64,
    pub original_title: String,
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
    pub id: u64,
    pub cast: Vec<MovieCastMember>,
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

    pub async fn movie_details(&self, id: u64) -> MovieDetailsResponse {
        self.client
            .get(format!("https://api.themoviedb.org/3/movie/{}", id))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    pub async fn movie_credits(&self, id: u64) -> MovieCreditsResponse {
        self.client
            .get(format!("https://api.themoviedb.org/3/movie/{}/credits", id))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}
