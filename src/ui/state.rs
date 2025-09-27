use serde::Deserialize;
use walkers::{
    HttpTiles, MapMemory,
    sources::{Mapbox, MapboxStyle},
};
use zip::ZipArchive;

use crate::ui::tmdb::{MovieCastMember, MovieCreditsResponse, MovieDetailsResponse};
use std::{io::Cursor, time::Instant};

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

pub struct State {
    pub search_text: String,
    pub movie_searches: Vec<MovieSearch>,
    pub last_change_time: Option<Instant>,
    pub current_movie: Option<MovieDetails>,
    pub show_map: bool,
    pub tiles: HttpTiles,
    pub map_memory: MapMemory,
    pub stops: Vec<Stop>,
}

impl State {
    pub fn new(token: String, ctx: egui::Context) -> Self {
        Self {
            search_text: String::new(),
            movie_searches: Vec::new(),
            last_change_time: None,
            current_movie: None,
            show_map: false,
            tiles: HttpTiles::new(
                Mapbox {
                    style: MapboxStyle::Dark,
                    high_resolution: false,
                    access_token: token,
                },
                ctx,
            ),
            map_memory: MapMemory::default(),
            stops: Vec::new(),
        }
    }
}

pub type StateMutation = Box<dyn Fn(&mut State) + Send + 'static>;

pub fn movie_search_mutation(movie_search: MovieSearch) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_searches.push(movie_search.clone()))
}

pub fn stops_mutation(stops: Vec<Stop>) -> StateMutation {
    Box::new(move |state: &mut State| state.stops = stops.clone())
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

const VBK_ZONE_ID: &str = "0100";

#[derive(Deserialize, Clone)]
pub struct Stop {
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
    pub zone_id: String,
}

pub async fn load_stops() -> Vec<Stop> {
    let resp = reqwest::get("https://projekte.kvv-efa.de/GTFS/google_transit.zip")
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let reader = Cursor::new(resp);

    let mut zip = ZipArchive::new(reader).unwrap();
    let mut stops_file = zip.by_name("stops.txt").unwrap();
    let mut stops_content = Vec::new();
    std::io::copy(&mut stops_file, &mut stops_content).unwrap();

    let mut csv_reader = csv::Reader::from_reader(Cursor::new(stops_content));
    let mut stops: Vec<Stop> = csv_reader.deserialize().map(|r| r.unwrap()).collect();
    stops.sort_by_key(|s| s.stop_name.clone());
    stops.dedup_by_key(|s| s.stop_name.clone());
    stops
        .into_iter()
        .filter(|s| s.zone_id == VBK_ZONE_ID)
        .collect()
}
