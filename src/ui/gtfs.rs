use std::io::Cursor;

use bytes::Bytes;
use serde::Deserialize;
use zip::ZipArchive;

const VBK_AGENCY_ID: &str = "02";
const ALBTAL_AGENCY_ID: &str = "01";

#[derive(Deserialize, Clone)]
pub struct StopTime {
    pub trip_id: String,
    pub stop_id: String,
}

#[derive(Deserialize, Clone)]
pub struct Route {
    pub route_id: String,
    pub agency_id: String,
    pub route_short_name: String,
    pub route_type: u8,
}

#[derive(Deserialize, Clone)]
pub struct Trip {
    pub trip_id: String,
    pub route_id: String,
}

#[derive(Deserialize, Clone)]
pub struct Stop {
    pub stop_id: String,
    pub stop_name: String,
    pub stop_lat: f64,
    pub stop_lon: f64,
}

#[derive(Deserialize, Clone, Default)]
pub struct TransitData {
    pub stops: Vec<Stop>,
    pub trips: Vec<Trip>,
    pub routes: Vec<Route>,
    pub stop_times: Vec<StopTime>,
}

impl TransitData {
    pub async fn load() -> Self {
        let resp = reqwest::get("https://projekte.kvv-efa.de/GTFS/google_transit.zip")
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        let mut zip = ZipArchive::new(Cursor::new(resp)).unwrap();

        Self {
            stops: load_stops(&mut zip),
            trips: load_trips(&mut zip),
            routes: load_routes(&mut zip),
            stop_times: load_stop_times(&mut zip),
        }
    }
}

fn load_stops(zip: &mut ZipArchive<Cursor<Bytes>>) -> Vec<Stop> {
    let stops_file = zip.by_name("stops.txt").unwrap();

    let mut csv_reader = csv::Reader::from_reader(stops_file);
    let mut stops: Vec<Stop> = csv_reader.deserialize().map(|r| r.unwrap()).collect();
    stops.sort_by_key(|s| s.stop_name.clone());
    stops.dedup_by_key(|s| s.stop_name.clone());
    stops
    //stops
    //    .into_iter()
    //    .filter(|s| s.zone_id == VBK_ZONE_ID)
    //    .collect()
}

fn load_trips(zip: &mut ZipArchive<Cursor<Bytes>>) -> Vec<Trip> {
    let trips_file = zip.by_name("trips.txt").unwrap();

    let mut csv_reader = csv::Reader::from_reader(trips_file);
    let trips: Vec<Trip> = csv_reader.deserialize().map(|r| r.unwrap()).collect();
    trips
}

fn load_routes(zip: &mut ZipArchive<Cursor<Bytes>>) -> Vec<Route> {
    let routes_file = zip.by_name("routes.txt").unwrap();

    let mut csv_reader = csv::Reader::from_reader(routes_file);
    let mut routes: Vec<Route> = csv_reader.deserialize().map(|r| r.unwrap()).collect();
    routes.sort_by_key(|r| r.route_short_name.clone());
    routes.dedup_by_key(|r| r.route_short_name.clone());
    routes
        .into_iter()
        .filter(|r| {
            (r.agency_id == VBK_AGENCY_ID || r.agency_id == ALBTAL_AGENCY_ID) && r.route_type < 3
        })
        .collect()
}

fn load_stop_times(zip: &mut ZipArchive<Cursor<Bytes>>) -> Vec<StopTime> {
    let stop_times_file = zip.by_name("stop_times.txt").unwrap();

    let mut csv_reader = csv::Reader::from_reader(stop_times_file);
    let stop_times: Vec<StopTime> = csv_reader.deserialize().map(|r| r.unwrap()).collect();
    stop_times
}
