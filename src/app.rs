use std::collections::HashMap;
use std::fs;

use eframe::egui;
use crate::tree::{FamilyTree, Gender, PersonId};
use crate::layout::LayoutEngine;

// 定数
const DEFAULT_RELATION_KIND: &str = "biological";
const NODE_CORNER_RADIUS: f32 = 6.0;
const EDGE_STROKE_WIDTH: f32 = 1.5;
const SPOUSE_LINE_OFFSET: f32 = 2.0;

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
    
    // 設定ウィンドウ
    show_settings: bool,
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
            relation_kind: DEFAULT_RELATION_KIND.to_string(),

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
            
            show_settings: false,
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

        self.tree.add_parent_child(a, c, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(b, c, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(c, e, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(d, e, DEFAULT_RELATION_KIND.into());
        self.tree.add_spouse(a, b, "1965".into());
        self.tree.add_spouse(c, d, "1994".into());

        self.status = "Added sample data".into();
    }

    fn clear_person_form(&mut self) {
        self.new_name.clear();
        self.new_gender = Gender::Unknown;
        self.new_birth.clear();
        self.new_memo.clear();
        self.new_deceased = false;
        self.new_death.clear();
    }

    fn parse_optional_field(s: &str) -> Option<String> {
        let trimmed = s.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    fn get_person_name(&self, id: &PersonId) -> String {
        self.tree.persons.get(id).map(|p| p.name.clone()).unwrap_or_else(|| "?".into())
    }

    fn show_relation_buttons(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        relations: &[(PersonId, String)],
        current_id: PersonId,
        is_parent: bool,
    ) {
        if !relations.is_empty() {
            ui.horizontal(|ui| {
                ui.label(label);
                for (id, name) in relations {
                    if ui.small_button(name).clicked() {
                        self.selected = Some(*id);
                    }
                    if ui.small_button("❌").on_hover_text("Remove relation").clicked() {
                        if is_parent {
                            self.tree.remove_parent_child(*id, current_id);
                        } else {
                            self.tree.remove_spouse(current_id, *id);
                        }
                        self.status = "Relation removed".into();
                    }
                }
            });
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
                    let birth = Self::parse_optional_field(&self.new_birth);
                    let death = self.new_deceased.then(|| Self::parse_optional_field(&self.new_death)).flatten();
                    let id = self.tree.add_person(
                        self.new_name.trim().to_string(),
                        self.new_gender,
                        birth,
                        self.new_memo.clone(),
                        self.new_deceased,
                        death,
                    );
                    self.selected = Some(id);
                    self.clear_person_form();
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
                    
                    self.show_relation_buttons(ui, "Father:", &fathers, sel, true);
                    self.show_relation_buttons(ui, "Mother:", &mothers, sel, true);
                    self.show_relation_buttons(ui, "Parent:", &other_parents, sel, true);
                    
                    let spouses: Vec<_> = self.tree.spouses_of(sel)
                        .into_iter()
                        .filter_map(|id| self.tree.persons.get(&id).map(|p| (id, p.name.clone())))
                        .collect();
                    self.show_relation_buttons(ui, "Spouses:", &spouses, sel, false);

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
                                        let name = self.get_person_name(id);
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
                                let kind = if self.relation_kind.trim().is_empty() {
                                    DEFAULT_RELATION_KIND
                                } else {
                                    self.relation_kind.trim()
                                };
                                self.tree.add_parent_child(parent, sel, kind.to_string());
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
                                        let name = self.get_person_name(id);
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
                                let kind = if self.relation_kind.trim().is_empty() {
                                    DEFAULT_RELATION_KIND
                                } else {
                                    self.relation_kind.trim()
                                };
                                self.tree.add_parent_child(sel, child, kind.to_string());
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
                                        let name = self.get_person_name(id);
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
            if ui.button("⚙ Settings").clicked() {
                self.show_settings = !self.show_settings;
            }
            });
        });

        // 設定ウィンドウ
        egui::Window::new("⚙ Settings")
            .open(&mut self.show_settings)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("General Settings");
                ui.separator();
                
                ui.label("Grid:");
                ui.checkbox(&mut self.show_grid, "Show Grid");
                ui.horizontal(|ui| {
                    ui.label("Grid Size:");
                    ui.add(egui::DragValue::new(&mut self.grid_size)
                        .speed(1.0)
                        .range(10.0..=200.0));
                });
                
                ui.separator();
                ui.label("Layout:");
                if ui.button("Reset All Positions").clicked() {
                    for person in self.tree.persons.values_mut() {
                        person.position = None;
                    }
                    self.status = "All positions reset".into();
                }
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
                LayoutEngine::snap_to_grid(base_origin, self.grid_size)
            } else {
                base_origin
            };
            
            if self.show_grid {
                LayoutEngine::draw_grid(&painter, rect, origin, self.zoom, self.pan, self.grid_size);
            }

            let nodes = LayoutEngine::compute_layout(&self.tree, origin);

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
                                    let relative_pos = egui::pos2(x - origin.x, y - origin.y);
                                    let snapped_rel = LayoutEngine::snap_to_grid(relative_pos, self.grid_size);
                                    
                                    let snapped_x = origin.x + snapped_rel.x;
                                    let snapped_y = origin.y + snapped_rel.y;
                                    
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
                    let perpendicular = egui::vec2(-dir.y, dir.x) * SPOUSE_LINE_OFFSET;
                    
                    painter.line_segment(
                        [a + perpendicular, b + perpendicular],
                        egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY),
                    );
                    painter.line_segment(
                        [a - perpendicular, b - perpendicular],
                        egui::Stroke::new(EDGE_STROKE_WIDTH, egui::Color32::LIGHT_GRAY),
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

                    painter.rect_filled(*r, NODE_CORNER_RADIUS, fill);
                    painter.rect_stroke(*r, NODE_CORNER_RADIUS, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::epaint::StrokeKind::Outside);

                    let text = LayoutEngine::person_label(&self.tree, n.id);
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
