use eframe::egui::Ui;

use crate::State;

pub fn render_lobby(ui: &mut Ui, state: &mut State) {
    let State::Lobby {
        join_existing,
        name_input,
    } = state
    else {
        return;
    };

    ui.horizontal(|ui| {
        ui.selectable_value(join_existing, false, "Create new session");
        ui.selectable_value(join_existing, true, "Join existing session");
    });
    ui.heading("Rusty Collab");
    ui.horizontal(|ui| {
        ui.label("Your name: ");
        ui.text_edit_singleline(name_input);
    });

    if !*join_existing {
        if ui.button("Create New Session").clicked() {
            *state = State::Session { doc: String::new() };
            return;
        }
    } else {
        if ui.button("Join Existing Session").clicked() {
            *state = State::Session { doc: String::new() };
            return;
        }
    }
}
