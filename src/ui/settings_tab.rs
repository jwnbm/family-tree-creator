use crate::app::App;
use crate::core::i18n::Language;
use crate::ui::NodeColorThemePreset;

/// 設定タブのUI描画トレイト
pub trait SettingsTabRenderer {
    fn render_settings_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String);
}

impl SettingsTabRenderer for App {
    fn render_settings_tab(&mut self, ui: &mut egui::Ui, t: impl Fn(&str) -> String) {
        let mut has_changed = false;

        ui.heading(t("settings"));
        ui.separator();
        
        ui.label(t("language"));
        ui.horizontal(|ui| {
            has_changed |= ui
                .radio_value(&mut self.ui.language, Language::Japanese, t("japanese"))
                .changed();
            has_changed |= ui
                .radio_value(&mut self.ui.language, Language::English, t("english"))
                .changed();
        });
        
        ui.separator();
        ui.label(t("grid"));
        has_changed |= ui.checkbox(&mut self.canvas.show_grid, t("show_grid")).changed();
        ui.horizontal(|ui| {
            ui.label(t("grid_size"));
            has_changed |= ui
                .add(
                    egui::DragValue::new(&mut self.canvas.grid_size)
                        .speed(1.0)
                        .range(10.0..=200.0),
                )
                .changed();
        });

        ui.separator();
        ui.label(t("node_color_theme"));
        ui.horizontal(|ui| {
            has_changed |= ui
                .radio_value(
                    &mut self.ui.node_color_theme,
                    NodeColorThemePreset::Default,
                    t("node_color_theme_default"),
                )
                .changed();
            has_changed |= ui
                .radio_value(
                    &mut self.ui.node_color_theme,
                    NodeColorThemePreset::HighContrast,
                    t("node_color_theme_high_contrast"),
                )
                .changed();
        });

        if has_changed {
            self.save_settings();
        }
    }
}
