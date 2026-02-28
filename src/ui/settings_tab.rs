use crate::app::App;
use crate::core::i18n::Language;
use crate::ui::NodeColorThemePreset;

/// 設定タブのUI描画トレイト
pub trait SettingsTabRenderer {
    fn render_settings_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl SettingsTabRenderer for App {
    fn render_settings_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        ui.heading(t("settings"));
        ui.separator();
        
        ui.label(t("language"));
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.ui.language, Language::Japanese, t("japanese"));
            ui.radio_value(&mut self.ui.language, Language::English, t("english"));
        });
        
        ui.separator();
        ui.label(t("grid"));
        ui.checkbox(&mut self.canvas.show_grid, t("show_grid"));
        ui.horizontal(|ui| {
            ui.label(t("grid_size"));
            ui.add(egui::DragValue::new(&mut self.canvas.grid_size)
                .speed(1.0)
                .range(10.0..=200.0));
        });

        ui.separator();
        ui.label(t("node_color_theme"));
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.ui.node_color_theme,
                NodeColorThemePreset::Default,
                t("node_color_theme_default"),
            );
            ui.radio_value(
                &mut self.ui.node_color_theme,
                NodeColorThemePreset::HighContrast,
                t("node_color_theme_high_contrast"),
            );
        });
    }
}
