use std::collections::{HashMap, VecDeque};
use eframe::egui;
use crate::core::tree::{FamilyTree, PersonId, EventId, Event};
use crate::core::i18n::{Language, Texts};

/// 画面上のノード情報
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: PersonId,
    #[allow(dead_code)]
    pub generation: usize, // 世代(0=ルート)
    #[allow(dead_code)]
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

        // 人物ノードの高さ：フォントサイズ14.0 + 上下パディング
        let font_size = 14.0;
        let padding_v = 16.0;
        let base_node_h = font_size + padding_v;
        let x_gap = 50.0;
        let y_gap = 80.0;

        let mut nodes = Vec::new();
        let mut gens: Vec<usize> = by_gen.keys().copied().collect();
        gens.sort();

        for g in gens {
            if let Some(ids) = by_gen.get(&g) {
                for (i, id) in ids.iter().enumerate() {
                    // 人物名からノード幅を計算
                    let person_name = tree.persons.get(id).map(|p| p.name.as_str()).unwrap_or("Unknown");
                    // 日本語も含めた文字列の表示幅を推定（1文字あたり約14ピクセル）
                    let char_count = person_name.chars().count();
                    let estimated_width = (char_count as f32 * 14.0).max(100.0).min(250.0);
                    let node_w = estimated_width;
                    let node_h = base_node_h;
                    
                    let (x, y) = if let Some(person) = tree.persons.get(id) {
                        person.position
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
            p.name.clone()
        } else {
            "Unknown".into()
        }
    }
    
    /// 人物の詳細情報をツールチップ用に生成
    pub fn person_tooltip(tree: &FamilyTree, id: PersonId, lang: Language) -> String {
        if let Some(p) = tree.persons.get(&id) {
            let mut tooltip = format!("{}: {}", Texts::get("tooltip_name", lang), p.name);
            
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
                    tooltip.push_str(&format!("\n{}: {}", Texts::get("tooltip_birth", lang), b));
                    
                    if p.deceased {
                        if let Some(age) = calculate_age(b, p.death.as_deref()) {
                            tooltip.push_str(&format!(" ({} {}{}) ", Texts::get("tooltip_died_at", lang), age, Texts::get("tooltip_age", lang)));
                        }
                    } else {
                        if let Some(age) = calculate_age(b, None) {
                            tooltip.push_str(&format!(" ({}{})", age, Texts::get("tooltip_age", lang)));
                        }
                    }
                }
            }
            
            if p.deceased {
                if let Some(d) = &p.death {
                    if !d.is_empty() {
                        tooltip.push_str(&format!("\n{}: {}", Texts::get("tooltip_death", lang), d));
                    } else {
                        tooltip.push_str(&format!("\n{}: {}", Texts::get("tooltip_deceased", lang), Texts::get("tooltip_yes", lang)));
                    }
                } else {
                    tooltip.push_str(&format!("\n{}: {}", Texts::get("tooltip_deceased", lang), Texts::get("tooltip_yes", lang)));
                }
            }
            
            if !p.memo.is_empty() {
                tooltip.push_str(&format!("\n{}: {}", Texts::get("tooltip_memo", lang), p.memo));
            }
            
            tooltip
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

    /// イベント名からノードサイズを計算
    pub fn calculate_event_node_size(event_name: &str, lang: Language) -> (f32, f32) {
        // イベントノードの高さ：フォントサイズ13.0 + 上下パディング
        let font_size = 13.0;
        let padding_v = 16.0;
        let base_node_h = font_size + padding_v;
        let padding_h = 20.0;
        
        let text = if event_name.is_empty() {
            Texts::get("new_event", lang)
        } else {
            event_name.to_string()
        };
        
        // 文字数から幅を推定（1文字あたり約13ピクセル）
        let char_count = text.chars().count();
        let estimated_width = (char_count as f32 * 13.0 + padding_h).max(120.0).min(250.0);
        
        (estimated_width, base_node_h)
    }

    /// イベントの画面矩形を計算
    pub fn calculate_event_screen_rect(
        event: &Event,
        origin: egui::Pos2,
        zoom: f32,
        pan: egui::Vec2,
        lang: Language,
    ) -> egui::Rect {
        let to_screen = |p: egui::Pos2| -> egui::Pos2 {
            let v = (p - origin) * zoom;
            origin + v + pan
        };
        
        let (node_w, node_h) = Self::calculate_event_node_size(&event.name, lang);
        let world_pos = egui::pos2(event.position.0, event.position.1);
        let screen_pos = to_screen(world_pos);
        
        egui::Rect::from_min_size(screen_pos, egui::vec2(node_w * zoom, node_h * zoom))
    }

    /// すべてのイベントの画面矩形を計算
    pub fn calculate_event_screen_rects(
        events: &HashMap<EventId, Event>,
        origin: egui::Pos2,
        zoom: f32,
        pan: egui::Vec2,
        lang: Language,
    ) -> HashMap<EventId, egui::Rect> {
        events
            .iter()
            .map(|(id, event)| {
                let rect = Self::calculate_event_screen_rect(event, origin, zoom, pan, lang);
                (*id, rect)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tree::{FamilyTree, Gender};

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
            (0.0, 0.0),
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
            (0.0, 0.0),
        );
        
        let label = LayoutEngine::person_label(&tree, id);
        assert_eq!(label, "John");
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
            (0.0, 0.0),
        );
        
        let label = LayoutEngine::person_label(&tree, id);
        assert_eq!(label, "Jane");
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
            (0.0, 0.0),
        );
        
        let label = LayoutEngine::person_label(&tree, id);
        assert_eq!(label, "Bob");
    }

    #[test]
    fn test_person_tooltip_basic() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "Test Person".to_string(),
            Gender::Unknown,
            None,
            "".to_string(),
            false,
            None,
            (0.0, 0.0),
        );
        
        let tooltip_ja = LayoutEngine::person_tooltip(&tree, id, Language::Japanese);
        assert!(tooltip_ja.contains("名前: Test Person"));
        
        let tooltip_en = LayoutEngine::person_tooltip(&tree, id, Language::English);
        assert!(tooltip_en.contains("Name: Test Person"));
    }

    #[test]
    fn test_person_tooltip_with_details() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "John".to_string(),
            Gender::Male,
            Some("1990-05-15".to_string()),
            "テストメモ".to_string(),
            false,
            None,
            (0.0, 0.0),
        );
        
        let tooltip_ja = LayoutEngine::person_tooltip(&tree, id, Language::Japanese);
        assert!(tooltip_ja.contains("名前: John"));
        assert!(tooltip_ja.contains("生年月日: 1990-05-15"));
        assert!(tooltip_ja.contains("36歳"));
        assert!(tooltip_ja.contains("メモ: テストメモ"));
        
        let tooltip_en = LayoutEngine::person_tooltip(&tree, id, Language::English);
        assert!(tooltip_en.contains("Name: John"));
        assert!(tooltip_en.contains("Birth: 1990-05-15"));
        assert!(tooltip_en.contains("36years old"));
        assert!(tooltip_en.contains("Memo: テストメモ"));
    }

    #[test]
    fn test_person_tooltip_deceased() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "Jane".to_string(),
            Gender::Female,
            Some("1950-01-01".to_string()),
            "".to_string(),
            true,
            Some("2020-12-31".to_string()),
            (0.0, 0.0),
        );
        
        let tooltip_ja = LayoutEngine::person_tooltip(&tree, id, Language::Japanese);
        assert!(tooltip_ja.contains("名前: Jane"));
        assert!(tooltip_ja.contains("生年月日: 1950-01-01"));
        assert!(tooltip_ja.contains("享年 70歳"));
        assert!(tooltip_ja.contains("没年月日: 2020-12-31"));
        
        let tooltip_en = LayoutEngine::person_tooltip(&tree, id, Language::English);
        assert!(tooltip_en.contains("Name: Jane"));
        assert!(tooltip_en.contains("Birth: 1950-01-01"));
        assert!(tooltip_en.contains("died at 70years old"));
        assert!(tooltip_en.contains("Death: 2020-12-31"));
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
            (50.0, 75.0),
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
            (0.0, 0.0),
        );
        let child = tree.add_person(
            "Child".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
            (0.0, 100.0),
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
        let _id = tree.add_person(
            "Positioned".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
            (100.0, 200.0),
        );
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = LayoutEngine::compute_layout(&tree, origin);
        
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].rect.left(), 100.0);
        assert_eq!(nodes[0].rect.top(), 200.0);
    }

    #[test]
    fn test_compute_layout_multiple_generations() {
        let mut tree = FamilyTree::default();
        let grandparent = tree.add_person("GP".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 0.0));
        let parent = tree.add_person("P".to_string(), Gender::Female, None, "".to_string(), false, None, (0.0, 100.0));
        let child = tree.add_person("C".to_string(), Gender::Unknown, None, "".to_string(), false, None, (0.0, 200.0));
        
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

    #[test]
    fn test_calculate_event_node_size_empty_name() {
        let (width, height) = LayoutEngine::calculate_event_node_size("", Language::Japanese);
        
        // 空の名前の場合、"New Event"が使用される
        assert!(width >= 120.0);
        assert!(width <= 250.0);
        assert_eq!(height, 29.0);
    }

    #[test]
    fn test_calculate_event_node_size_short_name() {
        let (width, height) = LayoutEngine::calculate_event_node_size("Test", Language::English);
        
        // 短い名前の場合、最小幅120.0が適用される
        assert_eq!(width, 120.0);
        assert_eq!(height, 29.0);
    }

    #[test]
    fn test_calculate_event_node_size_long_name() {
        let long_name = "This is a very long event name that should be truncated";
        let (width, height) = LayoutEngine::calculate_event_node_size(long_name, Language::English);
        
        // 長い名前の場合、最大幅250.0が適用される
        assert_eq!(width, 250.0);
        assert_eq!(height, 29.0);
    }

    #[test]
    fn test_calculate_event_node_size_japanese() {
        let (width, height) = LayoutEngine::calculate_event_node_size("イベント", Language::Japanese);
        
        assert!(width >= 120.0);
        assert!(width <= 250.0);
        assert_eq!(height, 29.0);
    }

    #[test]
    fn test_calculate_event_screen_rect() {
        let mut tree = FamilyTree::default();
        let event_id = tree.add_event(
            "Test Event".to_string(),
            None,
            "Description".to_string(),
            (100.0, 200.0),
            (255, 255, 200),
        );
        
        let event = tree.events.get(&event_id).unwrap();
        let origin = egui::pos2(0.0, 0.0);
        let zoom = 1.0;
        let pan = egui::vec2(0.0, 0.0);
        
        let rect = LayoutEngine::calculate_event_screen_rect(event, origin, zoom, pan, Language::English);
        
        assert_eq!(rect.left(), 100.0);
        assert_eq!(rect.top(), 200.0);
        assert_eq!(rect.height(), 29.0);
    }

    #[test]
    fn test_calculate_event_screen_rect_with_zoom() {
        let mut tree = FamilyTree::default();
        let event_id = tree.add_event(
            "Test".to_string(),
            None,
            "".to_string(),
            (100.0, 100.0),
            (255, 255, 200),
        );
        
        let event = tree.events.get(&event_id).unwrap();
        let origin = egui::pos2(0.0, 0.0);
        let zoom = 2.0;
        let pan = egui::vec2(0.0, 0.0);
        
        let rect = LayoutEngine::calculate_event_screen_rect(event, origin, zoom, pan, Language::English);
        
        // ズーム2.0の場合、位置とサイズが2倍になる
        assert_eq!(rect.left(), 200.0);
        assert_eq!(rect.top(), 200.0);
        assert_eq!(rect.height(), 58.0);
    }

    #[test]
    fn test_calculate_event_screen_rects() {
        let mut tree = FamilyTree::default();
        let event1_id = tree.add_event(
            "Event 1".to_string(),
            None,
            "".to_string(),
            (100.0, 100.0),
            (255, 255, 200),
        );
        let event2_id = tree.add_event(
            "Event 2".to_string(),
            None,
            "".to_string(),
            (200.0, 200.0),
            (200, 255, 255),
        );
        
        let origin = egui::pos2(0.0, 0.0);
        let zoom = 1.0;
        let pan = egui::vec2(0.0, 0.0);
        
        let rects = LayoutEngine::calculate_event_screen_rects(
            &tree.events,
            origin,
            zoom,
            pan,
            Language::Japanese,
        );
        
        assert_eq!(rects.len(), 2);
        assert!(rects.contains_key(&event1_id));
        assert!(rects.contains_key(&event2_id));
        
        let rect1 = rects.get(&event1_id).unwrap();
        let rect2 = rects.get(&event2_id).unwrap();
        
        assert_eq!(rect1.left(), 100.0);
        assert_eq!(rect1.top(), 100.0);
        assert_eq!(rect2.left(), 200.0);
        assert_eq!(rect2.top(), 200.0);
    }

    #[test]
    fn test_calculate_event_screen_rects_empty() {
        let tree = FamilyTree::default();
        let origin = egui::pos2(0.0, 0.0);
        let zoom = 1.0;
        let pan = egui::vec2(0.0, 0.0);
        
        let rects = LayoutEngine::calculate_event_screen_rects(
            &tree.events,
            origin,
            zoom,
            pan,
            Language::English,
        );
        
        assert_eq!(rects.len(), 0);
    }
}
