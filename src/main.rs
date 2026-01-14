mod core;
mod ui;
mod app;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Family Tree")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Family Tree",
        options,
        Box::new(|cc| {
            // 日本語フォントが含まれるようにする
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}

fn setup_fonts(ctx: &eframe::egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();
    
    // Noto Sans JPフォントを追加
    fonts.font_data.insert(
        "noto_sans_jp".to_owned(),
        std::sync::Arc::new(eframe::egui::FontData::from_static(include_bytes!(
            "../fonts/NotoSansJP-Regular.ttf"
        ))),
    );
    
    // Proportionalフォントファミリーの最優先に設定
    fonts
        .families
        .entry(eframe::egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_jp".to_owned());
    
    // Monospaceフォントファミリーにも追加
    fonts
        .families
        .entry(eframe::egui::FontFamily::Monospace)
        .or_default()
        .push("noto_sans_jp".to_owned());
    
    ctx.set_fonts(fonts);
}
