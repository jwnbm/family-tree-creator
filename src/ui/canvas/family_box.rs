use crate::app::App;
use crate::core::tree::PersonId;
use crate::core::i18n::Texts;
use crate::ui::{FamilyBoxRenderer, SideTab};
use std::collections::HashMap;

impl FamilyBoxRenderer for App {
    fn render_family_boxes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        for family in &self.tree.families {
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            
            for member_id in &family.members {
                if let Some(rect) = screen_rects.get(member_id) {
                    min_x = min_x.min(rect.min.x);
                    min_y = min_y.min(rect.min.y);
                    max_x = max_x.max(rect.max.x);
                    max_y = max_y.max(rect.max.y);
                }
            }
            
            if min_x < f32::MAX {
                let padding = 20.0;
                let label_height = 24.0;  // ラベルの高さ
                let label_padding = 8.0;   // ラベルと枠の間のスペース
                
                let family_rect = egui::Rect::from_min_max(
                    egui::pos2(min_x - padding, min_y - padding - label_height - label_padding),
                    egui::pos2(max_x + padding, max_y + padding)
                );
                
                let color = if let Some((r, g, b)) = family.color {
                    egui::Color32::from_rgba_unmultiplied(r, g, b, 30)
                } else {
                    egui::Color32::from_rgba_unmultiplied(200, 200, 255, 30)
                };
                
                let stroke_color = if let Some((r, g, b)) = family.color {
                    egui::Color32::from_rgb(r, g, b)
                } else {
                    egui::Color32::from_rgb(100, 100, 200)
                };
                
                painter.rect_filled(family_rect, 8.0, color);
                painter.rect_stroke(
                    family_rect,
                    8.0,
                    egui::Stroke::new(2.0, stroke_color),
                    egui::epaint::StrokeKind::Outside
                );
                
                // ラベルを枠の上部外側に配置
                let label_pos = egui::pos2(
                    family_rect.left() + padding,
                    family_rect.top() + 4.0
                );
                let label_size = egui::vec2(
                    (family_rect.width() - padding * 2.0).max(80.0),
                    label_height - 8.0
                );
                let label_rect = egui::Rect::from_min_size(label_pos, label_size);
                
                let resp = ui.interact(label_rect, egui::Id::new(("family_label", family.id)), egui::Sense::click());
                
                let bg_color = if resp.is_pointer_button_down_on() {
                    egui::Color32::from_rgba_unmultiplied(
                        stroke_color.r(), 
                        stroke_color.g(), 
                        stroke_color.b(), 
                        100
                    )
                } else if resp.hovered() {
                    egui::Color32::from_rgba_unmultiplied(
                        stroke_color.r(), 
                        stroke_color.g(), 
                        stroke_color.b(), 
                        60
                    )
                } else {
                    egui::Color32::from_rgba_unmultiplied(
                        stroke_color.r(), 
                        stroke_color.g(), 
                        stroke_color.b(), 
                        30
                    )
                };
                
                painter.rect_filled(label_rect, 3.0, bg_color);
                
                if resp.hovered() || resp.is_pointer_button_down_on() {
                    painter.rect_stroke(
                        label_rect,
                        3.0,
                        egui::Stroke::new(1.5, stroke_color),
                        egui::epaint::StrokeKind::Outside
                    );
                }
                
                let text_color = if resp.hovered() || resp.is_pointer_button_down_on() {
                    stroke_color
                } else {
                    egui::Color32::from_rgb(
                        (stroke_color.r() as f32 * 0.8) as u8,
                        (stroke_color.g() as f32 * 0.8) as u8,
                        (stroke_color.b() as f32 * 0.8) as u8,
                    )
                };
                
                painter.text(
                    label_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &family.name,
                    egui::FontId::proportional(11.0 * self.canvas.zoom.clamp(0.7, 1.2)),
                    text_color,
                );
                
                if resp.clicked() {
                    self.family_editor.selected_family = Some(family.id);
                    self.family_editor.new_family_name = family.name.clone();
                    if let Some((r, g, b)) = family.color {
                        self.family_editor.new_family_color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                    }
                    self.ui.side_tab = SideTab::Families;
                    let lang = self.ui.language;
                    let t = |key: &str| Texts::get(key, lang);
                    self.file.status = format!("{} {}", t("selected_family"), family.name);
                    self.log.add(format!("{}: {}", t("log_family_selected"), family.name));
                }
            }
        }
    }
}
