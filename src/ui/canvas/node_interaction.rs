use crate::app::App;
use crate::core::tree::PersonId;
use crate::core::layout::LayoutEngine;
use crate::core::i18n::Texts;
use crate::ui::{LogLevel, SideTab};
use super::NodeInteractionHandler;
use std::collections::HashMap;

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
        
        // Ctrlキーが押されているかチェック
        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);
        
        for n in nodes {
            if let Some(r) = screen_rects.get(&n.id) {
                let node_id = ui.id().with(n.id);
                let node_response = ui.interact(*r, node_id, egui::Sense::click_and_drag());
                
                if node_response.hovered() {
                    node_hovered = true;
                }
                
                if node_response.drag_started() {
                    // 複数選択されたノードのドラッグ開始
                    if !self.person_editor.selected_ids.is_empty() && 
                       self.person_editor.selected_ids.contains(&n.id) {
                        // 複数選択されたノードすべての初期位置を記録
                        self.canvas.multi_drag_starts.clear();
                        let mut names = Vec::new();
                        for id in &self.person_editor.selected_ids {
                            if let Some(person) = self.tree.persons.get(id) {
                                self.canvas.multi_drag_starts.insert(*id, person.position);
                                names.push(person.name.clone());
                            }
                        }
                        let lang = self.ui.language;
                        let t = |key: &str| Texts::get(key, lang);
                        self.log.add(format!("{}{}{}: {}", 
                            names.len(),
                            t("log_nodes_selected"),
                            t("log_node_drag_start"),
                            names.join(", ")
                        ), LogLevel::Debug);
                    } else {
                        // 単一ノードの場合もドラッグ開始位置を記録
                        self.canvas.multi_drag_starts.clear();
                        if let Some(person) = self.tree.persons.get(&n.id) {
                            self.canvas.multi_drag_starts.insert(n.id, person.position);
                            let lang = self.ui.language;
                            let t = |key: &str| Texts::get(key, lang);
                            self.log.add(format!("{}: {} (x:{:.0}, y:{:.0})",
                                t("log_node_drag_start"),
                                person.name,
                                person.position.0,
                                person.position.1
                            ), LogLevel::Debug);
                        }
                    }
                    self.canvas.dragging_node = Some(n.id);
                    self.canvas.node_drag_start = pointer_pos;
                }
                
                if node_response.dragged() && self.canvas.dragging_node == Some(n.id) {
                    any_node_dragged = true;
                    if let (Some(pos), Some(start)) = (pointer_pos, self.canvas.node_drag_start) {
                        let delta = (pos - start) / self.canvas.zoom;
                        
                        // ドラッグ開始時の位置からの累積移動量を使用
                        for (id, start_pos) in &self.canvas.multi_drag_starts {
                            if let Some(person) = self.tree.persons.get_mut(id) {
                                let new_x = start_pos.0 + delta.x;
                                let new_y = start_pos.1 + delta.y;
                                person.position = (new_x, new_y);
                            }
                        }
                    }
                }
                
                if node_response.drag_stopped() && self.canvas.dragging_node == Some(n.id) {
                    // ドラッグ完了のログを記録
                    let lang = self.ui.language;
                    let t = |key: &str| Texts::get(key, lang);
                    if self.canvas.multi_drag_starts.len() > 1 {
                        let mut moved_info = Vec::new();
                        for id in self.canvas.multi_drag_starts.keys() {
                            if let Some(person) = self.tree.persons.get(id) {
                                moved_info.push(format!("{} (x:{:.0}, y:{:.0})", 
                                    person.name,
                                    person.position.0,
                                    person.position.1
                                ));
                            }
                        }
                        self.log.add(format!("{}{}: {}", 
                            moved_info.len(),
                            t("log_nodes_moved"),
                            moved_info.join(", ")
                        ), LogLevel::Debug);
                    } else if let Some(person) = self.tree.persons.get(&n.id) {
                        let start_pos = self.canvas.multi_drag_starts.get(&n.id);
                        if let Some(start) = start_pos {
                            let delta_x = person.position.0 - start.0;
                            let delta_y = person.position.1 - start.1;
                            let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
                            self.log.add(format!("{}: {} → (x:{:.0}, y:{:.0}) {}:{:.0}px",
                                t("log_node_drag_start"),
                                person.name,
                                person.position.0,
                                person.position.1,
                                t("log_distance"),
                                distance
                            ), LogLevel::Debug);
                        }
                    }
                    
                    if self.canvas.show_grid {
                        // 複数選択されている場合は、すべてのノードをグリッドにスナップ
                        if !self.canvas.multi_drag_starts.is_empty() {
                            for id in self.canvas.multi_drag_starts.keys() {
                                if let Some(person) = self.tree.persons.get_mut(id) {
                                    let (x, y) = person.position;
                                    let relative_pos = egui::pos2(x - origin.x, y - origin.y);
                                    let snapped_rel = LayoutEngine::snap_to_grid(relative_pos, self.canvas.grid_size);
                                    
                                    let snapped_x = origin.x + snapped_rel.x;
                                    let snapped_y = origin.y + snapped_rel.y;
                                    
                                    person.position = (snapped_x, snapped_y);
                                }
                            }
                        } else {
                            if let Some(person) = self.tree.persons.get_mut(&n.id) {
                                let (x, y) = person.position;
                                let relative_pos = egui::pos2(x - origin.x, y - origin.y);
                                let snapped_rel = LayoutEngine::snap_to_grid(relative_pos, self.canvas.grid_size);
                                
                                let snapped_x = origin.x + snapped_rel.x;
                                let snapped_y = origin.y + snapped_rel.y;
                                
                                person.position = (snapped_x, snapped_y);
                            }
                        }
                    }
                    self.canvas.dragging_node = None;
                    self.canvas.node_drag_start = None;
                    self.canvas.multi_drag_starts.clear();
                }
                
                if node_response.clicked() {
                    // Ctrlキーが押されている場合は複数選択
                    if ctrl_pressed {
                        if let Some(idx) = self.person_editor.selected_ids.iter().position(|id| *id == n.id) {
                            // 既に選択されている場合は選択解除
                            self.person_editor.selected_ids.remove(idx);
                            let person_name = self.get_person_name(&n.id);
                            let lang = self.ui.language;
                            let t = |key: &str| Texts::get(key, lang);
                            self.log.add(format!("{}: {}", t("log_node_deselected"), person_name), LogLevel::Debug);
                            // 最後の選択を更新
                            if let Some(last_id) = self.person_editor.selected_ids.last() {
                                self.person_editor.selected = Some(*last_id);
                                if let Some(person) = self.tree.persons.get(last_id) {
                                    self.person_editor.new_name = person.name.clone();
                                    self.person_editor.new_gender = person.gender;
                                    self.person_editor.new_birth = person.birth.clone().unwrap_or_default();
                                    self.person_editor.new_memo = person.memo.clone();
                                    self.person_editor.new_deceased = person.deceased;
                                    self.person_editor.new_death = person.death.clone().unwrap_or_default();
                                    self.person_editor.new_photo_path = person.photo_path.clone().unwrap_or_default();
                                    self.person_editor.new_display_mode = person.display_mode;
                                    self.person_editor.new_photo_scale = person.photo_scale;
                                }
                            } else {
                                self.person_editor.selected = None;
                            }
                        } else {
                            // 新規選択を追加
                            self.ui.side_tab = SideTab::Persons;
                            self.person_editor.selected_ids.push(n.id);
                            self.person_editor.selected = Some(n.id);
                            let person_name = self.get_person_name(&n.id);
                            let lang = self.ui.language;
                            let t = |key: &str| Texts::get(key, lang);
                            self.log.add(format!("{}: {} ({} {}{})", t("log_node_added_to_selection"), person_name, t("log_total"), self.person_editor.selected_ids.len(), t("count_suffix")), LogLevel::Debug);
                            if let Some(person) = self.tree.persons.get(&n.id) {
                                self.person_editor.new_name = person.name.clone();
                                self.person_editor.new_gender = person.gender;
                                self.person_editor.new_birth = person.birth.clone().unwrap_or_default();
                                self.person_editor.new_memo = person.memo.clone();
                                self.person_editor.new_deceased = person.deceased;
                                self.person_editor.new_death = person.death.clone().unwrap_or_default();
                                self.person_editor.new_photo_path = person.photo_path.clone().unwrap_or_default();
                                self.person_editor.new_display_mode = person.display_mode;
                                self.person_editor.new_photo_scale = person.photo_scale;
                            }
                        }
                    } else {
                        // Ctrlキーが押されていない場合は単一選択
                        self.ui.side_tab = SideTab::Persons;
                        self.person_editor.selected_ids.clear();
                        self.person_editor.selected_ids.push(n.id);
                        self.person_editor.selected = Some(n.id);
                        let person_name = self.get_person_name(&n.id);
                        let lang = self.ui.language;
                        let t = |key: &str| Texts::get(key, lang);
                        self.log.add(format!("{}: {}", t("log_node_selected"), person_name), LogLevel::Debug);
                        if let Some(person) = self.tree.persons.get(&n.id) {
                            self.person_editor.new_name = person.name.clone();
                            self.person_editor.new_gender = person.gender;
                            self.person_editor.new_birth = person.birth.clone().unwrap_or_default();
                            self.person_editor.new_memo = person.memo.clone();
                            self.person_editor.new_deceased = person.deceased;
                            self.person_editor.new_death = person.death.clone().unwrap_or_default();
                            self.person_editor.new_photo_path = person.photo_path.clone().unwrap_or_default();
                            self.person_editor.new_display_mode = person.display_mode;
                            self.person_editor.new_photo_scale = person.photo_scale;
                        }
                    }
                }
            }
        }
        
        (node_hovered, any_node_dragged)
    }
}
