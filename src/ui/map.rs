use walkers::{
    Map, Plugin,
    extras::{
        GroupedPlaces, LabeledSymbol, LabeledSymbolGroup, LabeledSymbolGroupStyle,
        LabeledSymbolStyle, Symbol,
    },
};

use crate::ui::LichtApp;

use crate::ui::state::Stop;

pub fn show(app: &mut LichtApp, ui: &mut egui::Ui) {
    puffin::profile_function!();
    if ui.button("Back").clicked() {
        app.state.show_map = false;
    }
    ui.label(app.state.stops.len().to_string());
    let mut map = Map::new(
        Some(&mut app.state.tiles),
        &mut app.state.map_memory,
        walkers::lon_lat(8.404418866463923, 49.01376021753036),
    );

    map = map.with_plugin(stops_plugin(app.state.stops.clone()));
    ui.add(map);
}

fn stops_plugin(stops: Vec<Stop>) -> impl Plugin {
    puffin::profile_function!();

    let places = stops
        .iter()
        .map(|s| LabeledSymbol {
            position: walkers::lat_lon(s.stop_lat, s.stop_lon),
            label: s.stop_name.clone(),
            symbol: Some(Symbol::Circle("🚆".to_string())),
            style: LabeledSymbolStyle {
                symbol_size: 25.,
                ..Default::default()
            },
        })
        .collect();

    GroupedPlaces::new(
        places,
        LabeledSymbolGroup {
            style: LabeledSymbolGroupStyle::default(),
        },
    )
}

