use crate::tmdb::MovieSearchResponse;
use std::time::Instant;

pub struct State {
    pub search_text: String,
    pub movie_search_response: Option<MovieSearchResponse>,
    pub last_change_time: Instant,
}

impl State {
    pub fn new() -> Self {
        Self {
            search_text: String::new(),
            movie_search_response: None,
            last_change_time: Instant::now(),
        }
    }
}

pub type StateMutation = Box<dyn Fn(&mut State) + Send + 'static>;

pub fn movie_search_mutation(resp: MovieSearchResponse) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_search_response = Some(resp.clone()))
}
