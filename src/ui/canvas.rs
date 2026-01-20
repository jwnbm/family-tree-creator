use crate::app::{App, NODE_CORNER_RADIUS, SPOUSE_LINE_OFFSET, EDGE_STROKE_WIDTH};
use super::SideTab;
use crate::core::tree::{PersonId, Gender, EventId};
use crate::core::layout::LayoutEngine;
use crate::core::i18n::Texts;
use std::collections::HashMap;

/// キャンバスのメイン描画トレイト
pub trait CanvasRenderer {
    fn render_canvas(&mut self, ctx: &egui::Context);
}

/// ノード描画トレイト
pub trait NodeRenderer {
    fn render_canvas_nodes(
        &mut self,
        _ui: &mut egui::Ui,
        painter: &egui::Painter,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}

/// ノードインタラクショントレイト
pub trait NodeInteractionHandler {
    fn handle_node_interactions(
        &mut self,
        ui: &mut egui::Ui,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
        pointer_pos: Option<egui::Pos2>,
        origin: egui::Pos2,
    ) -> (bool, bool);
}

/// パン・ズーム処理トレイト
pub trait PanZoomHandler {
    fn handle_pan_zoom(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        pointer_pos: Option<egui::Pos2>,
        node_hovered: bool,
        any_node_dragged: bool,
        event_hovered: bool,
        any_event_dragged: bool,
    );
}

/// エッジ描画トレイト
pub trait EdgeRenderer {
    fn render_canvas_edges(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}

/// 家族の枠描画トレイト
pub trait FamilyBoxRenderer {
    fn render_family_boxes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}

/// イベントノード描画トレイト
pub trait EventNodeRenderer {
    fn render_event_nodes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
        pointer_pos: Option<egui::Pos2>,
    ) -> (bool, bool); // (event_hovered, any_event_dragged)
}

/// イベント関係線描画トレイト
pub trait EventRelationRenderer {
    fn render_event_relations(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}

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
                } else {
                    base_color
                };

                painter.rect_filled(*r, NODE_CORNER_RADIUS, fill);
                painter.rect_stroke(*r, NODE_CORNER_RADIUS, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::epaint::StrokeKind::Outside);

                // 写真表示モードの場合、写真を表示
                if let Some(person) = person {
                    if person.display_mode == crate::core::tree::PersonDisplayMode::NameAndPhoto {
                        if let Some(photo_path) = &person.photo_path {
                            if !photo_path.is_empty() {
                                // 写真領域（上部）
                                let photo_height = r.height() * 0.6;
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
                                
                                // 名前は下部に表示
                                let text_center = egui::pos2(r.center().x, r.min.y + photo_height + (r.height() - photo_height) / 2.0);
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

impl NodeInteractionHandler for App {
    fn handle_node_interactions(
        &mut self,
        ui: &mut egui::Ui,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
        pointer_pos: Option<egui::Pos2>,
        origin: egui::Pos2,
    ) -> (bool, bool) {
        let mut node_hovered = false;
        let mut any_node_dragged = false;
        
        for n in nodes {
            if let Some(r) = screen_rects.get(&n.id) {
                let node_id = ui.id().with(n.id);
                let node_response = ui.interact(*r, node_id, egui::Sense::click_and_drag());
                
                if node_response.hovered() {
                    node_hovered = true;
                }
                
                if node_response.drag_started() {
                    self.canvas.dragging_node = Some(n.id);
                    self.canvas.node_drag_start = pointer_pos;
                }
                
                if node_response.dragged() && self.canvas.dragging_node == Some(n.id) {
                    any_node_dragged = true;
                    if let (Some(pos), Some(start)) = (pointer_pos, self.canvas.node_drag_start) {
                        let delta = (pos - start) / self.canvas.zoom;
                        
                        if let Some(person) = self.tree.persons.get_mut(&n.id) {
                            let current_pos = person.position;
                            let new_x = current_pos.0 + delta.x;
                            let new_y = current_pos.1 + delta.y;
                            
                            person.position = (new_x, new_y);
                        }
                        self.canvas.node_drag_start = pointer_pos;
                    }
                }
                
                if node_response.drag_stopped() && self.canvas.dragging_node == Some(n.id) {
                    if self.canvas.show_grid {
                        if let Some(person) = self.tree.persons.get_mut(&n.id) {
                            let (x, y) = person.position;
                            let relative_pos = egui::pos2(x - origin.x, y - origin.y);
                            let snapped_rel = LayoutEngine::snap_to_grid(relative_pos, self.canvas.grid_size);
                            
                            let snapped_x = origin.x + snapped_rel.x;
                            let snapped_y = origin.y + snapped_rel.y;
                            
                            person.position = (snapped_x, snapped_y);
                        }
                    }
                    self.canvas.dragging_node = None;
                    self.canvas.node_drag_start = None;
                }
                
                if node_response.clicked() {
                    self.person_editor.selected = Some(n.id);
                    if let Some(person) = self.tree.persons.get(&n.id) {
                        self.person_editor.new_name = person.name.clone();
                        self.person_editor.new_gender = person.gender;
                        self.person_editor.new_birth = person.birth.clone().unwrap_or_default();
                        self.person_editor.new_memo = person.memo.clone();
                        self.person_editor.new_deceased = person.deceased;
                        self.person_editor.new_death = person.death.clone().unwrap_or_default();
                        self.person_editor.new_photo_path = person.photo_path.clone().unwrap_or_default();
                        self.person_editor.new_display_mode = person.display_mode;
                    }
                }
            }
        }
        
        (node_hovered, any_node_dragged)
    }
}

impl PanZoomHandler for App {
    fn handle_pan_zoom(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        pointer_pos: Option<egui::Pos2>,
        node_hovered: bool,
        any_node_dragged: bool,
        event_hovered: bool,
        any_event_dragged: bool,
    ) {
        let any_hovered = node_hovered || event_hovered;
        let any_dragged = any_node_dragged || any_event_dragged;
        let any_dragging = self.canvas.dragging_node.is_some() || self.canvas.dragging_event.is_some();
        
        if !any_hovered && !any_dragged && !any_dragging {
            if let Some(pos) = pointer_pos {
                let primary_down = ui.input(|i| i.pointer.primary_down());
                let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
                
                if primary_pressed && rect.contains(pos) {
                    self.canvas.dragging_pan = true;
                    self.canvas.last_pointer_pos = Some(pos);
                }
                
                if self.canvas.dragging_pan && primary_down {
                    if let Some(prev) = self.canvas.last_pointer_pos {
                        self.canvas.pan += pos - prev;
                        self.canvas.last_pointer_pos = Some(pos);
                    }
                }
                
                if !primary_down && self.canvas.dragging_pan {
                    self.canvas.dragging_pan = false;
                    self.canvas.last_pointer_pos = None;
                }
            }
        } else if !any_dragged {
            self.canvas.dragging_pan = false;
            self.canvas.last_pointer_pos = None;
        }
    }
}

impl EdgeRenderer for App {
    fn render_canvas_edges(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        // 配偶者の線
        for s in &self.tree.spouses {
            if let (Some(r1), Some(r2)) = (screen_rects.get(&s.person1), screen_rects.get(&s.person2)) {
                let a = r1.center();
                let b = r2.center();
                
                let dir = (b - a).normalized();
                let perpendicular = egui::vec2(-dir.y, dir.x) * SPOUSE_LINE_OFFSET;
                
                painter.line_segment(
                    [a + perpendicular, b + perpendicular],
                    egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY),
                );
                painter.line_segment(
                    [a - perpendicular, b - perpendicular],
                    egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY),
                );
                
                // メモがある場合、ツールチップを表示
                if !s.memo.is_empty() {
                    let mid = egui::pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
                    let line_rect = egui::Rect::from_center_size(
                        mid,
                        egui::vec2((b.x - a.x).abs().max(20.0), (b.y - a.y).abs().max(20.0))
                    );
                    let line_id = ui.id().with(("spouse_line", s.person1, s.person2));
                    let line_response = ui.interact(line_rect, line_id, egui::Sense::hover());
                    if line_response.hovered() {
                        line_response.on_hover_text(&s.memo);
                    }
                }
            }
        }

        // 親子の線
        let mut child_to_parents: HashMap<PersonId, Vec<PersonId>> = HashMap::new();
        for e in &self.tree.edges {
            child_to_parents.entry(e.child).or_default().push(e.parent);
        }

        let mut processed_children = std::collections::HashSet::new();

        for e in &self.tree.edges {
            let child_id = e.child;
            
            if processed_children.contains(&child_id) {
                continue;
            }
            
            if let Some(parents) = child_to_parents.get(&child_id) {
                let mut father_id = None;
                let mut mother_id = None;
                let mut other_parents = Vec::new();
                
                for parent_id in parents {
                    if let Some(parent) = self.tree.persons.get(parent_id) {
                        match parent.gender {
                            Gender::Male if father_id.is_none() => father_id = Some(*parent_id),
                            Gender::Female if mother_id.is_none() => mother_id = Some(*parent_id),
                            _ => other_parents.push(*parent_id),
                        }
                    }
                }
                
                if let (Some(father), Some(mother)) = (father_id, mother_id) {
                    let are_spouses = self.tree.spouses.iter().any(|s| {
                        (s.person1 == father && s.person2 == mother) ||
                        (s.person1 == mother && s.person2 == father)
                    });
                    
                    if are_spouses {
                        if let (Some(rf), Some(rm), Some(rc)) = (
                            screen_rects.get(&father),
                            screen_rects.get(&mother),
                            screen_rects.get(&child_id)
                        ) {
                            let father_center = rf.center();
                            let mother_center = rm.center();
                            let mid = egui::pos2(
                                (father_center.x + mother_center.x) / 2.0,
                                (father_center.y + mother_center.y) / 2.0
                            );
                            let child_top = rc.center_top();
                            
                            painter.line_segment([mid, child_top], egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY));
                        }
                    } else {
                        if let (Some(rf), Some(rm), Some(rc)) = (
                            screen_rects.get(&father),
                            screen_rects.get(&mother),
                            screen_rects.get(&child_id)
                        ) {
                            let father_center = rf.center();
                            let mother_center = rm.center();
                            
                            painter.line_segment(
                                [father_center, mother_center],
                                egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY)
                            );
                            
                            let mid = egui::pos2(
                                (father_center.x + mother_center.x) / 2.0,
                                (father_center.y + mother_center.y) / 2.0
                            );
                            let child_top = rc.center_top();
                            
                            painter.line_segment([mid, child_top], egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY));
                        }
                    }
                    processed_children.insert(child_id);
                    continue;
                }
            }
            
            if let (Some(rp), Some(rc)) = (screen_rects.get(&e.parent), screen_rects.get(&e.child)) {
                let a = rp.center_bottom();
                let b = rc.center_top();
                painter.line_segment([a, b], egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY));
            }
        }
    }
}

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
                }
            }
        }
    }
}

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
                self.event_editor.new_event_name = name;
                self.event_editor.new_event_date = date.unwrap_or_default();
                self.event_editor.new_event_description = description;
                let (r, g, b) = color;
                self.event_editor.new_event_color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                self.ui.side_tab = SideTab::Events;
            }
        }
        
        (event_hovered, any_event_dragged)
    }
}

impl EventRelationRenderer for App {
    fn render_event_relations(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        use crate::core::tree::EventRelationType;

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

            let nodes = LayoutEngine::compute_layout(&self.tree, origin);

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
