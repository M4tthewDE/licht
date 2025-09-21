use std::fs::File;

use serde::Deserialize;

mod tmdb;
mod ui;

#[derive(Deserialize, Clone)]
struct Config {
    token: String,
}

fn main() -> eframe::Result {
    env_logger::init();
    let config_file = File::open("config.json").unwrap();
    let config: Config = serde_json::from_reader(config_file).unwrap();
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Licht",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(ui::LichtApp::new(config)))
        }),
    )
}
