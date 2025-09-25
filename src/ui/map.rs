use walkers::{
    HttpTiles, Map, MapMemory, Plugin,
    extras::{
        GroupedPlaces, LabeledSymbol, LabeledSymbolGroup, LabeledSymbolGroupStyle,
        LabeledSymbolStyle, Symbol,
    },
    sources::{Mapbox, MapboxStyle},
};

use crate::ui::LichtApp;

pub struct State {
    tiles: HttpTiles,
    map_memory: MapMemory,
}

impl State {
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            tiles: HttpTiles::new(
                Mapbox {
                    style: MapboxStyle::Dark,
                    high_resolution: true,
                    access_token: "pk.eyJ1IjoibWF0dGhld2RlIiwiYSI6ImNrb3NsbHo0cDAycHgycHE3bjU1enNjNncifQ.bLfLqwHHxq0OWS-vYHBiKg".to_owned(),
                },
                ctx,
            ),
            map_memory: MapMemory::default(),
        }
    }
}

pub fn show(app: &mut LichtApp, ui: &mut egui::Ui) {
    if ui.button("Back").clicked() {
        app.state.show_map = false;
    }
    let mut map = Map::new(
        Some(&mut app.map_state.tiles),
        &mut app.map_state.map_memory,
        walkers::lon_lat(8.404418866463923, 49.01376021753036),
    );

    map = map.with_plugin(stations());
    ui.add(map);
}

pub fn stations() -> impl Plugin {
    GroupedPlaces::new(
        vec![LabeledSymbol {
            position: walkers::lat_lon(49.00592412192814, 8.40970780367037),
            label: "Ruppürrer Tor".to_owned(),
            symbol: Some(Symbol::Circle("🚆".to_string())),
            style: LabeledSymbolStyle {
                symbol_size: 25.,
                ..Default::default()
            },
        }],
        LabeledSymbolGroup {
            style: LabeledSymbolGroupStyle::default(),
        },
    )
}
