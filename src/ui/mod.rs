use egui::{
    Image, RichText, ScrollArea,
    TextStyle::{Body, Button, Heading},
};
use std::{
    collections::BTreeMap,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};
use tokio::runtime::{Builder, Runtime};

use egui::{Color32, FontId};

use crate::{
    Config,
    tmdb::{MovieSearchResponse, MovieSearchResult, TmdbClient},
};

pub struct LichtApp {
    tmdb_client: TmdbClient,
    search_text: String,
    rt: Runtime,
    rx: Receiver<MovieSearchResponse>,
    tx: Sender<MovieSearchResponse>,
    movie_search_response: Option<MovieSearchResponse>,
    last_change_time: Instant,
}

impl eframe::App for LichtApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(resp) = self.rx.try_recv() {
            self.movie_search_response = Some(resp);
        }

        let text_styles: BTreeMap<_, _> = [
            (Heading, FontId::new(25.0, egui::FontFamily::Proportional)),
            (Body, FontId::new(15.0, egui::FontFamily::Proportional)),
            (Button, FontId::new(15.0, egui::FontFamily::Proportional)),
        ]
        .into();

        ctx.all_styles_mut(|style| {
            style.text_styles = text_styles.clone();
            style.visuals.override_text_color = Some(Color32::WHITE);
            style.visuals.panel_fill = Color32::BLACK;
            style.visuals.text_edit_bg_color = Some(Color32::DARK_GRAY);
        });

        if Instant::now()
            .duration_since(self.last_change_time)
            .as_millis()
            >= 250
        {
            let tmdb_client = self.tmdb_client.clone();
            let search_text = self.search_text.clone();
            let tx = self.tx.clone();
            self.rt.spawn(async move {
                let resp = tmdb_client.search_movies(&search_text).await;
                tx.send(resp).unwrap();
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| self.show(ui));
    }
}

impl LichtApp {
    pub fn new(config: Config) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            tmdb_client: TmdbClient::new(config.token),
            search_text: String::new(),
            rt: Builder::new_multi_thread().enable_all().build().unwrap(),
            tx,
            rx,
            movie_search_response: None,
            last_change_time: Instant::now(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Licht");
        self.search(ui);
        ui.separator();
        self.movie_results(ui);
    }

    fn search(&mut self, ui: &mut egui::Ui) {
        if ui.text_edit_singleline(&mut self.search_text).changed() {
            self.last_change_time = Instant::now();
        }
    }

    fn movie_results(&mut self, ui: &mut egui::Ui) {
        if let Some(resp) = &self.movie_search_response {
            ScrollArea::vertical().show(ui, |ui| {
                for result in &resp.results {
                    self.movie_result(result, ui);
                    ui.separator();
                }
            });
        }
    }

    fn movie_result(&self, result: &MovieSearchResult, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(poster_path) = &result.poster_path {
                ui.add(Image::new(format!(
                    "https://image.tmdb.org/t/p/w600_and_h900_bestv2{}",
                    poster_path
                )).fit_to_exact_size(egui::vec2(90.0, 135.0)));
            } else {
                ui.add(Image::new("https://www.themoviedb.org/assets/2/v4/glyphicons/basic/glyphicons-basic-38-picture-grey-c2ebdbb057f2a7614185931650f8cee23fa137b93812ccb132b9df511df1cfac.svg")
                    .fit_to_exact_size(egui::vec2(90.0, 135.0)));
            }

            ui.vertical(|ui| {
                ui.label(&result.original_title);
                ui.label(RichText::new(format!("{}", result.release_date)).color(Color32::GRAY));
            });
        });
    }
}
