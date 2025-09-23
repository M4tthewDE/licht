use crate::tmdb::{MovieCreditsResponse, MovieDetailsResponse, MovieSearchResponse};
use std::time::Instant;

// TODO: define own structs instead of using tmdb responses
#[derive(Default)]
pub struct State {
    pub search_text: String,
    pub movie_search_response: Option<MovieSearchResponse>,
    pub last_change_time: Option<Instant>,
    pub movie_details: Vec<MovieDetailsResponse>,
    pub movie_credits: Vec<MovieCreditsResponse>,
    pub current_movie: Option<u64>,
}

impl State {
    pub fn details(&self, id: u64) -> Option<MovieDetailsResponse> {
        for details in &self.movie_details {
            if details.id == id {
                return Some(details.clone());
            }
        }

        None
    }

    pub fn credits(&self, id: u64) -> Option<MovieCreditsResponse> {
        for credits in &self.movie_credits {
            if credits.id == id {
                return Some(credits.clone());
            }
        }

        None
    }
}

pub type StateMutation = Box<dyn Fn(&mut State) + Send + 'static>;

pub fn movie_search_mutation(resp: MovieSearchResponse) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_search_response = Some(resp.clone()))
}

pub fn movie_details_mutation(resp: MovieDetailsResponse) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_details.push(resp.clone()))
}

pub fn movie_credits_mutation(resp: MovieCreditsResponse) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_credits.push(resp.clone()))
}
