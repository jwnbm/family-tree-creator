use crate::app::App;
use crate::core::tree::{PersonId, EventId};
use crate::core::layout::LayoutEngine;
use crate::core::i18n::Texts;
use crate::ui::{EventNodeRenderer, SideTab};
use std::collections::HashMap;

impl EventNodeRenderer for App {
    fn render_event_nodes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        _screen_rects: &HashMap<PersonId, egui::Rect>,
        pointer_pos: Option<egui::Pos2>,
    ) -> (bool, bool) {
        let mut event_hovered = false;
        let mut any_event_dragged = false;

        let origin = self.canvas.canvas_origin;
        let zoom = self.canvas.zoom;
        let lang = self.ui.language;

        let event_ids: Vec<EventId> = self.tree.events.keys().copied().collect();
        for event_id in event_ids {
            let event = self.tree.events.get(&event_id).unwrap();
            let (name, date, description, color, is_sel, is_dragging) = (
                event.name.clone(),
                event.date.clone(),
                event.description.clone(),
                event.color,
                self.event_editor.selected == Some(event_id),
                self.canvas.dragging_event == Some(event_id),
            );
            
            let rect = LayoutEngine::calculate_event_screen_rect(
                event,
                origin,
                zoom,
                self.canvas.pan,
                lang,
            );

            let (r, g, b) = color;
            let base_color = egui::Color32::from_rgb(r, g, b);

            let fill = if is_dragging {
                egui::Color32::from_rgb(
                    (base_color.r() as f32 * 0.85) as u8,
                    (base_color.g() as f32 * 0.85) as u8,
                    (base_color.b() as f32 * 0.7) as u8,
                )
            } else if is_sel {
                egui::Color32::from_rgb(
                    (base_color.r() as f32 * 1.0).min(255.0) as u8,
                    (base_color.g() as f32 * 0.98).min(255.0) as u8,
                    (base_color.b() as f32 * 0.78).min(255.0) as u8,
                )
            } else {
                base_color
            };

            // イベントノードは角を丸くせず、実線の枠で描画して人物ノードと区別
            painter.rect_filled(rect, 3.0, fill);
            painter.rect_stroke(rect, 3.0, egui::Stroke::new(2.0, egui::Color32::DARK_GRAY), egui::epaint::StrokeKind::Outside);

            let text = if name.is_empty() {
                Texts::get("new_event", lang)
            } else {
                name.clone()
            };
            
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(13.0 * zoom.clamp(0.7, 1.2)),
                egui::Color32::BLACK,
            );

            // ツールチップ
            let event_node_id = ui.id().with(("event", event_id));
            let event_response = ui.interact(rect, event_node_id, egui::Sense::hover());
            if event_response.hovered() {
                let mut tooltip_text = format!("{}\n", name);
                if let Some(d) = &date {
                    tooltip_text.push_str(&format!("{}: {}\n", Texts::get("date", self.ui.language), d));
                }
                if !description.is_empty() {
                    tooltip_text.push_str(&format!("{}: {}", Texts::get("description", self.ui.language), description));
                }
                event_response.on_hover_text(tooltip_text);
            }

            // インタラクション処理
            let event_interact_id = ui.id().with(("event_interact", event_id));
            let interact_response = ui.interact(rect, event_interact_id, egui::Sense::click_and_drag());

            if interact_response.hovered() {
                event_hovered = true;
            }

            if interact_response.drag_started() {
                self.canvas.dragging_event = Some(event_id);
                self.canvas.event_drag_start = pointer_pos;
                let event_name = if name.is_empty() {
                    Texts::get("new_event", lang).to_string()
                } else {
                    name.clone()
                };
                let t = |key: &str| Texts::get(key, lang);
                self.log.add(format!("{}: {}", t("log_event_drag_started"), event_name));
            }

            if interact_response.dragged() && self.canvas.dragging_event == Some(event_id) {
                any_event_dragged = true;
                if let (Some(pos), Some(start)) = (pointer_pos, self.canvas.event_drag_start) {
                    let delta = (pos - start) / self.canvas.zoom;
                    
                    if let Some(event) = self.tree.events.get_mut(&event_id) {
                        let current_pos = event.position;
                        event.position.0 = current_pos.0 + delta.x;
                        event.position.1 = current_pos.1 + delta.y;
                    }
                    self.canvas.event_drag_start = pointer_pos;
                }
            }

            if interact_response.drag_stopped() && self.canvas.dragging_event == Some(event_id) {
                let event_name = if name.is_empty() {
                    Texts::get("new_event", lang).to_string()
                } else {
                    name.clone()
                };
                let t = |key: &str| Texts::get(key, lang);
                self.log.add(format!("{}: {}", t("log_event_moved"), event_name));
                
                if self.canvas.show_grid {
                    if let Some(event) = self.tree.events.get_mut(&event_id) {
                        let (x, y) = event.position;
                        let relative_pos = egui::pos2(x - origin.x, y - origin.y);
                        let snapped_rel = LayoutEngine::snap_to_grid(relative_pos, self.canvas.grid_size);
                        event.position = (origin.x + snapped_rel.x, origin.y + snapped_rel.y);
                    }
                }
                self.canvas.dragging_event = None;
                self.canvas.event_drag_start = None;
            }

            if interact_response.clicked() {
                self.event_editor.selected = Some(event_id);
                self.event_editor.new_event_name = name.clone();
                self.event_editor.new_event_date = date.unwrap_or_default();
                self.event_editor.new_event_description = description;
                let (r, g, b) = color;
                self.event_editor.new_event_color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                self.ui.side_tab = SideTab::Events;
                
                let event_name = if name.is_empty() {
                    Texts::get("new_event", lang).to_string()
                } else {
                    name
                };
                let t = |key: &str| Texts::get(key, lang);
                self.log.add(format!("{}: {}", t("log_event_selected"), event_name));
            }
        }
        
        (event_hovered, any_event_dragged)
    }
}
