mod tree;
mod layout;
mod app;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Family Tree (egui MVP)")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Family Tree (egui MVP)",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
