use std::collections::HashMap;

use eframe::egui;

use crate::application::{AppSettings, TreeFileService};
use crate::core::i18n::{self as i18n, Texts};
use crate::core::layout::LayoutEngine;
use crate::core::tree::{FamilyTree, PersonId};
use crate::infrastructure::read_image_dimensions;
use crate::infrastructure::MultiFormatTreeRepository;
use crate::ui::{
    CanvasRenderer, CanvasState, EventEditorState, EventsTabRenderer, FamiliesTabRenderer,
    FamilyEditorState, FileMenuRenderer, FileState, HelpMenuRenderer, LogLevel, LogState,
    PersonEditorState, PersonsTabRenderer, RelationEditorState, SettingsTabRenderer, SideTab,
    UiState, ViewMenuRenderer,
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

        app.load_settings_on_startup();
        
        let t = |key: &str| Texts::get(key, app.ui.language);
        app.log.add(t("log_app_started"), LogLevel::Debug);
        app
    }
}

impl App {
    fn apply_settings(&mut self, settings: AppSettings) {
        self.ui.language = settings.language;
        self.canvas.show_grid = settings.show_grid;
        self.canvas.grid_size = settings.grid_size.clamp(10.0, 200.0);
        self.ui.node_color_theme = settings.node_color_theme;
    }

    fn collect_settings(&self) -> AppSettings {
        AppSettings {
            language: self.ui.language,
            show_grid: self.canvas.show_grid,
            grid_size: self.canvas.grid_size,
            node_color_theme: self.ui.node_color_theme,
        }
    }

    fn load_settings_on_startup(&mut self) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);

        match AppSettings::load_from_default_path() {
            Ok(Some(settings)) => {
                self.apply_settings(settings);
                self.log
                    .add(t("log_settings_loaded"), LogLevel::Debug);
            }
            Ok(None) => {
                self.apply_settings(AppSettings::default());
            }
            Err(error) => {
                self.apply_settings(AppSettings::default());
                self.log.add(
                    format!("{}: {error}", t("log_settings_load_failed")),
                    LogLevel::Warning,
                );
            }
        }
    }

    pub(crate) fn save_settings(&mut self) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);

        let settings = self.collect_settings();
        if let Err(error) = settings.save_to_default_path() {
            self.log.add(
                format!("{}: {error}", t("log_settings_save_failed")),
                LogLevel::Error,
            );
        }
    }

    fn set_error_status_and_log(&mut self, status_prefix: &str, error: &str) {
        let message = format!("{status_prefix}: {error}");
        self.file.status = message.clone();
        self.log.add(message, LogLevel::Error);
    }

    pub(crate) fn visible_canvas_left_top(&self) -> (f32, f32) {
        if self.canvas.canvas_rect == egui::Rect::NOTHING {
            return (100.0, 100.0);
        }

        let screen_position = self.canvas.canvas_rect.left_top() + egui::vec2(50.0, 50.0);
        let world_position = self.canvas.canvas_origin
            + (screen_position - self.canvas.canvas_origin - self.canvas.pan) / self.canvas.zoom;
        (world_position.x, world_position.y)
    }

    pub fn save(&mut self) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        let service = TreeFileService::new(MultiFormatTreeRepository::new());

        if let Err(error) = service.save_tree(&self.file.file_path, &self.tree) {
            self.set_error_status_and_log(&t("save_error"), &error.to_string());
            return;
        }

        self.file.status = format!("{}: {}", t("saved"), self.file.file_path);
        self.log
            .add(
                format!("{}: {}", t("log_file_saved"), self.file.file_path),
                LogLevel::Debug,
            );
    }

    pub fn load(&mut self) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        let service = TreeFileService::new(MultiFormatTreeRepository::new());
        let tree = match service.load_tree(&self.file.file_path) {
            Ok(tree) => tree,
            Err(error) => {
                self.set_error_status_and_log(&t("load_error"), &error.to_string());
                return;
            }
        };

        self.tree = tree;
        self.person_editor.selected = None;
        self.file.status = format!("{}: {}", t("loaded"), self.file.file_path);
        self.log
            .add(
                format!("{}: {}", t("log_file_loaded"), self.file.file_path),
                LogLevel::Debug,
            );
    }

    pub fn clear_person_form(&mut self) {
        self.person_editor.clear();
    }

    pub fn parse_optional_field(s: &str) -> Option<String> {
        let trimmed = s.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    pub fn get_person_name(&self, id: &PersonId) -> String {
        let lang = self.ui.language;
        self.tree.persons.get(id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| Texts::get("unknown", lang))
    }

    pub fn fit_canvas_to_contents(&mut self) {
        if self.canvas.canvas_rect == egui::Rect::NOTHING {
            return;
        }

        if self.tree.persons.is_empty() && self.tree.events.is_empty() {
            self.canvas.zoom = 1.0;
            self.canvas.pan = egui::Vec2::ZERO;
            return;
        }

        let base_origin = self.canvas.canvas_rect.left_top() + egui::vec2(24.0, 24.0);
        let origin = if self.canvas.show_grid {
            LayoutEngine::snap_to_grid(base_origin, self.canvas.grid_size)
        } else {
            base_origin
        };

        let photo_dimensions: HashMap<PersonId, (u32, u32)> = self
            .tree
            .persons
            .iter()
            .filter_map(|(person_id, person)| {
                if person.display_mode != crate::core::tree::PersonDisplayMode::NameAndPhoto {
                    return None;
                }

                person
                    .photo_path
                    .as_deref()
                    .and_then(read_image_dimensions)
                    .map(|dimensions| (*person_id, dimensions))
            })
            .collect();

        let nodes = LayoutEngine::compute_layout(&self.tree, origin, &photo_dimensions);

        let mut world_bounds: Option<egui::Rect> = None;
        for node in &nodes {
            world_bounds = Some(match world_bounds {
                Some(bounds) => bounds.union(node.rect),
                None => node.rect,
            });
        }

        let lang = self.ui.language;
        for event in self.tree.events.values() {
            let (width, height) = LayoutEngine::calculate_event_node_size(&event.name, lang);
            let event_rect = egui::Rect::from_min_size(
                egui::pos2(event.position.0, event.position.1),
                egui::vec2(width, height),
            );

            world_bounds = Some(match world_bounds {
                Some(bounds) => bounds.union(event_rect),
                None => event_rect,
            });
        }

        let Some(bounds) = world_bounds else {
            return;
        };

        let margin = 40.0;
        let min_width = 1.0;
        let min_height = 1.0;
        let content_width = bounds.width().max(min_width);
        let content_height = bounds.height().max(min_height);
        let available_width = (self.canvas.canvas_rect.width() - margin * 2.0).max(min_width);
        let available_height = (self.canvas.canvas_rect.height() - margin * 2.0).max(min_height);

        let fit_zoom_x = available_width / content_width;
        let fit_zoom_y = available_height / content_height;
        self.canvas.zoom = fit_zoom_x.min(fit_zoom_y).clamp(0.3, 3.0);

        let world_center = bounds.center();
        let screen_center = self.canvas.canvas_rect.center();
        self.canvas.pan = screen_center - origin - (world_center - origin) * self.canvas.zoom;

        let t = |key: &str| Texts::get(key, lang);
        self.file.status = t("fit_to_view_done");
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        
        // i18n警告をログに出力
        for warning in i18n::take_warnings() {
            self.log.add(warning, LogLevel::Warning);
        }
        
        // メニューバー
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.render_file_menu(ui, ctx);
                self.render_view_menu(ui);
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
                    ui.heading(t("log_panel_title"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t("clear")).clicked() {
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