use walkers::{
    Map, Plugin,
    extras::{LabeledSymbol, LabeledSymbolStyle, Places, Symbol},
};

use crate::ui::{LichtApp, state::Route};

pub fn show(app: &mut LichtApp, ui: &mut egui::Ui) {
    puffin::profile_function!();
    if ui.button("Back").clicked() {
        app.state.show_map = false;
    }

    let mut map = Map::new(
        Some(&mut app.state.tiles),
        &mut app.state.map_memory,
        walkers::lon_lat(8.404418866463923, 49.01376021753036),
    );

    map = map.with_plugin(stops_plugin(&app.state.routes));
    ui.add(map);
}

fn stops_plugin(routes: &[Route]) -> impl Plugin {
    puffin::profile_function!();

    let places = routes
        .iter()
        .flat_map(|r| r.stations.clone())
        .map(|s| LabeledSymbol {
            position: walkers::lat_lon(s.lat, s.lon),
            label: s.name.clone(),
            symbol: Some(Symbol::Circle("🚆".to_string())),
            style: LabeledSymbolStyle {
                symbol_size: 25.,
                ..Default::default()
            },
        })
        .collect();

    Places::new(places)
}
