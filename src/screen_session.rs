use eframe::egui::{self, TextEdit, Ui, text::CCursor};

use crate::{
    App,
    awareness::{LoroCursors, broadcast_awareness},
    task_leave_session::task_leave_session,
    task_start_session::SessionState,
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

    ui.horizontal(|ui| {
        let _ = ui.button(format!("Me: {}", state.own_name));

        state
            .awareness_cache
            .iter()
            .for_each(|(_, (awareness, _))| {
                let _ = ui.button(format!("{}", awareness.name));
            });
    });

    let doc_text = state.loro_doc.get_text("text");
    let mut text_content = doc_text.to_string();

    let text_edit = TextEdit::multiline(&mut text_content).show(ui);

    if state.egui_cursors_needs_update {
        state.egui_cursors_needs_update = false;
        update_egui_from_loro_cursors(ui, text_edit.response.id, &state.loro_doc, &state.cursors);
    } else {
        let new_cursors = get_loro_cursors_from_egui(&text_edit, &doc_text);
        if new_cursors != state.cursors {
            state.cursors = new_cursors;
            let _ = broadcast_awareness(state);
        }
    }

    if text_edit.response.changed() {
        let _ = doc_text.update(&text_content, Default::default());
        state.loro_doc.commit();
    }
}

fn update_egui_from_loro_cursors(
    ui: &mut Ui,
    text_edit_id: egui::Id,
    loro_doc: &loro::LoroDoc,
    cursors: &LoroCursors,
) {
    // Based on https://github.com/emilk/egui/blob/main/crates/egui_demo_lib/src/demo/text_edit.rs

    let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) else {
        return;
    };

    let egui_cursor_range = if let Some((primary, secondary)) = cursors.as_ref()
        && let Ok(primary_pos) = loro_doc.get_cursor_pos(primary)
        && let Ok(secondary_pos) = loro_doc.get_cursor_pos(secondary)
    {
        Some(egui::text::CCursorRange::two(
            CCursor::new(secondary_pos.current.pos),
            CCursor::new(primary_pos.current.pos),
        ))
    } else {
        None
    };

    state.cursor.set_char_range(egui_cursor_range);
    state.store(ui.ctx(), text_edit_id);
    ui.memory_mut(|mem| mem.request_focus(text_edit_id));
}

fn get_loro_cursors_from_egui(
    output: &egui::text_edit::TextEditOutput,
    doc_text: &loro::LoroText,
) -> LoroCursors {
    let Some(cursor_range) = output.cursor_range else {
        return None;
    };

    let primary_idx = cursor_range.primary.index;
    let secondary_idx = cursor_range.secondary.index;

    let Some(primary) = doc_text.get_cursor(primary_idx, loro::cursor::Side::Left) else {
        return None;
    };
    let Some(secondary) = doc_text.get_cursor(secondary_idx, loro::cursor::Side::Left) else {
        return None;
    };

    Some((primary.clone(), secondary.clone()))
}
