use std::collections::HashMap;

use crate::app::App;
use crate::core::layout::LayoutEngine;
use crate::core::tree::PersonId;
use crate::infrastructure::read_image_dimensions;

use super::{CanvasRenderer, NodeRenderer, NodeInteractionHandler, PanZoomHandler, EdgeRenderer, FamilyBoxRenderer, EventNodeRenderer, EventRelationRenderer};

impl CanvasRenderer for App {
    fn render_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click());
            let pointer_pos = ui.input(|i| i.pointer.interact_pos());
            
            // キャンバス情報を保存
            self.canvas.canvas_rect = rect;

            // ズーム処理
            ctx.input(|i| {
                if i.modifiers.ctrl && i.raw_scroll_delta.y.abs() > 0.0 {
                    let factor = (i.raw_scroll_delta.y / 400.0).exp();
                    self.canvas.zoom = (self.canvas.zoom * factor).clamp(0.3, 3.0);
                }
            });

            let painter = ui.painter_at(rect);

            let to_screen = |p: egui::Pos2, zoom: f32, pan: egui::Vec2, origin: egui::Pos2| -> egui::Pos2 {
                let v = (p - origin) * zoom;
                origin + v + pan
            };

            let base_origin = rect.left_top() + egui::vec2(24.0, 24.0);
            let origin = if self.canvas.show_grid {
                LayoutEngine::snap_to_grid(base_origin, self.canvas.grid_size)
            } else {
                base_origin
            };
            
            // originを保存
            self.canvas.canvas_origin = origin;
            
            if self.canvas.show_grid {
                LayoutEngine::draw_grid(&painter, rect, origin, self.canvas.zoom, self.canvas.pan, self.canvas.grid_size);
            }

            let photo_dimensions: HashMap<PersonId, (u32, u32)> = self
                .tree
                .persons
                .iter()
                .filter_map(|(person_id, person)| {
                    if person.display_mode != crate::core::tree::PersonDisplayMode::NameAndPhoto {
                        return None;
                    }

                    person
                        .photo_path
                        .as_deref()
                        .and_then(read_image_dimensions)
                        .map(|dimensions| (*person_id, dimensions))
                })
                .collect();

            let nodes = LayoutEngine::compute_layout(&self.tree, origin, &photo_dimensions);

            let mut screen_rects: HashMap<PersonId, egui::Rect> = HashMap::new();
            for n in &nodes {
                let min = to_screen(n.rect.min, self.canvas.zoom, self.canvas.pan, origin);
                let max = to_screen(n.rect.max, self.canvas.zoom, self.canvas.pan, origin);
                screen_rects.insert(n.id, egui::Rect::from_min_max(min, max));
            }

            // ノードのインタラクション処理
            let (node_hovered, any_node_dragged) = self.handle_node_interactions(ui, &nodes, &screen_rects, pointer_pos, origin);
            
            // イベントノード描画（ホバー/ドラッグ状態を先に取得）
            let (event_hovered, any_event_dragged) = self.render_event_nodes(ui, &painter, &screen_rects, pointer_pos);
            
            // パン・ズーム処理
            self.handle_pan_zoom(ui, rect, pointer_pos, node_hovered, any_node_dragged, event_hovered, any_event_dragged);

            // エッジ（関係線）描画
            self.render_canvas_edges(ui, &painter, &screen_rects);

            // 家族の枠描画
            self.render_family_boxes(ui, &painter, &screen_rects);

            // ノード描画
            self.render_canvas_nodes(ui, &painter, &nodes, &screen_rects);

            // イベント関係線描画
            self.render_event_relations(ui, &painter, &screen_rects);

            // ズーム表示
            painter.text(
                rect.right_top() + egui::vec2(-10.0, 10.0),
                egui::Align2::RIGHT_TOP,
                format!("zoom: {:.2}", self.canvas.zoom),
                egui::FontId::proportional(12.0),
                egui::Color32::DARK_GRAY,
            );
        });
    }
}
