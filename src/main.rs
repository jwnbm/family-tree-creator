mod tree;

use std::collections::{HashMap, VecDeque};
use std::fs;

use eframe::egui;
use tree::{FamilyTree, Gender, PersonId};

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
    new_deceased: bool,
    new_death: String,

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
    
    // グリッド
    show_grid: bool,
    grid_size: f32,
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
            new_deceased: false,
            new_death: String::new(),

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
            
            show_grid: true,
            grid_size: 50.0,
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
        let a = self.tree.add_person("Grandpa".into(), Gender::Male, Some("1940-01-01".into()), "".into(), true, Some("2020-01-01".into()));
        let b = self.tree.add_person("Grandma".into(), Gender::Female, Some("1942-02-02".into()), "".into(), true, Some("2022-05-15".into()));
        let c = self.tree.add_person("Father".into(), Gender::Male, Some("1968-03-03".into()), "".into(), false, None);
        let d = self.tree.add_person("Mother".into(), Gender::Female, Some("1970-04-04".into()), "".into(), false, None);
        let e = self.tree.add_person("Me".into(), Gender::Unknown, Some("1995-05-05".into()), "Hello".into(), false, None);

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
        let node_h = 50.0;  // グリッドに合わせて調整
        let x_gap = 50.0;   // グリッドサイズの倍数に
        let y_gap = 50.0;   // グリッドサイズの倍数に

        let mut nodes = Vec::new();
        let mut gens: Vec<usize> = by_gen.keys().copied().collect();
        gens.sort();

        for g in gens {
            if let Some(ids) = by_gen.get(&g) {
                for (i, id) in ids.iter().enumerate() {
                    // 手動配置があればそれを使用、なければ自動レイアウト
                    let (x, y) = if let Some(person) = self.tree.persons.get(id) {
                        if let Some((px, py)) = person.position {
                            (px, py)
                        } else {
                            // 自動レイアウト
                            let auto_x = origin.x + (i as f32) * (node_w + x_gap);
                            let auto_y = origin.y + (g as f32) * (node_h + y_gap);
                            (auto_x, auto_y)
                        }
                    } else {
                        // フォールバック
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

    fn person_label(&self, id: PersonId) -> String {
        if let Some(p) = self.tree.persons.get(&id) {
            let mut label = p.name.clone();
            
            // 年齢を計算するヘルパー関数
            let calculate_age = |birth: &str, end_date: Option<&str>| -> Option<i32> {
                let birth_year = birth.split('-').next()?.parse::<i32>().ok()?;
                let end_year = if let Some(ed) = end_date {
                    ed.split('-').next()?.parse::<i32>().ok()?
                } else {
                    2026 // 現在の年
                };
                Some(end_year - birth_year)
            };
            
            // 誕生日を追加
            if let Some(b) = &p.birth {
                if !b.is_empty() {
                    label.push_str(&format!("\n{}", b));
                    
                    // 年齢を計算
                    if p.deceased {
                        // 死亡している場合、享年を表示
                        if let Some(age) = calculate_age(b, p.death.as_deref()) {
                            label.push_str(&format!(" (died at {})", age));
                        }
                    } else {
                        // 生きている場合、現在の年齢を表示
                        if let Some(age) = calculate_age(b, None) {
                            label.push_str(&format!(" (age {})", age));
                        }
                    }
                }
            }
            
            // 死亡している場合は命日を追加
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
    
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect, origin: egui::Pos2) {
        let grid_size = self.grid_size * self.zoom;
        
        // グリッドの原点（origin + pan）
        let grid_origin = origin + self.pan;
        
        // グリッド線の描画範囲を計算
        let start_x = ((rect.left() - grid_origin.x) / grid_size).floor() * grid_size + grid_origin.x;
        let start_y = ((rect.top() - grid_origin.y) / grid_size).floor() * grid_size + grid_origin.y;
        
        // 縦線
        let mut x = start_x;
        while x <= rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(0.5, egui::Color32::from_gray(200)),
            );
            x += grid_size;
        }
        
        // 横線
        let mut y = start_y;
        while y <= rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(200)),
            );
            y += grid_size;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 左パネル：編集 UI
        egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
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
            ui.checkbox(&mut self.new_deceased, "Deceased");
            if self.new_deceased {
                ui.label("Death (YYYY-MM-DD, optional):");
                ui.text_edit_singleline(&mut self.new_death);
            }
            ui.label("Memo:");
            ui.text_edit_multiline(&mut self.new_memo);
            if ui.button("Add").clicked() {
                if !self.new_name.trim().is_empty() {
                    let birth = self.new_birth.trim();
                    let birth = (!birth.is_empty()).then(|| birth.to_string());
                    let death = if self.new_deceased {
                        let death_str = self.new_death.trim();
                        (!death_str.is_empty()).then(|| death_str.to_string())
                    } else {
                        None
                    };
                    let id = self.tree.add_person(
                        self.new_name.trim().to_string(),
                        self.new_gender,
                        birth,
                        self.new_memo.clone(),
                        self.new_deceased,
                        death,
                    );
                    self.selected = Some(id);
                    self.new_name.clear();
                    self.new_gender = Gender::Unknown;
                    self.new_birth.clear();
                    self.new_memo.clear();
                    self.new_deceased = false;
                    self.new_death.clear();
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
                    ui.checkbox(&mut p.deceased, "Deceased");
                    if p.deceased {
                        ui.horizontal(|ui| {
                            ui.label("Death:");
                            let mut d = p.death.clone().unwrap_or_default();
                            if ui.text_edit_singleline(&mut d).changed() {
                                let d = d.trim().to_string();
                                p.death = (!d.is_empty()).then_some(d);
                            }
                        });
                    }
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
            
            ui.separator();
            ui.label("Grid Settings:");
            ui.checkbox(&mut self.show_grid, "Show Grid");
            ui.horizontal(|ui| {
                ui.label("Grid Size:");
                ui.add(egui::DragValue::new(&mut self.grid_size)
                    .speed(1.0)
                    .range(10.0..=200.0));
            });
            
            if ui.button("Reset All Positions").clicked() {
                for person in self.tree.persons.values_mut() {
                    person.position = None;
                }
                self.status = "All positions reset".into();
            }
            });
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

            // originをグリッドに揃える
            let base_origin = rect.left_top() + egui::vec2(24.0, 24.0);
            let origin = if self.show_grid {
                egui::pos2(
                    (base_origin.x / self.grid_size).round() * self.grid_size,
                    (base_origin.y / self.grid_size).round() * self.grid_size,
                )
            } else {
                base_origin
            };
            
            // グリッド描画
            if self.show_grid {
                self.draw_grid(&painter, rect, origin);
            }

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
                                // 現在の位置を取得（ノードの左上座標）
                                let current_pos = person.position.unwrap_or((n.rect.left(), n.rect.top()));
                                let new_x = current_pos.0 + delta.x;
                                let new_y = current_pos.1 + delta.y;
                                
                                // ドラッグ中は自由に動かす（スナップなし）
                                person.position = Some((new_x, new_y));
                            }
                            self.node_drag_start = pointer_pos;
                        }
                    }
                    
                    // ドラッグ終了時にグリッドスナップ
                    if node_response.drag_stopped() && self.dragging_node == Some(n.id) {
                        if self.show_grid {
                            if let Some(person) = self.tree.persons.get_mut(&n.id) {
                                if let Some((x, y)) = person.position {
                                    // グリッドの原点（origin）を基準にスナップ
                                    let relative_x = x - origin.x;
                                    let relative_y = y - origin.y;
                                    
                                    let snapped_rel_x = (relative_x / self.grid_size).round() * self.grid_size;
                                    let snapped_rel_y = (relative_y / self.grid_size).round() * self.grid_size;
                                    
                                    let snapped_x = origin.x + snapped_rel_x;
                                    let snapped_y = origin.y + snapped_rel_y;
                                    
                                    person.position = Some((snapped_x, snapped_y));
                                }
                            }
                        }
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
                    
                    // 父母が両方いる場合
                    if let (Some(father), Some(mother)) = (father_id, mother_id) {
                        let are_spouses = self.tree.spouses.iter().any(|s| {
                            (s.person1 == father && s.person2 == mother) ||
                            (s.person1 == mother && s.person2 == father)
                        });
                        
                        if are_spouses {
                            // 配偶者関係がある場合：配偶者線の中点から子への線を引く
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
                        } else {
                            // 配偶者関係がない場合：父母を結ぶ線を引き、その中点から子への線を引く
                            if let (Some(rf), Some(rm), Some(rc)) = (
                                screen_rects.get(&father),
                                screen_rects.get(&mother),
                                screen_rects.get(&child_id)
                            ) {
                                let father_center = rf.center();
                                let mother_center = rm.center();
                                
                                // 父母を結ぶ線
                                painter.line_segment(
                                    [father_center, mother_center],
                                    egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY)
                                );
                                
                                // 中点から子への線
                                let mid = egui::pos2(
                                    (father_center.x + mother_center.x) / 2.0,
                                    (father_center.y + mother_center.y) / 2.0
                                );
                                let child_top = rc.center_top();
                                
                                painter.line_segment([mid, child_top], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
                            }
                        }
                        processed_children.insert(child_id);
                        continue;
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
                    
                    // 性別に応じた基本色を決定
                    let gender = self.tree.persons.get(&n.id).map(|p| p.gender).unwrap_or(Gender::Unknown);
                    let base_color = match gender {
                        Gender::Male => egui::Color32::from_rgb(173, 216, 230),   // 水色
                        Gender::Female => egui::Color32::from_rgb(255, 182, 193), // ピンク
                        Gender::Unknown => egui::Color32::from_rgb(245, 245, 245), // グレー
                    };
                    
                    let fill = if is_dragging {
                        egui::Color32::from_rgb(255, 220, 180) // ドラッグ中はオレンジ系
                    } else if is_sel {
                        // 選択時は基本色を明るくする
                        match gender {
                            Gender::Male => egui::Color32::from_rgb(200, 235, 255),
                            Gender::Female => egui::Color32::from_rgb(255, 220, 230),
                            Gender::Unknown => egui::Color32::from_rgb(200, 230, 255),
                        }
                    } else {
                        base_color
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
