use eframe::egui::{self, RichText, Ui};

use crate::{task_start_session::task_start_session, App};

pub struct LobbyState {
    pub join_existing: bool,
    pub name_input: String,
    pub existing_peer_input: String,
}

pub fn render_lobby(ui: &mut Ui, app: App, state: &mut LobbyState) {
    // Center everything vertically
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);

        // Title
        ui.label(
            RichText::new("Rusty Collab")
                .size(48.0)
                .strong()
                .color(egui::Color32::from_rgb(30, 41, 59)),
        );

        ui.add_space(8.0);

        ui.label(
            RichText::new("Collaborative text editing")
                .size(16.0)
                .color(egui::Color32::from_rgb(100, 116, 139)),
        );

        ui.add_space(40.0);

        // Container with max width
        ui.allocate_ui_with_layout(
            egui::vec2(400.0, f32::INFINITY),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                // Mode selection tabs
                ui.horizontal(|ui| {
                    ui.set_width(400.0);

                    let create_button =
                        egui::Button::new(RichText::new("Create Session").size(16.0).strong())
                            .min_size(egui::vec2(196.0, 48.0))
                            .corner_radius(8)
                            .selected(!state.join_existing)
                            .fill(if !state.join_existing {
                                egui::Color32::from_rgb(59, 130, 246)
                            } else {
                                egui::Color32::from_rgb(241, 245, 249)
                            });

                    let join_button =
                        egui::Button::new(RichText::new("Join Session").size(16.0).strong())
                            .min_size(egui::vec2(196.0, 48.0))
                            .corner_radius(8)
                            .selected(state.join_existing)
                            .fill(if state.join_existing {
                                egui::Color32::from_rgb(59, 130, 246)
                            } else {
                                egui::Color32::from_rgb(241, 245, 249)
                            });

                    if ui.add(create_button).clicked() {
                        state.join_existing = false;
                    }

                    ui.add_space(8.0);

                    if ui.add(join_button).clicked() {
                        state.join_existing = true;
                    }
                });

                ui.add_space(32.0);

                // Name input
                ui.horizontal(|ui| {
                    ui.set_width(400.0);
                    ui.label(
                        RichText::new("Your name")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(71, 85, 105)),
                    );
                });

                ui.add_space(4.0);

                let name_edit = egui::TextEdit::singleline(&mut state.name_input)
                    .desired_width(400.0)
                    .font(egui::FontId::new(16.0, egui::FontFamily::Proportional))
                    .margin(egui::vec2(12.0, 12.0));
                ui.add(name_edit);

                ui.add_space(20.0);

                // Peer ID input (only for join)
                if state.join_existing {
                    ui.horizontal(|ui| {
                        ui.set_width(400.0);
                        ui.label(
                            RichText::new("Peer ID to join")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(71, 85, 105)),
                        );
                    });

                    ui.add_space(4.0);

                    let peer_edit = egui::TextEdit::singleline(&mut state.existing_peer_input)
                        .desired_width(400.0)
                        .font(egui::FontId::new(16.0, egui::FontFamily::Proportional))
                        .margin(egui::vec2(12.0, 12.0));
                    ui.add(peer_edit);

                    ui.add_space(20.0);
                }

                // Action button
                let button_text = if state.join_existing {
                    "Join Session"
                } else {
                    "Create Session"
                };

                let button = egui::Button::new(RichText::new(button_text).size(16.0).strong())
                    .min_size(egui::vec2(400.0, 48.0))
                    .corner_radius(8);

                if ui.add(button).clicked() {
                    if state.join_existing {
                        tokio::spawn(task_start_session(
                            app,
                            state.name_input.clone(),
                            Some(state.existing_peer_input.clone()),
                        ));
                    } else {
                        tokio::spawn(task_start_session(app, state.name_input.clone(), None));
                    }
                }
            },
        );
    });
}
