use crate::Config;
use egui::{
    Image, ImageButton, RichText, ScrollArea,
    TextStyle::{Body, Button, Heading},
};
use state::{MovieDetails, MovieSearch, State, StateMutation};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};
use tmdb::TmdbClient;
use tokio::runtime::{Builder, Runtime};

use egui::{Color32, FontId};

mod map;
mod state;
mod tmdb;

pub struct LichtApp {
    tmdb_client: TmdbClient,
    rt: Runtime,
    rx: Receiver<StateMutation>,
    tx: Sender<StateMutation>,
    state: State,
    map_state: map::State,
}

impl eframe::App for LichtApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(modifier) = self.rx.try_recv() {
            modifier(&mut self.state);
        }

        ctx.all_styles_mut(|style| {
            style.text_styles = [
                (Heading, FontId::new(25.0, egui::FontFamily::Proportional)),
                (Body, FontId::new(15.0, egui::FontFamily::Proportional)),
                (Button, FontId::new(15.0, egui::FontFamily::Proportional)),
            ]
            .into();
            style.visuals.override_text_color = Some(Color32::WHITE);
            style.visuals.panel_fill = Color32::BLACK;
            style.visuals.text_edit_bg_color = Some(Color32::DARK_GRAY);
        });

        if let Some(last_change_time) = self.state.last_change_time
            && Instant::now().duration_since(last_change_time).as_millis() >= 250
        {
            self.state.last_change_time = None;
            self.do_search();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.show_map {
                map::show(self, ui);
            } else {
                self.show(ui);
            }
        });
    }
}

impl LichtApp {
    pub fn new(ctx: egui::Context, config: Config) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tmdb_client: TmdbClient::new(config.token),
            rt: Builder::new_multi_thread().enable_all().build().unwrap(),
            tx,
            rx,
            state: State::default(),
            map_state: map::State::new(ctx),
        }
    }

    fn do_search(&self) {
        let tmdb_client = self.tmdb_client.clone();
        let search_text = self.state.search_text.clone();
        let tx = self.tx.clone();
        self.rt.spawn(async move {
            let movie_search_resp = tmdb_client.search_movies(&search_text).await;

            for result in movie_search_resp.results {
                let movie_details = tmdb_client.movie_details(result.id, tmdb::ENGLISH).await;
                let german_movie_details = tmdb_client.movie_details(result.id, tmdb::GERMAN).await;
                let credits = tmdb_client.movie_credits(result.id).await;
                let movie_search = MovieSearch::new(movie_details, german_movie_details, credits);
                tx.send(state::movie_search_mutation(movie_search)).unwrap();
            }
        });
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        if ui.button("Map").clicked() {
            self.state.show_map = true;
            return;
        }

        if let Some(current_movie) = self.state.current_movie.clone() {
            if ui.button("Back").clicked() {
                self.state.current_movie = None;
            }

            ui.separator();

            self.current_movie(&current_movie, ui);
        } else {
            ui.heading("Search");
            self.search(ui);
            ui.separator();
            self.movie_results(ui);
        }
    }

    fn search(&mut self, ui: &mut egui::Ui) {
        if ui
            .text_edit_singleline(&mut self.state.search_text)
            .changed()
        {
            self.state.last_change_time = Some(Instant::now());
        }
    }

    fn movie_results(&mut self, ui: &mut egui::Ui) {
        if !self.state.movie_searches.is_empty() {
            ScrollArea::both().show(ui, |ui| {
                for movie_search in self.state.movie_searches.clone() {
                    self.movie_search_result(movie_search.clone(), ui);
                    ui.separator();
                }
            });
        }
    }

    fn movie_search_result(&mut self, movie_search: MovieSearch, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let image = Image::new(&movie_search.details.poster_url)
                .fit_to_exact_size(egui::vec2(120.0, 160.0));

            if ui.add(ImageButton::new(image)).clicked() {
                self.state.current_movie = Some(movie_search.details.clone());
            };

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&movie_search.details.original_title)
                        .context_menu(|ui| {
                            if ui.button("Copy German title").clicked() {
                                ui.ctx().copy_text(movie_search.german_details.title);
                            }
                        });
                    ui.label(
                        RichText::new(
                            movie_search
                                .details
                                .release_date
                                .clone()
                                .unwrap_or_default(),
                        )
                        .color(Color32::GRAY),
                    );
                });

                if movie_search.details.runtime != 0 {
                    ui.label(
                        RichText::new(humanize_runtime(movie_search.details.runtime))
                            .color(Color32::GRAY),
                    );
                }

                ui.add_space(10.0);

                if !movie_search.credits.credits.is_empty() {
                    ui.horizontal(|ui| {
                        for credit in movie_search.credits.credits {
                            let image = Image::new(&credit.profile_photo_url)
                                .fit_to_exact_size(egui::vec2(60.0, 90.0));

                            ui.add(image).on_hover_ui(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(&credit.name);
                                    ui.label(RichText::new(&credit.character).color(Color32::GRAY));
                                });
                            });
                        }
                    });
                }
            });
        });
    }

    fn current_movie(&self, movie_details: &MovieDetails, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let image =
                Image::new(&movie_details.poster_url).fit_to_exact_size(egui::vec2(120.0, 160.0));

            ui.add(image);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(&movie_details.original_title);
                    ui.label(
                        RichText::new(movie_details.release_date.clone().unwrap_or_default())
                            .color(Color32::GRAY),
                    );
                });

                ui.label(
                    RichText::new(humanize_runtime(movie_details.runtime)).color(Color32::GRAY),
                );
                ui.label(movie_details.tagline.clone().unwrap_or_default());
                ui.separator();
                ui.label(movie_details.overview.clone().unwrap_or_default());
            });
        });
    }
}

fn humanize_runtime(runtime: u64) -> String {
    let hours = runtime / 60;
    let minutes = runtime % 60;

    if hours == 0 {
        format!("{minutes}m")
    } else {
        format!("{hours}h{minutes}m")
    }
}
