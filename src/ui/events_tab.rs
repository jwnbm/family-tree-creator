use eframe::egui;
use crate::app::App;
use crate::core::tree::EventRelationType;

pub trait EventsTabRenderer {
    fn render_events_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl EventsTabRenderer for App {
    fn render_events_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        ui.heading(t("manage_events"));
        
        // 新しいイベント追加
        if ui.button(t("add_new_event")).clicked() {
            self.event_editor.clear();
            self.event_editor.selected = None;
        }
        
        ui.separator();
        
        // イベントエディタ
        ui.heading(t("event_editor"));
        
        ui.label(t("name"));
        ui.text_edit_singleline(&mut self.event_editor.new_event_name);
        
        ui.label(t("date"));
        ui.text_edit_singleline(&mut self.event_editor.new_event_date);
        
        ui.label(t("description"));
        ui.text_edit_multiline(&mut self.event_editor.new_event_description);
        
        ui.label(t("color"));
        ui.color_edit_button_rgb(&mut self.event_editor.new_event_color);
        
        ui.horizontal(|ui| {
            // 追加または更新
            if self.event_editor.selected.is_none() {
                if ui.button(t("add")).clicked() {
                    // 現在表示されているキャンバスの左上を計算
                    let visible_left_top = if self.canvas.canvas_rect != egui::Rect::NOTHING {
                        let screen_pos = self.canvas.canvas_rect.left_top() + egui::vec2(50.0, 50.0);
                        let world_pos = self.canvas.canvas_origin + (screen_pos - self.canvas.canvas_origin - self.canvas.pan) / self.canvas.zoom;
                        (world_pos.x, world_pos.y)
                    } else {
                        (100.0, 100.0)
                    };
                    
                    let name = self.event_editor.new_event_name.clone();
                    let date = App::parse_optional_field(&self.event_editor.new_event_date);
                    let description = self.event_editor.new_event_description.clone();
                    let color = (
                        (self.event_editor.new_event_color[0] * 255.0) as u8,
                        (self.event_editor.new_event_color[1] * 255.0) as u8,
                        (self.event_editor.new_event_color[2] * 255.0) as u8,
                    );
                    
                    let event_id = self.tree.add_event(name.clone(), date, description, visible_left_top, color);
                    self.event_editor.selected = Some(event_id);
                    self.file.status = t("new_event_added");
                    self.log.add(format!("新しいイベントを追加しました: {}", if name.is_empty() { t("new_event") } else { name }));
                }
            } else {
                if ui.button(t("update")).clicked() {
                    if let Some(event_id) = self.event_editor.selected {
                        if let Some(event) = self.tree.events.get_mut(&event_id) {
                            let old_name = event.name.clone();
                            event.name = self.event_editor.new_event_name.clone();
                            event.date = App::parse_optional_field(&self.event_editor.new_event_date);
                            event.description = self.event_editor.new_event_description.clone();
                            event.color = (
                                (self.event_editor.new_event_color[0] * 255.0) as u8,
                                (self.event_editor.new_event_color[1] * 255.0) as u8,
                                (self.event_editor.new_event_color[2] * 255.0) as u8,
                            );
                            self.file.status = t("event_updated");
                            self.log.add(format!("イベントを更新しました: {} -> {}", old_name, event.name));
                        }
                    }
                }
                
                if ui.button(t("delete")).clicked() {
                    if let Some(event_id) = self.event_editor.selected {
                        let event_name = self.tree.events.get(&event_id)
                            .map(|e| e.name.clone())
                            .unwrap_or_else(|| t("unknown"));
                        self.tree.remove_event(event_id);
                        self.event_editor.selected = None;
                        self.event_editor.clear();
                        self.file.status = t("event_deleted");
                        self.log.add(format!("イベントを削除しました: {}", event_name));
                    }
                }
            }
            
            if ui.button(t("cancel")).clicked() {
                self.event_editor.selected = None;
                self.event_editor.clear();
            }
        });
        
        // イベントと人物の関係
        if let Some(event_id) = self.event_editor.selected {
            ui.separator();
            ui.heading(t("event_relations"));
            
            // 既存の関係を表示
            let relations: Vec<_> = self.tree.event_relations_of(event_id)
                .into_iter()
                .map(|r| (r.person, r.relation_type, r.memo.clone()))
                .collect();
            
            for (person_id, relation_type, memo) in relations {
                let person_name = self.get_person_name(&person_id);
                let relation_type_str = match relation_type {
                    EventRelationType::Line => t("line"),
                    EventRelationType::ArrowToPerson => t("arrow_to_person"),
                    EventRelationType::ArrowToEvent => t("arrow_to_event"),
                };
                
                ui.horizontal(|ui| {
                    ui.label(format!("→ {} ({})", person_name, relation_type_str));
                    if !memo.is_empty() {
                        ui.label(format!("[{}]", memo));
                    }
                    if ui.small_button(t("remove_relation")).clicked() {
                        let event_name = self.tree.events.get(&event_id)
                            .map(|e| e.name.clone())
                            .unwrap_or_else(|| t("unknown"));
                        self.tree.remove_event_relation(event_id, person_id);
                        self.file.status = t("relation_removed");
                        self.log.add(format!("イベント関係を削除しました: {} <-> {}", event_name, person_name));
                    }
                });
            }
            
            ui.separator();
            
            // 新しい関係を追加
            ui.label(t("add_person_to_event"));
            
            egui::ComboBox::from_id_salt("event_person_pick")
                .selected_text(
                    self.event_editor.person_pick
                        .map(|pid| self.get_person_name(&pid))
                        .unwrap_or_else(|| t("select"))
                )
                .show_ui(ui, |ui| {
                    for pid in self.tree.persons.keys() {
                        let name = self.get_person_name(pid);
                        ui.selectable_value(&mut self.event_editor.person_pick, Some(*pid), name);
                    }
                });
            
            ui.label(t("relation_type"));
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.event_editor.relation_type, EventRelationType::Line, t("line"));
                ui.radio_value(&mut self.event_editor.relation_type, EventRelationType::ArrowToPerson, t("arrow_to_person"));
                ui.radio_value(&mut self.event_editor.relation_type, EventRelationType::ArrowToEvent, t("arrow_to_event"));
            });
            
            ui.label(t("memo"));
            ui.text_edit_singleline(&mut self.event_editor.relation_memo);
            
            if ui.button(t("add")).clicked() {
                if let Some(person_id) = self.event_editor.person_pick {
                    let event_name = self.tree.events.get(&event_id)
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| t("unknown"));
                    let person_name = self.get_person_name(&person_id);
                    self.tree.add_event_relation(
                        event_id,
                        person_id,
                        self.event_editor.relation_type,
                        self.event_editor.relation_memo.clone()
                    );
                    self.event_editor.person_pick = None;
                    self.event_editor.relation_memo.clear();
                    self.file.status = t("relation_added");
                    self.log.add(format!("イベント関係を追加しました: {} <-> {}", event_name, person_name));
                }
            }
        }
    }
}
