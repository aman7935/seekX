pub mod styles;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;
#[cfg(feature = "layer-shell")]
use gtk4_layer_shell as layer_shell;
#[cfg(feature = "layer-shell")]
use gtk4_layer_shell::LayerShell;

use crate::application::{Launcher, RankedApp};
use crate::ui::styles::EMBEDDED_CSS;
#[derive(Clone)]
pub enum ResultItem {
    App(RankedApp),
    Folder { name: String, path: String },
    File { name: String, path: String },
    WebSearch { query: String },
}

const RESULT_LIMIT: usize = 9;
const WINDOW_WIDTH: i32 = 580;
const SEARCH_BOX_HEIGHT: i32 = 62;
const RESULTS_AREA_HEIGHT: i32 = 200;
const ANIMATION_MS: u32 = 200;

#[derive(Default)]
struct UiState {
    results: Vec<ResultItem>,
}

pub fn run(launcher: Launcher) {
    let app = gtk::Application::builder()
        .application_id("com.seekx.launcher")
        .build();

    app.connect_activate(move |app| {
        if let Some(window) = app.active_window() {
            // If already visible on screen, do NOT open a second instance.
            if window.is_visible() {
                return;
            }
            // Hidden (closed via hide-on-close): just re-present it.
            // The window's own `connect_show` handler will clear state.
            window.present();
        } else {
            build_ui(app, launcher.clone());
        }
    });

    app.run();
}

fn build_ui(app: &gtk::Application, launcher: Launcher) {
    install_css();

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("seekX")
        .default_width(WINDOW_WIDTH)
        .default_height(SEARCH_BOX_HEIGHT)
        .resizable(false)
        .decorated(false)
        .build();
    window.add_css_class("seekx-window");
    window.remove_css_class("background");
    window.remove_css_class("solid-csd");
    window.set_hide_on_close(true);
    setup_layer_shell(&window);

    // ── Outer wrapper: transparent, vertical, holds both boxes ──
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 8);
    outer.add_css_class("seekx-outer");

    // ── Search box ──
    let search_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    search_box.add_css_class("seekx-search-box");

    let entry = gtk::Entry::builder()
        .placeholder_text("Search Anything")
        .hexpand(true)
        .build();
    entry.add_css_class("seekx-entry");
    search_box.append(&entry);

    // ── Results box ──
    let results_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    results_box.add_css_class("seekx-results-box");

    let list = gtk::ListBox::new();
    list.add_css_class("seekx-list");
    list.set_selection_mode(gtk::SelectionMode::Single);
    list.set_activate_on_single_click(false);

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .child(&list)
        .build();
    scroller.set_has_frame(false);
    scroller.set_min_content_height(RESULTS_AREA_HEIGHT);
    scroller.set_max_content_height(RESULTS_AREA_HEIGHT);
    scroller.add_css_class("seekx-scroll");

    results_box.append(&scroller);

    // ── Revealer for animated show/hide of results ──
    let revealer = gtk::Revealer::new();
    revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(ANIMATION_MS);
    revealer.set_reveal_child(false);
    revealer.set_child(Some(&results_box));

    // ── Assemble ──
    outer.append(&search_box);
    outer.append(&revealer);
    window.set_child(Some(&outer));

    let state = Rc::new(RefCell::new(UiState::default()));

    {
        let state = state.clone();
        let launcher = launcher.clone();
        let list = list.clone();
        let revealer = revealer.clone();
        entry.connect_changed(move |entry| {
            refresh_results(&launcher, entry, &list, &revealer, &state);
        });
    }

    {
        let launcher = launcher.clone();
        let state = state.clone();
        let entry = entry.clone();
        let window = window.clone();
        list.connect_row_activated(move |_, row| {
            let idx = row.index().max(0) as usize;
            let selected = state.borrow().results.get(idx).cloned();
            if let Some(item) = selected {
                match item {
                    ResultItem::App(app) => {
                        launcher.launch_app(&app.app);
                    }
                    ResultItem::Folder { path, .. } => {
                        std::process::Command::new("xdg-open")
                            .arg(path)
                            .spawn()
                            .ok();
                    }
                    ResultItem::File { path, .. } => {
                        std::process::Command::new("xdg-open")
                            .arg(path)
                            .spawn()
                            .ok();
                    }
                    ResultItem::WebSearch { query } => {
                        launcher.web_search(&query);
                    }
                }
                window.close();
            } else if launcher.web_search(entry.text().as_str()) {
                window.close();
            }
        });
    }

    {
        let launcher = launcher.clone();
        let state = state.clone();
        let list = list.clone();
        let entry = entry.clone();
        let entry_for_signal = entry.clone();
        let window = window.clone();
        entry_for_signal.connect_activate(move |_| {
            trigger_primary_action(&launcher, &state, &list, &entry, &window);
        });
    }

    let key_controller = gtk::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    {
        let launcher = launcher.clone();
        let state = state.clone();
        let list = list.clone();
        let entry = entry.clone();
        let window = window.clone();
        let scroller_clone = scroller.clone();
        key_controller.connect_key_pressed(move |_, key, _, mods| match key {
            gdk::Key::Escape => {
                window.close();
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Tab => {
                // Deselect text and move cursor to end, keeping the existing text.
                let len = entry.text().len() as i32;
                entry.select_region(len, len);
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Down => {
                move_selection(&list, &scroller_clone, &state, 1);
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Up => {
                move_selection(&list, &scroller_clone, &state, -1);
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Return => {
                if mods.contains(gdk::ModifierType::ALT_MASK) {
                    let q = entry.text().to_string();
                    if launcher.web_search(&q) {
                        window.close();
                    }
                    return gtk::glib::Propagation::Stop;
                }
                gtk::glib::Propagation::Proceed
            }
            _ => gtk::glib::Propagation::Proceed,
        });
    }
    window.add_controller(key_controller);

    // On re-open: keep previous text but select it all so one Backspace clears it.
    // Pressing Tab deselects and moves cursor to end.
    {
        let entry = entry.clone();
        let list = list.clone();
        let revealer = revealer.clone();
        let state = state.clone();
        let launcher = launcher.clone();
        window.connect_map(move |_| {
            entry.grab_focus();
            // Select all existing text so the user can clear it with one Backspace.
            entry.select_region(0, -1);
            // Refresh results in case the index has changed since last open.
            refresh_results(&launcher, &entry, &list, &revealer, &state);
        });
    }

    window.present();
}

#[cfg(feature = "layer-shell")]
fn setup_layer_shell(window: &gtk::ApplicationWindow) {
    if !layer_shell::is_supported() {
        return;
    }

    window.init_layer_shell();
    window.set_layer(layer_shell::Layer::Overlay);
    window.set_keyboard_mode(layer_shell::KeyboardMode::Exclusive);
    window.set_namespace("seekx");
    center_layer_shell(window, SEARCH_BOX_HEIGHT);
}

#[cfg(not(feature = "layer-shell"))]
fn setup_layer_shell(_window: &gtk::ApplicationWindow) {}

#[cfg(feature = "layer-shell")]
fn center_layer_shell(window: &gtk::ApplicationWindow, _height: i32) {
    let display = match gdk::Display::default() {
        Some(display) => display,
        None => return,
    };

    let monitor_from_surface = window
        .surface()
        .and_then(|surface| display.monitor_at_surface(&surface));

    let monitor = monitor_from_surface.or_else(|| {
        let monitors = display.monitors();
        monitors
            .item(0)
            .and_then(|obj| obj.downcast::<gdk::Monitor>().ok())
    });

    let Some(monitor) = monitor else {
        return;
    };

    let geometry = monitor.geometry();
    let left = ((geometry.width() - WINDOW_WIDTH) / 2).max(0);
    let top = (geometry.height() / 3).max(0);

    window.set_anchor(layer_shell::Edge::Left, true);
    window.set_anchor(layer_shell::Edge::Right, false);
    window.set_anchor(layer_shell::Edge::Top, true);
    window.set_anchor(layer_shell::Edge::Bottom, false);
    window.set_margin(layer_shell::Edge::Left, left);
    window.set_margin(layer_shell::Edge::Top, top);
}

#[cfg(not(feature = "layer-shell"))]
#[allow(dead_code)]
fn center_layer_shell(_window: &gtk::ApplicationWindow, _height: i32) {}

// fn trigger_primary_action(
//     launcher: &Launcher,
//     state: &Rc<RefCell<UiState>>,
//     list: &gtk::ListBox,
//     entry: &gtk::Entry,
//     window: &gtk::ApplicationWindow,
// ) {
//     if let Some(row) = list.selected_row() {
//         let idx = row.index().max(0) as usize;
//         if let Some(item) = state.borrow().results.get(idx).cloned() {
//             launcher.launch_app(&item.app);
//             window.close();
//             return;
//         }
//     }
//
//     let q = entry.text().to_string();
//     if launcher.web_search(&q) {
//         window.close();
//     }
// }
fn trigger_primary_action(
    launcher: &Launcher,
    state: &Rc<RefCell<UiState>>,
    list: &gtk::ListBox,
    entry: &gtk::Entry,
    window: &gtk::ApplicationWindow,
) {
    if let Some(row) = list.selected_row() {
        let idx = row.index().max(0) as usize;

        if let Some(item) = state.borrow().results.get(idx).cloned() {
            match item {
                ResultItem::App(app) => {
                    launcher.launch_app(&app.app);
                }
                ResultItem::Folder { path, .. } => {
                    std::process::Command::new("xdg-open")
                        .arg(path)
                        .spawn()
                        .ok();
                }
                ResultItem::File { path, .. } => {
                    std::process::Command::new("xdg-open")
                        .arg(path)
                        .spawn()
                        .ok();
                }
                ResultItem::WebSearch { query } => {
                    launcher.web_search(&query);
                }
            }

            window.close();
            return;
        }
    }

    let q = entry.text().to_string();
    if launcher.web_search(&q) {
        window.close();
    }
}

fn refresh_results(
    launcher: &Launcher,
    entry: &gtk::Entry,
    list: &gtk::ListBox,
    revealer: &gtk::Revealer,
    state: &Rc<RefCell<UiState>>,
) {
    let query = entry.text().to_string();
    let trimmed = query.trim();

    // let results: Vec<ResultItem> = if trimmed.starts_with('/') {
    //     launcher.rank_folders(trimmed, RESULT_LIMIT)
    // } else {
    //     launcher
    //         .rank(trimmed, RESULT_LIMIT)
    //         .into_iter()
    //         .map(ResultItem::App)
    //         .collect()
    // };
    //

    let results: Vec<ResultItem> = if trimmed.starts_with("//") {
        launcher.rank_files(trimmed, RESULT_LIMIT)
    } else if trimmed.starts_with("/") {
        launcher.rank_folders(trimmed, RESULT_LIMIT)
    } else {
        let mut apps: Vec<ResultItem> = launcher
            .rank(trimmed, RESULT_LIMIT.saturating_sub(1))
            .into_iter()
            .map(ResultItem::App)
            .collect();

        apps.push(ResultItem::WebSearch {
            query: trimmed.to_string(),
        });

        apps
    };

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    for result in &results {
        let row = gtk::ListBoxRow::new();
        row.add_css_class("seekx-row");

        let container_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        container_box.set_margin_start(4);
        container_box.set_margin_end(4);

        match result {
            ResultItem::App(app) => {
                if let Some(icon_name) = &app.app.icon {
                    let image = gtk::Image::builder()
                        .icon_name(icon_name)
                        .pixel_size(32)
                        .build();
                    container_box.append(&image);
                } else {
                    let image = gtk::Image::builder()
                        .icon_name("application-x-executable")
                        .pixel_size(32)
                        .build();
                    container_box.append(&image);
                }

                let label = gtk::Label::new(None);
                label.set_markup(&format_highlighted_label(&app.app.name, trimmed));
                label.set_xalign(0.0);
                label.add_css_class("seekx-label");
                container_box.append(&label);
            }

            ResultItem::Folder { name, .. } => {
                let image = gtk::Image::builder()
                    .icon_name("folder")
                    .pixel_size(32)
                    .build();

                container_box.append(&image);

                let label = gtk::Label::new(None);
                label.set_markup(&format_highlighted_label(name, trimmed));
                label.set_xalign(0.0);
                label.add_css_class("seekx-label");
                container_box.append(&label);
            }
            // ResultItem::File { name, .. } => {
            //     let image = gtk::Image::builder()
            //         .icon_name("text-x-generic")
            //         .pixel_size(32)
            //         .build();
            //
            //     container_box.append(&image);
            //
            //     let label = gtk::Label::new(Some(&name));
            //     label.set_xalign(0.0);
            //     label.add_css_class("seekx-label");
            //     container_box.append(&label);
            // }
            ResultItem::File { name, path } => {
                let image = gtk::Image::builder()
                    .icon_name("text-x-generic")
                    .pixel_size(32)
                    .build();

                container_box.append(&image);

                // vertical layout for name + path
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);

                // filename
                let name_label = gtk::Label::new(None);
                name_label.set_markup(&format_highlighted_label(name, trimmed));
                name_label.set_xalign(0.0);
                name_label.add_css_class("seekx-label");

                let home = std::env::var("HOME").unwrap_or_default();

                // extract parent folder
                let parent = std::path::Path::new(path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                // convert /home/user → ~
                let display_path = parent.replace(&home, "~");

                // path label
                let path_label = gtk::Label::new(Some(&display_path));
                path_label.set_xalign(0.0);
                path_label.add_css_class("seekx-path");
                path_label.set_ellipsize(gtk::pango::EllipsizeMode::Start);

                vbox.append(&name_label);
                vbox.append(&path_label);

                container_box.append(&vbox);
            }
            ResultItem::WebSearch { query } => {
                let image = gtk::Image::builder()
                    .icon_name("applications-internet")
                    .pixel_size(32)
                    .build();

                container_box.append(&image);

                let label_text = if query.is_empty() {
                    "Browse web".to_string()
                } else {
                    format!("Search web for '{}'", query)
                };

                let label = gtk::Label::new(Some(&label_text));
                label.set_xalign(0.0);
                label.add_css_class("seekx-label");
                label.add_css_class("seekx-web-label");
                container_box.append(&label);
            }
        }

        row.set_child(Some(&container_box));
        list.append(&row);
    }

    if let Some(row) = list.row_at_index(0) {
        list.select_row(Some(&row));
    }

    let show = !trimmed.is_empty() && !results.is_empty();
    revealer.set_reveal_child(show);

    state.borrow_mut().results = results;
}

fn move_selection(
    list: &gtk::ListBox,
    scroller: &gtk::ScrolledWindow,
    state: &Rc<RefCell<UiState>>,
    delta: i32,
) {
    let total = state.borrow().results.len();
    if total == 0 {
        return;
    }

    let current = list.selected_row().map(|row| row.index()).unwrap_or(0);
    let next = (current + delta).clamp(0, total.saturating_sub(1) as i32);
    if let Some(row) = list.row_at_index(next) {
        list.select_row(Some(&row));

        let adj = scroller.vadjustment();
        if let Some(bounds) = row.compute_bounds(list) {
            let y = bounds.y() as f64;
            let h = bounds.height() as f64;
            let val = adj.value();
            let page = adj.page_size();

            if val > y {
                adj.set_value(y);
            } else if val + page < y + h {
                adj.set_value(y + h - page);
            }
        }
    }
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(EMBEDDED_CSS);

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn format_highlighted_label(text: &str, query: &str) -> String {
    if query.is_empty() {
        return gtk::glib::markup_escape_text(text).to_string();
    }

    let escaped_text = gtk::glib::markup_escape_text(text);

    if text.to_lowercase().starts_with(&query.to_lowercase()) {
        let (prefix, suffix) = text.split_at(query.len());
        format!(
            "<span foreground='#FF8C00'>{}</span>{}",
            gtk::glib::markup_escape_text(prefix),
            gtk::glib::markup_escape_text(suffix)
        )
    } else {
        escaped_text.to_string()
    }
}
