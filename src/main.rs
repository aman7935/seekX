mod desktop;
mod launcher;
mod search;
mod ui;

fn main() {
    let apps = desktop::load_installed_apps();
    let launcher = launcher::Launcher::new(apps);
    ui::run(launcher);
}
