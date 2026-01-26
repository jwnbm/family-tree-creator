use crate::app::{App, NODE_CORNER_RADIUS};
use crate::core::tree::{PersonId, Gender};
use crate::core::layout::LayoutEngine;
use crate::ui::NodeRenderer;
use std::collections::HashMap;

impl NodeRenderer for App {
    fn render_canvas_nodes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        for n in nodes {
            if let Some(r) = screen_rects.get(&n.id) {
                let is_sel = self.person_editor.selected == Some(n.id);
                let is_multi_selected = self.person_editor.selected_ids.contains(&n.id);
                let is_dragging = self.canvas.dragging_node == Some(n.id);
                
                let person = self.tree.persons.get(&n.id);
                let gender = person.map(|p| p.gender).unwrap_or(Gender::Unknown);
                let base_color = match gender {
                    Gender::Male => egui::Color32::from_rgb(173, 216, 230),
                    Gender::Female => egui::Color32::from_rgb(255, 182, 193),
                    Gender::Unknown => egui::Color32::from_rgb(245, 245, 245),
                };
                
                let fill = if is_dragging {
                    egui::Color32::from_rgb(255, 220, 180)
                } else if is_sel {
                    match gender {
                        Gender::Male => egui::Color32::from_rgb(200, 235, 255),
                        Gender::Female => egui::Color32::from_rgb(255, 220, 230),
                        Gender::Unknown => egui::Color32::from_rgb(200, 230, 255),
                    }
                } else if is_multi_selected {
                    // 複数選択されているが最後の選択ではないノードは薄い色
                    match gender {
                        Gender::Male => egui::Color32::from_rgb(190, 225, 245),
                        Gender::Female => egui::Color32::from_rgb(255, 210, 220),
                        Gender::Unknown => egui::Color32::from_rgb(225, 240, 255),
                    }
                } else {
                    base_color
                };

                painter.rect_filled(*r, NODE_CORNER_RADIUS, fill);
                
                // 複数選択されているノードには太い枠線
                let stroke_width = if is_multi_selected { 2.0 } else { 1.0 };
                let stroke_color = if is_sel {
                    egui::Color32::from_rgb(0, 100, 200) // 最後の選択は濃い青
                } else if is_multi_selected {
                    egui::Color32::from_rgb(100, 150, 200) // 他の選択は薄い青
                } else {
                    egui::Color32::GRAY
                };
                painter.rect_stroke(*r, NODE_CORNER_RADIUS, egui::Stroke::new(stroke_width, stroke_color), egui::epaint::StrokeKind::Outside);

                // 写真表示モードの場合、写真を表示
                if let Some(person) = person {
                    if person.display_mode == crate::core::tree::PersonDisplayMode::NameAndPhoto {
                        if let Some(photo_path) = &person.photo_path {
                            if !photo_path.is_empty() {
                                // 名前表示領域の高さを固定（30.0ピクセル）
                                let name_area_height = 30.0;
                                // 写真領域は残りの高さ全体
                                let photo_height = r.height() - name_area_height;
                                let photo_rect = egui::Rect::from_min_size(
                                    r.min,
                                    egui::vec2(r.width(), photo_height),
                                );
                                
                                // 写真読み込みを試みる
                                if let Ok(image_data) = std::fs::read(photo_path) {
                                    if let Ok(image) = image::load_from_memory(&image_data) {
                                        let size = [image.width() as _, image.height() as _];
                                        let rgba = image.to_rgba8();
                                        let pixels = rgba.as_flat_samples();
                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                            size,
                                            pixels.as_slice(),
                                        );
                                        
                                        let texture = ui.ctx().load_texture(
                                            format!("person_photo_{}", n.id),
                                            color_image,
                                            Default::default(),
                                        );
                                        
                                        painter.image(
                                            texture.id(),
                                            photo_rect,
                                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                            egui::Color32::WHITE,
                                        );
                                    }
                                }
                                
                                // 名前は下部の固定領域に表示
                                let text_center = egui::pos2(r.center().x, r.min.y + photo_height + name_area_height / 2.0);
                                let text = LayoutEngine::person_label(&self.tree, n.id);
                                painter.text(
                                    text_center,
                                    egui::Align2::CENTER_CENTER,
                                    text,
                                    egui::FontId::proportional(14.0 * self.canvas.zoom.clamp(0.7, 1.2)),
                                    egui::Color32::BLACK,
                                );
                            } else {
                                // 写真パスが空の場合は名前のみ表示
                                let text = LayoutEngine::person_label(&self.tree, n.id);
                                painter.text(
                                    r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    text,
                                    egui::FontId::proportional(14.0 * self.canvas.zoom.clamp(0.7, 1.2)),
                                    egui::Color32::BLACK,
                                );
                            }
                        } else {
                            // photo_pathがNoneの場合は名前のみ表示
                            let text = LayoutEngine::person_label(&self.tree, n.id);
                            painter.text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::proportional(14.0 * self.canvas.zoom.clamp(0.7, 1.2)),
                                egui::Color32::BLACK,
                            );
                        }
                    } else {
                        // 名前のみモード
                        let text = LayoutEngine::person_label(&self.tree, n.id);
                        painter.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(14.0 * self.canvas.zoom.clamp(0.7, 1.2)),
                            egui::Color32::BLACK,
                        );
                    }
                }
                
                // ツールチップを表示
                let node_id = ui.id().with(n.id);
                let node_response = ui.interact(*r, node_id, egui::Sense::hover());
                if node_response.hovered() {
                    let tooltip_text = LayoutEngine::person_tooltip(&self.tree, n.id, self.ui.language);
                    node_response.on_hover_text(tooltip_text);
                }
            }
        }
    }
}


