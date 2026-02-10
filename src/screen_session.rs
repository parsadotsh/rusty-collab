use eframe::egui::{TextEdit, Ui};

use crate::{App, task_leave_session::task_leave_session};

pub struct SessionState {
    pub doc: String,
}

pub fn render_session(ui: &mut Ui, app: App, state: &mut SessionState) {
    if ui.button("Leave Session").clicked() {
        tokio::spawn(task_leave_session(app));
    }

    TextEdit::multiline(&mut state.doc).show(ui);
}
