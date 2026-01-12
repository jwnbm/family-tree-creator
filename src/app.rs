use std::collections::{HashMap, VecDeque};
use std::fs;

use eframe::egui;
use crate::tree::{FamilyTree, Gender, PersonId};

/// 画面上のノード情報
#[derive(Debug, Clone)]
struct LayoutNode {
    id: PersonId,
    generation: usize, // 世代(0=ルート)
    pos: egui::Pos2,
    rect: egui::Rect,
}

pub struct App {
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
                let new_g = g + 1;
                let entry = gen_map.entry(ch).or_insert(new_g);
                if new_g < *entry {
                    *entry = new_g;
                }
                q.push_back(ch);
            }
        }

        if gen_map.is_empty() {
            for id in self.tree.persons.keys() {
                gen_map.insert(*id, 0);
            }
        } else {
            for id in self.tree.persons.keys() {
                gen_map.entry(*id).or_insert(0);
            }
        }

        let mut by_gen: HashMap<usize, Vec<PersonId>> = HashMap::new();
        for (id, g) in &gen_map {
            by_gen.entry(*g).or_default().push(*id);
        }

        for ids in by_gen.values_mut() {
            ids.sort_by_key(|id| self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or_default());
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
                    let (x, y) = if let Some(person) = self.tree.persons.get(id) {
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

    fn person_label(&self, id: PersonId) -> String {
        if let Some(p) = self.tree.persons.get(&id) {
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
    
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect, origin: egui::Pos2) {
        let grid_size = self.grid_size * self.zoom;
        let grid_origin = origin + self.pan;
        
        let start_x = ((rect.left() - grid_origin.x) / grid_size).floor() * grid_size + grid_origin.x;
        let start_y = ((rect.top() - grid_origin.y) / grid_size).floor() * grid_size + grid_origin.y;
        
        let mut x = start_x;
        while x <= rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(0.5, egui::Color32::from_gray(200)),
            );
            x += grid_size;
        }
        
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
        egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Family Tree (MVP)");
                ui.separator();

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
                    
                    ui.label("Relations:");
                    
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

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click());

            let pointer_pos = ui.input(|i| i.pointer.interact_pos());

            ctx.input(|i| {
                if i.modifiers.ctrl && i.raw_scroll_delta.y.abs() > 0.0 {
                    let factor = (i.raw_scroll_delta.y / 400.0).exp();
                    self.zoom = (self.zoom * factor).clamp(0.3, 3.0);
                }
            });

            let painter = ui.painter_at(rect);

            let to_screen = |p: egui::Pos2, zoom: f32, pan: egui::Vec2, origin: egui::Pos2| -> egui::Pos2 {
                let v = (p - origin) * zoom;
                origin + v + pan
            };

            let base_origin = rect.left_top() + egui::vec2(24.0, 24.0);
            let origin = if self.show_grid {
                egui::pos2(
                    (base_origin.x / self.grid_size).round() * self.grid_size,
                    (base_origin.y / self.grid_size).round() * self.grid_size,
                )
            } else {
                base_origin
            };
            
            if self.show_grid {
                self.draw_grid(&painter, rect, origin);
            }

            let nodes = self.compute_layout(origin);

            let mut screen_rects: HashMap<PersonId, egui::Rect> = HashMap::new();
            for n in &nodes {
                let min = to_screen(n.rect.min, self.zoom, self.pan, origin);
                let max = to_screen(n.rect.max, self.zoom, self.pan, origin);
                screen_rects.insert(n.id, egui::Rect::from_min_max(min, max));
            }

            let mut node_hovered = false;
            let mut any_node_dragged = false;
            
            for n in &nodes {
                if let Some(r) = screen_rects.get(&n.id) {
                    let node_id = ui.id().with(n.id);
                    let node_response = ui.interact(*r, node_id, egui::Sense::click_and_drag());
                    
                    if node_response.hovered() {
                        node_hovered = true;
                    }
                    
                    if node_response.drag_started() {
                        self.dragging_node = Some(n.id);
                        self.node_drag_start = pointer_pos;
                    }
                    
                    if node_response.dragged() && self.dragging_node == Some(n.id) {
                        any_node_dragged = true;
                        if let (Some(pos), Some(start)) = (pointer_pos, self.node_drag_start) {
                            let delta = (pos - start) / self.zoom;
                            
                            if let Some(person) = self.tree.persons.get_mut(&n.id) {
                                let current_pos = person.position.unwrap_or((n.rect.left(), n.rect.top()));
                                let new_x = current_pos.0 + delta.x;
                                let new_y = current_pos.1 + delta.y;
                                
                                person.position = Some((new_x, new_y));
                            }
                            self.node_drag_start = pointer_pos;
                        }
                    }
                    
                    if node_response.drag_stopped() && self.dragging_node == Some(n.id) {
                        if self.show_grid {
                            if let Some(person) = self.tree.persons.get_mut(&n.id) {
                                if let Some((x, y)) = person.position {
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
                    
                    if node_response.clicked() {
                        self.selected = Some(n.id);
                    }
                }
            }
            
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
                self.dragging_pan = false;
                self.last_pointer_pos = None;
            }

            for s in &self.tree.spouses {
                if let (Some(r1), Some(r2)) = (screen_rects.get(&s.person1), screen_rects.get(&s.person2)) {
                    let a = r1.center();
                    let b = r2.center();
                    
                    let dir = (b - a).normalized();
                    let perpendicular = egui::vec2(-dir.y, dir.x) * 2.0;
                    
                    painter.line_segment(
                        [a + perpendicular, b + perpendicular],
                        egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY),
                    );
                    painter.line_segment(
                        [a - perpendicular, b - perpendicular],
                        egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY),
                    );
                    
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
                                
                                painter.line_segment([mid, child_top], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
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
                                    egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY)
                                );
                                
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
                
                if let (Some(rp), Some(rc)) = (screen_rects.get(&e.parent), screen_rects.get(&e.child)) {
                    let a = rp.center_bottom();
                    let b = rc.center_top();
                    painter.line_segment([a, b], egui::Stroke::new(1.5, egui::Color32::LIGHT_GRAY));
                }
            }

            for n in &nodes {
                if let Some(r) = screen_rects.get(&n.id) {
                    let is_sel = self.selected == Some(n.id);
                    let is_dragging = self.dragging_node == Some(n.id);
                    
                    let gender = self.tree.persons.get(&n.id).map(|p| p.gender).unwrap_or(Gender::Unknown);
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

                    painter.rect_filled(*r, 6.0, fill);
                    painter.rect_stroke(*r, 6.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::epaint::StrokeKind::Outside);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Gender;

    #[test]
    fn test_person_label_basic() {
        let mut app = App::default();
        let id = app.tree.add_person(
            "Test Person".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
        );
        
        let label = app.person_label(id);
        assert_eq!(label, "Test Person");
    }

    #[test]
    fn test_person_label_with_birth() {
        let mut app = App::default();
        let id = app.tree.add_person(
            "John".to_string(),
            Gender::Male,
            Some("1990-05-15".to_string()),
            "".to_string(),
            false,
            None,
        );
        
        let label = app.person_label(id);
        assert!(label.contains("John"));
        assert!(label.contains("1990-05-15"));
        assert!(label.contains("(age 36)"));
    }

    #[test]
    fn test_person_label_deceased() {
        let mut app = App::default();
        let id = app.tree.add_person(
            "Jane".to_string(),
            Gender::Female,
            Some("1950-01-01".to_string()),
            "".to_string(),
            true,
            Some("2020-12-31".to_string()),
        );
        
        let label = app.person_label(id);
        assert!(label.contains("Jane"));
        assert!(label.contains("1950-01-01"));
        assert!(label.contains("(died at 70)"));
        assert!(label.contains("† 2020-12-31"));
    }

    #[test]
    fn test_person_label_deceased_without_death_date() {
        let mut app = App::default();
        let id = app.tree.add_person(
            "Bob".to_string(),
            Gender::Male,
            Some("1960-06-10".to_string()),
            "".to_string(),
            true,
            None,
        );
        
        let label = app.person_label(id);
        assert!(label.contains("Bob"));
        assert!(label.contains("1960-06-10"));
        assert!(label.contains("†"));
    }

    #[test]
    fn test_compute_layout_single_person() {
        let mut app = App::default();
        app.tree.add_person(
            "Solo".to_string(),
            Gender::Unknown,
            None,
            "".to_string(),
            false,
            None,
        );
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = app.compute_layout(origin);
        
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].generation, 0);
    }

    #[test]
    fn test_compute_layout_parent_child() {
        let mut app = App::default();
        let parent = app.tree.add_person(
            "Parent".to_string(),
            Gender::Female,
            None,
            "".to_string(),
            false,
            None,
        );
        let child = app.tree.add_person(
            "Child".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
        );
        
        app.tree.add_parent_child(parent, child, "biological".to_string());
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = app.compute_layout(origin);
        
        assert_eq!(nodes.len(), 2);
        
        let parent_node = nodes.iter().find(|n| n.id == parent).unwrap();
        let child_node = nodes.iter().find(|n| n.id == child).unwrap();
        
        assert_eq!(parent_node.generation, 0);
        assert_eq!(child_node.generation, 1);
    }

    #[test]
    fn test_compute_layout_with_manual_position() {
        let mut app = App::default();
        let id = app.tree.add_person(
            "Positioned".to_string(),
            Gender::Male,
            None,
            "".to_string(),
            false,
            None,
        );
        
        // 手動位置を設定
        if let Some(person) = app.tree.persons.get_mut(&id) {
            person.position = Some((100.0, 200.0));
        }
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = app.compute_layout(origin);
        
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].rect.left(), 100.0);
        assert_eq!(nodes[0].rect.top(), 200.0);
    }

    #[test]
    fn test_compute_layout_multiple_generations() {
        let mut app = App::default();
        let grandparent = app.tree.add_person("GP".to_string(), Gender::Male, None, "".to_string(), false, None);
        let parent = app.tree.add_person("P".to_string(), Gender::Female, None, "".to_string(), false, None);
        let child = app.tree.add_person("C".to_string(), Gender::Unknown, None, "".to_string(), false, None);
        
        app.tree.add_parent_child(grandparent, parent, "biological".to_string());
        app.tree.add_parent_child(parent, child, "biological".to_string());
        
        let origin = egui::pos2(0.0, 0.0);
        let nodes = app.compute_layout(origin);
        
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
        let app = App::default();
        let fake_id = uuid::Uuid::new_v4();
        
        let label = app.person_label(fake_id);
        assert_eq!(label, "Unknown");
    }

    #[test]
    fn test_grid_settings() {
        let mut app = App::default();
        
        assert_eq!(app.show_grid, true);
        assert_eq!(app.grid_size, 50.0);
        
        app.show_grid = false;
        app.grid_size = 100.0;
        
        assert_eq!(app.show_grid, false);
        assert_eq!(app.grid_size, 100.0);
    }
}
