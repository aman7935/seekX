mod desktop;
mod launcher;
mod search;
mod ui;

use eframe::egui;

fn main() {
    if let Err(err) = run() {
        eprintln!("seekX failed: {err}");
    }
}

fn run() -> Result<(), eframe::Error> {
    let apps = desktop::load_installed_apps();
    let launcher = launcher::Launcher::new(apps);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 520.0])
            .with_min_inner_size([680.0, 420.0])
            .with_decorations(false)
            .with_transparent(false)
            .with_title("seekX"),
        ..Default::default()
    };

    eframe::run_native(
        "seekX",
        options,
        Box::new(move |_cc| Ok(Box::new(ui::SeekXApp::new(launcher)))),
    )?;

    Ok(())
}
