use std::collections::HashMap;
use std::fs;

use eframe::egui;
use crate::core::tree::{FamilyTree, Gender, PersonId};
use crate::core::layout::LayoutEngine;
use crate::core::i18n::{Language, Texts};
use uuid::Uuid;

// 定数
const DEFAULT_RELATION_KIND: &str = "biological";
const NODE_CORNER_RADIUS: f32 = 6.0;
const EDGE_STROKE_WIDTH: f32 = 1.5;
const SPOUSE_LINE_OFFSET: f32 = 2.0;

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
    
    // 配偶者メモ編集
    editing_spouse_memo: Option<(PersonId, PersonId)>,
    edit_spouse_memo_text: String,
    
    // 親子関係の種類編集
    editing_parent_kind: Option<(PersonId, PersonId)>, // (parent, child)
    edit_parent_kind_text: String,

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
    
    // キャンバス情報
    canvas_rect: egui::Rect,
    canvas_origin: egui::Pos2,
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
            
            editing_spouse_memo: None,
            edit_spouse_memo_text: String::new(),
            
            editing_parent_kind: None,
            edit_parent_kind_text: String::new(),

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
            
            canvas_rect: egui::Rect::NOTHING,
            canvas_origin: egui::Pos2::ZERO,
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
        // 第1世代（祖父母）: 横並び
        let grandpa = self.tree.add_person("Grandpa".into(), Gender::Male, Some("1940-01-01".into()), "".into(), true, Some("2020-01-01".into()), (0.0, 0.0));
        let grandma = self.tree.add_person("Grandma".into(), Gender::Female, Some("1942-02-02".into()), "".into(), true, Some("2022-05-15".into()), (230.0, 0.0));
        
        // 第2世代（両親と叔父叔母）: 横並び、第1世代の下
        let uncle = self.tree.add_person("Uncle".into(), Gender::Male, Some("1965-06-15".into()), "".into(), false, None, (-230.0, 110.0));
        let aunt = self.tree.add_person("Aunt".into(), Gender::Female, Some("1967-08-20".into()), "".into(), false, None, (-460.0, 110.0));
        let father = self.tree.add_person("Father".into(), Gender::Male, Some("1968-03-03".into()), "".into(), false, None, (0.0, 110.0));
        let mother = self.tree.add_person("Mother".into(), Gender::Female, Some("1970-04-04".into()), "".into(), false, None, (230.0, 110.0));
        
        // 第3世代（兄弟と自分、従兄弟）: 第2世代の下
        let cousin = self.tree.add_person("Cousin".into(), Gender::Male, Some("1992-11-11".into()), "".into(), false, None, (-345.0, 220.0));
        let brother = self.tree.add_person("Brother".into(), Gender::Male, Some("1993-07-10".into()), "".into(), false, None, (0.0, 220.0));
        let me = self.tree.add_person("Me".into(), Gender::Unknown, Some("1995-05-05".into()), "Hello".into(), false, None, (115.0, 220.0));
        let sister = self.tree.add_person("Sister".into(), Gender::Female, Some("1998-09-20".into()), "".into(), false, None, (230.0, 220.0));
        
        // 第4世代（自分の子供）: 第3世代の下
        let my_spouse = self.tree.add_person("My Spouse".into(), Gender::Female, Some("1996-03-15".into()), "".into(), false, None, (345.0, 220.0));
        let my_son = self.tree.add_person("My Son".into(), Gender::Male, Some("2020-01-10".into()), "".into(), false, None, (172.5, 330.0));
        let my_daughter = self.tree.add_person("My Daughter".into(), Gender::Female, Some("2022-06-25".into()), "".into(), false, None, (287.5, 330.0));

        // 祖父母の関係
        self.tree.add_spouse(grandpa, grandma, "1965".into());
        
        // 叔父叔母の関係
        self.tree.add_parent_child(grandpa, uncle, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(grandma, uncle, DEFAULT_RELATION_KIND.into());
        self.tree.add_spouse(uncle, aunt, "1990".into());
        self.tree.add_parent_child(uncle, cousin, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(aunt, cousin, DEFAULT_RELATION_KIND.into());
        
        // 両親の関係
        self.tree.add_parent_child(grandpa, father, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(grandma, father, DEFAULT_RELATION_KIND.into());
        self.tree.add_spouse(father, mother, "1992".into());
        
        // 兄弟の関係
        self.tree.add_parent_child(father, brother, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(mother, brother, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(father, me, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(mother, me, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(father, sister, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(mother, sister, DEFAULT_RELATION_KIND.into());
        
        // 自分の配偶者と子供の関係
        self.tree.add_spouse(me, my_spouse, "2018".into());
        self.tree.add_parent_child(me, my_son, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(my_spouse, my_son, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(me, my_daughter, DEFAULT_RELATION_KIND.into());
        self.tree.add_parent_child(my_spouse, my_daughter, DEFAULT_RELATION_KIND.into());

        // グループを作成
        let grandparents_group = self.tree.add_family("Grandparents".into(), Some((200, 150, 100)));
        let uncle_family_group = self.tree.add_family("Uncle's Family".into(), Some((150, 200, 150)));
        let parents_group = self.tree.add_family("Parents".into(), Some((100, 150, 200)));
        let my_family_group = self.tree.add_family("My Family".into(), Some((200, 100, 150)));
        
        // グループにメンバーを追加
        self.tree.add_member_to_family(grandparents_group, grandpa);
        self.tree.add_member_to_family(grandparents_group, grandma);
        
        self.tree.add_member_to_family(uncle_family_group, uncle);
        self.tree.add_member_to_family(uncle_family_group, aunt);
        self.tree.add_member_to_family(uncle_family_group, cousin);
        
        self.tree.add_member_to_family(parents_group, father);
        self.tree.add_member_to_family(parents_group, mother);
        self.tree.add_member_to_family(parents_group, brother);
        self.tree.add_member_to_family(parents_group, sister);
        
        self.tree.add_member_to_family(my_family_group, me);
        self.tree.add_member_to_family(my_family_group, my_spouse);
        self.tree.add_member_to_family(my_family_group, my_son);
        self.tree.add_member_to_family(my_family_group, my_daughter);

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
}

impl App {
    /// 個人管理タブのUI
    fn render_persons_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
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
            // 現在表示されているキャンバスの左上を計算
            let visible_left_top = if self.canvas_rect != egui::Rect::NOTHING {
                let screen_pos = self.canvas_rect.left_top() + egui::vec2(50.0, 50.0);
                let world_pos = self.canvas_origin + (screen_pos - self.canvas_origin - self.pan) / self.zoom;
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
            self.selected = Some(id);
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
            
            // 父親の表示（種類編集機能付き）
            if !fathers.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(&t("father"));
                });
                for (parent_id, parent_name) in &fathers {
                    // 関係の種類を取得
                    let kind = self.tree.edges.iter()
                        .find(|e| e.parent == *parent_id && e.child == sel)
                        .map(|e| e.kind.clone())
                        .unwrap_or_default();
                    
                    ui.horizontal(|ui| {
                        if ui.small_button(parent_name).clicked() {
                            self.selected = Some(*parent_id);
                        }
                        
                        // 種類の表示
                        if !kind.is_empty() && kind != "biological" {
                            ui.label(format!("({})", kind));
                        }
                        
                        // 編集ボタン
                        if ui.small_button("✏️").on_hover_text(&t("edit_kind")).clicked() {
                            self.editing_parent_kind = Some((*parent_id, sel));
                            self.edit_parent_kind_text = kind.clone();
                        }
                        
                        // 削除ボタン
                        if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                            self.tree.remove_parent_child(*parent_id, sel);
                            self.status = t("relation_removed");
                        }
                    });
                    
                    // 種類編集UI
                    if self.editing_parent_kind == Some((*parent_id, sel)) {
                        ui.horizontal(|ui| {
                            ui.label(&t("kind"));
                            ui.text_edit_singleline(&mut self.edit_parent_kind_text);
                            if ui.button(&t("save")).clicked() {
                                // 親子関係の種類を更新
                                if let Some(edge) = self.tree.edges.iter_mut().find(|e| {
                                    e.parent == *parent_id && e.child == sel
                                }) {
                                    edge.kind = if self.edit_parent_kind_text.trim().is_empty() {
                                        "biological".to_string()
                                    } else {
                                        self.edit_parent_kind_text.trim().to_string()
                                    };
                                    self.status = t("relation_kind_updated");
                                }
                                self.editing_parent_kind = None;
                                self.edit_parent_kind_text.clear();
                            }
                            if ui.button(&t("cancel")).clicked() {
                                self.editing_parent_kind = None;
                                self.edit_parent_kind_text.clear();
                            }
                        });
                    }
                }
            }
            
            // 母親の表示（種類編集機能付き）
            if !mothers.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(&t("mother"));
                });
                for (parent_id, parent_name) in &mothers {
                    // 関係の種類を取得
                    let kind = self.tree.edges.iter()
                        .find(|e| e.parent == *parent_id && e.child == sel)
                        .map(|e| e.kind.clone())
                        .unwrap_or_default();
                    
                    ui.horizontal(|ui| {
                        if ui.small_button(parent_name).clicked() {
                            self.selected = Some(*parent_id);
                        }
                        
                        // 種類の表示
                        if !kind.is_empty() && kind != "biological" {
                            ui.label(format!("({})", kind));
                        }
                        
                        // 編集ボタン
                        if ui.small_button("✏️").on_hover_text(&t("edit_kind")).clicked() {
                            self.editing_parent_kind = Some((*parent_id, sel));
                            self.edit_parent_kind_text = kind.clone();
                        }
                        
                        // 削除ボタン
                        if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                            self.tree.remove_parent_child(*parent_id, sel);
                            self.status = t("relation_removed");
                        }
                    });
                    
                    // 種類編集UI
                    if self.editing_parent_kind == Some((*parent_id, sel)) {
                        ui.horizontal(|ui| {
                            ui.label(&t("kind"));
                            ui.text_edit_singleline(&mut self.edit_parent_kind_text);
                            if ui.button(&t("save")).clicked() {
                                // 親子関係の種類を更新
                                if let Some(edge) = self.tree.edges.iter_mut().find(|e| {
                                    e.parent == *parent_id && e.child == sel
                                }) {
                                    edge.kind = if self.edit_parent_kind_text.trim().is_empty() {
                                        "biological".to_string()
                                    } else {
                                        self.edit_parent_kind_text.trim().to_string()
                                    };
                                    self.status = t("relation_kind_updated");
                                }
                                self.editing_parent_kind = None;
                                self.edit_parent_kind_text.clear();
                            }
                            if ui.button(&t("cancel")).clicked() {
                                self.editing_parent_kind = None;
                                self.edit_parent_kind_text.clear();
                            }
                        });
                    }
                }
            }
            
            // その他の親の表示（種類編集機能付き）
            if !other_parents.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(&t("parent"));
                });
                for (parent_id, parent_name) in &other_parents {
                    // 関係の種類を取得
                    let kind = self.tree.edges.iter()
                        .find(|e| e.parent == *parent_id && e.child == sel)
                        .map(|e| e.kind.clone())
                        .unwrap_or_default();
                    
                    ui.horizontal(|ui| {
                        if ui.small_button(parent_name).clicked() {
                            self.selected = Some(*parent_id);
                        }
                        
                        // 種類の表示
                        if !kind.is_empty() && kind != "biological" {
                            ui.label(format!("({})", kind));
                        }
                        
                        // 編集ボタン
                        if ui.small_button("✏️").on_hover_text(&t("edit_kind")).clicked() {
                            self.editing_parent_kind = Some((*parent_id, sel));
                            self.edit_parent_kind_text = kind.clone();
                        }
                        
                        // 削除ボタン
                        if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                            self.tree.remove_parent_child(*parent_id, sel);
                            self.status = t("relation_removed");
                        }
                    });
                    
                    // 種類編集UI
                    if self.editing_parent_kind == Some((*parent_id, sel)) {
                        ui.horizontal(|ui| {
                            ui.label(&t("kind"));
                            ui.text_edit_singleline(&mut self.edit_parent_kind_text);
                            if ui.button(&t("save")).clicked() {
                                // 親子関係の種類を更新
                                if let Some(edge) = self.tree.edges.iter_mut().find(|e| {
                                    e.parent == *parent_id && e.child == sel
                                }) {
                                    edge.kind = if self.edit_parent_kind_text.trim().is_empty() {
                                        "biological".to_string()
                                    } else {
                                        self.edit_parent_kind_text.trim().to_string()
                                    };
                                    self.status = t("relation_kind_updated");
                                }
                                self.editing_parent_kind = None;
                                self.edit_parent_kind_text.clear();
                            }
                            if ui.button(&t("cancel")).clicked() {
                                self.editing_parent_kind = None;
                                self.edit_parent_kind_text.clear();
                            }
                        });
                    }
                }
            }
            
            // 配偶者の表示（メモ付き）
            let spouse_ids = self.tree.spouses_of(sel);
            if !spouse_ids.is_empty() {
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
                            self.selected = Some(*spouse_id);
                        }
                        
                        // メモの表示と編集
                        if !spouse_memo.is_empty() {
                            ui.label(format!("({})", spouse_memo));
                        }
                        
                        // 編集ボタン
                        if ui.small_button("✏️").on_hover_text(&t("edit_memo")).clicked() {
                            self.editing_spouse_memo = Some((sel, *spouse_id));
                            self.edit_spouse_memo_text = spouse_memo.clone();
                        }
                        
                        // 削除ボタン
                        if ui.small_button("❌").on_hover_text(&t("remove_relation")).clicked() {
                            self.tree.remove_spouse(sel, *spouse_id);
                            self.status = t("relation_removed");
                        }
                    });
                    
                    // メモ編集UI
                    if self.editing_spouse_memo == Some((sel, *spouse_id)) {
                        ui.horizontal(|ui| {
                            ui.label(&t("memo"));
                            ui.text_edit_singleline(&mut self.edit_spouse_memo_text);
                            if ui.button(&t("save")).clicked() {
                                // 配偶者関係のメモを更新
                                if let Some(spouse_rel) = self.tree.spouses.iter_mut().find(|s| {
                                    (s.person1 == sel && s.person2 == *spouse_id) ||
                                    (s.person1 == *spouse_id && s.person2 == sel)
                                }) {
                                    spouse_rel.memo = self.edit_spouse_memo_text.clone();
                                    self.status = t("spouse_memo_updated");
                                }
                                self.editing_spouse_memo = None;
                                self.edit_spouse_memo_text.clear();
                            }
                            if ui.button(&t("cancel")).clicked() {
                                self.editing_spouse_memo = None;
                                self.edit_spouse_memo_text.clear();
                            }
                        });
                    }
                }
            }

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

    /// 家族管理タブのUI
    fn render_families_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        ui.heading(t("manage_families"));
        
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
    
        // 家族エディタ
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

        // アクションボタン
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

    /// 設定タブのUI
    fn render_settings_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
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
    }

    /// キャンバスのノード描画
    fn render_canvas_nodes(
        &mut self,
        _ui: &mut egui::Ui,
        painter: &egui::Painter,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        for n in nodes {
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
    }

    /// ノードとのインタラクション処理
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
        
        for n in nodes {
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
                            let current_pos = person.position;
                            let new_x = current_pos.0 + delta.x;
                            let new_y = current_pos.1 + delta.y;
                            
                            person.position = (new_x, new_y);
                        }
                        self.node_drag_start = pointer_pos;
                    }
                }
                
                if node_response.drag_stopped() && self.dragging_node == Some(n.id) {
                    if self.show_grid {
                        if let Some(person) = self.tree.persons.get_mut(&n.id) {
                            let (x, y) = person.position;
                            let relative_pos = egui::pos2(x - origin.x, y - origin.y);
                            let snapped_rel = LayoutEngine::snap_to_grid(relative_pos, self.grid_size);
                            
                            let snapped_x = origin.x + snapped_rel.x;
                            let snapped_y = origin.y + snapped_rel.y;
                            
                            person.position = (snapped_x, snapped_y);
                        }
                    }
                    self.dragging_node = None;
                    self.node_drag_start = None;
                }
                
                if node_response.clicked() {
                    self.selected = Some(n.id);
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
        
        (node_hovered, any_node_dragged)
    }

    /// パン・ズーム処理
    fn handle_pan_zoom(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        pointer_pos: Option<egui::Pos2>,
        node_hovered: bool,
        any_node_dragged: bool,
    ) {
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
    }

    /// 関係線（エッジ）の描画
    fn render_canvas_edges(
        &self,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        // 配偶者の線
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

        // 親子の線
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
    }

    /// 家族の枠描画
    fn render_family_boxes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        for family in &self.tree.families {
            if family.members.len() < 2 {
                continue;
            }
            
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
                
                let label_pos = family_rect.left_top() + egui::vec2(10.0, 5.0);
                let label_size = egui::vec2(family_rect.width() * 0.5, 20.0);
                let label_rect = egui::Rect::from_min_size(label_pos, label_size);
                
                let resp = ui.interact(label_rect, egui::Id::new(("family_label", family.id)), egui::Sense::click());
                
                let bg_color = if resp.is_pointer_button_down_on() {
                    egui::Color32::from_rgba_unmultiplied(
                        stroke_color.r(), 
                        stroke_color.g(), 
                        stroke_color.b(), 
                        100
                    )
                } else if resp.hovered() {
                    egui::Color32::from_rgba_unmultiplied(
                        stroke_color.r(), 
                        stroke_color.g(), 
                        stroke_color.b(), 
                        60
                    )
                } else {
                    egui::Color32::from_rgba_unmultiplied(
                        stroke_color.r(), 
                        stroke_color.g(), 
                        stroke_color.b(), 
                        30
                    )
                };
                
                painter.rect_filled(label_rect, 3.0, bg_color);
                
                if resp.hovered() || resp.is_pointer_button_down_on() {
                    painter.rect_stroke(
                        label_rect,
                        3.0,
                        egui::Stroke::new(1.5, stroke_color),
                        egui::epaint::StrokeKind::Outside
                    );
                }
                
                let text_color = if resp.hovered() || resp.is_pointer_button_down_on() {
                    stroke_color
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
    }

    /// キャンバス描画
    fn render_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click());
            let pointer_pos = ui.input(|i| i.pointer.interact_pos());
            
            // キャンバス情報を保存
            self.canvas_rect = rect;

            // ズーム処理
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
            
            // originを保存
            self.canvas_origin = origin;
            
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

            // ノードのインタラクション処理
            let (node_hovered, any_node_dragged) = self.handle_node_interactions(ui, &nodes, &screen_rects, pointer_pos, origin);
            
            // パン・ズーム処理
            self.handle_pan_zoom(ui, rect, pointer_pos, node_hovered, any_node_dragged);

            // エッジ（関係線）描画
            self.render_canvas_edges(&painter, &screen_rects);

            // 家族の枠描画
            self.render_family_boxes(ui, &painter, &screen_rects);

            // ノード描画
            self.render_canvas_nodes(ui, &painter, &nodes, &screen_rects);

            // ズーム表示
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.language;
        let t = |key: &str| Texts::get(key, lang);
        
        // サイドパネル
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
                    SideTab::Persons => self.render_persons_tab(ui, t),
                    SideTab::Families => self.render_families_tab(ui, t),
                    SideTab::Settings => self.render_settings_tab(ui, t),
                }
            });
        });

        // キャンバス
        self.render_canvas(ctx);
    }
}