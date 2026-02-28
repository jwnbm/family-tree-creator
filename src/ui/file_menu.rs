use eframe::egui;
use crate::app::App;
use crate::core::tree::FamilyTree;

pub trait FileMenuRenderer {
    fn render_file_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context);
}

impl FileMenuRenderer for App {
    fn render_file_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let lang = self.ui.language;
        let t = |key: &str| crate::core::i18n::Texts::get(key, lang);
        
        ui.menu_button(t("file_menu"), |ui| {
            // 新規作成
            if ui.button(t("new")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Family Tree", &["json", "sqlite", "db"])
                    .add_filter("JSON", &["json"])
                    .add_filter("SQLite", &["sqlite", "db"])
                    .set_file_name("tree.json")
                    .save_file()
                {
                    self.tree = FamilyTree::default();
                    self.person_editor.selected = None;
                    self.family_editor.selected_family = None;
                    self.event_editor.selected = None;
                    self.file.file_path = path.display().to_string();
                    self.file.status = t("new_tree_created");
                    self.save();
                }
                ui.close();
            }
            
            // 開く
            if ui.button(format!("{} (Ctrl+O)", t("open"))).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Family Tree", &["json", "sqlite", "db"])
                    .add_filter("JSON", &["json"])
                    .add_filter("SQLite", &["sqlite", "db"])
                    .pick_file()
                {
                    self.file.file_path = path.display().to_string();
                    self.load();
                }
                ui.close();
            }
            
            // 保存
            if ui.button(format!("{} (Ctrl+S)", t("save"))).clicked() {
                // ファイルパスが存在しない場合は名前を付けて保存
                if self.file.file_path.is_empty() || !std::path::Path::new(&self.file.file_path).exists() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Family Tree", &["json", "sqlite", "db"])
                        .add_filter("JSON", &["json"])
                        .add_filter("SQLite", &["sqlite", "db"])
                        .set_file_name(if self.file.file_path.is_empty() { "tree.json" } else { &self.file.file_path })
                        .save_file()
                    {
                        self.file.file_path = path.display().to_string();
                        self.save();
                    }
                } else {
                    self.save();
                }
                ui.close();
            }
            
            // 名前を付けて保存
            if ui.button(t("save_as")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Family Tree", &["json", "sqlite", "db"])
                    .add_filter("JSON", &["json"])
                    .add_filter("SQLite", &["sqlite", "db"])
                    .set_file_name(&self.file.file_path)
                    .save_file()
                {
                    self.file.file_path = path.display().to_string();
                    self.save();
                }
                ui.close();
            }
        });
        
        // キーボードショートカット
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            // ファイルパスが存在しない場合は名前を付けて保存
            if self.file.file_path.is_empty() || !std::path::Path::new(&self.file.file_path).exists() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Family Tree", &["json", "sqlite", "db"])
                    .add_filter("JSON", &["json"])
                    .add_filter("SQLite", &["sqlite", "db"])
                    .set_file_name(if self.file.file_path.is_empty() { "tree.json" } else { &self.file.file_path })
                    .save_file()
                {
                    self.file.file_path = path.display().to_string();
                    self.save();
                }
            } else {
                self.save();
            }
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Family Tree", &["json", "sqlite", "db"])
                .add_filter("JSON", &["json"])
                .add_filter("SQLite", &["sqlite", "db"])
                .pick_file()
            {
                self.file.file_path = path.display().to_string();
                self.load();
            }
        }
    }
}
