use std::collections::HashMap;
use std::fs;

use eframe::egui;
use crate::tree::{FamilyTree, Gender, PersonId};
use crate::layout::LayoutEngine;
use uuid::Uuid;

// 定数
const DEFAULT_RELATION_KIND: &str = "biological";
const NODE_CORNER_RADIUS: f32 = 6.0;
const EDGE_STROKE_WIDTH: f32 = 1.5;
const SPOUSE_LINE_OFFSET: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Japanese,
    English,
}

struct Texts;

impl Texts {
    fn get(key: &str, lang: Language) -> String {
        match lang {
            Language::Japanese => Self::ja(key),
            Language::English => Self::en(key),
        }
    }
    
    fn ja(key: &str) -> String {
        match key {
            "title" => "家系図 (MVP)",
            "persons" => "👤 個人",
            "families" => "👪 家族",
            "settings" => "⚙ 設定",
            "file" => "ファイル:",
            "save" => "保存",
            "load" => "読込",
            "sample" => "サンプル",
            "add_new_person" => "➕ 新しい個人を追加",
            "person_editor" => "個人エディタ",
            "name" => "名前:",
            "gender" => "性別:",
            "male" => "男性",
            "female" => "女性",
            "unknown" => "不明",
            "birth" => "生年月日:",
            "deceased" => "故人",
            "death" => "没年月日:",
            "memo" => "メモ:",
            "update" => "更新",
            "cancel" => "キャンセル",
            "delete" => "削除",
            "relations" => "関係:",
            "father" => "父親:",
            "mother" => "母親:",
            "parent" => "親:",
            "spouses" => "配偶者:",
            "add_relations" => "関係を追加:",
            "add_parent" => "親を追加:",
            "add_child" => "子を追加:",
            "add_spouse" => "配偶者を追加:",
            "kind" => "種類:",
            "add" => "追加",
            "select" => "(選択)",
            "view_controls" => "操作: キャンバスをドラッグでパン、Ctrl+ホイールでズーム",
            "drag_nodes" => "ノードをドラッグして位置を調整",
            "manage_families" => "家族管理",
            "add_new_family" => "➕ 新しい家族を追加",
            "family_editor" => "家族エディタ",
            "color" => "色:",
            "members" => "メンバー",
            "no_members" => "(メンバーなし)",
            "no_family_selected" => "(家族が選択されていません)",
            "add_member" => "メンバーを追加:",
            "delete_family" => "家族を削除",
            "grid" => "グリッド:",
            "show_grid" => "グリッドを表示",
            "grid_size" => "グリッドサイズ:",
            "layout" => "レイアウト:",
            "reset_positions" => "すべての位置をリセット",
            "language" => "言語:",
            "japanese" => "日本語",
            "english" => "English",
            "new_person_added" => "新しい個人を追加しました",
            "person_updated" => "個人情報を更新しました",
            "name_required" => "名前は必須です",
            "person_deleted" => "個人を削除しました",
            "relation_removed" => "関係を削除しました",
            "parent_added" => "親を追加しました",
            "child_added" => "子を追加しました",
            "spouse_added" => "配偶者を追加しました",
            "new_family_added" => "新しい家族を追加しました",
            "member_removed" => "メンバーを削除しました",
            "member_added" => "メンバーを追加しました",
            "family_updated" => "家族情報を更新しました",
            "family_deleted" => "家族を削除しました",
            "positions_reset" => "すべての位置をリセットしました",
            "saved" => "保存しました",
            "loaded" => "読み込みました",
            "sample_added" => "サンプルデータを追加しました",
            "edit" => "編集:",
            "remove_relation" => "関係を削除",
            "selected_family" => "選択した家族:",
            "new_person" => "New Person",
            "new_family" => "New Family",
            _ => key,
        }.to_string()
    }
    
    fn en(key: &str) -> String {
        match key {
            "title" => "Family Tree (MVP)",
            "persons" => "👤 Persons",
            "families" => "👪 Families",
            "settings" => "⚙ Settings",
            "file" => "File:",
            "save" => "Save",
            "load" => "Load",
            "sample" => "Sample",
            "add_new_person" => "➕ Add New Person",
            "person_editor" => "Person Editor",
            "name" => "Name:",
            "gender" => "Gender:",
            "male" => "Male",
            "female" => "Female",
            "unknown" => "Unknown",
            "birth" => "Birth:",
            "deceased" => "Deceased",
            "death" => "Death:",
            "memo" => "Memo:",
            "update" => "Update",
            "cancel" => "Cancel",
            "delete" => "Delete",
            "relations" => "Relations:",
            "father" => "Father:",
            "mother" => "Mother:",
            "parent" => "Parent:",
            "spouses" => "Spouses:",
            "add_relations" => "Add Relations:",
            "add_parent" => "Add Parent:",
            "add_child" => "Add Child:",
            "add_spouse" => "Add Spouse:",
            "kind" => "Kind:",
            "add" => "Add",
            "select" => "(select)",
            "view_controls" => "View controls: Drag on canvas to pan, Ctrl+Wheel to zoom",
            "drag_nodes" => "Drag nodes to manually adjust positions",
            "manage_families" => "Manage Families",
            "add_new_family" => "➕ Add New Family",
            "family_editor" => "Family Editor",
            "color" => "Color:",
            "members" => "Members",
            "no_members" => "(No members)",
            "no_family_selected" => "(No family selected)",
            "add_member" => "Add member:",
            "delete_family" => "Delete Family",
            "grid" => "Grid:",
            "show_grid" => "Show Grid",
            "grid_size" => "Grid Size:",
            "layout" => "Layout:",
            "reset_positions" => "Reset All Positions",
            "language" => "Language:",
            "japanese" => "日本語",
            "english" => "English",
            "new_person_added" => "New person added",
            "person_updated" => "Person updated",
            "name_required" => "Name is required",
            "person_deleted" => "Person deleted",
            "relation_removed" => "Relation removed",
            "parent_added" => "Parent added",
            "child_added" => "Child added",
            "spouse_added" => "Spouse added",
            "new_family_added" => "New family added",
            "member_removed" => "Member removed",
            "member_added" => "Member added",
            "family_updated" => "Family updated",
            "family_deleted" => "Family deleted",
            "positions_reset" => "All positions reset",
            "saved" => "Saved",
            "loaded" => "Loaded",
            "sample_added" => "Added sample data",
            "edit" => "Edit:",
            "remove_relation" => "Remove relation",
            "selected_family" => "Selected family:",
            "new_person" => "New Person",
            "new_family" => "New Family",
            _ => key,
        }.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SideTab {
    Persons,
    Families,
    Settings,
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
    
    // サイドパネルタブ
    side_tab: SideTab,
    
    // 言語
    language: Language,
    
    // 家族管理
    selected_family: Option<Uuid>,
    new_family_name: String,
    new_family_color: [f32; 3],
    family_member_pick: Option<PersonId>,
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
            
            side_tab: SideTab::Persons,
            
            language: Language::Japanese,
            
            new_family_name: String::new(),
            new_family_color: [0.8, 0.8, 1.0],
            selected_family: None,
            family_member_pick: None,
        }
    }
}

impl App {
    fn save(&mut self) {
        let lang = self.language;
        let t = |key: &str| Texts::get(key, lang);
        match serde_json::to_string_pretty(&self.tree) {
            Ok(s) => match fs::write(&self.file_path, s) {
                Ok(_) => self.status = format!("{}: {}", t("saved"), self.file_path),
                Err(e) => self.status = format!("Save error: {e}"),
            },
            Err(e) => self.status = format!("Serialize error: {e}"),
        }
    }

    fn load(&mut self) {
        let lang = self.language;
        let t = |key: &str| Texts::get(key, lang);
        match fs::read_to_string(&self.file_path) {
            Ok(s) => match serde_json::from_str::<FamilyTree>(&s) {
                Ok(tree) => {
                    self.tree = tree;
                    self.selected = None;
                    self.status = format!("{}: {}", t("loaded"), self.file_path);
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

        let lang = self.language;
        let t = |key: &str| Texts::get(key, lang);
        self.status = t("sample_added");
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
                    let lang = self.language;
                    let t = |key: &str| Texts::get(key, lang);
                    if ui.small_button("❌").on_hover_text(t("remove_relation")).clicked() {
                        if is_parent {
                            self.tree.remove_parent_child(*id, current_id);
                        } else {
                            self.tree.remove_spouse(current_id, *id);
                        }
                        self.status = t("relation_removed");
                    }
                }
            });
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.language;
        let t = |key: &str| Texts::get(key, lang);
        
        egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(t("title"));
                
                // タブ選択
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.side_tab, SideTab::Persons, t("persons"));
                    ui.selectable_value(&mut self.side_tab, SideTab::Families, t("families"));
                    ui.selectable_value(&mut self.side_tab, SideTab::Settings, t("settings"));
                });
                ui.separator();

                match self.side_tab {
                    SideTab::Persons => {
                        // 個人管理タブ
                        ui.horizontal(|ui| {
                            ui.label(t("file"));
                            ui.text_edit_singleline(&mut self.file_path);
                        });
                        ui.horizontal(|ui| {
                            if ui.button(t("save")).clicked() {
                                self.save();
                            }
                            if ui.button(t("load")).clicked() {
                                self.load();
                            }
                            if ui.button(t("sample")).clicked() {
                                self.add_sample();
                            }
                        });
                        if !self.status.is_empty() {
                            ui.label(&self.status);
                        }

                        ui.separator();

                        // Add New Button
                        if ui.button(t("add_new_person")).clicked() {
                            // 空の新しい個人を作成
                            let id = self.tree.add_person(
                                t("new_person"),
                                Gender::Unknown,
                                None,
                                String::new(),
                                false,
                                None,
                            );
                            self.selected = Some(id);
                            // フォームに読み込む
                            if let Some(person) = self.tree.persons.get(&id) {
                                self.new_name = person.name.clone();
                                self.new_gender = person.gender;
                                self.new_birth = person.birth.clone().unwrap_or_default();
                                self.new_memo = person.memo.clone();
                                self.new_deceased = person.deceased;
                                self.new_death = person.death.clone().unwrap_or_default();
                            }
                            self.status = t("new_person_added");
                        }

                        ui.separator();
            
                        // Person Editor
                        if self.selected.is_some() {
                            if let Some(person) = self.selected.and_then(|id| self.tree.persons.get(&id)) {
                                ui.heading(format!("{} {}", t("edit"), person.name));
                            }
                        } else {
                            ui.heading(t("person_editor"));
                        }
            
                        ui.horizontal(|ui| {
                            ui.label(t("name"));
                            ui.text_edit_singleline(&mut self.new_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label(t("gender"));
                            ui.radio_value(&mut self.new_gender, Gender::Male, t("male"));
                            ui.radio_value(&mut self.new_gender, Gender::Female, t("female"));
                            ui.radio_value(&mut self.new_gender, Gender::Unknown, t("unknown"));
                        });
                        ui.horizontal(|ui| {
                            ui.label(t("birth"));
                            ui.text_edit_singleline(&mut self.new_birth);
                        });
                        ui.checkbox(&mut self.new_deceased, t("deceased"));
                        if self.new_deceased {
                            ui.horizontal(|ui| {
                                ui.label(t("death"));
                                ui.text_edit_singleline(&mut self.new_death);
                            });
                        }
                        ui.label(t("memo"));
                        ui.text_edit_multiline(&mut self.new_memo);

                        ui.horizontal(|ui| {
                            if self.selected.is_some() {
                                if ui.button(t("update")).clicked() {
                                    if let Some(sel) = self.selected {
                                        if let Some(p) = self.tree.persons.get_mut(&sel) {
                                            if !self.new_name.trim().is_empty() {
                                                p.name = self.new_name.trim().to_string();
                                                p.gender = self.new_gender;
                                                p.birth = Self::parse_optional_field(&self.new_birth);
                                                p.memo = self.new_memo.clone();
                                                p.deceased = self.new_deceased;
                                                p.death = self.new_deceased
                                                    .then(|| Self::parse_optional_field(&self.new_death))
                                                    .flatten();
                                                self.status = t("person_updated");
                                            } else {
                                                self.status = t("name_required");
                                            }
                                        }
                                    }
                                }
                                if ui.button(t("cancel")).clicked() {
                                    self.selected = None;
                                    self.clear_person_form();
                                }
                                if ui.button(t("delete")).clicked() {
                                    if let Some(sel) = self.selected {
                                        self.tree.remove_person(sel);
                                        self.selected = None;
                                        self.clear_person_form();
                                        self.status = t("person_deleted");
                                    }
                                }
                            }
                        });

                        // 関係管理（編集モードの場合のみ表示）
                        if let Some(sel) = self.selected {
                            ui.separator();
                            ui.label(t("relations"));
                            
                            let all_ids: Vec<PersonId> = self.tree.persons.keys().copied().collect();
                            
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
                            
                            self.show_relation_buttons(ui, &t("father"), &fathers, sel, true);
                            self.show_relation_buttons(ui, &t("mother"), &mothers, sel, true);
                            self.show_relation_buttons(ui, &t("parent"), &other_parents, sel, true);
                            
                            let spouses: Vec<_> = self.tree.spouses_of(sel)
                                .into_iter()
                                .filter_map(|id| self.tree.persons.get(&id).map(|p| (id, p.name.clone())))
                                .collect();
                            self.show_relation_buttons(ui, &t("spouses"), &spouses, sel, false);

                            ui.separator();
                            ui.label(t("add_relations"));
                            
                            ui.horizontal(|ui| {
                                ui.label(t("add_parent"));
                                egui::ComboBox::from_id_salt("add_parent")
                                    .selected_text(
                                        self.parent_pick
                                            .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                                            .unwrap_or_else(|| t("select")),
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
                                ui.label(t("kind"));
                                ui.text_edit_singleline(&mut self.relation_kind);
                                if ui.button(t("add")).clicked() {
                                    if let Some(parent) = self.parent_pick {
                                        let kind = if self.relation_kind.trim().is_empty() {
                                            DEFAULT_RELATION_KIND
                                        } else {
                                            self.relation_kind.trim()
                                        };
                                        self.tree.add_parent_child(parent, sel, kind.to_string());
                                        self.parent_pick = None;
                                        self.status = t("parent_added");
                                    }
                                }
                            });

                            ui.add_space(4.0);
                            
                            ui.horizontal(|ui| {
                                ui.label(t("add_child"));
                                egui::ComboBox::from_id_salt("add_child")
                                    .selected_text(
                                        self.child_pick
                                            .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                                            .unwrap_or_else(|| t("select")),
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
                                ui.label(t("kind"));
                                ui.text_edit_singleline(&mut self.relation_kind);
                                if ui.button(t("add")).clicked() {
                                    if let Some(child) = self.child_pick {
                                        let kind = if self.relation_kind.trim().is_empty() {
                                            DEFAULT_RELATION_KIND
                                        } else {
                                            self.relation_kind.trim()
                                        };
                                        self.tree.add_parent_child(sel, child, kind.to_string());
                                        self.child_pick = None;
                                        self.status = t("child_added");
                                    }
                                }
                            });

                            ui.add_space(4.0);
                            
                            ui.horizontal(|ui| {
                                ui.label(t("add_spouse"));
                                egui::ComboBox::from_id_salt("add_spouse")
                                    .selected_text(
                                        self.spouse1_pick
                                            .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.clone()))
                                            .unwrap_or_else(|| t("select")),
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
                                ui.label(t("memo"));
                                ui.text_edit_singleline(&mut self.spouse_memo);
                                if ui.button(t("add")).clicked() {
                                    if let Some(spouse) = self.spouse1_pick {
                                        self.tree.add_spouse(sel, spouse, self.spouse_memo.clone());
                                        self.spouse1_pick = None;
                                        self.spouse_memo.clear();
                                        self.status = t("spouse_added");
                                    }
                                }
                            });
                        }

                        ui.separator();
                        ui.label(t("view_controls"));
                        ui.label(t("drag_nodes"));
                    }

                    SideTab::Families => {
                        // 家族管理タブ
                        ui.heading(t("manage_families"));
                        
                        // Add New Familyボタン
                        if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(t("add_new_family"))).clicked() {
                            let color = (
                                (self.new_family_color[0] * 255.0) as u8,
                                (self.new_family_color[1] * 255.0) as u8,
                                (self.new_family_color[2] * 255.0) as u8,
                            );
                            let new_id = self.tree.add_family(t("new_family"), Some(color));
                            self.selected_family = Some(new_id);
                            self.new_family_name = t("new_family");
                            self.status = t("new_family_added");
                        }
                    
                        ui.separator();
                    
                        // 家族エディタ（統合）
                        if self.selected_family.is_some() {
                            if let Some(family) = self.selected_family.and_then(|id| self.tree.families.iter().find(|f| f.id == id)) {
                                ui.heading(format!("{} {}", t("edit"), family.name));
                            }
                        } else {
                            ui.heading(t("family_editor"));
                        }
                        
                        ui.horizontal(|ui| {
                            ui.label(t("name"));
                            ui.text_edit_singleline(&mut self.new_family_name);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label(t("color"));
                            ui.color_edit_button_rgb(&mut self.new_family_color);
                        });
                        
                        ui.separator();
                        ui.heading(t("members"));
                    
                        // メンバーリスト
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            if let Some(family_id) = self.selected_family {
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
                                                        self.status = t("member_removed");
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
                        if self.selected_family.is_some() {
                            ui.horizontal(|ui| {
                                ui.label(t("add_member"));
                                egui::ComboBox::from_id_salt("family_member_pick")
                                    .selected_text(
                                        self.family_member_pick
                                            .and_then(|id| self.tree.persons.get(&id).map(|p| p.name.as_str()))
                                            .unwrap_or(&t("select"))
                                    )
                                    .show_ui(ui, |ui| {
                                        if let Some(family_id) = self.selected_family {
                                            if let Some(family) = self.tree.families.iter().find(|f| f.id == family_id) {
                                                for (id, person) in &self.tree.persons {
                                                    if !family.members.contains(id) {
                                                        ui.selectable_value(&mut self.family_member_pick, Some(*id), &person.name);
                                                    }
                                                }
                                            }
                                        }
                                    });
                                    
                                if let Some(pid) = self.family_member_pick {
                                    if ui.small_button(t("add")).clicked() {
                                        if let Some(family_id) = self.selected_family {
                                            self.tree.add_member_to_family(family_id, pid);
                                            self.family_member_pick = None;
                                            self.status = t("member_added");
                                        }
                                    }
                                }
                            });
                        }

                        ui.separator();

                        // アクションボタン（選択時のみ表示）
                        if let Some(family_id) = self.selected_family {
                            ui.horizontal(|ui| {
                                if ui.button(t("update")).clicked() && !self.new_family_name.trim().is_empty() {
                                    if let Some(family) = self.tree.families.iter_mut().find(|f| f.id == family_id) {
                                        family.name = self.new_family_name.clone();
                                        family.color = Some((
                                            (self.new_family_color[0] * 255.0) as u8,
                                            (self.new_family_color[1] * 255.0) as u8,
                                            (self.new_family_color[2] * 255.0) as u8,
                                        ));
                                        self.status = t("family_updated");
                                    }
                                }
                                
                                if ui.button(t("cancel")).clicked() {
                                    self.selected_family = None;
                                    self.new_family_name.clear();
                                    self.family_member_pick = None;
                                }
                                
                                if ui.button(t("delete_family")).clicked() {
                                    self.tree.remove_family(family_id);
                                    self.selected_family = None;
                                    self.new_family_name.clear();
                                    self.family_member_pick = None;
                                    self.status = t("family_deleted");
                                }
                            });
                        }
                    }

                    SideTab::Settings => {
                        // 設定タブ
                        ui.heading(t("settings"));
                        ui.separator();
                        
                        ui.label(t("language"));
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.language, Language::Japanese, t("japanese"));
                            ui.radio_value(&mut self.language, Language::English, t("english"));
                        });
                        
                        ui.separator();
                        ui.label(t("grid"));
                        ui.checkbox(&mut self.show_grid, t("show_grid"));
                        ui.horizontal(|ui| {
                            ui.label(t("grid_size"));
                            ui.add(egui::DragValue::new(&mut self.grid_size)
                                .speed(1.0)
                                .range(10.0..=200.0));
                        });
                        
                        ui.separator();
                        ui.label(t("layout"));
                        if ui.button(t("reset_positions")).clicked() {
                            for person in self.tree.persons.values_mut() {
                                person.position = None;
                            }
                            self.status = t("positions_reset");
                        }
                    }
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
                        // 選択した人物の情報をフォームに読み込む
                        if let Some(person) = self.tree.persons.get(&n.id) {
                            self.new_name = person.name.clone();
                            self.new_gender = person.gender;
                            self.new_birth = person.birth.clone().unwrap_or_default();
                            self.new_memo = person.memo.clone();
                            self.new_deceased = person.deceased;
                            self.new_death = person.death.clone().unwrap_or_default();
                        }
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

            // 家族の枠を描画
            for family in &self.tree.families {
                if family.members.len() < 2 {
                    continue; // メンバーが1人以下の場合は枠を描画しない
                }
                
                // メンバー全員を囲む矩形を計算
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                
                for member_id in &family.members {
                    if let Some(rect) = screen_rects.get(member_id) {
                        min_x = min_x.min(rect.min.x);
                        min_y = min_y.min(rect.min.y);
                        max_x = max_x.max(rect.max.x);
                        max_y = max_y.max(rect.max.y);
                    }
                }
                
                if min_x < f32::MAX {
                    let padding = 20.0;
                    let family_rect = egui::Rect::from_min_max(
                        egui::pos2(min_x - padding, min_y - padding),
                        egui::pos2(max_x + padding, max_y + padding)
                    );
                    
                    let color = if let Some((r, g, b)) = family.color {
                        egui::Color32::from_rgba_unmultiplied(r, g, b, 30)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(200, 200, 255, 30)
                    };
                    
                    let stroke_color = if let Some((r, g, b)) = family.color {
                        egui::Color32::from_rgb(r, g, b)
                    } else {
                        egui::Color32::from_rgb(100, 100, 200)
                    };
                    
                    painter.rect_filled(family_rect, 8.0, color);
                    painter.rect_stroke(
                        family_rect,
                        8.0,
                        egui::Stroke::new(2.0, stroke_color),
                        egui::epaint::StrokeKind::Outside
                    );
                    
                    // 家族名をラベル表示（クリック可能）
                    let label_pos = family_rect.left_top() + egui::vec2(10.0, 5.0);
                    let label_size = egui::vec2(family_rect.width() * 0.5, 20.0);
                    let label_rect = egui::Rect::from_min_size(label_pos, label_size);
                    
                    // クリック検出とホバー効果
                    let resp = ui.interact(label_rect, egui::Id::new(("family_label", family.id)), egui::Sense::click());
                    
                    // 背景とボーダーを描画（ボタンのように見せる）
                    let bg_color = if resp.is_pointer_button_down_on() {
                        // クリック中
                        egui::Color32::from_rgba_unmultiplied(
                            stroke_color.r(), 
                            stroke_color.g(), 
                            stroke_color.b(), 
                            100
                        )
                    } else if resp.hovered() {
                        // ホバー中
                        egui::Color32::from_rgba_unmultiplied(
                            stroke_color.r(), 
                            stroke_color.g(), 
                            stroke_color.b(), 
                            60
                        )
                    } else {
                        // 通常
                        egui::Color32::from_rgba_unmultiplied(
                            stroke_color.r(), 
                            stroke_color.g(), 
                            stroke_color.b(), 
                            30
                        )
                    };
                    
                    painter.rect_filled(label_rect, 3.0, bg_color);
                    
                    // ホバー時にボーダーを追加
                    if resp.hovered() || resp.is_pointer_button_down_on() {
                        painter.rect_stroke(
                            label_rect,
                            3.0,
                            egui::Stroke::new(1.5, stroke_color),
                            egui::epaint::StrokeKind::Outside
                        );
                    }
                    
                    // テキストを描画
                    let text_color = if resp.hovered() || resp.is_pointer_button_down_on() {
                        stroke_color // ホバー時は濃い色
                    } else {
                        egui::Color32::from_rgb(
                            (stroke_color.r() as f32 * 0.8) as u8,
                            (stroke_color.g() as f32 * 0.8) as u8,
                            (stroke_color.b() as f32 * 0.8) as u8,
                        )
                    };
                    
                    painter.text(
                        label_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &family.name,
                        egui::FontId::proportional(11.0 * self.zoom.clamp(0.7, 1.2)),
                        text_color,
                    );
                    
                    // クリック処理
                    if resp.clicked() {
                        self.selected_family = Some(family.id);
                        self.new_family_name = family.name.clone();
                        if let Some((r, g, b)) = family.color {
                            self.new_family_color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                        }
                        self.side_tab = SideTab::Families;
                        let lang = self.language;
                        let t = |key: &str| Texts::get(key, lang);
                        self.status = format!("{} {}", t("selected_family"), family.name);
                    }
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
