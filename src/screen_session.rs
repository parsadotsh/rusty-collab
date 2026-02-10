use eframe::egui::{TextEdit, Ui};

use crate::{
    App,
    task_leave_session::task_leave_session,
    task_start_session::{GossipMessage, SessionState},
};

pub fn render_session(ui: &mut Ui, app: App, state: &mut SessionState) {
    if ui.button("Leave Session").clicked() {
        tokio::spawn(task_leave_session(app));
    }

    let text_edit = TextEdit::multiline(&mut state.doc).show(ui);

    if text_edit.response.changed() {
        let _ = state.outbound_queue.send(GossipMessage::Update {
            new_doc: state.doc.clone(),
        });
    }
}
