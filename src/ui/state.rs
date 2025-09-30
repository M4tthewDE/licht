use tracing::info;
use walkers::{
    HttpTiles, MapMemory,
    sources::{Mapbox, MapboxStyle},
};

use crate::ui::{
    gtfs::TransitData,
    tmdb::{MovieCastMember, MovieCreditsResponse, MovieDetailsResponse},
};
use std::{collections::HashSet, time::Instant};

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
    pub routes: Vec<Route>,
    pub current_route: Option<Route>,
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
            routes: Vec::new(),
            current_route: None,
        }
    }
}

pub type StateMutation = Box<dyn Fn(&mut State) + Send + 'static>;

pub fn movie_search_mutation(movie_search: MovieSearch) -> StateMutation {
    Box::new(move |state: &mut State| state.movie_searches.push(movie_search.clone()))
}

pub fn routes_mutation(routes: Vec<Route>) -> StateMutation {
    Box::new(move |state: &mut State| {
        state.routes = routes.clone();
        state.current_route = routes.first().cloned();
    })
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

#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    pub name: String,
    pub lon: f64,
    pub lat: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub stations: Vec<Station>,
    pub name: String,
}

impl Route {
    async fn new(td: &TransitData, route_id: &str, name: String) -> Self {
        let trip_ids: HashSet<&String> = td
            .trips
            .iter()
            .filter(|t| t.route_id == route_id)
            .map(|t| &t.trip_id)
            .collect();

        let stop_ids: HashSet<&String> = td
            .stop_times
            .iter()
            .filter(|st| trip_ids.contains(&st.trip_id))
            .map(|st| &st.stop_id)
            .collect();

        let stations: Vec<Station> = td
            .stops
            .iter()
            .filter(|s| stop_ids.contains(&s.stop_id))
            .map(|s| Station {
                name: s.stop_name.clone(),
                lon: s.stop_lon,
                lat: s.stop_lat,
            })
            .collect();

        info!(name);
        calculate_geopoints(&stations).await;
        Route { stations, name }
    }
}

#[tracing::instrument(skip(transit_data))]
pub async fn load_routes(transit_data: &TransitData) -> Vec<Route> {
    let mut routes = Vec::new();
    for r in &transit_data.routes {
        routes.push(Route::new(transit_data, &r.route_id, r.route_short_name.clone()).await);
    }

    routes
}

#[tracing::instrument(skip(stations))]
async fn calculate_geopoints(stations: &[Station]) {
    let bounding_box = bounding_box(stations);
    info!(
        "{},{}",
        bounding_box.top_left.lat, bounding_box.top_left.lon
    );
    info!(
        "{},{}",
        bounding_box.bottom_right.lat, bounding_box.bottom_right.lon
    );
}

#[derive(Clone, Debug)]
struct GeoPoint {
    lat: f64,
    lon: f64,
}

#[derive(Clone, Debug)]
struct BoundingBox {
    top_left: GeoPoint,
    bottom_right: GeoPoint,
}

fn bounding_box(stations: &[Station]) -> BoundingBox {
    let mut top = -85.;
    let mut bottom = 85.;
    let mut left = 180.0;
    let mut right = -180.0;

    for station in stations.iter().skip(1) {
        if station.lat > top {
            top = station.lat
        }

        if station.lat < bottom {
            bottom = station.lat
        }

        if station.lon < left {
            left = station.lon
        }

        if station.lon > right {
            right = station.lon
        }
    }

    BoundingBox {
        top_left: GeoPoint {
            lat: top,
            lon: left,
        },
        bottom_right: GeoPoint {
            lat: bottom,
            lon: right,
        },
    }
}
