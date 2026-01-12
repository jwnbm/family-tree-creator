use std::collections::{HashMap, VecDeque};
use std::fs;

use eframe::egui;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type PersonId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    id: PersonId,
    name: String,
    birth: Option<String>, // "YYYY-MM-DD" など
    memo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParentChild {
    parent: PersonId,
    child: PersonId,
    kind: String, // "biological" / "adoptive" 等、今は自由文字列
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FamilyTree {
    persons: HashMap<PersonId, Person>,
    edges: Vec<ParentChild>,
}

impl FamilyTree {
    fn add_person(&mut self, name: String, birth: Option<String>, memo: String) -> PersonId {
        let id = Uuid::new_v4();
        self.persons.insert(
            id,
            Person {
                id,
                name,
                birth,
                memo,
            },
        );
        id
    }

    fn remove_person(&mut self, id: PersonId) {
        self.persons.remove(&id);
        self.edges.retain(|e| e.parent != id && e.child != id);
    }

    fn add_parent_child(&mut self, parent: PersonId, child: PersonId, kind: String) {
        // 重複エッジ防止（同じ親子・同じkindなら追加しない）
        if self
            .edges
            .iter()
            .any(|e| e.parent == parent && e.child == child && e.kind == kind)
        {
            return;
        }
        self.edges.push(ParentChild { parent, child, kind });
    }

    fn parents_of(&self, child: PersonId) -> Vec<PersonId> {
        self.edges
            .iter()
            .filter(|e| e.child == child)
            .map(|e| e.parent)
            .collect()
    }

    fn children_of(&self, parent: PersonId) -> Vec<PersonId> {
        self.edges
            .iter()
            .filter(|e| e.parent == parent)
            .map(|e| e.child)
            .collect()
    }

    /// ルート（親がいない人物）を返す
    fn roots(&self) -> Vec<PersonId> {
        let mut has_parent = HashMap::<PersonId, bool>::new();
        for id in self.persons.keys() {
            has_parent.insert(*id, false);
        }
        for e in &self.edges {
            has_parent.insert(e.child, true);
        }
        has_parent
            .into_iter()
            .filter_map(|(id, hp)| (!hp).then_some(id))
            .collect()
    }
}

/// 画面上のノード情報
#[derive(Debug, Clone)]
struct LayoutNode {
    id: PersonId,
    generation: usize, // 世代(0=ルート)
    pos: egui::Pos2,
    rect: egui::Rect,
}

struct App {
    tree: FamilyTree,
    selected: Option<PersonId>,

    // 入力フォーム
    new_name: String,
    new_birth: String,
    new_memo: String,

    // 親子関係追加フォーム
    parent_pick: Option<PersonId>,
    child_pick: Option<PersonId>,
    relation_kind: String,

    // 保存/読込
    file_path: String,
    status: String,

    // 表示
    zoom: f32,
    pan: egui::Vec2,
    dragging_pan: bool,
    last_pointer_pos: Option<egui::Pos2>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tree: FamilyTree::default(),
            selected: None,

            new_name: String::new(),
            new_birth: String::new(),
            new_memo: String::new(),

            parent_pick: None,
            child_pick: None,
            relation_kind: "biological".to_string(),

            file_path: "tree.json".to_string(),
            status: String::new(),

            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            dragging_pan: false,
            last_pointer_pos: None,
        }
    }
}

impl App {
    fn save(&mut self) {
        match serde_json::to_string_pretty(&self.tree) {
            Ok(s) => match fs::write(&self.file_path, s) {
                Ok(_) => self.status = format!("Saved: {}", self.file_path),
                Err(e) => self.status = format!("Save error: {e}"),
            },
            Err(e) => self.status = format!("Serialize error: {e}"),
        }
    }

    fn load(&mut self) {
        match fs::read_to_string(&self.file_path) {
            Ok(s) => match serde_json::from_str::<FamilyTree>(&s) {
                Ok(t) => {
                    self.tree = t;
                    self.selected = None;
                    self.status = format!("Loaded: {}", self.file_path);
                }
                Err(e) => self.status = format!("Parse error: {e}"),
            },
            Err(e) => self.status = format!("Read error: {e}"),
        }
    }

    fn add_sample(&mut self) {
        let a = self.tree.add_person("Grandpa".into(), Some("1940-01-01".into()), "".into());
        let b = self.tree.add_person("Grandma".into(), Some("1942-02-02".into()), "".into());
        let c = self.tree.add_person("Father".into(), Some("1968-03-03".into()), "".into());
        let d = self.tree.add_person("Mother".into(), Some("1970-04-04".into()), "".into());
        let e = self.tree.add_person("Me".into(), Some("1995-05-05".into()), "Hello".into());

        self.tree.add_parent_child(a, c, "biological".into());
        self.tree.add_parent_child(b, c, "biological".into());
        self.tree.add_parent_child(c, e, "biological".into());
        self.tree.add_parent_child(d, e, "biological".into());

        self.status = "Added sample data".into();
    }

    /// BFSで世代(gen)を決めて、簡易レイアウト（行=世代）を作る
    fn compute_layout(&self, origin: egui::Pos2) -> Vec<LayoutNode> {
        // 世代計算：ルートを0として子へ+1
        let roots = self.tree.roots();
        let mut gen_map: HashMap<PersonId, usize> = HashMap::new();
        let mut q = VecDeque::new();

        for r in &roots {
            gen_map.insert(*r, 0);
            q.push_back(*r);
        }

        while let Some(pid) = q.pop_front() {
            let g = gen_map[&pid];
            for ch in self.tree.children_of(pid) {
                // 既に世代がある場合は、より小さい方を優先（雑だがMVPとしてOK）
                let new_g = g + 1;
                let entry = gen_map.entry(ch).or_insert(new_g);
                if new_g < *entry {
                    *entry = new_g;
                }
                q.push_back(ch);
            }
        }

        // ルートが無い（全員に親がいる等）ケース：とりあえず全員0に
        if gen_map.is_empty() {
            for id in self.tree.persons.keys() {
                gen_map.insert(*id, 0);
            }
        } else {
            // BFSで到達しない人物も0にしておく
            for id in self.tree.persons.keys() {
                gen_map.entry(*id).or_insert(0);
            }
        }

        // 世代ごとに並べる
        let mut by_gen: HashMap<usize, Vec<PersonId>> = HashMap::new();
        for (id, g) in &gen_map {
            by_gen.entry(*g).or_default().push(*id);
        }

        // 見た目の安定のため、名前でソート
        for ids in by_gen.values_mut() {
            ids.sort_by_key(|id| self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or_default());
        }

        let node_w = 140.0;
        let node_h = 48.0;
        let x_gap = 28.0;
        let y_gap = 64.0;

        let mut nodes = Vec::new();
        let mut gens: Vec<usize> = by_gen.keys().copied().collect();
        gens.sort();

        for g in gens {
            if let Some(ids) = by_gen.get(&g) {
                for (i, id) in ids.iter().enumerate() {
                    let x = origin.x + (i as f32) * (node_w + x_gap);
                    let y = origin.y + (g as f32) * (node_h + y_gap);
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

    fn person_label(&self, id: PersonId) -> String {
        if let Some(p) = self.tree.persons.get(&id) {
            match &p.birth {
                Some(b) if !b.is_empty() => format!("{}\n{}", p.name, b),
                _ => p.name.clone(),
            }
        } else {
            "Unknown".into()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 左パネル：編集 UI
        egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
            ui.heading("Family Tree (MVP)");
            ui.separator();

            // 保存/読込
            ui.horizontal(|ui| {
                ui.label("File:");
                ui.text_edit_singleline(&mut self.file_path);
            });
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.save();
                }
                if ui.button("Load").clicked() {
                    self.load();
                }
                if ui.button("Sample").clicked() {
                    self.add_sample();
                }
            });
            if !self.status.is_empty() {
                ui.label(&self.status);
            }

            ui.separator();
            ui.label("Add Person");
            ui.text_edit_singleline(&mut self.new_name);
            ui.label("Birth (YYYY-MM-DD, optional):");
            ui.text_edit_singleline(&mut self.new_birth);
            ui.label("Memo:");
            ui.text_edit_multiline(&mut self.new_memo);
            if ui.button("Add").clicked() {
                if !self.new_name.trim().is_empty() {
                    let birth = self.new_birth.trim();
                    let birth = (!birth.is_empty()).then(|| birth.to_string());
                    let id = self.tree.add_person(
                        self.new_name.trim().to_string(),
                        birth,
                        self.new_memo.clone(),
                    );
                    self.selected = Some(id);
                    self.new_name.clear();
                    self.new_birth.clear();
                    self.new_memo.clear();
                } else {
                    self.status = "Name is required".into();
                }
            }

            ui.separator();
            ui.label("Persons");
            // 人物一覧
            let mut ids: Vec<PersonId> = self.tree.persons.keys().copied().collect();
            ids.sort_by_key(|id| self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or_default());

            egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                for id in ids {
                    let name = self.tree.persons.get(&id).map(|p| p.name.as_str()).unwrap_or("?");
                    let selected = self.selected == Some(id);
                    if ui.selectable_label(selected, name).clicked() {
                        self.selected = Some(id);
                    }
                }
            });

            // 選択人物の編集
            ui.separator();
            ui.label("Selected");
            if let Some(sel) = self.selected {
                if let Some(p) = self.tree.persons.get_mut(&sel) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut p.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Birth:");
                        let mut b = p.birth.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut b).changed() {
                            let b = b.trim().to_string();
                            p.birth = (!b.is_empty()).then_some(b);
                        }
                    });
                    ui.label("Memo:");
                    ui.text_edit_multiline(&mut p.memo);

                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            self.tree.remove_person(sel);
                            self.selected = None;
                        }
                    });
                }
            } else {
                ui.label("(none)");
            }

            ui.separator();
            ui.label("Add Parent-Child");
            // 親子関係追加
            let mut all_ids: Vec<PersonId> = self.tree.persons.keys().copied().collect();
            all_ids.sort_by_key(|id| self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or_default());

            egui::ComboBox::from_label("Parent")
                .selected_text(
                    self.parent_pick
                        .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                        .unwrap_or_else(|| "(select)".into()),
                )
                .show_ui(ui, |ui| {
                    for id in &all_ids {
                        let name = self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or("?".into());
                        ui.selectable_value(&mut self.parent_pick, Some(*id), name);
                    }
                });

            egui::ComboBox::from_label("Child")
                .selected_text(
                    self.child_pick
                        .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                        .unwrap_or_else(|| "(select)".into()),
                )
                .show_ui(ui, |ui| {
                    for id in &all_ids {
                        let name = self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or("?".into());
                        ui.selectable_value(&mut self.child_pick, Some(*id), name);
                    }
                });

            ui.label("Relation kind (biological...):");
            ui.text_edit_singleline(&mut self.relation_kind);

            if ui.button("Add Relation").clicked() {
                match (self.parent_pick, self.child_pick) {
                    (Some(pa), Some(ch)) if pa != ch => {
                        let k = self.relation_kind.trim();
                        let k = if k.is_empty() { "biological" } else { k };
                        self.tree.add_parent_child(pa, ch, k.to_string());
                    }
                    _ => self.status = "Pick parent and child (different persons)".into(),
                }
            }

            ui.separator();
            ui.label("View controls: Drag on canvas to pan, Ctrl+Wheel to zoom");
        });

        // 中央：キャンバス描画
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

            // パン操作（ドラッグ）
            if response.drag_started() {
                self.dragging_pan = true;
                self.last_pointer_pos = response.interact_pointer_pos();
            }
            if self.dragging_pan && response.dragged() {
                if let (Some(prev), Some(now)) = (self.last_pointer_pos, response.interact_pointer_pos()) {
                    self.pan += now - prev;
                    self.last_pointer_pos = Some(now);
                }
            }
            if !response.dragged() && self.dragging_pan {
                self.dragging_pan = false;
                self.last_pointer_pos = None;
            }

            // ズーム（Ctrl + Wheel）
            ctx.input(|i| {
                if i.modifiers.ctrl && i.raw_scroll_delta.y.abs() > 0.0 {
                    let factor = (i.raw_scroll_delta.y / 400.0).exp();
                    self.zoom = (self.zoom * factor).clamp(0.3, 3.0);
                }
            });

            let painter = ui.painter_at(rect);

            // 変換（ワールド→スクリーン）
            let to_screen = |p: egui::Pos2, zoom: f32, pan: egui::Vec2, origin: egui::Pos2| -> egui::Pos2 {
                // originを基準にズーム
                let v = (p - origin) * zoom;
                origin + v + pan
            };

            let origin = rect.left_top() + egui::vec2(24.0, 24.0);

            // レイアウト（ワールド座標）
            let nodes = self.compute_layout(origin);

            // id -> rect (screen)
            let mut screen_rects: HashMap<PersonId, egui::Rect> = HashMap::new();
            for n in &nodes {
                let min = to_screen(n.rect.min, self.zoom, self.pan, origin);
                let max = to_screen(n.rect.max, self.zoom, self.pan, origin);
                screen_rects.insert(n.id, egui::Rect::from_min_max(min, max));
            }

            // 線描画（親→子）
            for e in &self.tree.edges {
                if let (Some(rp), Some(rc)) = (screen_rects.get(&e.parent), screen_rects.get(&e.child)) {
                    let a = rp.center_bottom();
                    let b = rc.center_top();
                    painter.line_segment([a, b], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
                }
            }

            // ノード描画 + クリックで選択
            for n in &nodes {
                if let Some(r) = screen_rects.get(&n.id) {
                    let is_sel = self.selected == Some(n.id);
                    let fill = if is_sel {
                        egui::Color32::from_rgb(200, 230, 255)
                    } else {
                        egui::Color32::from_rgb(245, 245, 245)
                    };

                    painter.rect_filled(*r, 6.0, fill);
                    painter.rect_stroke(*r, 6.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::epaint::StrokeKind::Outside);

                    // テキスト
                    let text = self.person_label(n.id);
                    painter.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::proportional(14.0 * self.zoom.clamp(0.7, 1.2)),
                        egui::Color32::BLACK,
                    );

                    // クリック判定
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        if r.contains(pos) && ctx.input(|i| i.pointer.any_click()) {
                            self.selected = Some(n.id);
                        }
                    }
                }
            }

            // 情報表示
            painter.text(
                rect.right_top() + egui::vec2(-10.0, 10.0),
                egui::Align2::RIGHT_TOP,
                format!("zoom: {:.2}", self.zoom),
                egui::FontId::proportional(12.0),
                egui::Color32::DARK_GRAY,
            );
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Family Tree (egui MVP)")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Family Tree (egui MVP)",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
