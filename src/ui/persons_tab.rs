use std::collections::HashMap;

use eframe::egui;
use crate::app::App;
use crate::core::tree::{Gender, Person, PersonDisplayMode, PersonId};

const DEFAULT_RELATION_KIND: &str = "biological";

pub trait PersonsTabRenderer {
    fn render_persons_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl PersonsTabRenderer for App {
    fn render_persons_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        self.render_persons_tab_header(ui, &t);
        self.render_persons_tab_editor_section(ui, &t);

        // 関係管理（編集モードの場合のみ表示）
        if let Some(sel) = self.person_editor.selected {
            self.render_persons_tab_relations_section(ui, sel, &t);
        }

        self.render_persons_tab_actions_section(ui, &t);
        self.render_persons_tab_footer(ui, &t);
    }
}

impl App {
    fn render_persons_tab_header(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.heading(t("manage_persons"));
        if ui.button(t("add_new_person")).clicked() {
            self.add_new_person(t);
        }
        ui.separator();
    }

    fn add_new_person(&mut self, t: &impl Fn(&str) -> String) {
        let visible_left_top = self.visible_canvas_left_top();
        let person_id = self.tree.add_person(
            t("new_person"),
            Gender::Unknown,
            None,
            String::new(),
            false,
            None,
            visible_left_top,
        );
        self.person_editor.selected = Some(person_id);
        self.load_selected_person_into_form(person_id);
        self.file.status = t("new_person_added");
        self.log
            .add(format!("{}: {}", t("log_person_added"), t("new_person")));
    }

    fn load_selected_person_into_form(&mut self, person_id: PersonId) {
        if let Some(person) = self.tree.persons.get(&person_id) {
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

    fn render_persons_tab_editor_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        self.render_person_editor_heading(ui, t);
        self.render_person_basic_fields(ui, t);
        self.render_person_photo_fields(ui, t);
        self.render_person_display_fields(ui, t);
    }

    fn render_persons_tab_actions_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        self.render_person_action_buttons(ui, t);
    }

    fn render_person_editor_heading(&self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        if let Some(person) = self
            .person_editor
            .selected
            .and_then(|id| self.tree.persons.get(&id))
        {
            ui.heading(format!("{} {}", t("edit"), person.name));
            return;
        }
        ui.heading(t("person_editor"));
    }

    fn render_person_basic_fields(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.horizontal(|ui| {
            ui.label(t("name"));
            ui.text_edit_singleline(&mut self.person_editor.new_name);
        });
        ui.horizontal(|ui| {
            ui.label(t("gender"));
            ui.radio_value(&mut self.person_editor.new_gender, Gender::Male, t("male"));
            ui.radio_value(
                &mut self.person_editor.new_gender,
                Gender::Female,
                t("female"),
            );
            ui.radio_value(
                &mut self.person_editor.new_gender,
                Gender::Unknown,
                t("unknown"),
            );
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
    }

    fn render_person_photo_fields(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.horizontal(|ui| {
            ui.label(t("photo_path"));
            ui.text_edit_singleline(&mut self.person_editor.new_photo_path);
            if ui.button(t("choose_photo")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
                    .pick_file()
                {
                    self.person_editor.new_photo_path = path.display().to_string();
                }
            }
            if !self.person_editor.new_photo_path.is_empty() && ui.button(t("clear_photo")).clicked() {
                self.person_editor.new_photo_path.clear();
            }
        });
    }

    fn render_person_display_fields(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.horizontal(|ui| {
            ui.label(t("display_mode"));
            ui.radio_value(
                &mut self.person_editor.new_display_mode,
                PersonDisplayMode::NameOnly,
                t("name_only"),
            );
            ui.radio_value(
                &mut self.person_editor.new_display_mode,
                PersonDisplayMode::NameAndPhoto,
                t("name_and_photo"),
            );
        });

        if self.person_editor.new_display_mode == PersonDisplayMode::NameAndPhoto {
            ui.horizontal(|ui| {
                ui.label(t("photo_scale"));
                ui.add(egui::Slider::new(&mut self.person_editor.new_photo_scale, 0.1..=3.0).text("×"));
            });
        }
    }

    fn render_person_action_buttons(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.horizontal(|ui| {
            if self.person_editor.selected.is_none() {
                return;
            }
            if ui.button(t("update")).clicked() {
                self.update_selected_person(t);
            }
            if ui.button(t("cancel")).clicked() {
                self.cancel_person_edit();
            }
            if ui.button(t("delete")).clicked() {
                self.delete_selected_person(t);
            }
        });
    }

    fn update_selected_person(&mut self, t: &impl Fn(&str) -> String) {
        if self.person_editor.new_name.trim().is_empty() {
            self.file.status = t("name_required");
            return;
        }

        let Some(person_id) = self.person_editor.selected else {
            return;
        };

        if let Some(person) = self.tree.persons.get_mut(&person_id) {
            person.name = self.person_editor.new_name.trim().to_string();
            person.gender = self.person_editor.new_gender;
            person.birth = App::parse_optional_field(&self.person_editor.new_birth);
            person.memo = self.person_editor.new_memo.clone();
            person.deceased = self.person_editor.new_deceased;
            person.death = self
                .person_editor
                .new_deceased
                .then(|| App::parse_optional_field(&self.person_editor.new_death))
                .flatten();
            person.photo_path = if self.person_editor.new_photo_path.trim().is_empty() {
                None
            } else {
                Some(self.person_editor.new_photo_path.trim().to_string())
            };
            person.display_mode = self.person_editor.new_display_mode;
            person.photo_scale = self.person_editor.new_photo_scale.clamp(0.1, 3.0);
            self.file.status = t("person_updated");
        }
    }

    fn cancel_person_edit(&mut self) {
        self.person_editor.selected = None;
        self.clear_person_form();
    }

    fn delete_selected_person(&mut self, t: &impl Fn(&str) -> String) {
        let Some(person_id) = self.person_editor.selected else {
            return;
        };

        let person_name = self.get_person_name(&person_id);
        self.tree.remove_person(person_id);
        self.person_editor.selected = None;
        self.person_editor.selected_ids.clear();
        self.clear_person_form();
        self.file.status = t("deleted");
        self.log
            .add(format!("{}: {}", t("log_person_deleted"), person_name));
    }

    fn render_persons_tab_footer(&self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.separator();
        ui.label(t("view_controls"));
        ui.label(t("drag_nodes"));
    }

    fn selected_person_name_or_select(
        persons: &HashMap<PersonId, Person>,
        selected: Option<PersonId>,
        t: &impl Fn(&str) -> String,
    ) -> String {
        selected
            .and_then(|id| persons.get(&id).map(|person| person.name.clone()))
            .unwrap_or_else(|| t("select"))
    }

    fn render_relation_target_picker(
        ui: &mut egui::Ui,
        persons: &HashMap<PersonId, Person>,
        combo_id: &str,
        selected: &mut Option<PersonId>,
        current_person: PersonId,
        all_ids: &[PersonId],
        t: &impl Fn(&str) -> String,
    ) {
        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(Self::selected_person_name_or_select(persons, *selected, t))
            .show_ui(ui, |ui| {
                for id in all_ids {
                    if *id != current_person {
                        let person_name = persons
                            .get(id)
                            .map(|person| person.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        ui.selectable_value(selected, Some(*id), person_name);
                    }
                }
            });
    }

    fn relation_kind_or_default(&self) -> String {
        let kind = self.relation_editor.relation_kind.trim();
        if kind.is_empty() {
            DEFAULT_RELATION_KIND.to_string()
        } else {
            kind.to_string()
        }
    }

    fn start_parent_kind_edit(&mut self, parent_id: PersonId, child_id: PersonId, current_kind: &str) {
        self.relation_editor.editing_parent_kind = Some((parent_id, child_id));
        self.relation_editor.temp_kind = current_kind.to_string();
    }

    fn clear_parent_kind_edit(&mut self) {
        self.relation_editor.editing_parent_kind = None;
        self.relation_editor.temp_kind.clear();
    }

    fn remove_parent_relation(&mut self, parent_id: PersonId, child_id: PersonId, t: &impl Fn(&str) -> String) {
        self.tree.remove_parent_child(parent_id, child_id);
        self.file.status = t("relation_removed");
    }

    fn save_parent_relation_kind(&mut self, parent_id: PersonId, child_id: PersonId, t: &impl Fn(&str) -> String) {
        if let Some(edge) = self
            .tree
            .edges
            .iter_mut()
            .find(|edge| edge.parent == parent_id && edge.child == child_id)
        {
            edge.kind = if self.relation_editor.temp_kind.trim().is_empty() {
                "biological".to_string()
            } else {
                self.relation_editor.temp_kind.trim().to_string()
            };
            self.file.status = t("relation_kind_updated");
        }
        self.clear_parent_kind_edit();
    }

    fn start_spouse_memo_edit(&mut self, person1: PersonId, person2: PersonId, current_memo: &str) {
        self.relation_editor.editing_spouse_memo = Some((person1, person2));
        self.relation_editor.temp_spouse_memo = current_memo.to_string();
    }

    fn clear_spouse_memo_edit(&mut self) {
        self.relation_editor.editing_spouse_memo = None;
        self.relation_editor.temp_spouse_memo.clear();
    }

    fn remove_spouse_relation(&mut self, person1: PersonId, person2: PersonId, t: &impl Fn(&str) -> String) {
        self.tree.remove_spouse(person1, person2);
        self.file.status = t("relation_removed");
    }

    fn save_spouse_relation_memo(&mut self, person1: PersonId, person2: PersonId, t: &impl Fn(&str) -> String) {
        if let Some(spouse_relation) = self
            .tree
            .spouses
            .iter_mut()
            .find(|spouse_relation| {
                (spouse_relation.person1 == person1 && spouse_relation.person2 == person2)
                    || (spouse_relation.person1 == person2 && spouse_relation.person2 == person1)
            })
        {
            spouse_relation.memo = self.relation_editor.temp_spouse_memo.clone();
            self.file.status = t("spouse_memo_updated");
        }
        self.clear_spouse_memo_edit();
    }

    fn render_persons_tab_relations_section(
        &mut self,
        ui: &mut egui::Ui,
        sel: PersonId,
        t: &impl Fn(&str) -> String,
    ) {
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
                    self.start_parent_kind_edit(*parent_id, sel, &kind);
                }
                
                // 削除ボタン
                if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                    self.remove_parent_relation(*parent_id, sel, t);
                }
            });
            
            // 種類編集UI
            if self.relation_editor.editing_parent_kind == Some((*parent_id, sel)) {
                ui.horizontal(|ui| {
                    ui.label(&t("kind"));
                    ui.text_edit_singleline(&mut self.relation_editor.temp_kind);
                    if ui.button(&t("save")).clicked() {
                        self.save_parent_relation_kind(*parent_id, sel, t);
                    }
                    if ui.button(&t("cancel")).clicked() {
                        self.clear_parent_kind_edit();
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
                    self.start_spouse_memo_edit(sel, *spouse_id, &spouse_memo);
                }
                
                // 削除ボタン
                if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                    self.remove_spouse_relation(sel, *spouse_id, t);
                }
            });
            
            // メモ編集UI
            if self.relation_editor.editing_spouse_memo == Some((sel, *spouse_id)) {
                ui.horizontal(|ui| {
                    ui.label(&t("memo"));
                    ui.text_edit_singleline(&mut self.relation_editor.temp_spouse_memo);
                    if ui.button(&t("save")).clicked() {
                        self.save_spouse_relation_memo(sel, *spouse_id, t);
                    }
                    if ui.button(&t("cancel")).clicked() {
                        self.clear_spouse_memo_edit();
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
            Self::render_relation_target_picker(
                ui,
                &self.tree.persons,
                "add_parent",
                &mut self.relation_editor.parent_pick,
                sel,
                all_ids,
                t,
            );
        });
        ui.horizontal(|ui| {
            ui.label(t("kind"));
            ui.text_edit_singleline(&mut self.relation_editor.relation_kind);
            if ui.button(t("add")).clicked() {
                if let Some(parent) = self.relation_editor.parent_pick {
                    let relation_kind = self.relation_kind_or_default();
                    self.tree.add_parent_child(parent, sel, relation_kind);
                    self.relation_editor.parent_pick = None;
                    self.file.status = t("parent_added");
                }
            }
        });

        ui.add_space(4.0);
        
        // 子を追加
        ui.horizontal(|ui| {
            ui.label(t("add_child"));
            Self::render_relation_target_picker(
                ui,
                &self.tree.persons,
                "add_child",
                &mut self.relation_editor.child_pick,
                sel,
                all_ids,
                t,
            );
        });
        ui.horizontal(|ui| {
            ui.label(t("kind"));
            ui.text_edit_singleline(&mut self.relation_editor.relation_kind);
            if ui.button(t("add")).clicked() {
                if let Some(child) = self.relation_editor.child_pick {
                    let relation_kind = self.relation_kind_or_default();
                    self.tree.add_parent_child(sel, child, relation_kind);
                    self.relation_editor.child_pick = None;
                    self.file.status = t("child_added");
                }
            }
        });

        ui.add_space(4.0);
        
        // 配偶者を追加
        ui.horizontal(|ui| {
            ui.label(t("add_spouse"));
            Self::render_relation_target_picker(
                ui,
                &self.tree.persons,
                "add_spouse",
                &mut self.relation_editor.spouse_pick,
                sel,
                all_ids,
                t,
            );
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
