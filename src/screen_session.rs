use std::sync::Arc;

use eframe::egui::{self, Color32, LayerId, RichText, TextEdit, Ui, UiBuilder, text::CCursor};

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
        let text = RichText::new(&state.own_name).color(generate_peer_color(&state.own_id));
        let _ = ui.button(text);

        state
            .awareness_cache
            .iter()
            .for_each(|(_, (awareness, _))| {
                let text = RichText::new(&awareness.name)
                    .color(generate_peer_color(&awareness.endpoint_id));
                let _ = ui.button(text);
            });
    });

    let doc_text = state.loro_doc.get_text("text");
    let mut text_content = doc_text.to_string();

    let text_edit = {
        let front_layer_id = LayerId::new(ui.layer_id().order, ui.id().with("front"));
        ui.ctx().set_sublayer(ui.layer_id(), front_layer_id);

        ui.scope_builder(UiBuilder::new().layer_id(front_layer_id), |ui| {
            TextEdit::multiline(&mut text_content)
                .background_color(Color32::TRANSPARENT)
                .show(ui)
        })
        .inner
    };

    render_peer_cursors(ui, &text_edit, &state.awareness_cache, &state.loro_doc);

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

fn render_peer_cursors(
    ui: &mut egui::Ui,
    text_edit_output: &egui::text_edit::TextEditOutput,
    awareness_cache: &crate::awareness::AwarenessCache,
    loro_doc: &loro::LoroDoc,
) {
    let painter = ui.painter_at(text_edit_output.text_clip_rect);
    let galley = &text_edit_output.galley;
    let galley_pos = text_edit_output.galley_pos;

    for (endpoint_id, (awareness, _)) in awareness_cache.iter() {
        ui.horizontal(|ui| {
            if let Some((cursor_primary, cursor_secondary)) = &awareness.loro_cursors {
                ui.label(format!("{:?}", loro_doc.get_cursor_pos(cursor_primary)));
                ui.label(format!("{:?}", loro_doc.get_cursor_pos(cursor_secondary)));
            } else {
                ui.label("No cursors");
            }
        });

        if let Some((cursor_primary, cursor_secondary)) = &awareness.loro_cursors
            && let Ok(primary) = loro_doc.get_cursor_pos(cursor_primary)
            && let Ok(secondary) = loro_doc.get_cursor_pos(cursor_secondary)
        {
            let color = generate_peer_color(endpoint_id);
            let selection_color =
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);

            paint_awareness_selection(
                &painter,
                galley,
                galley_pos,
                primary.current.pos,
                secondary.current.pos,
                selection_color,
            );

            let cursor_rect = galley
                .pos_from_cursor(CCursor::new(primary.current.pos))
                .translate(galley_pos.to_vec2());

            paint_awareness_cursor(&painter, cursor_rect, color);
        }
    }
}

fn generate_peer_color(endpoint_id: &[u8; 32]) -> Color32 {
    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color32 {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match h as i32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color32::from_rgb(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }
    // Generate hue from hash of full endpoint_id for better distribution
    let mut hash: u32 = 2166136261;
    for byte in endpoint_id {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }

    let hue = (hash % 360) as f32; // 0-360 degrees
    let saturation = 0.85; // High saturation for vibrant colors
    let lightness = 0.35; // Dark enough for white background (0-0.5 range)

    hsl_to_rgb(hue, saturation, lightness)
}

fn paint_awareness_cursor(painter: &egui::Painter, cursor_rect: egui::Rect, color: Color32) {
    let top = cursor_rect.center_top();
    let bottom = cursor_rect.center_bottom();

    painter.line_segment([top, bottom], (2.0, color));

    // Half circle pointing right, positioned at the top of the bar
    let center = top + egui::vec2(0.0, 4.0);
    let radius = 4.0;

    // Clip to only draw the right half of the circle
    let clip_rect = egui::Rect::from_min_max(
        egui::pos2(center.x, center.y - radius - 1.0),
        egui::pos2(center.x + radius + 1.0, center.y + radius + 1.0),
    );
    let clipped_painter = painter.with_clip_rect(clip_rect);
    clipped_painter.circle_filled(center, radius, color);
}

fn paint_awareness_selection(
    painter: &egui::Painter,
    galley: &Arc<egui::Galley>,
    galley_pos: egui::Pos2,
    primary: usize,
    secondary: usize,
    color: Color32,
) {
    if primary == secondary {
        return;
    }

    let (min_idx, max_idx) = (primary.min(secondary), primary.max(secondary));
    let min_layout = galley.layout_from_cursor(CCursor::new(min_idx));
    let max_layout = galley.layout_from_cursor(CCursor::new(max_idx));

    for row_idx in min_layout.row..=max_layout.row {
        let placed_row = &galley.rows[row_idx];
        let row = &placed_row.row;

        let left = if row_idx == min_layout.row {
            row.x_offset(min_layout.column)
        } else {
            0.0
        };

        let right = if row_idx == max_layout.row {
            row.x_offset(max_layout.column)
        } else {
            let newline_bonus = if placed_row.ends_with_newline {
                row.height() / 2.0
            } else {
                0.0
            };
            row.size.x + newline_bonus
        };

        let rect = egui::Rect::from_min_max(
            egui::pos2(left, placed_row.pos.y),
            egui::pos2(right, placed_row.pos.y + row.size.y),
        );
        painter.rect_filled(rect.translate(galley_pos.to_vec2()), 0.0, color);
    }
}
