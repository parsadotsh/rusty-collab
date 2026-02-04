use eframe::egui;

mod screen_lobby;
mod screen_session;

use screen_lobby::render_lobby;
use screen_session::render_session;

struct App {
    state: State,
}

pub enum State {
    // Initial state, for user to select "New Session" or "Join Existing"
    Lobby {
        join_existing: bool,
        name_input: String,
    },
    // Active collaborative editing session
    Session {
        doc: String,
    },
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Rusty Collab",
        options,
        Box::new(|_cc| {
            // Setup
            let app = App {
                state: State::Lobby {
                    join_existing: false,
                    name_input: String::new(),
                },
            };
            Ok(Box::new(app))
        }),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_ui(ui, self);
        });
    }
}

fn render_ui(ui: &mut egui::Ui, app: &mut App) {
    match app.state {
        State::Lobby { .. } => render_lobby(ui, &mut app.state),
        State::Session { .. } => render_session(ui, &mut app.state),
    }
}
