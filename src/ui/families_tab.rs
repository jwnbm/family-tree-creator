use crate::app::App;

use uuid::Uuid;

/// 家族タブのUI描画トレイト
pub trait FamiliesTabRenderer {
    fn render_families_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl FamiliesTabRenderer for App {
    fn render_families_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        self.render_families_tab_header(ui, &t);
        self.render_families_tab_editor_section(ui, &t);
        self.render_families_tab_relations_section(ui, &t);
        self.render_families_tab_actions_section(ui, &t);
        self.render_families_tab_footer(ui, &t);
    }
}

impl App {
    fn render_families_tab_header(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.heading(t("manage_families"));
        if ui.button(t("add_new_family")).clicked() {
            self.add_new_family(t);
        }
        ui.separator();
    }

    fn add_new_family(&mut self, t: &impl Fn(&str) -> String) {
        let color = self.family_editor_color_rgb();
        let family_id = self.tree.add_family(t("new_family"), Some(color));
        self.family_editor.selected_family = Some(family_id);
        self.family_editor.new_family_name = t("new_family");
        self.file.status = t("new_family_added");
        self.log
            .add(format!("{}: {}", t("log_family_added"), t("new_family")));
    }

    fn render_families_tab_editor_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        self.render_family_editor_heading(ui, t);
        self.render_family_editor_fields(ui, t);
    }

    fn render_family_editor_heading(&self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        if let Some(family) = self
            .family_editor
            .selected_family
            .and_then(|id| self.tree.families.iter().find(|family| family.id == id))
        {
            ui.heading(format!("{} {}", t("edit"), family.name));
            return;
        }
        ui.heading(t("family_editor"));
    }

    fn render_family_editor_fields(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.horizontal(|ui| {
            ui.label(t("name"));
            ui.text_edit_singleline(&mut self.family_editor.new_family_name);
        });

        ui.horizontal(|ui| {
            ui.label(t("color"));
            ui.color_edit_button_rgb(&mut self.family_editor.new_family_color);
        });
    }

    fn render_families_tab_relations_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.separator();
        ui.heading(t("members"));
        self.render_family_members_list(ui, t);
        ui.separator();
        self.render_add_family_member_section(ui, t);
    }

    fn render_family_members_list(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            if let Some(family_id) = self.family_editor.selected_family {
                if let Some(family) = self.tree.families.iter().find(|family| family.id == family_id) {
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
                                        self.remove_member_from_selected_family(
                                            family_id,
                                            *member_id,
                                            &person_name,
                                            t,
                                        );
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
    }

    fn remove_member_from_selected_family(
        &mut self,
        family_id: Uuid,
        member_id: crate::core::tree::PersonId,
        person_name: &str,
        t: &impl Fn(&str) -> String,
    ) {
        let family_name = self.family_name_or_default(family_id);
        self.tree.remove_member_from_family(family_id, member_id);
        self.file.status = t("member_removed");
        self.log.add(format!(
            "{}: {} {} {}",
            t("log_family_member_removed"),
            person_name,
            t("log_from"),
            family_name
        ));
    }

    fn render_add_family_member_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        if self.family_editor.selected_family.is_none() {
            return;
        }

        ui.horizontal(|ui| {
            ui.label(t("add_member"));
            egui::ComboBox::from_id_salt("family_member_pick")
                .selected_text(
                    self.family_editor
                        .family_member_pick
                        .and_then(|id| self.tree.persons.get(&id).map(|person| person.name.as_str()))
                        .unwrap_or(&t("select")),
                )
                .show_ui(ui, |ui| {
                    if let Some(family_id) = self.family_editor.selected_family {
                        if let Some(family) = self.tree.families.iter().find(|family| family.id == family_id) {
                            for (person_id, person) in &self.tree.persons {
                                if !family.members.contains(person_id) {
                                    ui.selectable_value(
                                        &mut self.family_editor.family_member_pick,
                                        Some(*person_id),
                                        &person.name,
                                    );
                                }
                            }
                        }
                    }
                });

            if let Some(person_id) = self.family_editor.family_member_pick {
                if ui.small_button(t("add")).clicked() {
                    self.add_member_to_selected_family(person_id, t);
                }
            }
        });
    }

    fn add_member_to_selected_family(
        &mut self,
        person_id: crate::core::tree::PersonId,
        t: &impl Fn(&str) -> String,
    ) {
        let Some(family_id) = self.family_editor.selected_family else {
            return;
        };

        let family_name = self.family_name_or_default(family_id);
        let person_name = self.get_person_name(&person_id);
        self.tree.add_member_to_family(family_id, person_id);
        self.family_editor.family_member_pick = None;
        self.file.status = t("member_added");
        self.log.add(format!(
            "{}: {} {} {}",
            t("log_family_member_added"),
            person_name,
            t("log_to"),
            family_name
        ));
    }

    fn render_families_tab_actions_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.separator();
        let Some(family_id) = self.family_editor.selected_family else {
            return;
        };

        ui.horizontal(|ui| {
            if ui.button(t("update")).clicked() {
                self.update_selected_family(family_id, t);
            }

            if ui.button(t("cancel")).clicked() {
                self.clear_family_editor_selection();
            }

            if ui.button(t("delete_family")).clicked() {
                self.delete_selected_family(family_id, t);
            }
        });
    }

    fn render_families_tab_footer(&self, _ui: &mut egui::Ui, _t: &impl Fn(&str) -> String) {
    }

    fn update_selected_family(&mut self, family_id: Uuid, t: &impl Fn(&str) -> String) {
        if self.family_editor.new_family_name.trim().is_empty() {
            return;
        }

        let new_name = self.family_editor.new_family_name.clone();
        let color = self.family_editor_color_rgb();
        if let Some(family) = self
            .tree
            .families
            .iter_mut()
            .find(|family| family.id == family_id)
        {
            let old_name = family.name.clone();
            family.name = new_name;
            family.color = Some(color);
            self.file.status = t("family_updated");
            self.log.add(format!(
                "{}: {} {} {}",
                t("log_family_updated"),
                old_name,
                t("log_to"),
                family.name
            ));
        }
    }

    fn delete_selected_family(&mut self, family_id: Uuid, t: &impl Fn(&str) -> String) {
        let family_name = self.family_name_or_default(family_id);
        self.tree.remove_family(family_id);
        self.clear_family_editor_selection();
        self.file.status = t("family_deleted");
        self.log
            .add(format!("{}: {}", t("log_family_deleted"), family_name));
    }

    fn family_editor_color_rgb(&self) -> (u8, u8, u8) {
        (
            (self.family_editor.new_family_color[0] * 255.0) as u8,
            (self.family_editor.new_family_color[1] * 255.0) as u8,
            (self.family_editor.new_family_color[2] * 255.0) as u8,
        )
    }

    fn family_name_or_default(&self, family_id: Uuid) -> String {
        self.tree
            .families
            .iter()
            .find(|family| family.id == family_id)
            .map(|family| family.name.clone())
            .unwrap_or_default()
    }

    fn clear_family_editor_selection(&mut self) {
        self.family_editor.selected_family = None;
        self.family_editor.new_family_name.clear();
        self.family_editor.family_member_pick = None;
    }
}
