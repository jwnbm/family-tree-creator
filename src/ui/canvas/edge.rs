use crate::app::{App, EDGE_STROKE_WIDTH, SPOUSE_LINE_OFFSET};
use crate::core::tree::{PersonId, Gender};
use crate::ui::EdgeRenderer;
use std::collections::HashMap;

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
