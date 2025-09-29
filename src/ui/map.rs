use egui::{Align2, ComboBox, FontId, Window};
use walkers::{
    Map, Plugin,
    extras::{LabeledSymbol, LabeledSymbolStyle, Places, Symbol},
};

use crate::ui::{LichtApp, state::Route};

pub fn show(app: &mut LichtApp, ui: &mut egui::Ui) {
    puffin::profile_function!();

    let mut map = Map::new(
        Some(&mut app.state.tiles),
        &mut app.state.map_memory,
        walkers::lon_lat(8.404418866463923, 49.01376021753036),
    );

    if let Some(route) = &app.state.current_route {
        map = map.with_plugin(stops_plugin(route));
    }

    ui.add(map);

    controls(app, ui);
}

fn stops_plugin(route: &Route) -> impl Plugin {
    puffin::profile_function!();

    let places = route
        .stations
        .iter()
        .map(|s| LabeledSymbol {
            position: walkers::lat_lon(s.lat, s.lon),
            label: "".to_string(),
            symbol: Some(Symbol::Circle("🚆".to_string())),
            style: LabeledSymbolStyle {
                label_font: FontId::proportional(12.0),
                symbol_size: 20.,
                ..Default::default()
            },
        })
        .collect();

    Places::new(places)
}

fn controls(app: &mut LichtApp, ui: &egui::Ui) {
    Window::new("Controls")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
            if ui.button("Back").clicked() {
                app.state.show_map = false;
            }

            ui.separator();

            let selected_text: &str = match &app.state.current_route {
                Some(r) => &r.name,
                None => "",
            };

            ComboBox::from_label("Route")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for route in &app.state.routes {
                        ui.selectable_value(
                            &mut app.state.current_route,
                            Some(route.clone()),
                            &route.name,
                        );
                    }
                });
        });
}
