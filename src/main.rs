use std::sync::Arc;

use eframe::egui;
use parking_lot::Mutex;

mod screen_lobby;
mod screen_session;
mod task_leave_session;
mod task_start_session;

use screen_lobby::render_lobby;
use screen_session::render_session;

use crate::{screen_lobby::LobbyState, task_start_session::SessionState};

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
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "Rusty Collab",
            options,
            Box::new(|cc| {
                // Setup
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
        egui::CentralPanel::default().show(ctx, |ui| {
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
