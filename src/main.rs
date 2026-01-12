use std::collections::{HashMap, VecDeque};
use std::fs;

use eframe::egui;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type PersonId = Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum Gender {
    Male,
    Female,
    Unknown,
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    id: PersonId,
    name: String,
    #[serde(default)]
    gender: Gender,
    birth: Option<String>, // "YYYY-MM-DD" など
    memo: String,
    #[serde(default)]
    manual_offset: Option<(f32, f32)>, // 手動配置のオフセット
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParentChild {
    parent: PersonId,
    child: PersonId,
    kind: String, // "biological" / "adoptive" 等、今は自由文字列
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Spouse {
    person1: PersonId,
    person2: PersonId,
    memo: String, // 結婚年月日などのメモ
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FamilyTree {
    persons: HashMap<PersonId, Person>,
    edges: Vec<ParentChild>,
    #[serde(default)]
    spouses: Vec<Spouse>,
}

impl FamilyTree {
    fn add_person(&mut self, name: String, gender: Gender, birth: Option<String>, memo: String) -> PersonId {
        let id = Uuid::new_v4();
        self.persons.insert(
            id,
            Person {
                id,
                name,
                gender,
                birth,
                memo,
                manual_offset: None,
            },
        );
        id
    }

    fn remove_person(&mut self, id: PersonId) {
        self.persons.remove(&id);
        self.edges.retain(|e| e.parent != id && e.child != id);
        self.spouses.retain(|s| s.person1 != id && s.person2 != id);
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

    fn add_spouse(&mut self, person1: PersonId, person2: PersonId, memo: String) {
        // 重複防止（順序に関わらず同じペアなら追加しない）
        if self.spouses.iter().any(|s| {
            (s.person1 == person1 && s.person2 == person2)
                || (s.person1 == person2 && s.person2 == person1)
        }) {
            return;
        }
        self.spouses.push(Spouse {
            person1,
            person2,
            memo,
        });
    }

    fn remove_parent_child(&mut self, parent: PersonId, child: PersonId) {
        self.edges.retain(|e| !(e.parent == parent && e.child == child));
    }

    fn remove_spouse(&mut self, person1: PersonId, person2: PersonId) {
        self.spouses.retain(|s| {
            !((s.person1 == person1 && s.person2 == person2)
                || (s.person1 == person2 && s.person2 == person1))
        });
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

    fn spouses_of(&self, person: PersonId) -> Vec<PersonId> {
        self.spouses
            .iter()
            .filter_map(|s| {
                if s.person1 == person {
                    Some(s.person2)
                } else if s.person2 == person {
                    Some(s.person1)
                } else {
                    None
                }
            })
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
    new_gender: Gender,
    new_birth: String,
    new_memo: String,

    // 親子関係追加フォーム
    parent_pick: Option<PersonId>,
    child_pick: Option<PersonId>,
    relation_kind: String,

    // 配偶者関係追加フォーム
    spouse1_pick: Option<PersonId>,
    spouse_memo: String,

    // 保存/読込
    file_path: String,
    status: String,

    // 表示
    zoom: f32,
    pan: egui::Vec2,
    dragging_pan: bool,
    last_pointer_pos: Option<egui::Pos2>,
    
    // ノードドラッグ
    dragging_node: Option<PersonId>,
    node_drag_start: Option<egui::Pos2>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tree: FamilyTree::default(),
            selected: None,

            new_name: String::new(),
            new_gender: Gender::Unknown,
            new_birth: String::new(),
            new_memo: String::new(),

            parent_pick: None,
            child_pick: None,
            relation_kind: "biological".to_string(),

            spouse1_pick: None,
            spouse_memo: String::new(),

            file_path: "tree.json".to_string(),
            status: String::new(),

            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            dragging_pan: false,
            last_pointer_pos: None,
            
            dragging_node: None,
            node_drag_start: None,
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
        let a = self.tree.add_person("Grandpa".into(), Gender::Male, Some("1940-01-01".into()), "".into());
        let b = self.tree.add_person("Grandma".into(), Gender::Female, Some("1942-02-02".into()), "".into());
        let c = self.tree.add_person("Father".into(), Gender::Male, Some("1968-03-03".into()), "".into());
        let d = self.tree.add_person("Mother".into(), Gender::Female, Some("1970-04-04".into()), "".into());
        let e = self.tree.add_person("Me".into(), Gender::Unknown, Some("1995-05-05".into()), "Hello".into());

        // 親子関係
        self.tree.add_parent_child(a, c, "biological".into());
        self.tree.add_parent_child(b, c, "biological".into());
        self.tree.add_parent_child(c, e, "biological".into());
        self.tree.add_parent_child(d, e, "biological".into());

        // 配偶者関係
        self.tree.add_spouse(a, b, "1965".into());
        self.tree.add_spouse(c, d, "1994".into());

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
                    
                    // 手動オフセットを適用
                    let (offset_x, offset_y) = self.tree.persons.get(id)
                        .and_then(|p| p.manual_offset)
                        .unwrap_or((0.0, 0.0));
                    
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(x + offset_x, y + offset_y),
                        egui::vec2(node_w, node_h),
                    );
                    nodes.push(LayoutNode {
                        id: *id,
                        generation: g,
                        pos: egui::pos2(x + offset_x, y + offset_y),
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
            ui.horizontal(|ui| {
                ui.label("Gender:");
                ui.radio_value(&mut self.new_gender, Gender::Male, "Male");
                ui.radio_value(&mut self.new_gender, Gender::Female, "Female");
                ui.radio_value(&mut self.new_gender, Gender::Unknown, "Unknown");
            });
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
                        self.new_gender,
                        birth,
                        self.new_memo.clone(),
                    );
                    self.selected = Some(id);
                    self.new_name.clear();
                    self.new_gender = Gender::Unknown;
                    self.new_birth.clear();
                    self.new_memo.clear();
                } else {
                    self.status = "Name is required".into();
                }
            }

            ui.separator();
            ui.label("Persons");
            // 人物一覧
            let mut all_ids: Vec<PersonId> = self.tree.persons.keys().copied().collect();
            all_ids.sort_by_key(|id| self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or_default());

            egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                for id in &all_ids {
                    let name = self.tree.persons.get(&id).map(|p| p.name.as_str()).unwrap_or("?");
                    let selected = self.selected == Some(*id);
                    if ui.selectable_label(selected, name).clicked() {
                        self.selected = Some(*id);
                    }
                }
            });

            // 選択人物の編集
            ui.separator();
            ui.label("Selected Person");
            if let Some(sel) = self.selected {
                if let Some(p) = self.tree.persons.get_mut(&sel) {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut p.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Gender:");
                        ui.radio_value(&mut p.gender, Gender::Male, "Male");
                        ui.radio_value(&mut p.gender, Gender::Female, "Female");
                        ui.radio_value(&mut p.gender, Gender::Unknown, "Unknown");
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

                    ui.separator();
                    
                    // 既存の関係を表示
                    ui.label("Relations:");
                    
                    // 父母を性別で区別して表示
                    let parents = self.tree.parents_of(sel);
                    let mut fathers = Vec::new();
                    let mut mothers = Vec::new();
                    let mut other_parents = Vec::new();
                    
                    for parent_id in &parents {
                        if let Some(parent) = self.tree.persons.get(parent_id) {
                            match parent.gender {
                                Gender::Male => fathers.push((*parent_id, parent.name.clone())),
                                Gender::Female => mothers.push((*parent_id, parent.name.clone())),
                                Gender::Unknown => other_parents.push((*parent_id, parent.name.clone())),
                            }
                        }
                    }
                    
                    if !fathers.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Father:");
                            for (id, name) in &fathers {
                                if ui.small_button(name).clicked() {
                                    self.selected = Some(*id);
                                }
                                if ui.small_button("❌").on_hover_text("Remove parent relation").clicked() {
                                    self.tree.remove_parent_child(*id, sel);
                                    self.status = "Parent relation removed".into();
                                }
                            }
                        });
                    }
                    
                    if !mothers.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Mother:");
                            for (id, name) in &mothers {
                                if ui.small_button(name).clicked() {
                                    self.selected = Some(*id);
                                }
                                if ui.small_button("❌").on_hover_text("Remove parent relation").clicked() {
                                    self.tree.remove_parent_child(*id, sel);
                                    self.status = "Parent relation removed".into();
                                }
                            }
                        });
                    }
                    
                    if !other_parents.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Parent:");
                            for (id, name) in &other_parents {
                                if ui.small_button(name).clicked() {
                                    self.selected = Some(*id);
                                }
                                if ui.small_button("❌").on_hover_text("Remove parent relation").clicked() {
                                    self.tree.remove_parent_child(*id, sel);
                                    self.status = "Parent relation removed".into();
                                }
                            }
                        });
                    }
                    
                    // 配偶者を表示
                    let spouses = self.tree.spouses_of(sel);
                    if !spouses.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Spouses:");
                            for spouse_id in &spouses {
                                if let Some(spouse) = self.tree.persons.get(spouse_id) {
                                    if ui.small_button(&spouse.name).clicked() {
                                        self.selected = Some(*spouse_id);
                                    }
                                    if ui.small_button("❌").on_hover_text("Remove spouse relation").clicked() {
                                        self.tree.remove_spouse(sel, *spouse_id);
                                        self.status = "Spouse relation removed".into();
                                    }
                                }
                            }
                        });
                    }

                    ui.separator();
                    ui.label("Add Relations:");
                    
                    // 親を追加
                    ui.horizontal(|ui| {
                        ui.label("Add Parent:");
                        egui::ComboBox::from_id_salt("add_parent")
                            .selected_text(
                                self.parent_pick
                                    .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                                    .unwrap_or_else(|| "(select)".into()),
                            )
                            .show_ui(ui, |ui| {
                                for id in &all_ids {
                                    if *id != sel {
                                        let name = self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or("?".into());
                                        ui.selectable_value(&mut self.parent_pick, Some(*id), name);
                                    }
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Kind:");
                        ui.text_edit_singleline(&mut self.relation_kind);
                        if ui.button("Add").clicked() {
                            if let Some(parent) = self.parent_pick {
                                let k = self.relation_kind.trim();
                                let k = if k.is_empty() { "biological" } else { k };
                                self.tree.add_parent_child(parent, sel, k.to_string());
                                self.parent_pick = None;
                                self.status = "Parent added".into();
                            }
                        }
                    });

                    ui.add_space(4.0);
                    
                    // 子を追加
                    ui.horizontal(|ui| {
                        ui.label("Add Child:");
                        egui::ComboBox::from_id_salt("add_child")
                            .selected_text(
                                self.child_pick
                                    .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                                    .unwrap_or_else(|| "(select)".into()),
                            )
                            .show_ui(ui, |ui| {
                                for id in &all_ids {
                                    if *id != sel {
                                        let name = self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or("?".into());
                                        ui.selectable_value(&mut self.child_pick, Some(*id), name);
                                    }
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Kind:");
                        ui.text_edit_singleline(&mut self.relation_kind);
                        if ui.button("Add").clicked() {
                            if let Some(child) = self.child_pick {
                                let k = self.relation_kind.trim();
                                let k = if k.is_empty() { "biological" } else { k };
                                self.tree.add_parent_child(sel, child, k.to_string());
                                self.child_pick = None;
                                self.status = "Child added".into();
                            }
                        }
                    });

                    ui.add_space(4.0);
                    
                    // 配偶者を追加
                    ui.horizontal(|ui| {
                        ui.label("Add Spouse:");
                        egui::ComboBox::from_id_salt("add_spouse")
                            .selected_text(
                                self.spouse1_pick
                                    .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                                    .unwrap_or_else(|| "(select)".into()),
                            )
                            .show_ui(ui, |ui| {
                                for id in &all_ids {
                                    if *id != sel {
                                        let name = self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or("?".into());
                                        ui.selectable_value(&mut self.spouse1_pick, Some(*id), name);
                                    }
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Memo:");
                        ui.text_edit_singleline(&mut self.spouse_memo);
                        if ui.button("Add").clicked() {
                            if let Some(spouse) = self.spouse1_pick {
                                self.tree.add_spouse(sel, spouse, self.spouse_memo.clone());
                                self.spouse1_pick = None;
                                self.spouse_memo.clear();
                                self.status = "Spouse added".into();
                            }
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Delete Person").clicked() {
                            self.tree.remove_person(sel);
                            self.selected = None;
                        }
                    });
                }
            } else {
                ui.label("(none)");
            }

            ui.separator();
            ui.label("View controls: Drag on canvas to pan, Ctrl+Wheel to zoom");
            ui.label("Drag nodes to manually adjust positions");
            if ui.button("Reset All Positions").clicked() {
                for person in self.tree.persons.values_mut() {
                    person.manual_offset = None;
                }
                self.status = "All positions reset".into();
            }
        });

        // 中央：キャンバス描画
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click());

            let pointer_pos = ui.input(|i| i.pointer.interact_pos());

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

            // ノードのインタラクション判定（ドラッグ優先）
            let mut node_hovered = false;
            let mut any_node_dragged = false;
            
            for n in &nodes {
                if let Some(r) = screen_rects.get(&n.id) {
                    let node_id = ui.id().with(n.id);
                    let node_response = ui.interact(*r, node_id, egui::Sense::click_and_drag());
                    
                    if node_response.hovered() {
                        node_hovered = true;
                    }
                    
                    // ドラッグ開始
                    if node_response.drag_started() {
                        self.dragging_node = Some(n.id);
                        self.node_drag_start = pointer_pos;
                    }
                    
                    // ドラッグ中
                    if node_response.dragged() && self.dragging_node == Some(n.id) {
                        any_node_dragged = true;
                        if let (Some(pos), Some(start)) = (pointer_pos, self.node_drag_start) {
                            let delta = (pos - start) / self.zoom;
                            
                            if let Some(person) = self.tree.persons.get_mut(&n.id) {
                                let current_offset = person.manual_offset.unwrap_or((0.0, 0.0));
                                person.manual_offset = Some((
                                    current_offset.0 + delta.x,
                                    current_offset.1 + delta.y,
                                ));
                            }
                            self.node_drag_start = pointer_pos;
                        }
                    }
                    
                    // ドラッグ終了
                    if node_response.drag_stopped() && self.dragging_node == Some(n.id) {
                        self.dragging_node = None;
                        self.node_drag_start = None;
                    }
                    
                    // クリック
                    if node_response.clicked() {
                        self.selected = Some(n.id);
                    }
                }
            }
            
            // キャンバスのパン操作（ノードがホバー/ドラッグされていない場合のみ）
            if !node_hovered && !any_node_dragged && self.dragging_node.is_none() {
                if let Some(pos) = pointer_pos {
                    let primary_down = ui.input(|i| i.pointer.primary_down());
                    let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
                    
                    if primary_pressed && rect.contains(pos) {
                        self.dragging_pan = true;
                        self.last_pointer_pos = Some(pos);
                    }
                    
                    if self.dragging_pan && primary_down {
                        if let Some(prev) = self.last_pointer_pos {
                            self.pan += pos - prev;
                            self.last_pointer_pos = Some(pos);
                        }
                    }
                    
                    if !primary_down && self.dragging_pan {
                        self.dragging_pan = false;
                        self.last_pointer_pos = None;
                    }
                }
            } else if !any_node_dragged {
                // ノードがホバー中はキャンバスパンをキャンセル
                self.dragging_pan = false;
                self.last_pointer_pos = None;
            }

            // 配偶者関係の線描画（二重線）
            for s in &self.tree.spouses {
                if let (Some(r1), Some(r2)) = (screen_rects.get(&s.person1), screen_rects.get(&s.person2)) {
                    let a = r1.center();
                    let b = r2.center();
                    
                    // 二重線を描画（平行な2本の線）
                    let dir = (b - a).normalized();
                    let perpendicular = egui::vec2(-dir.y, dir.x) * 2.0; // 垂直方向のオフセット
                    
                    painter.line_segment(
                        [a + perpendicular, b + perpendicular],
                        egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY),
                    );
                    painter.line_segment(
                        [a - perpendicular, b - perpendicular],
                        egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY),
                    );
                    
                    // メモがあれば中点に表示
                    if !s.memo.is_empty() {
                        let mid = egui::pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
                        painter.text(
                            mid,
                            egui::Align2::CENTER_CENTER,
                            &s.memo,
                            egui::FontId::proportional(10.0 * self.zoom.clamp(0.7, 1.2)),
                            egui::Color32::DARK_GRAY,
                        );
                    }
                }
            }

            // 線描画（親→子）
            // まず、各子について父母のペアを特定
            let mut child_to_parents: HashMap<PersonId, Vec<PersonId>> = HashMap::new();
            for e in &self.tree.edges {
                child_to_parents.entry(e.child).or_default().push(e.parent);
            }

            let mut processed_children = std::collections::HashSet::new();

            for e in &self.tree.edges {
                let child_id = e.child;
                
                // すでに処理済みの子はスキップ
                if processed_children.contains(&child_id) {
                    continue;
                }
                
                if let Some(parents) = child_to_parents.get(&child_id) {
                    // 父母を性別で分類
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
                    
                    // 父母が両方いて、かつ配偶者関係にある場合
                    if let (Some(father), Some(mother)) = (father_id, mother_id) {
                        let are_spouses = self.tree.spouses.iter().any(|s| {
                            (s.person1 == father && s.person2 == mother) ||
                            (s.person1 == mother && s.person2 == father)
                        });
                        
                        if are_spouses {
                            // 配偶者線の中点から子への線を引く
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
                                
                                painter.line_segment([mid, child_top], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
                            }
                            processed_children.insert(child_id);
                            continue;
                        }
                    }
                }
                
                // 配偶者関係にない場合や、その他のケースは従来通り個別に描画
                if let (Some(rp), Some(rc)) = (screen_rects.get(&e.parent), screen_rects.get(&e.child)) {
                    let a = rp.center_bottom();
                    let b = rc.center_top();
                    painter.line_segment([a, b], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
                }
            }

            // ノード描画
            for n in &nodes {
                if let Some(r) = screen_rects.get(&n.id) {
                    let is_sel = self.selected == Some(n.id);
                    let is_dragging = self.dragging_node == Some(n.id);
                    let fill = if is_dragging {
                        egui::Color32::from_rgb(255, 220, 180)
                    } else if is_sel {
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
