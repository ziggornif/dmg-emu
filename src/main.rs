use emulator::gui::GameBoyApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "DMG Emu",
        native_options,
        Box::new(|cc| Ok(Box::new(GameBoyApp::new(cc)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
