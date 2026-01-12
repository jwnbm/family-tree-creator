use std::collections::{HashMap, VecDeque};
use eframe::egui;
use crate::tree::{FamilyTree, PersonId};

/// 画面上のノード情報
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: PersonId,
    pub generation: usize, // 世代(0=ルート)
    pub pos: egui::Pos2,
    pub rect: egui::Rect,
}

/// レイアウト計算とラベル生成を担当するモジュール
pub struct LayoutEngine;

impl LayoutEngine {
    /// ノードのレイアウトを計算
    pub fn compute_layout(tree: &FamilyTree, origin: egui::Pos2) -> Vec<LayoutNode> {
        // 世代計算：ルートを0として子へ+1
        let roots = tree.roots();
        let mut gen_map: HashMap<PersonId, usize> = HashMap::new();
        let mut q = VecDeque::new();

        for r in &roots {
            gen_map.insert(*r, 0);
            q.push_back(*r);
        }

        while let Some(pid) = q.pop_front() {
            let g = gen_map[&pid];
            for ch in tree.children_of(pid) {
                let new_g = g + 1;
                let entry = gen_map.entry(ch).or_insert(new_g);
                if new_g < *entry {
                    *entry = new_g;
                }
                q.push_back(ch);
            }
        }

        if gen_map.is_empty() {
            for id in tree.persons.keys() {
                gen_map.insert(*id, 0);
            }
        } else {
            for id in tree.persons.keys() {
                gen_map.entry(*id).or_insert(0);
            }
        }

        let mut by_gen: HashMap<usize, Vec<PersonId>> = HashMap::new();
        for (id, g) in &gen_map {
            by_gen.entry(*g).or_default().push(*id);
        }

        for ids in by_gen.values_mut() {
            ids.sort_by_key(|id| tree.persons.get(id).map(|p| p.name.clone()).unwrap_or_default());
        }

        let node_w = 140.0;
        let node_h = 50.0;
        let x_gap = 50.0;
        let y_gap = 50.0;

        let mut nodes = Vec::new();
        let mut gens: Vec<usize> = by_gen.keys().copied().collect();
        gens.sort();

        for g in gens {
            if let Some(ids) = by_gen.get(&g) {
                for (i, id) in ids.iter().enumerate() {
                    let (x, y) = if let Some(person) = tree.persons.get(id) {
                        if let Some((px, py)) = person.position {
                            (px, py)
                        } else {
                            let auto_x = origin.x + (i as f32) * (node_w + x_gap);
                            let auto_y = origin.y + (g as f32) * (node_h + y_gap);
                            (auto_x, auto_y)
                        }
                    } else {
                        let auto_x = origin.x + (i as f32) * (node_w + x_gap);
                        let auto_y = origin.y + (g as f32) * (node_h + y_gap);
                        (auto_x, auto_y)
                    };
                    
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(node_w, node_h),
                    );
                    nodes.push(LayoutNode {
                        id: *id,
                        generation: g,
                        pos: egui::pos2(x, y),
                        rect,
                    });
                }
            }
        }

        nodes
    }

    /// 人物のラベル（表示テキスト）を生成
    pub fn person_label(tree: &FamilyTree, id: PersonId) -> String {
        if let Some(p) = tree.persons.get(&id) {
            let mut label = p.name.clone();
            
            let calculate_age = |birth: &str, end_date: Option<&str>| -> Option<i32> {
                let birth_year = birth.split('-').next()?.parse::<i32>().ok()?;
                let end_year = if let Some(ed) = end_date {
                    ed.split('-').next()?.parse::<i32>().ok()?
                } else {
                    2026
                };
                Some(end_year - birth_year)
            };
            
            if let Some(b) = &p.birth {
                if !b.is_empty() {
                    label.push_str(&format!("\n{}", b));
                    
                    if p.deceased {
                        if let Some(age) = calculate_age(b, p.death.as_deref()) {
                            label.push_str(&format!(" (died at {})", age));
                        }
                    } else {
                        if let Some(age) = calculate_age(b, None) {
                            label.push_str(&format!(" (age {})", age));
                        }
                    }
                }
            }
            
            if p.deceased {
                if let Some(d) = &p.death {
                    if !d.is_empty() {
                        label.push_str(&format!("\n† {}", d));
                    } else {
                        label.push_str("\n†");
                    }
                } else {
                    label.push_str("\n†");
                }
            }
            
            label
        } else {
            "Unknown".into()
        }
    }

    /// グリッド線を描画
    pub fn draw_grid(
        painter: &egui::Painter,
        rect: egui::Rect,
        origin: egui::Pos2,
        zoom: f32,
        pan: egui::Vec2,
        grid_size: f32,
    ) {
        let grid_size = grid_size * zoom;
        let grid_origin = origin + pan;
        
        let start_x = ((rect.left() - grid_origin.x) / grid_size).floor() * grid_size + grid_origin.x;
        let start_y = ((rect.top() - grid_origin.y) / grid_size).floor() * grid_size + grid_origin.y;
        
        let mut x = start_x;
        while x <= rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(0.5, egui::Color32::from_gray(220)),
            );
            x += grid_size;
        }
        
        let mut y = start_y;
        while y <= rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(220)),
            );
            y += grid_size;
        }
    }

    /// 座標をグリッドにスナップ
    pub fn snap_to_grid(pos: egui::Pos2, grid_size: f32) -> egui::Pos2 {
        let x = (pos.x / grid_size).round() * grid_size;
        let y = (pos.y / grid_size).round() * grid_size;
        egui::pos2(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{FamilyTree, Gender};

    #[test]
    fn test_person_label_basic() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "Test Person".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
        );
        
        let label = LayoutEngine::person_label(&tree, id);
        assert_eq!(label, "Test Person");
    }

    #[test]
    fn test_person_label_with_birth() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "John".to_string(),
            Gender::Male,
            Some("1990-05-15".to_string()),
            "".to_string(),
            false,
            None,
        );
        
        let label = LayoutEngine::person_label(&tree, id);
        assert!(label.contains("John"));
        assert!(label.contains("1990-05-15"));
        assert!(label.contains("(age 36)"));
    }

    #[test]
    fn test_person_label_deceased() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "Jane".to_string(),
            Gender::Female,
            Some("1950-01-01".to_string()),
            "".to_string(),
            true,
            Some("2020-12-31".to_string()),
        );
        
        let label = LayoutEngine::person_label(&tree, id);
        assert!(label.contains("Jane"));
        assert!(label.contains("1950-01-01"));
        assert!(label.contains("(died at 70)"));
        assert!(label.contains("† 2020-12-31"));
    }

    #[test]
    fn test_person_label_deceased_without_death_date() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "Bob".to_string(),
            Gender::Male,
            Some("1960-06-10".to_string()),
            "".to_string(),
            true,
            None,
        );
        
        let label = LayoutEngine::person_label(&tree, id);
        assert!(label.contains("Bob"));
        assert!(label.contains("1960-06-10"));
        assert!(label.contains("†"));
    }

    #[test]
    fn test_compute_layout_single_person() {
        let mut tree = FamilyTree::default();
        tree.add_person(
            "Solo".to_string(),
            Gender::Unknown,
            None,
            "".to_string(),
            false,
            None,
        );
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = LayoutEngine::compute_layout(&tree, origin);
        
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].generation, 0);
    }

    #[test]
    fn test_compute_layout_parent_child() {
        let mut tree = FamilyTree::default();
        let parent = tree.add_person(
            "Parent".to_string(),
            Gender::Female,
            None,
            "".to_string(),
            false,
            None,
        );
        let child = tree.add_person(
            "Child".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
        );
        
        tree.add_parent_child(parent, child, "biological".to_string());
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = LayoutEngine::compute_layout(&tree, origin);
        
        assert_eq!(nodes.len(), 2);
        
        let parent_node = nodes.iter().find(|n| n.id == parent).unwrap();
        let child_node = nodes.iter().find(|n| n.id == child).unwrap();
        
        assert_eq!(parent_node.generation, 0);
        assert_eq!(child_node.generation, 1);
    }

    #[test]
    fn test_compute_layout_with_manual_position() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "Positioned".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
        );
        
        // 手動位置を設定
        if let Some(person) = tree.persons.get_mut(&id) {
            person.position = Some((100.0, 200.0));
        }
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = LayoutEngine::compute_layout(&tree, origin);
        
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].rect.left(), 100.0);
        assert_eq!(nodes[0].rect.top(), 200.0);
    }

    #[test]
    fn test_compute_layout_multiple_generations() {
        let mut tree = FamilyTree::default();
        let grandparent = tree.add_person("GP".to_string(), Gender::Male, None, "".to_string(), false, None);
        let parent = tree.add_person("P".to_string(), Gender::Female, None, "".to_string(), false, None);
        let child = tree.add_person("C".to_string(), Gender::Unknown, None, "".to_string(), false, None);
        
        tree.add_parent_child(grandparent, parent, "biological".to_string());
        tree.add_parent_child(parent, child, "biological".to_string());
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = LayoutEngine::compute_layout(&tree, origin);
        
        assert_eq!(nodes.len(), 3);
        
        let gp_node = nodes.iter().find(|n| n.id == grandparent).unwrap();
        let p_node = nodes.iter().find(|n| n.id == parent).unwrap();
        let c_node = nodes.iter().find(|n| n.id == child).unwrap();
        
        assert_eq!(gp_node.generation, 0);
        assert_eq!(p_node.generation, 1);
        assert_eq!(c_node.generation, 2);
        
        // Y座標が世代順になっていることを確認
        assert!(gp_node.rect.top() < p_node.rect.top());
        assert!(p_node.rect.top() < c_node.rect.top());
    }

    #[test]
    fn test_person_label_unknown_id() {
        let tree = FamilyTree::default();
        let fake_id = uuid::Uuid::new_v4();
        
        let label = LayoutEngine::person_label(&tree, fake_id);
        assert_eq!(label, "Unknown");
    }

    #[test]
    fn test_snap_to_grid() {
        let pos = egui::pos2(123.4, 567.8);
        let snapped = LayoutEngine::snap_to_grid(pos, 50.0);
        
        assert_eq!(snapped.x, 100.0);
        assert_eq!(snapped.y, 550.0);
    }
}
