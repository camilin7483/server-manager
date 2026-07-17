// Server Manager v0.1.0
// Plataforma profesional de administración de infraestructura
// Arquitectura modular: 11 crates, separación estricta de responsabilidades

use eframe::egui;
use sm_ui::ServerManagerUi;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Server Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "Server Manager",
        options,
        Box::new(|_cc| Ok(Box::new(ServerManagerUi::default()))),
    )
}
