use std::sync::Arc;

use eframe::egui;
use parking_lot::Mutex;

mod awareness;
mod gossip_message;
mod screen_lobby;
mod screen_session;
mod task_leave_session;
mod task_start_session;

use screen_lobby::render_lobby;
use screen_session::render_session;

use crate::{screen_lobby::LobbyState, task_start_session::SessionState};

fn setup_custom_style(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    ctx.set_visuals(egui::Visuals::light());

    let mut style = (*ctx.style()).clone();

    // White/light background
    style.visuals.dark_mode = false;
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(40, 40, 40));
    style.visuals.panel_fill = egui::Color32::from_rgb(250, 250, 250);
    style.visuals.window_fill = egui::Color32::from_rgb(250, 250, 250);
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(250, 250, 250);

    // Rounded corners
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);

    // Spacing
    style.spacing.button_padding = egui::vec2(16.0, 12.0);
    style.spacing.item_spacing = egui::vec2(12.0, 16.0);

    ctx.set_style(style);
}

#[derive(Clone)]
struct App {
    state: Arc<Mutex<State>>,
    egui_ctx: egui::Context,
}

impl App {
    pub fn replace_state(&self, state: State) -> State {
        let mut state_lock = self.state.lock();
        let old_state = std::mem::replace(&mut *state_lock, state);
        self.egui_ctx.request_repaint();

        old_state
    }
}

pub enum State {
    Lobby(LobbyState),
    Loading,
    Session(SessionState),
}

#[tokio::main]
async fn main() -> eframe::Result {
    tokio::task::block_in_place(|| {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 700.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Rusty Collab",
            options,
            Box::new(|cc| {
                // Setup custom styling
                setup_custom_style(&cc.egui_ctx);

                let app = App {
                    state: Arc::new(Mutex::new(State::Lobby(LobbyState {
                        join_existing: false,
                        name_input: String::new(),
                        existing_peer_input: String::new(),
                    }))),
                    egui_ctx: cc.egui_ctx.clone(),
                };
                Ok(Box::new(app))
            }),
        )
    })
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(250, 250, 250))
                    .inner_margin(egui::vec2(40.0, 24.0)),
            )
            .show(ctx, |ui| {
                render_ui(ui, self);
            });
    }
}

fn render_ui(ui: &mut egui::Ui, app: &mut App) {
    let mut state = app.state.lock();

    match &mut *state {
        State::Lobby(state) => render_lobby(ui, app.clone(), state),
        State::Loading => {
            ui.spinner();
        }
        State::Session(state) => render_session(ui, app.clone(), state),
    }
}
