use crate::app::{App, EDGE_STROKE_WIDTH};
use crate::core::tree::{PersonId, EventRelationType};
use crate::core::layout::LayoutEngine;
use crate::ui::EventRelationRenderer;
use std::collections::HashMap;

impl EventRelationRenderer for App {
    fn render_event_relations(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        // イベント矩形を計算
        let origin = self.canvas.canvas_origin;
        let zoom = self.canvas.zoom;
        let lang = self.ui.language;
        
        let event_rects = LayoutEngine::calculate_event_screen_rects(
            &self.tree.events,
            origin,
            zoom,
            self.canvas.pan,
            lang,
        );

        for relation in &self.tree.event_relations {
            if let (Some(event_rect), Some(person_rect)) = (event_rects.get(&relation.event), screen_rects.get(&relation.person)) {
                // イベントの色を取得
                let (r, g, b) = self.tree.events.get(&relation.event)
                    .map(|e| e.color)
                    .unwrap_or((255, 255, 200));
                let event_color = egui::Color32::from_rgb(r, g, b);
                
                let event_center = event_rect.center();
                let person_center = person_rect.center();
                
                // ノードの端から線を引くための計算（矩形との交点を求める）
                let dir = (person_center - event_center).normalized();
                
                // イベントノードの境界との交点を計算
                let t_x_event = if dir.x.abs() > 0.001 {
                    (event_rect.width() / 2.0) / dir.x.abs()
                } else {
                    f32::INFINITY
                };
                let t_y_event = if dir.y.abs() > 0.001 {
                    (event_rect.height() / 2.0) / dir.y.abs()
                } else {
                    f32::INFINITY
                };
                let t_event = t_x_event.min(t_y_event);
                let start = event_center + dir * (t_event + 2.0); // 2ピクセルの余白を追加
                
                // 人物ノードの境界との交点を計算
                let t_x_person = if dir.x.abs() > 0.001 {
                    (person_rect.width() / 2.0) / dir.x.abs()
                } else {
                    f32::INFINITY
                };
                let t_y_person = if dir.y.abs() > 0.001 {
                    (person_rect.height() / 2.0) / dir.y.abs()
                } else {
                    f32::INFINITY
                };
                let t_person = t_x_person.min(t_y_person);
                let end = person_center - dir * (t_person + 2.0); // 2ピクセルの余白を追加

                let stroke = egui::Stroke::new(EDGE_STROKE_WIDTH, event_color);

                match relation.relation_type {
                    EventRelationType::Line => {
                        painter.line_segment([start, end], stroke);
                    }
                    EventRelationType::ArrowToPerson => {
                        // イベント → 人物（矢印は人物側）
                        painter.line_segment([start, end], stroke);
                        let arrow_dir = dir;
                        let arrow_size = 10.0;
                        let arrow_angle = std::f32::consts::PI / 6.0;
                        let perp1 = egui::vec2(
                            arrow_dir.x * arrow_angle.cos() - arrow_dir.y * arrow_angle.sin(),
                            arrow_dir.x * arrow_angle.sin() + arrow_dir.y * arrow_angle.cos(),
                        );
                        let perp2 = egui::vec2(
                            arrow_dir.x * arrow_angle.cos() + arrow_dir.y * arrow_angle.sin(),
                            -arrow_dir.x * arrow_angle.sin() + arrow_dir.y * arrow_angle.cos(),
                        );
                        painter.line_segment([end, end - perp1 * arrow_size], stroke);
                        painter.line_segment([end, end - perp2 * arrow_size], stroke);
                    }
                    EventRelationType::ArrowToEvent => {
                        // 人物 → イベント（矢印はイベント側）
                        painter.line_segment([start, end], stroke);
                        let arrow_dir = -dir;
                        let arrow_size = 10.0;
                        let arrow_angle = std::f32::consts::PI / 6.0;
                        let perp1 = egui::vec2(
                            arrow_dir.x * arrow_angle.cos() - arrow_dir.y * arrow_angle.sin(),
                            arrow_dir.x * arrow_angle.sin() + arrow_dir.y * arrow_angle.cos(),
                        );
                        let perp2 = egui::vec2(
                            arrow_dir.x * arrow_angle.cos() + arrow_dir.y * arrow_angle.sin(),
                            -arrow_dir.x * arrow_angle.sin() + arrow_dir.y * arrow_angle.cos(),
                        );
                        painter.line_segment([start, start - perp1 * arrow_size], stroke);
                        painter.line_segment([start, start - perp2 * arrow_size], stroke);
                    }
                }

                // メモのツールチップ
                if !relation.memo.is_empty() {
                    let mid_point = (start + end.to_vec2()) / 2.0;
                    let line_rect = egui::Rect::from_center_size(mid_point, egui::vec2(20.0, 20.0));
                    let line_id = ui.id().with(("event_relation", relation.event, relation.person));
                    let line_response = ui.interact(line_rect, line_id, egui::Sense::hover());
                    if line_response.hovered() {
                        line_response.on_hover_text(&relation.memo);
                    }
                }
            }
        }
    }
}
