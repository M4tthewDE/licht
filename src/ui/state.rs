use crate::ui::tmdb::{MovieCastMember, MovieCreditsResponse, MovieDetailsResponse};
use std::time::Instant;

#[derive(Clone)]
pub struct MovieSearch {
    pub details: MovieDetails,
    pub german_details: MovieDetails,
    pub credits: MovieCredits,
}

impl MovieSearch {
    pub fn new(
        details: MovieDetailsResponse,
        german_details: MovieDetailsResponse,
        credits: MovieCreditsResponse,
    ) -> Self {
        Self {
            details: details.into(),
            german_details: german_details.into(),
            credits: credits.into(),
        }
    }
}

#[derive(Clone)]
pub struct MovieDetails {
    pub original_title: String,
    pub title: String,
    pub release_date: Option<String>,
    pub poster_url: String,
    pub runtime: u64,
    pub tagline: Option<String>,
    pub overview: Option<String>,
}

impl From<MovieDetailsResponse> for MovieDetails {
    fn from(details: MovieDetailsResponse) -> Self {
        Self {
            original_title: details.original_title,
            title: details.title,
            release_date: details.release_date,
            poster_url: build_poster_url(details.poster_path),
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
    pub character: String,
}

impl From<MovieCastMember> for MovieCredit {
    fn from(cast_member: MovieCastMember) -> Self {
        Self {
            name: cast_member.name,
            profile_photo_url: build_poster_url(cast_member.profile_path),
            character: cast_member.character,
        }
    }
}

#[derive(Clone)]
pub struct MovieCredits {
    pub credits: Vec<MovieCredit>,
}

impl From<MovieCreditsResponse> for MovieCredits {
    fn from(credits: MovieCreditsResponse) -> Self {
        Self {
            credits: credits.cast.iter().map(|c| c.clone().into()).collect(),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub search_text: String,
    pub movie_searches: Vec<MovieSearch>,
    pub last_change_time: Option<Instant>,
    pub current_movie: Option<MovieDetails>,
}

pub type StateMutation = Box<dyn Fn(&mut State) + Send + 'static>;

pub fn movie_search_mutation(movie_searches: Vec<MovieSearch>) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_searches = movie_searches.clone())
}

fn build_poster_url(poster_path: Option<String>) -> String {
    if let Some(poster_path) = poster_path {
        format!(
            "https://image.tmdb.org/t/p/w600_and_h900_bestv2{}",
            poster_path
        )
    } else {
        "https://www.themoviedb.org/assets/2/v4/glyphicons/basic/glyphicons-basic-38-picture-grey-c2ebdbb057f2a7614185931650f8cee23fa137b93812ccb132b9df511df1cfac.svg".to_string()
    }
}
