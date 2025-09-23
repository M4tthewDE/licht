use crate::tmdb::{MovieCastMember, MovieCreditsResponse, MovieDetailsResponse, MovieSearchResult};
use std::time::Instant;

use super::util;

pub type MovieId = u64;

#[derive(Clone)]
pub struct MovieSearch {
    pub id: MovieId,
    pub original_title: String,
    pub release_date: Option<String>,
    pub poster_url: String,
}

impl From<MovieSearchResult> for MovieSearch {
    fn from(search: MovieSearchResult) -> Self {
        Self {
            id: search.id,
            original_title: search.original_title,
            release_date: search.release_date,
            poster_url: util::build_poster_url(search.poster_path),
        }
    }
}

#[derive(Clone)]
pub struct MovieDetails {
    pub id: MovieId,
    pub original_title: String,
    pub release_date: Option<String>,
    pub poster_url: String,
    pub runtime: u64,
    pub tagline: Option<String>,
    pub overview: Option<String>,
}

impl From<MovieDetailsResponse> for MovieDetails {
    fn from(details: MovieDetailsResponse) -> Self {
        Self {
            id: details.id,
            original_title: details.original_title,
            release_date: details.release_date,
            poster_url: util::build_poster_url(details.poster_path),
            runtime: details.runtime,
            tagline: details.tagline,
            overview: details.overview,
        }
    }
}

#[derive(Clone)]
pub struct MovieCredit {
    pub name: String,
    pub profile_photo_url: String,
}

impl From<MovieCastMember> for MovieCredit {
    fn from(cast_member: MovieCastMember) -> Self {
        Self {
            name: cast_member.name,
            profile_photo_url: util::build_poster_url(cast_member.profile_path),
        }
    }
}

#[derive(Clone)]
pub struct MovieCredits {
    pub movie_id: MovieId,
    pub credits: Vec<MovieCredit>,
}

impl From<MovieCreditsResponse> for MovieCredits {
    fn from(credits: MovieCreditsResponse) -> Self {
        Self {
            movie_id: credits.id,
            credits: credits.cast.iter().map(|c| c.clone().into()).collect(),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub search_text: String,
    pub movie_searches: Vec<MovieSearch>,
    pub last_change_time: Option<Instant>,
    pub movie_details: Vec<MovieDetails>,
    pub movie_credits: Vec<MovieCredits>,
    pub current_movie: Option<MovieId>,
}

impl State {
    pub fn details(&self, id: u64) -> Option<MovieDetails> {
        for details in &self.movie_details {
            if details.id == id {
                return Some(details.clone());
            }
        }

        None
    }

    pub fn credits(&self, id: MovieId) -> Option<MovieCredits> {
        for credits in &self.movie_credits {
            if credits.movie_id == id {
                return Some(credits.clone());
            }
        }

        None
    }
}

pub type StateMutation = Box<dyn Fn(&mut State) + Send + 'static>;

pub fn movie_search_mutation(movie_searches: Vec<MovieSearch>) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_searches = movie_searches.clone())
}

pub fn movie_details_mutation(movie_details: MovieDetails) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_details.push(movie_details.clone()))
}

pub fn movie_credits_mutation(movie_credits: MovieCredits) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_credits.push(movie_credits.clone()))
}
