use eframe::egui;
use crate::app::App;
use crate::core::i18n::Texts;

pub trait HelpMenuRenderer {
    fn render_help_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context);
}

impl HelpMenuRenderer for App {
    fn render_help_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);
        
        ui.menu_button(t("help_menu"), |ui| {
            if ui.button(t("about")).clicked() {
                self.ui.show_about_dialog = true;
                ui.close();
            }
            if ui.button(t("license")).clicked() {
                self.ui.show_license_dialog = true;
                ui.close();
            }
        });
        
        // バージョン情報ダイアログ
        if self.ui.show_about_dialog {
            egui::Window::new(t("about"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(t("app_name"));
                        ui.label(format!("{}: {}", t("version"), env!("CARGO_PKG_VERSION")));
                        ui.add_space(10.0);
                        ui.label(t("app_description"));
                        ui.add_space(10.0);
                        if ui.button(t("close")).clicked() {
                            self.ui.show_about_dialog = false;
                        }
                    });
                });
        }
        
        // ライセンス情報ダイアログ
        if self.ui.show_license_dialog {
            egui::Window::new(t("license"))
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .default_height(400.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(t("license_text"));
                    });
                    ui.add_space(10.0);
                    if ui.button(t("close")).clicked() {
                        self.ui.show_license_dialog = false;
                    }
                });
        }
    }
}
