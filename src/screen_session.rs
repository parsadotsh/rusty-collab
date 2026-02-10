use eframe::egui::{TextEdit, Ui};

use crate::{App, task_leave_session::task_leave_session, task_start_session::SessionState};

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

    let doc_text = state.loro_doc.get_text("text");
    let mut text_content = doc_text.to_string();

    let text_edit = TextEdit::multiline(&mut text_content).show(ui);

    if text_edit.response.changed() {
        let _ = doc_text.update(&text_content, Default::default());
        state.loro_doc.commit();
    }
}
