use crate::app::{App, EDGE_STROKE_WIDTH, SPOUSE_LINE_OFFSET};
use crate::core::tree::{Gender, PersonId};
use crate::ui::EdgeRenderer;
use std::collections::HashMap;

fn draw_vertical_polyline(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
) {
    let mid_y = (start.y + end.y) * 0.5;
    let bend_start = egui::pos2(start.x, mid_y);
    let bend_end = egui::pos2(end.x, mid_y);

    painter.line_segment([start, bend_start], stroke);
    painter.line_segment([bend_start, bend_end], stroke);
    painter.line_segment([bend_end, end], stroke);
}

fn draw_horizontal_polyline(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
) {
    let mid_x = (start.x + end.x) * 0.5;
    let bend_start = egui::pos2(mid_x, start.y);
    let bend_end = egui::pos2(mid_x, end.y);

    painter.line_segment([start, bend_start], stroke);
    painter.line_segment([bend_start, bend_end], stroke);
    painter.line_segment([bend_end, end], stroke);
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
                let stroke = egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY);

                let offset = egui::vec2(0.0, SPOUSE_LINE_OFFSET);
                let a_upper = a + offset;
                let b_upper = b + offset;
                let a_lower = a - offset;
                let b_lower = b - offset;

                draw_horizontal_polyline(painter, a_upper, b_upper, stroke);
                draw_horizontal_polyline(painter, a_lower, b_lower, stroke);
                
                // メモがある場合、ツールチップを表示
                if !s.memo.is_empty() {
                    let min_x = a.x.min(b.x);
                    let max_x = a.x.max(b.x);
                    let min_y = a.y.min(b.y);
                    let max_y = a.y.max(b.y);
                    let line_rect = egui::Rect::from_min_max(
                        egui::pos2(min_x, min_y),
                        egui::pos2(max_x, max_y),
                    )
                    .expand(12.0);
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
                            let stroke = egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY);

                            draw_vertical_polyline(painter, mid, child_top, stroke);
                        }
                    } else {
                        if let (Some(rf), Some(rm), Some(rc)) = (
                            screen_rects.get(&father),
                            screen_rects.get(&mother),
                            screen_rects.get(&child_id)
                        ) {
                            let father_center = rf.center();
                            let mother_center = rm.center();
                            let stroke = egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY);

                            draw_horizontal_polyline(painter, father_center, mother_center, stroke);
                            
                            let mid = egui::pos2(
                                (father_center.x + mother_center.x) / 2.0,
                                (father_center.y + mother_center.y) / 2.0
                            );
                            let child_top = rc.center_top();

                            draw_vertical_polyline(painter, mid, child_top, stroke);
                        }
                    }
                    processed_children.insert(child_id);
                    continue;
                }
            }
            
            if let (Some(rp), Some(rc)) = (screen_rects.get(&e.parent), screen_rects.get(&e.child)) {
                let a = rp.center_bottom();
                let b = rc.center_top();
                draw_vertical_polyline(
                    painter,
                    a,
                    b,
                    egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY),
                );
            }
        }
    }
}
