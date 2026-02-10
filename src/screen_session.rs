use std::sync::Arc;

use eframe::egui::{self, Color32, LayerId, RichText, TextEdit, Ui, UiBuilder, text::CCursor};

use crate::{
    App,
    awareness::{LoroCursors, broadcast_awareness},
    task_leave_session::task_leave_session,
    task_start_session::SessionState,
};

pub fn render_session(ui: &mut Ui, app: App, state: &mut SessionState) {
    ui.vertical_centered(|ui| {
        // Header with leave button
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());

            ui.label(
                RichText::new("Collaborative Session")
                    .size(24.0)
                    .strong()
                    .color(egui::Color32::from_rgb(30, 41, 59)),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let leave_button = egui::Button::new(
                    RichText::new("Leave Session")
                        .size(14.0)
                        .color(egui::Color32::WHITE),
                )
                .min_size(egui::vec2(120.0, 36.0))
                .corner_radius(8)
                .fill(egui::Color32::from_rgb(239, 68, 68));

                if ui.add(leave_button).clicked() {
                    tokio::spawn(task_leave_session(app));
                }
            });
        });

        ui.add_space(16.0);

        // Peer ID display with copy button
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            ui.set_height(32.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Your Peer ID:")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(71, 85, 105)),
                );

                let id = state.iroh_endpoint.id().to_string();
                ui.label(
                    RichText::new(&id[..id.len().min(32)])
                        .size(14.0)
                        .monospace()
                        .color(egui::Color32::from_rgb(100, 116, 139)),
                );

                ui.add_space(8.0);

                let copy_button = egui::Button::new(RichText::new("📋 Copy").size(12.0))
                    .min_size(egui::vec2(80.0, 28.0))
                    .corner_radius(6);

                if ui.add(copy_button).clicked() {
                    ui.ctx().copy_text(id);
                }
            });
        });

        ui.add_space(16.0);

        // Active users
        ui.horizontal(|ui| {
            ui.set_width(ui.available_width());
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Active users:")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(71, 85, 105)),
                );

                ui.add_space(8.0);

                // Own user
                let own_color = generate_peer_color(&state.own_id);
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(
                        own_color.r(),
                        own_color.g(),
                        own_color.b(),
                        30,
                    ))
                    .stroke(egui::Stroke::new(1.0, own_color))
                    .corner_radius(12)
                    .inner_margin(egui::vec2(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&state.own_name).size(12.0).color(own_color));
                    });

                // Other peers
                state
                    .awareness_cache
                    .iter()
                    .for_each(|(_, (awareness, _))| {
                        ui.add_space(6.0);
                        let peer_color = generate_peer_color(&awareness.endpoint_id);
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(
                                peer_color.r(),
                                peer_color.g(),
                                peer_color.b(),
                                30,
                            ))
                            .stroke(egui::Stroke::new(1.0, peer_color))
                            .corner_radius(12)
                            .inner_margin(egui::vec2(8.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(&awareness.name).size(12.0).color(peer_color),
                                );
                            });
                    });
            });
        });

        ui.add_space(24.0);

        // Text editor in a styled frame
        let editor_frame = egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(226, 232, 240),
            ))
            .corner_radius(8)
            .inner_margin(egui::vec2(16.0, 16.0));

        {
            let doc_text = state.loro_doc.get_text("text");
            let mut text_content = doc_text.to_string();

            let text_edit_id = ui.id().with("text_edit");

            if state.egui_cursors_needs_update {
                state.egui_cursors_needs_update = false;
                update_egui_from_loro_cursors(ui, text_edit_id, &state.loro_doc, &state.cursors);
            }

            let output = editor_frame
                .show(ui, |ui| {
                    let front_layer_id = LayerId::new(ui.layer_id().order, ui.id().with("front"));
                    ui.ctx().set_sublayer(ui.layer_id(), front_layer_id);

                    ui.scope_builder(UiBuilder::new().layer_id(front_layer_id), |ui| {
                        TextEdit::multiline(&mut text_content)
                            .id(text_edit_id)
                            .frame(false)
                            .background_color(Color32::TRANSPARENT)
                            .font(egui::FontId::new(18.0, egui::FontFamily::Proportional))
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .show(ui)
                    })
                    .inner
                })
                .inner;

            if output.response.changed() {
                let _ = doc_text.update(&text_content, Default::default());
                state.loro_doc.commit();
            }

            if state.egui_cursors_needs_update {
                state.egui_cursors_needs_update = false;
            } else {
                let new_cursors = get_loro_cursors_from_egui(&output, &doc_text);
                if new_cursors != state.cursors {
                    state.cursors = new_cursors;
                    let _ = broadcast_awareness(state);
                }
            }

            render_peer_cursors(ui, &output, &state.awareness_cache, &state.loro_doc);
        }
    });
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
