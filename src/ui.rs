use std::cell::RefCell;
use std::rc::Rc;

use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;
#[cfg(feature = "layer-shell")]
use gtk4_layer_shell as layer_shell;
#[cfg(feature = "layer-shell")]
use gtk4_layer_shell::LayerShell;

use crate::launcher::{Launcher, RankedApp};

const RESULT_LIMIT: usize = 9;
const WINDOW_WIDTH: i32 = 580;
const SEARCH_BOX_HEIGHT: i32 = 62;
const RESULTS_AREA_HEIGHT: i32 = 200;
const ANIMATION_MS: u32 = 200;

#[derive(Default)]
struct UiState {
    results: Vec<RankedApp>,
}

pub fn run(launcher: Launcher) {
    let app = gtk::Application::builder()
        .application_id("com.seekx.launcher")
        .build();

    app.connect_activate(move |app| {
        build_ui(app, launcher.clone());
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
        .placeholder_text("Search apps or web")
        .hexpand(true)
        .build();
    entry.add_css_class("seekx-entry");
    search_box.append(&entry);

    // ── Results box ──
    let results_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    results_box.add_css_class("seekx-results-box");

    let status = gtk::Label::new(Some("Matches: 0"));
    status.set_xalign(0.0);
    status.add_css_class("seekx-status");

    let list = gtk::ListBox::new();
    list.add_css_class("seekx-list");
    list.set_selection_mode(gtk::SelectionMode::Single);
    list.set_activate_on_single_click(false);

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .child(&list)
        .build();
    scroller.set_has_frame(false);
    scroller.set_min_content_height(RESULTS_AREA_HEIGHT);
    scroller.set_max_content_height(RESULTS_AREA_HEIGHT);
    scroller.add_css_class("seekx-scroll");

    results_box.append(&status);
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
        let status = status.clone();
        let revealer = revealer.clone();
        entry.connect_changed(move |entry| {
            refresh_results(
                &launcher,
                entry,
                &list,
                &status,
                &revealer,
                &state,
            );
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
                launcher.launch_app(&item.app);
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
        key_controller.connect_key_pressed(move |_, key, _, mods| match key {
            gdk::Key::Escape => {
                window.close();
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Down => {
                move_selection(&list, &state, 1);
                gtk::glib::Propagation::Stop
            }
            gdk::Key::Up => {
                move_selection(&list, &state, -1);
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

    refresh_results(
        &launcher,
        &entry,
        &list,
        &status,
        &revealer,
        &state,
    );
    window.present();
    entry.grab_focus();
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
            launcher.launch_app(&item.app);
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
    status: &gtk::Label,
    revealer: &gtk::Revealer,
    state: &Rc<RefCell<UiState>>,
) {
    let query = entry.text().to_string();
    let trimmed = query.trim();
    let results = launcher.rank(trimmed, RESULT_LIMIT);

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    for result in &results {
        let row = gtk::ListBoxRow::new();
        row.add_css_class("seekx-row");

        let container_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        container_box.set_margin_start(4);
        container_box.set_margin_end(4);

        if let Some(icon_name) = &result.app.icon {
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
        let label = gtk::Label::new(Some(&result.app.name));

        label.set_xalign(0.0);
        label.add_css_class("seekx-label");
        container_box.append(&label);
        row.set_child(Some(&container_box));
        list.append(&row);
    }

    // status.set_text(&format!(
    //     "Installed: {} | Matches: {}",
    //     launcher.app_count(),
    //     results.len()
    // ));

    if let Some(row) = list.row_at_index(0) {
        list.select_row(Some(&row));
    }

    // Animate the results box in/out
    let show = !results.is_empty() || !trimmed.is_empty();
    revealer.set_reveal_child(show);

    state.borrow_mut().results = results;
}

fn move_selection(list: &gtk::ListBox, state: &Rc<RefCell<UiState>>, delta: i32) {
    let total = state.borrow().results.len();
    if total == 0 {
        return;
    }

    let current = list.selected_row().map(|row| row.index()).unwrap_or(0);
    let next = (current + delta).clamp(0, total.saturating_sub(1) as i32);
    if let Some(row) = list.row_at_index(next) {
        list.select_row(Some(&row));
    }
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
window.seekx-window,
window.seekx-window.background,
window.seekx-window > * {
  background-color: transparent;
  background: none;
}

*,
*:focus,
*:focus-visible,
*:selected {
  outline: none;
  box-shadow: none;
}

.seekx-outer {
  background-color: transparent;
  background: none;
}

.seekx-search-box {
  background-color: #0e0e14;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  padding: 10px 18px;
}

.seekx-results-box {
  background-color: #0e0e14;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  padding: 10px 16px;
}

entry.seekx-entry,
entry.seekx-entry text {
  background: transparent;
  color: #e4e2ec;
  border: none;
  border-radius: 0;
  font-size: 18px;
  font-weight: 500;
  box-shadow: none;
  outline: none;
}

entry.seekx-entry {
  min-height: 40px;
  padding: 0 4px;
}

entry.seekx-entry:focus {
  outline: none;
  box-shadow: none;
  border: none;
}

entry.seekx-entry:focus-visible,
row.seekx-row:focus,
row.seekx-row:focus-visible,
list.seekx-list:focus,
list.seekx-list:focus-visible,
scrolledwindow.seekx-scroll:focus,
scrolledwindow.seekx-scroll:focus-visible {
  outline: none;
  box-shadow: none;
}

scrolledwindow.seekx-scroll,
scrolledwindow.seekx-scroll > viewport,
scrolledwindow.seekx-scroll > viewport > * {
  background: transparent;
  border: none;
  box-shadow: none;
}

scrolledwindow.seekx-scroll scrollbar {
  background: transparent;
  border: none;
}

scrolledwindow.seekx-scroll scrollbar slider {
  background-color: rgba(255, 255, 255, 0.08);
  border-radius: 99px;
  min-width: 4px;
  min-height: 24px;
}

scrolledwindow.seekx-scroll scrollbar slider:hover {
  background-color: rgba(255, 255, 255, 0.16);
}

list.seekx-list {
  background: transparent;
  border: none;
}

row.seekx-row {
  background-color: transparent;
  border: none;
  border-radius: 8px;
  margin-top: 1px;
  margin-bottom: 1px;
  padding: 8px 10px;
}

row.seekx-row:hover {
  background-color: rgba(255, 255, 255, 0.04);
}

row.seekx-row:selected {
  background-color: rgba(124, 110, 240, 0.15);
  border: none;
}

row.seekx-row:selected:hover {
  background-color: rgba(124, 110, 240, 0.22);
}

label.seekx-label {
  color: rgba(228, 226, 236, 0.9);
  font-size: 14px;
  font-weight: 400;
}

row.seekx-row:selected label.seekx-label {
  color: #d8d4f5;
  font-weight: 500;
}

label.seekx-status {
  color: rgba(180, 175, 200, 0.45);
  font-size: 11px;
  font-weight: 300;
  padding-top: 2px;
  padding-bottom: 4px;
  padding-left: 4px;
}
",
    );

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
