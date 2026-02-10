use eframe::egui::Ui;

use crate::{App, State, task_start_session::task_start_session};

pub struct LobbyState {
    pub join_existing: bool,
    pub name_input: String,
    pub existing_peer_input: String,
}

pub fn render_lobby(ui: &mut Ui, app: App, state: &mut LobbyState) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.join_existing, false, "Create new session");
        ui.selectable_value(&mut state.join_existing, true, "Join existing session");
    });
    ui.heading("Rusty Collab");
    ui.horizontal(|ui| {
        ui.label("Your name: ");
        ui.text_edit_singleline(&mut state.name_input);
    });

    if state.join_existing {
        ui.horizontal(|ui| {
            ui.label("Existing peer ID: ");
            ui.text_edit_singleline(&mut state.existing_peer_input);
        });
    }

    if !state.join_existing {
        if ui.button("Create New Session").clicked() {
            tokio::spawn(task_start_session(app, state.name_input.clone(), None));
        }
    } else {
        if ui.button("Join Existing Session").clicked() {
            tokio::spawn(task_start_session(
                app,
                state.name_input.clone(),
                Some(state.existing_peer_input.clone()),
            ));
            return;
        }
    }
}
