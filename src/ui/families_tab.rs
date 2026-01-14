use crate::app::App;

/// 家族タブのUI描画トレイト
pub trait FamiliesTabRenderer {
    fn render_families_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl FamiliesTabRenderer for App {
    fn render_families_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        ui.heading(t("manage_families"));
        
        if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(t("add_new_family"))).clicked() {
            let color = (
                (self.family_editor.new_family_color[0] * 255.0) as u8,
                (self.family_editor.new_family_color[1] * 255.0) as u8,
                (self.family_editor.new_family_color[2] * 255.0) as u8,
            );
            let new_id = self.tree.add_family(t("new_family"), Some(color));
            self.family_editor.selected_family = Some(new_id);
            self.family_editor.new_family_name = t("new_family");
            self.file.status = t("new_family_added");
        }
    
        ui.separator();
    
        // 家族エディタ
        if self.family_editor.selected_family.is_some() {
            if let Some(family) = self.family_editor.selected_family.and_then(|id| self.tree.families.iter().find(|f| f.id == id)) {
                ui.heading(format!("{} {}", t("edit"), family.name));
            }
        } else {
            ui.heading(t("family_editor"));
        }
        
        ui.horizontal(|ui| {
            ui.label(t("name"));
            ui.text_edit_singleline(&mut self.family_editor.new_family_name);
        });
        
        ui.horizontal(|ui| {
            ui.label(t("color"));
            ui.color_edit_button_rgb(&mut self.family_editor.new_family_color);
        });
        
        ui.separator();
        ui.heading(t("members"));
    
        // メンバーリスト
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            if let Some(family_id) = self.family_editor.selected_family {
                if let Some(family) = self.tree.families.iter().find(|f| f.id == family_id) {
                    if family.members.is_empty() {
                        ui.label(t("no_members"));
                    } else {
                        let members = family.members.clone();
                        for member_id in &members {
                            if let Some(person) = self.tree.persons.get(member_id) {
                                let person_name = person.name.clone();
                                ui.horizontal(|ui| {
                                    ui.label(&person_name);
                                    if ui.small_button("➖").clicked() {
                                        self.tree.remove_member_from_family(family_id, *member_id);
                                        self.file.status = t("member_removed");
                                    }
                                });
                            }
                        }
                    }
                }
            } else {
                ui.label(t("no_family_selected"));
            }
        });
    
        ui.separator();
        
        // メンバー追加
        if self.family_editor.selected_family.is_some() {
            ui.horizontal(|ui| {
                ui.label(t("add_member"));
                egui::ComboBox::from_id_salt("family_member_pick")
                    .selected_text(
                        self.family_editor.family_member_pick
                            .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.as_str()))
                            .unwrap_or(&t("select"))
                    )
                    .show_ui(ui, |ui| {
                        if let Some(family_id) = self.family_editor.selected_family {
                            if let Some(family) = self.tree.families.iter().find(|f| f.id == family_id) {
                                for (id, person) in &self.tree.persons {
                                    if !family.members.contains(id) {
                                        ui.selectable_value(&mut self.family_editor.family_member_pick, Some(*id), &person.name);
                                    }
                                }
                            }
                        }
                    });
                    
                if let Some(pid) = self.family_editor.family_member_pick {
                    if ui.small_button(t("add")).clicked() {
                        if let Some(family_id) = self.family_editor.selected_family {
                            self.tree.add_member_to_family(family_id, pid);
                            self.family_editor.family_member_pick = None;
                            self.file.status = t("member_added");
                        }
                    }
                }
            });
        }

        ui.separator();

        // アクションボタン
        if let Some(family_id) = self.family_editor.selected_family {
            ui.horizontal(|ui| {
                if ui.button(t("update")).clicked() && !self.family_editor.new_family_name.trim().is_empty() {
                    if let Some(family) = self.tree.families.iter_mut().find(|f| f.id == family_id) {
                        family.name = self.family_editor.new_family_name.clone();
                        family.color = Some((
                            (self.family_editor.new_family_color[0] * 255.0) as u8,
                            (self.family_editor.new_family_color[1] * 255.0) as u8,
                            (self.family_editor.new_family_color[2] * 255.0) as u8,
                        ));
                        self.file.status = t("family_updated");
                    }
                }
                
                if ui.button(t("cancel")).clicked() {
                    self.family_editor.selected_family = None;
                    self.family_editor.new_family_name.clear();
                    self.family_editor.family_member_pick = None;
                }
                
                if ui.button(t("delete_family")).clicked() {
                    self.tree.remove_family(family_id);
                    self.family_editor.selected_family = None;
                    self.family_editor.new_family_name.clear();
                    self.family_editor.family_member_pick = None;
                    self.file.status = t("family_deleted");
                }
            });
        }
    }
}
