use std::io::Cursor;

use serde::Deserialize;
use zip::ZipArchive;

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
