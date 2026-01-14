use std::fs;

use eframe::egui;
use crate::core::tree::{FamilyTree, Gender, PersonId};
use crate::core::i18n::{Language, Texts};
use crate::ui::{
    PersonsTabRenderer, FamiliesTabRenderer, SettingsTabRenderer, CanvasRenderer
};
use uuid::Uuid;

// 定数
pub const DEFAULT_RELATION_KIND: &str = "biological";
pub const NODE_CORNER_RADIUS: f32 = 6.0;
pub const EDGE_STROKE_WIDTH: f32 = 1.5;
pub const SPOUSE_LINE_OFFSET: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideTab {
    Persons,
    Families,
    Settings,
}

pub struct App {
    pub tree: FamilyTree,
    pub selected: Option<PersonId>,

    // 入力フォーム
    pub new_name: String,
    pub new_gender: Gender,
    pub new_birth: String,
    pub new_memo: String,
    pub new_deceased: bool,
    pub new_death: String,

    // 親子関係追加フォーム
    pub parent_pick: Option<PersonId>,
    pub child_pick: Option<PersonId>,
    pub relation_kind: String,

    // 配偶者関係追加フォーム
    pub spouse_pick: Option<PersonId>,
    pub spouse_memo: String,
    
    // 配偶者メモ編集
    pub editing_spouse_memo: Option<(PersonId, PersonId)>,
    pub temp_spouse_memo: String,
    
    // 親子関係の種類編集
    pub editing_parent_kind: Option<(PersonId, PersonId)>, // (parent, child)
    pub temp_kind: String,

    // 保存/読込
    pub file_path: String,
    pub status: String,

    // 表示
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub dragging_pan: bool,
    pub last_pointer_pos: Option<egui::Pos2>,
    
    // ノードドラッグ
    pub dragging_node: Option<PersonId>,
    pub node_drag_start: Option<egui::Pos2>,
    
    // グリッド
    pub show_grid: bool,
    pub grid_size: f32,
    
    // サイドパネルタブ
    pub side_tab: SideTab,
    
    // 言語
    pub language: Language,
    
    // 家族管理
    pub selected_family: Option<Uuid>,
    pub new_family_name: String,
    pub new_family_color: [f32; 3],
    pub family_member_pick: Option<PersonId>,
    
    // キャンバス情報
    pub canvas_rect: egui::Rect,
    pub canvas_origin: egui::Pos2,
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

            spouse_pick: None,
            spouse_memo: String::new(),
            
            editing_spouse_memo: None,
            temp_spouse_memo: String::new(),
            
            editing_parent_kind: None,
            temp_kind: String::new(),

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
    pub fn save(&mut self) {
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

    pub fn load(&mut self) {
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

    pub fn add_sample(&mut self) {
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

    pub fn clear_person_form(&mut self) {
        self.new_name.clear();
        self.new_gender = Gender::Unknown;
        self.new_birth.clear();
        self.new_memo.clear();
        self.new_deceased = false;
        self.new_death.clear();
    }

    pub fn parse_optional_field(s: &str) -> Option<String> {
        let trimmed = s.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    pub fn get_person_name(&self, id: &PersonId) -> String {
        self.tree.persons.get(id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
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