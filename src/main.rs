fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "Wpass GUI",
        native_options,
        Box::new(|cc| {
            let mut wpass_gui = wpass_gui::WPassApp::new(cc);
            wpass_gui.init();
            Box::new(wpass_gui)
        }),
    )
}
