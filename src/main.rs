use std::fs::File;

use serde::Deserialize;

mod ui;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Deserialize, Clone)]
struct Config {
    tmdb_token: String,
    mapbox_token: String,
}

fn main() -> eframe::Result {
    env_logger::init();
    puffin::set_scopes_on(true);

    let _puffin_server = if std::env::var("PROFILER").is_ok() {
        eprintln!("running with profiler");
        let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
        Some(puffin_http::Server::new(&server_addr).unwrap())
    } else {
        None
    };

    let config_file = File::open("config.json").unwrap();
    let config: Config = serde_json::from_reader(config_file).unwrap();
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Licht",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(ui::LichtApp::new(cc.egui_ctx.clone(), config)))
        }),
    )
}
