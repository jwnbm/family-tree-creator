use eframe::egui;

use crate::app::App;
use crate::core::i18n::Texts;

pub trait ViewMenuRenderer {
    fn render_view_menu(&mut self, ui: &mut egui::Ui);
}

impl ViewMenuRenderer for App {
    fn render_view_menu(&mut self, ui: &mut egui::Ui) {
        let lang = self.ui.language;
        let t = |key: &str| Texts::get(key, lang);

        ui.menu_button(t("view_menu"), |ui| {
            if ui.button(t("fit_to_view")).clicked() {
                self.fit_canvas_to_contents();
                ui.close();
            }
        });
    }
}