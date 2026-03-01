use eframe::egui;
use crate::app::App;
use crate::core::tree::EventRelationType;
use crate::ui::LogLevel;

pub trait EventsTabRenderer {
    fn render_events_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl EventsTabRenderer for App {
    fn render_events_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        self.render_events_tab_header(ui, &t);
        self.render_events_tab_editor_section(ui, &t);

        if let Some(event_id) = self.event_editor.selected {
            self.render_events_tab_relations_section(ui, event_id, &t);
        }

        self.render_events_tab_actions_section(ui, &t);
        self.render_events_tab_footer(ui, &t);
    }
}

impl App {
    fn render_events_tab_header(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.heading(t("manage_events"));
        if ui.button(t("add_new_event")).clicked() {
            self.clear_event_editor_selection();
        }
        ui.separator();
    }

    fn render_events_tab_editor_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.heading(t("event_editor"));
        self.render_event_form_fields(ui, t);
    }

    fn render_events_tab_relations_section(
        &mut self,
        ui: &mut egui::Ui,
        event_id: crate::core::tree::EventId,
        t: &impl Fn(&str) -> String,
    ) {
        self.render_event_relations_section(ui, event_id, t);
    }

    fn render_events_tab_actions_section(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        self.render_event_action_buttons(ui, t);
    }

    fn render_events_tab_footer(&self, _ui: &mut egui::Ui, _t: &impl Fn(&str) -> String) {
    }

    fn render_event_form_fields(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.label(t("name"));
        ui.text_edit_singleline(&mut self.event_editor.new_event_name);

        ui.label(t("date"));
        ui.text_edit_singleline(&mut self.event_editor.new_event_date);

        ui.label(t("description"));
        ui.text_edit_multiline(&mut self.event_editor.new_event_description);

        ui.label(t("color"));
        ui.color_edit_button_rgb(&mut self.event_editor.new_event_color);
    }

    fn render_event_action_buttons(&mut self, ui: &mut egui::Ui, t: &impl Fn(&str) -> String) {
        ui.horizontal(|ui| {
            if self.event_editor.selected.is_none() {
                if ui.button(t("add")).clicked() {
                    self.add_event_from_editor(t);
                }
            } else {
                if ui.button(t("update")).clicked() {
                    self.update_selected_event(t);
                }
                if ui.button(t("delete")).clicked() {
                    self.delete_selected_event(t);
                }
            }

            if ui.button(t("cancel")).clicked() {
                self.clear_event_editor_selection();
            }
        });
    }

    fn add_event_from_editor(&mut self, t: &impl Fn(&str) -> String) {
        let visible_left_top = self.visible_canvas_left_top();
        let event_name = self.event_editor.new_event_name.clone();
        let event_date = App::parse_optional_field(&self.event_editor.new_event_date);
        let event_description = self.event_editor.new_event_description.clone();
        let event_color = self.event_editor_color_rgb();

        let event_id = self.tree.add_event(
            event_name.clone(),
            event_date,
            event_description,
            visible_left_top,
            event_color,
        );
        self.event_editor.selected = Some(event_id);
        self.file.status = t("new_event_added");
        self.log.add(format!(
            "{}: {}",
            t("log_event_added"),
            if event_name.is_empty() {
                t("new_event")
            } else {
                event_name
            }
        ), LogLevel::Debug);
    }

    fn update_selected_event(&mut self, t: &impl Fn(&str) -> String) {
        let Some(event_id) = self.event_editor.selected else {
            return;
        };

        let event_color = self.event_editor_color_rgb();
        if let Some(event) = self.tree.events.get_mut(&event_id) {
            let old_name = event.name.clone();
            event.name = self.event_editor.new_event_name.clone();
            event.date = App::parse_optional_field(&self.event_editor.new_event_date);
            event.description = self.event_editor.new_event_description.clone();
            event.color = event_color;
            self.file.status = t("event_updated");
            self.log.add(format!(
                "{}: {} {} {}",
                t("log_event_updated"),
                old_name,
                t("log_to"),
                event.name
            ), LogLevel::Debug);
        }
    }

    fn delete_selected_event(&mut self, t: &impl Fn(&str) -> String) {
        let Some(event_id) = self.event_editor.selected else {
            return;
        };

        let event_name = self.event_name_or_unknown(event_id, t);
        self.tree.remove_event(event_id);
        self.clear_event_editor_selection();
        self.file.status = t("event_deleted");
        self.log
            .add(
                format!("{}: {}", t("log_event_deleted"), event_name),
                LogLevel::Debug,
            );
    }

    fn render_event_relations_section(
        &mut self,
        ui: &mut egui::Ui,
        event_id: crate::core::tree::EventId,
        t: &impl Fn(&str) -> String,
    ) {
        ui.separator();
        ui.heading(t("event_relations"));
        self.render_existing_event_relations(ui, event_id, t);
        ui.separator();
        self.render_add_event_relation_section(ui, event_id, t);
    }

    fn render_existing_event_relations(
        &mut self,
        ui: &mut egui::Ui,
        event_id: crate::core::tree::EventId,
        t: &impl Fn(&str) -> String,
    ) {
        let relations: Vec<_> = self
            .tree
            .event_relations_of(event_id)
            .into_iter()
            .map(|relation| (relation.person, relation.relation_type, relation.memo.clone()))
            .collect();

        for (person_id, relation_type, memo) in relations {
            let person_name = self.get_person_name(&person_id);
            let relation_type_str = Self::event_relation_type_label(relation_type, t);

            ui.horizontal(|ui| {
                ui.label(format!("→ {} ({})", person_name, relation_type_str));
                if !memo.is_empty() {
                    ui.label(format!("[{}]", memo));
                }
                if ui.small_button(t("remove_relation")).clicked() {
                    self.remove_event_relation_and_log(event_id, person_id, &person_name, t);
                }
            });
        }
    }

    fn render_add_event_relation_section(
        &mut self,
        ui: &mut egui::Ui,
        event_id: crate::core::tree::EventId,
        t: &impl Fn(&str) -> String,
    ) {
        ui.label(t("add_person_to_event"));

        egui::ComboBox::from_id_salt("event_person_pick")
            .selected_text(
                self.event_editor
                    .person_pick
                    .map(|person_id| self.get_person_name(&person_id))
                    .unwrap_or_else(|| t("select")),
            )
            .show_ui(ui, |ui| {
                for person_id in self.tree.persons.keys() {
                    let person_name = self.get_person_name(person_id);
                    ui.selectable_value(
                        &mut self.event_editor.person_pick,
                        Some(*person_id),
                        person_name,
                    );
                }
            });

        ui.label(t("relation_type"));
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.event_editor.relation_type, EventRelationType::Line, t("line"));
            ui.radio_value(
                &mut self.event_editor.relation_type,
                EventRelationType::ArrowToPerson,
                t("arrow_to_person"),
            );
            ui.radio_value(
                &mut self.event_editor.relation_type,
                EventRelationType::ArrowToEvent,
                t("arrow_to_event"),
            );
        });

        ui.label(t("memo"));
        ui.text_edit_singleline(&mut self.event_editor.relation_memo);

        if ui.button(t("add")).clicked() {
            if let Some(person_id) = self.event_editor.person_pick {
                self.add_event_relation_from_editor(event_id, person_id, t);
            }
        }
    }

    fn event_editor_color_rgb(&self) -> (u8, u8, u8) {
        (
            (self.event_editor.new_event_color[0] * 255.0) as u8,
            (self.event_editor.new_event_color[1] * 255.0) as u8,
            (self.event_editor.new_event_color[2] * 255.0) as u8,
        )
    }

    fn clear_event_editor_selection(&mut self) {
        self.event_editor.selected = None;
        self.event_editor.clear();
    }

    fn event_name_or_unknown(&self, event_id: crate::core::tree::EventId, t: &impl Fn(&str) -> String) -> String {
        self.tree
            .events
            .get(&event_id)
            .map(|event| event.name.clone())
            .unwrap_or_else(|| t("unknown"))
    }

    fn event_relation_type_label(relation_type: EventRelationType, t: &impl Fn(&str) -> String) -> String {
        match relation_type {
            EventRelationType::Line => t("line"),
            EventRelationType::ArrowToPerson => t("arrow_to_person"),
            EventRelationType::ArrowToEvent => t("arrow_to_event"),
        }
    }

    fn remove_event_relation_and_log(
        &mut self,
        event_id: crate::core::tree::EventId,
        person_id: crate::core::tree::PersonId,
        person_name: &str,
        t: &impl Fn(&str) -> String,
    ) {
        let event_name = self.event_name_or_unknown(event_id, t);
        self.tree.remove_event_relation(event_id, person_id);
        self.file.status = t("relation_removed");
        self.log.add(format!(
            "{}: {} <-> {}",
            t("log_event_relation_removed"),
            event_name,
            person_name
        ), LogLevel::Debug);
    }

    fn add_event_relation_from_editor(
        &mut self,
        event_id: crate::core::tree::EventId,
        person_id: crate::core::tree::PersonId,
        t: &impl Fn(&str) -> String,
    ) {
        let event_name = self.event_name_or_unknown(event_id, t);
        let person_name = self.get_person_name(&person_id);
        self.tree.add_event_relation(
            event_id,
            person_id,
            self.event_editor.relation_type,
            self.event_editor.relation_memo.clone(),
        );
        self.event_editor.person_pick = None;
        self.event_editor.relation_memo.clear();
        self.file.status = t("relation_added");
        self.log.add(format!(
            "{}: {} <-> {}",
            t("log_event_relation_added"),
            event_name,
            person_name
        ), LogLevel::Debug);
    }
}
