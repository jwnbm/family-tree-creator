use std::fs;

use eframe::egui;
use crate::core::tree::{FamilyTree, PersonId};
use crate::core::i18n::Texts;
use crate::ui::{
    FileMenuRenderer, PersonsTabRenderer, FamiliesTabRenderer, SettingsTabRenderer, CanvasRenderer,
    PersonEditorState, RelationEditorState, FamilyEditorState, 
    CanvasState, FileState, UiState, SideTab
};

// 定数
pub const NODE_CORNER_RADIUS: f32 = 6.0;
pub const EDGE_STROKE_WIDTH: f32 = 1.5;
pub const SPOUSE_LINE_OFFSET: f32 = 2.0;

pub struct App {
    pub tree: FamilyTree,
    
    // 状態管理（機能ごとに分離）
    pub person_editor: PersonEditorState,
    pub relation_editor: RelationEditorState,
    pub family_editor: FamilyEditorState,
    pub canvas: CanvasState,
    pub file: FileState,
    pub ui: UiState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tree: FamilyTree::default(),
            person_editor: PersonEditorState::default(),
            relation_editor: RelationEditorState::new(),
            family_editor: FamilyEditorState::new(),
            canvas: CanvasState::default(),
            file: FileState::new(),
            ui: UiState::default(),
        }
    }
}

impl App {
    pub fn save(&mut self) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        match serde_json::to_string_pretty(&self.tree) {
            Ok(s) => match fs::write(&self.file.file_path, s) {
                Ok(_) => self.file.status = format!("{}: {}", t("saved"), self.file.file_path),
                Err(e) => self.file.status = format!("Save error: {e}"),
            },
            Err(e) => self.file.status = format!("Serialize error: {e}"),
        }
    }

    pub fn load(&mut self) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        match fs::read_to_string(&self.file.file_path) {
            Ok(s) => match serde_json::from_str::<FamilyTree>(&s) {
                Ok(tree) => {
                    self.tree = tree;
                    self.person_editor.selected = None;
                    self.file.status = format!("{}: {}", t("loaded"), self.file.file_path);
                }
                Err(e) => self.file.status = format!("Parse error: {e}"),
            },
            Err(e) => self.file.status = format!("Read error: {e}"),
        }
    }

    pub fn clear_person_form(&mut self) {
        self.person_editor.clear();
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
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        
        // メニューバー
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.render_file_menu(ui, ctx);
            });
        });
        
        // サイドパネル
        egui::SidePanel::left("left_panel").resizable(true).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(t("title"));
                
                // タブ選択
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.ui.side_tab, SideTab::Persons, t("persons"));
                    ui.selectable_value(&mut self.ui.side_tab, SideTab::Families, t("families"));
                    ui.selectable_value(&mut self.ui.side_tab, SideTab::Settings, t("settings"));
                });
                ui.separator();

                match self.ui.side_tab {
                    SideTab::Persons => self.render_persons_tab(ui, t),
                    SideTab::Families => self.render_families_tab(ui, t),
                    SideTab::Settings => self.render_settings_tab(ui, t),
                }
            });
        });

        // キャンバス
        self.render_canvas(ctx);

        // ステータスバー
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if !self.file.status.is_empty() {
                    ui.label(&self.file.status);
                } else {
                    ui.label(""); // 空の場合でもスペースを確保
                }
            });
        });
    }
}