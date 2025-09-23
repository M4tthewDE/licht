use egui::{
    Image, ImageButton, RichText, ScrollArea,
    TextStyle::{Body, Button, Heading},
};
use state::{State, StateMutation};
use std::{
    collections::BTreeMap,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};
use tokio::runtime::{Builder, Runtime};

use egui::{Color32, FontId};

use crate::{
    Config,
    tmdb::{MovieDetailsResponse, MovieSearchResult, TmdbClient},
};

mod state;

pub struct LichtApp {
    tmdb_client: TmdbClient,
    rt: Runtime,
    rx: Receiver<StateMutation>,
    tx: Sender<StateMutation>,
    state: State,
}

impl eframe::App for LichtApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(modifier) = self.rx.try_recv() {
            modifier(&mut self.state);
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

        if let Some(last_change_time) = self.state.last_change_time
            && Instant::now().duration_since(last_change_time).as_millis() >= 250
        {
            self.state.last_change_time = None;

            let tmdb_client = self.tmdb_client.clone();
            let search_text = self.state.search_text.clone();
            let tx = self.tx.clone();
            self.rt.spawn(async move {
                let resp = tmdb_client.search_movies(&search_text).await;
                tx.send(state::movie_search_mutation(resp.clone())).unwrap();

                for movie_result in &resp.results {
                    let details = tmdb_client.movie_details(movie_result.id).await;
                    tx.send(state::movie_details_mutation(details.clone()))
                        .unwrap();

                    let credits = tmdb_client.movie_credits(movie_result.id).await;
                    tx.send(state::movie_credits_mutation(credits.clone()))
                        .unwrap();
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| self.show(ctx, ui));
    }
}

impl LichtApp {
    pub fn new(config: Config) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tmdb_client: TmdbClient::new(config.token),
            rt: Builder::new_multi_thread().enable_all().build().unwrap(),
            tx,
            rx,
            state: State::default(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(current_movie) = self.state.current_movie {
            if ui.button("Back").clicked() {
                self.state.current_movie = None;
            }

            ui.separator();

            if let Some(movie_details) = self.state.details(current_movie) {
                self.movie(ctx, &movie_details, ui);
            }
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
        if let Some(resp) = self.state.movie_search_response.clone() {
            ScrollArea::vertical().show(ui, |ui| {
                for result in &resp.results {
                    self.movie_result(result, ui);
                    ui.separator();
                }
            });
        }
    }

    fn movie_result(&mut self, result: &MovieSearchResult, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let url = if let Some(poster_path) = &result.poster_path {
                format!("https://image.tmdb.org/t/p/w600_and_h900_bestv2{}", poster_path)
            } else {
                "https://www.themoviedb.org/assets/2/v4/glyphicons/basic/glyphicons-basic-38-picture-grey-c2ebdbb057f2a7614185931650f8cee23fa137b93812ccb132b9df511df1cfac.svg".to_string()
            };

            let image = Image::new(&url).fit_to_exact_size(egui::vec2(120.0, 160.0));

            if ui.add(ImageButton::new(image)).clicked() {
                self.state.current_movie = Some(result.id);
            };

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&result.original_title);
                    ui.label(RichText::new(result.release_date.clone().unwrap_or_default()).color(Color32::GRAY));
                });

                if let Some(details) = self.state.details(result.id) && details.runtime != 0 {
                    ui.label(RichText::new(humanize_runtime(details.runtime)).color(Color32::GRAY));
                }

                ui.add_space(10.0);

                if let Some(credits) = self.state.credits(result.id) {
                    ui.horizontal(|ui| {
                        for credit in credits.cast.iter().take(10) {
                            let url = if let Some(poster_path) = &credit.profile_path {
                                format!("https://image.tmdb.org/t/p/w600_and_h900_bestv2{}", poster_path)
                            } else {
                                "https://www.themoviedb.org/assets/2/v4/glyphicons/basic/glyphicons-basic-38-picture-grey-c2ebdbb057f2a7614185931650f8cee23fa137b93812ccb132b9df511df1cfac.svg".to_string()
                            };

                            let image = Image::new(&url).fit_to_exact_size(egui::vec2(60.0, 90.0));

                            ui.add(image).on_hover_ui(|ui| {
                                ui.label(&credit.name);
                            });
                        }
                    });
                }
            });
        });
    }

    fn movie(&self, ctx: &egui::Context, movie_details: &MovieDetailsResponse, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let url = if let Some(poster_path) = &movie_details.poster_path {
                format!(
                    "https://image.tmdb.org/t/p/w600_and_h900_bestv2{}",
                    poster_path
                )
            } else {
                "https://www.themoviedb.org/assets/2/v4/glyphicons/basic/glyphicons-basic-38-picture-grey-c2ebdbb057f2a7614185931650f8cee23fa137b93812ccb132b9df511df1cfac.svg".to_string()
            };

            let image = Image::new(&url).fit_to_exact_size(egui::vec2(120.0, 160.0));

            if ui.add(ImageButton::new(image)).clicked() {
                ctx.open_url(egui::OpenUrl {
                    url,
                    new_tab: false,
                })
            };
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(&movie_details.original_title);
                    ui.label(RichText::new(movie_details.release_date.clone().unwrap_or_default()).color(Color32::GRAY));
                });

                ui.label(RichText::new(humanize_runtime(movie_details.runtime)).color(Color32::GRAY));
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
