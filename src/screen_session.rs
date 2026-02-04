use eframe::egui::{TextEdit, Ui};

use crate::State;

pub fn render_session(ui: &mut Ui, state: &mut State) {
    let State::Session { doc } = state else {
        return;
    };

    if ui.button("Leave Session").clicked() {
        *state = State::Lobby {
            join_existing: false,
            name_input: String::new(),
        };
        return;
    }

    TextEdit::multiline(doc).show(ui);
}
