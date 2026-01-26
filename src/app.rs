use std::fs;

use eframe::egui;
use crate::core::tree::{FamilyTree, PersonId};
use crate::core::i18n::{Texts, self as i18n};
use crate::ui::{
    FileMenuRenderer, HelpMenuRenderer, PersonsTabRenderer, FamiliesTabRenderer, EventsTabRenderer, SettingsTabRenderer, CanvasRenderer,
    PersonEditorState, RelationEditorState, FamilyEditorState, EventEditorState,
    CanvasState, FileState, UiState, SideTab, LogState, LogLevel
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
    pub event_editor: EventEditorState,
    pub canvas: CanvasState,
    pub file: FileState,
    pub ui: UiState,
    pub log: LogState,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            tree: FamilyTree::default(),
            person_editor: PersonEditorState::default(),
            relation_editor: RelationEditorState::new(),
            family_editor: FamilyEditorState::new(),
            event_editor: EventEditorState::default(),
            canvas: CanvasState::default(),
            file: FileState::new(),
            ui: UiState::default(),
            log: LogState::default(),
        };
        
        // logディレクトリを作成し、ログファイルを初期化
        if let Err(e) = app.log.set_log_file("logs") {
            eprintln!("Failed to create log directory: {}", e);
        }
        
        let t = |key: &str| Texts::get(key, app.ui.language);
        app.log.add(t("log_app_started"));
        app
    }
}

impl App {
    pub fn save(&mut self) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        match serde_json::to_string_pretty(&self.tree) {
            Ok(s) => match fs::write(&self.file.file_path, s) {
                Ok(_) => {
                    self.file.status = format!("{}: {}", t("saved"), self.file.file_path);
                    self.log.add(format!("{}: {}", t("log_file_saved"), self.file.file_path));
                },
                Err(e) => {
                    self.file.status = format!("Save error: {e}");
                    self.log.add_with_level(format!("Save error: {e}"), LogLevel::Error);
                },
            },
            Err(e) => {
                self.file.status = format!("Serialize error: {e}");
                self.log.add_with_level(format!("Serialize error: {e}"), LogLevel::Error);
            },
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
                    self.log.add(format!("{}: {}", t("log_file_loaded"), self.file.file_path));
                }
                Err(e) => {
                    self.file.status = format!("Parse error: {e}");
                    self.log.add_with_level(format!("Parse error: {e}"), LogLevel::Error);
                },
            },
            Err(e) => {
                self.file.status = format!("Read error: {e}");
                self.log.add_with_level(format!("Read error: {e}"), LogLevel::Error);
            },
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
        
        // i18n警告をログに出力
        for warning in i18n::take_warnings() {
            self.log.add_with_level(warning, LogLevel::Warning);
        }
        
        // メニューバー
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.render_file_menu(ui, ctx);
                self.render_help_menu(ui, ctx);
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
                    ui.selectable_value(&mut self.ui.side_tab, SideTab::Events, t("events"));
                    ui.selectable_value(&mut self.ui.side_tab, SideTab::Settings, t("settings"));
                });
                ui.separator();

                match self.ui.side_tab {
                    SideTab::Persons => self.render_persons_tab(ui, t),
                    SideTab::Families => self.render_families_tab(ui, t),
                    SideTab::Events => self.render_events_tab(ui, t),
                    SideTab::Settings => self.render_settings_tab(ui, t),
                }
            });
        });

        // ログパネル（下部）
        egui::TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .default_height(120.0)
            .min_height(60.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📋 ログ");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("クリア").clicked() {
                            self.log.clear();
                        }
                    });
                });
                ui.separator();
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for msg in &self.log.messages {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&msg.timestamp)
                                        .color(egui::Color32::GRAY)
                                        .monospace()
                                );
                                ui.label(
                                    egui::RichText::new(format!("[{}]", msg.level.as_str()))
                                        .color(msg.level.color())
                                        .monospace()
                                );
                                ui.label(&msg.message);
                            });
                        }
                    });
            });

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
        
        // キャンバス（最後に描画することで他のパネルの後ろに配置）
        self.render_canvas(ctx);
    }
}