use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;

use crate::launcher::{Launcher, RankedApp};

const RESULT_LIMIT: usize = 9;
const WINDOW_WIDTH: i32 = 760;
const WINDOW_HEIGHT_COLLAPSED: i32 = 84;
const WINDOW_HEIGHT_EXPANDED: i32 = 360;

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
        .default_height(WINDOW_HEIGHT_COLLAPSED)
        .resizable(false)
        .decorated(false)
        .build();
    window.add_css_class("seekx-window");
    window.set_modal(true);
    window.set_hide_on_close(true);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    container.add_css_class("seekx-root");

    let entry = gtk::Entry::builder()
        .placeholder_text("Search apps or web")
        .hexpand(true)
        .build();
    entry.add_css_class("seekx-entry");

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
    scroller.set_visible(false);
    scroller.add_css_class("seekx-scroll");

    container.append(&entry);
    container.append(&scroller);
    window.set_child(Some(&container));

    let state = Rc::new(RefCell::new(UiState::default()));

    {
        let state = state.clone();
        let launcher = launcher.clone();
        let list = list.clone();
        let scroller = scroller.clone();
        let window = window.clone();
        entry.connect_changed(move |entry| {
            refresh_results(&launcher, entry, &list, &scroller, &window, &state);
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

    refresh_results(&launcher, &entry, &list, &scroller, &window, &state);
    window.present();
    entry.grab_focus();
}

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
    scroller: &gtk::ScrolledWindow,
    window: &gtk::ApplicationWindow,
    state: &Rc<RefCell<UiState>>,
) {
    let query = entry.text().to_string();
    let trimmed = query.trim();
    let results = if trimmed.is_empty() {
        Vec::new()
    } else {
        launcher.rank(trimmed, RESULT_LIMIT)
    };

    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    for result in &results {
        let row = gtk::ListBoxRow::new();
        row.add_css_class("seekx-row");

        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let icon = app_icon_widget(result.app.icon.as_deref());
        row_box.append(&icon);

        let label = gtk::Label::new(Some(&result.app.name));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.add_css_class("seekx-label");
        row_box.append(&label);

        row.set_child(Some(&row_box));
        list.append(&row);
    }

    if let Some(row) = list.row_at_index(0) {
        list.select_row(Some(&row));
    }

    let has_results = !results.is_empty();
    scroller.set_visible(has_results);
    if has_results {
        window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT_EXPANDED);
    } else {
        window.set_default_size(WINDOW_WIDTH, WINDOW_HEIGHT_COLLAPSED);
    }

    state.borrow_mut().results = results;
}

fn app_icon_widget(icon: Option<&str>) -> gtk::Image {
    let image = if let Some(icon_name) = icon {
        if Path::new(icon_name).is_absolute() && Path::new(icon_name).exists() {
            gtk::Image::from_file(icon_name)
        } else {
            gtk::Image::from_icon_name(icon_name)
        }
    } else {
        gtk::Image::from_icon_name("application-x-executable")
    };

    image.set_pixel_size(20);
    image.add_css_class("seekx-icon");
    image
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
window.seekx-window > * {
  background: #000000;
}

.seekx-root {
  background: #000000;
  border: 0.5px solid #ffffff;
  border-radius: 10px;
  padding: 16px;
}

entry.seekx-entry,
entry.seekx-entry text {
  background: transparent;
  color: #ffffff;
  border: none;
  border-radius: 0;
  font-size: 17px;
  box-shadow: none;
  outline: none;
}

entry.seekx-entry {
  min-height: 40px;
  padding: 0;
}

entry.seekx-entry:focus {
  outline: none;
  box-shadow: none;
  border: none;
}

scrolledwindow.seekx-scroll,
scrolledwindow.seekx-scroll > viewport,
scrolledwindow.seekx-scroll > viewport > * {
  background: transparent;
  border: none;
  box-shadow: none;
}

list.seekx-list {
  background: transparent;
  border: none;
}

row.seekx-row {
  background: transparent;
  border: none;
  border-radius: 0;
  margin-top: 4px;
  margin-bottom: 4px;
  padding: 0;
}

row.seekx-row:selected {
  background: transparent;
  border: none;
}

label.seekx-label {
  color: #ffffff;
  font-size: 15px;
}

image.seekx-icon {
  color: #ffffff;
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
