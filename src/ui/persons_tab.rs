use eframe::egui;
use crate::app::App;
use crate::core::tree::{Gender, PersonId};

const DEFAULT_RELATION_KIND: &str = "biological";

pub trait PersonsTabRenderer {
    fn render_persons_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl PersonsTabRenderer for App {
    fn render_persons_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        ui.heading(t("manage_persons"));
        
        // 新規人物追加ボタン
        if ui.button(t("add_new_person")).clicked() {
            // 現在表示されているキャンバスの左上を計算
            let visible_left_top = if self.canvas.canvas_rect != egui::Rect::NOTHING {
                let screen_pos = self.canvas.canvas_rect.left_top() + egui::vec2(50.0, 50.0);
                let world_pos = self.canvas.canvas_origin + (screen_pos - self.canvas.canvas_origin - self.canvas.pan) / self.canvas.zoom;
                (world_pos.x, world_pos.y)
            } else {
                (100.0, 100.0)
            };
            
            let id = self.tree.add_person(
                t("new_person"),
                Gender::Unknown,
                None,
                String::new(),
                false,
                None,
                visible_left_top,
            );
            self.person_editor.selected = Some(id);
            if let Some(person) = self.tree.persons.get(&id) {
                self.person_editor.new_name = person.name.clone();
                self.person_editor.new_gender = person.gender;
                self.person_editor.new_birth = person.birth.clone().unwrap_or_default();
                self.person_editor.new_memo = person.memo.clone();
                self.person_editor.new_deceased = person.deceased;
                self.person_editor.new_death = person.death.clone().unwrap_or_default();
            }
            self.file.status = t("new_person_added");
        }

        ui.separator();

        // 人物エディタ
        if self.person_editor.selected.is_some() {
            if let Some(person) = self.person_editor.selected.and_then(|id| self.tree.persons.get(&id)) {
                ui.heading(format!("{} {}", t("edit"), person.name));
            }
        } else {
            ui.heading(t("person_editor"));
        }

        ui.horizontal(|ui| {
            ui.label(t("name"));
            ui.text_edit_singleline(&mut self.person_editor.new_name);
        });
        ui.horizontal(|ui| {
            ui.label(t("gender"));
            ui.radio_value(&mut self.person_editor.new_gender, Gender::Male, t("male"));
            ui.radio_value(&mut self.person_editor.new_gender, Gender::Female, t("female"));
            ui.radio_value(&mut self.person_editor.new_gender, Gender::Unknown, t("unknown"));
        });
        ui.horizontal(|ui| {
            ui.label(t("birth"));
            ui.text_edit_singleline(&mut self.person_editor.new_birth);
        });
        ui.checkbox(&mut self.person_editor.new_deceased, t("deceased"));
        if self.person_editor.new_deceased {
            ui.horizontal(|ui| {
                ui.label(t("death"));
                ui.text_edit_singleline(&mut self.person_editor.new_death);
            });
        }
        ui.label(t("memo"));
        ui.text_edit_multiline(&mut self.person_editor.new_memo);

        // 更新・キャンセル・削除ボタン
        ui.horizontal(|ui| {
            if self.person_editor.selected.is_some() {
                if ui.button(t("update")).clicked() {
                    if let Some(sel) = self.person_editor.selected {
                        if let Some(p) = self.tree.persons.get_mut(&sel) {
                            if !self.person_editor.new_name.trim().is_empty() {
                                p.name = self.person_editor.new_name.trim().to_string();
                                p.gender = self.person_editor.new_gender;
                                p.birth = App::parse_optional_field(&self.person_editor.new_birth);
                                p.memo = self.person_editor.new_memo.clone();
                                p.deceased = self.person_editor.new_deceased;
                                p.death = self.person_editor.new_deceased
                                    .then(|| App::parse_optional_field(&self.person_editor.new_death))
                                    .flatten();
                                self.file.status = t("person_updated");
                            } else {
                                self.file.status = t("name_required");
                            }
                        }
                    }
                }
                if ui.button(t("cancel")).clicked() {
                    self.person_editor.selected = None;
                    self.clear_person_form();
                }
                if ui.button(t("delete")).clicked() {
                    if let Some(sel) = self.person_editor.selected {
                        self.tree.remove_person(sel);
                        self.person_editor.selected = None;
                        self.clear_person_form();
                        self.file.status = t("person_deleted");
                    }
                }
            }
        });

        // 関係管理（編集モードの場合のみ表示）
        if let Some(sel) = self.person_editor.selected {
            self.render_relations_section(ui, sel, &t);
        }

        ui.separator();
        ui.label(t("view_controls"));
        ui.label(t("drag_nodes"));
    }
}

impl App {
    fn render_relations_section(&mut self, ui: &mut egui::Ui, sel: PersonId, t: &impl Fn(&str) -> String) {
        ui.separator();
        ui.label(t("relations"));
        
        let all_ids: Vec<PersonId> = self.tree.persons.keys().copied().collect();
        
        // 親の分類
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
        
        // 父親の表示
        self.render_parent_relations(ui, sel, &fathers, &t("father"), t);
        
        // 母親の表示
        self.render_parent_relations(ui, sel, &mothers, &t("mother"), t);
        
        // その他の親の表示
        self.render_parent_relations(ui, sel, &other_parents, &t("parent"), t);
        
        // 配偶者の表示
        self.render_spouse_relations(ui, sel, t);

        // 新しい関係を追加
        self.render_add_relations(ui, sel, &all_ids, t);
    }

    fn render_parent_relations(
        &mut self,
        ui: &mut egui::Ui,
        sel: PersonId,
        parents: &[(PersonId, String)],
        label: &str,
        t: &impl Fn(&str) -> String,
    ) {
        if parents.is_empty() {
            return;
        }

        ui.horizontal(|ui| {
            ui.label(label);
        });
        
        for (parent_id, parent_name) in parents {
            // 関係の種類を取得
            let kind = self.tree.edges.iter()
                .find(|e| e.parent == *parent_id && e.child == sel)
                .map(|e| e.kind.clone())
                .unwrap_or_default();
            
            ui.horizontal(|ui| {
                if ui.small_button(parent_name).clicked() {
                    self.person_editor.selected = Some(*parent_id);
                }
                
                // 種類の表示
                if !kind.is_empty() && kind != "biological" {
                    ui.label(format!("({})", kind));
                }
                
                // 編集ボタン
                if ui.small_button("✏️").on_hover_text(&t("edit_kind")).clicked() {
                    self.relation_editor.editing_parent_kind = Some((*parent_id, sel));
                    self.relation_editor.temp_kind = kind.clone();
                }
                
                // 削除ボタン
                if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                    self.tree.remove_parent_child(*parent_id, sel);
                    self.file.status = t("relation_removed");
                }
            });
            
            // 種類編集UI
            if self.relation_editor.editing_parent_kind == Some((*parent_id, sel)) {
                ui.horizontal(|ui| {
                    ui.label(&t("kind"));
                    ui.text_edit_singleline(&mut self.relation_editor.temp_kind);
                    if ui.button(&t("save")).clicked() {
                        // 親子関係の種類を更新
                        if let Some(edge) = self.tree.edges.iter_mut().find(|e| {
                            e.parent == *parent_id && e.child == sel
                        }) {
                            edge.kind = if self.relation_editor.temp_kind.trim().is_empty() {
                                "biological".to_string()
                            } else {
                                self.relation_editor.temp_kind.trim().to_string()
                            };
                            self.file.status = t("relation_kind_updated");
                        }
                        self.relation_editor.editing_parent_kind = None;
                        self.relation_editor.temp_kind.clear();
                    }
                    if ui.button(&t("cancel")).clicked() {
                        self.relation_editor.editing_parent_kind = None;
                        self.relation_editor.temp_kind.clear();
                    }
                });
            }
        }
    }

    fn render_spouse_relations(&mut self, ui: &mut egui::Ui, sel: PersonId, t: &impl Fn(&str) -> String) {
        let spouse_ids = self.tree.spouses_of(sel);
        if spouse_ids.is_empty() {
            return;
        }

        ui.horizontal(|ui| {
            ui.label(&t("spouses"));
        });
        
        for spouse_id in &spouse_ids {
            // 先に必要な情報をクローンしておく
            let spouse_name = self.tree.persons.get(spouse_id)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            
            // 配偶者関係のメモを取得
            let spouse_memo = self.tree.spouses.iter()
                .find(|s| {
                    (s.person1 == sel && s.person2 == *spouse_id) ||
                    (s.person1 == *spouse_id && s.person2 == sel)
                })
                .map(|s| s.memo.clone())
                .unwrap_or_default();
            
            ui.horizontal(|ui| {
                if ui.small_button(&spouse_name).clicked() {
                    self.person_editor.selected = Some(*spouse_id);
                }
                
                // メモの表示と編集
                if !spouse_memo.is_empty() {
                    ui.label(format!("({})", spouse_memo));
                }
                
                // 編集ボタン
                if ui.small_button("✏️").on_hover_text(&t("edit_memo")).clicked() {
                    self.relation_editor.editing_spouse_memo = Some((sel, *spouse_id));
                    self.relation_editor.temp_spouse_memo = spouse_memo.clone();
                }
                
                // 削除ボタン
                if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                    self.tree.remove_spouse(sel, *spouse_id);
                    self.file.status = t("relation_removed");
                }
            });
            
            // メモ編集UI
            if self.relation_editor.editing_spouse_memo == Some((sel, *spouse_id)) {
                ui.horizontal(|ui| {
                    ui.label(&t("memo"));
                    ui.text_edit_singleline(&mut self.relation_editor.temp_spouse_memo);
                    if ui.button(&t("save")).clicked() {
                        // 配偶者関係のメモを更新
                        if let Some(spouse_rel) = self.tree.spouses.iter_mut().find(|s| {
                            (s.person1 == sel && s.person2 == *spouse_id) ||
                            (s.person1 == *spouse_id && s.person2 == sel)
                        }) {
                            spouse_rel.memo = self.relation_editor.temp_spouse_memo.clone();
                            self.file.status = t("spouse_memo_updated");
                        }
                        self.relation_editor.editing_spouse_memo = None;
                        self.relation_editor.temp_spouse_memo.clear();
                    }
                    if ui.button(&t("cancel")).clicked() {
                        self.relation_editor.editing_spouse_memo = None;
                        self.relation_editor.temp_spouse_memo.clear();
                    }
                });
            }
        }
    }

    fn render_add_relations(
        &mut self,
        ui: &mut egui::Ui,
        sel: PersonId,
        all_ids: &[PersonId],
        t: &impl Fn(&str) -> String,
    ) {
        ui.separator();
        ui.label(t("add_relations"));
        
        // 親を追加
        ui.horizontal(|ui| {
            ui.label(t("add_parent"));
            egui::ComboBox::from_id_salt("add_parent")
                .selected_text(
                    self.relation_editor.parent_pick
                        .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                        .unwrap_or_else(|| t("select")),
                )
                .show_ui(ui, |ui| {
                    for id in all_ids {
                        if *id != sel {
                            let name = self.get_person_name(id);
                            ui.selectable_value(&mut self.relation_editor.parent_pick, Some(*id), name);
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label(t("kind"));
            ui.text_edit_singleline(&mut self.relation_editor.relation_kind);
            if ui.button(t("add")).clicked() {
                if let Some(parent) = self.relation_editor.parent_pick {
                    let kind = if self.relation_editor.relation_kind.trim().is_empty() {
                        DEFAULT_RELATION_KIND
                    } else {
                        self.relation_editor.relation_kind.trim()
                    };
                    self.tree.add_parent_child(parent, sel, kind.to_string());
                    self.relation_editor.parent_pick = None;
                    self.file.status = t("parent_added");
                }
            }
        });

        ui.add_space(4.0);
        
        // 子を追加
        ui.horizontal(|ui| {
            ui.label(t("add_child"));
            egui::ComboBox::from_id_salt("add_child")
                .selected_text(
                    self.relation_editor.child_pick
                        .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                        .unwrap_or_else(|| t("select")),
                )
                .show_ui(ui, |ui| {
                    for id in all_ids {
                        if *id != sel {
                            let name = self.get_person_name(id);
                            ui.selectable_value(&mut self.relation_editor.child_pick, Some(*id), name);
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label(t("kind"));
            ui.text_edit_singleline(&mut self.relation_editor.relation_kind);
            if ui.button(t("add")).clicked() {
                if let Some(child) = self.relation_editor.child_pick {
                    let kind = if self.relation_editor.relation_kind.trim().is_empty() {
                        DEFAULT_RELATION_KIND
                    } else {
                        self.relation_editor.relation_kind.trim()
                    };
                    self.tree.add_parent_child(sel, child, kind.to_string());
                    self.relation_editor.child_pick = None;
                    self.file.status = t("child_added");
                }
            }
        });

        ui.add_space(4.0);
        
        // 配偶者を追加
        ui.horizontal(|ui| {
            ui.label(t("add_spouse"));
            egui::ComboBox::from_id_salt("add_spouse")
                .selected_text(
                    self.relation_editor.spouse_pick
                        .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                        .unwrap_or_else(|| t("select")),
                )
                .show_ui(ui, |ui| {
                    for id in all_ids {
                        if *id != sel {
                            let name = self.get_person_name(id);
                            ui.selectable_value(&mut self.relation_editor.spouse_pick, Some(*id), name);
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label(t("memo"));
            ui.text_edit_singleline(&mut self.relation_editor.spouse_memo);
            if ui.button(t("add")).clicked() {
                if let Some(spouse) = self.relation_editor.spouse_pick {
                    self.tree.add_spouse(sel, spouse, self.relation_editor.spouse_memo.clone());
                    self.relation_editor.spouse_pick = None;
                    self.relation_editor.spouse_memo.clear();
                    self.file.status = t("spouse_added");
                }
            }
        });
    }
}
