use std::fmt::Debug;

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

    ui.horizontal(|ui| {
        ui.label("My ID:");
        let id = state.iroh_endpoint.id().to_string();
        ui.label(&id);
        if ui.button("📋").clicked() {
            ui.ctx().copy_text(id);
        }
    });

    let text_edit = TextEdit::multiline(&mut state.doc).show(ui);

    if text_edit.response.changed() {
        let _ = state.outbound_queue.send(GossipMessage::Update {
            new_doc: state.doc.clone(),
        });
    }
}
