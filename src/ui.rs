use eframe::egui;
use eframe::egui::{Color32, Key, RichText, Stroke, ViewportCommand};

use crate::launcher::{Launcher, RankedApp};

const RESULT_LIMIT: usize = 9;

pub struct SeekXApp {
    launcher: Launcher,
    query: String,
    selected: usize,
    cached_query: String,
    cached_results: Vec<RankedApp>,
    status_line: String,
    request_close: bool,
}

impl SeekXApp {
    pub fn new(launcher: Launcher) -> Self {
        let app_count = launcher.app_count();
        let status_line = format!("Indexed {app_count} apps. Enter to launch, Alt+Enter for web search, Esc to close.");

        Self {
            launcher,
            query: String::new(),
            selected: 0,
            cached_query: String::new(),
            cached_results: Vec::new(),
            status_line,
            request_close: false,
        }
    }

    fn refresh_results(&mut self) {
        if self.query == self.cached_query {
            return;
        }
        self.cached_results = self.launcher.rank(&self.query, RESULT_LIMIT);
        self.cached_query = self.query.clone();
        if self.selected >= self.cached_results.len() {
            self.selected = self.cached_results.len().saturating_sub(1);
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let mut launch_selected = false;
        let mut web_search = false;

        ctx.input(|input| {
            if input.key_pressed(Key::Escape) {
                self.request_close = true;
            }
            if input.key_pressed(Key::ArrowDown)
                || (input.key_pressed(Key::J) && input.modifiers.ctrl)
            {
                if !self.cached_results.is_empty() {
                    self.selected = (self.selected + 1).min(self.cached_results.len() - 1);
                }
            }
            if input.key_pressed(Key::ArrowUp)
                || (input.key_pressed(Key::K) && input.modifiers.ctrl)
            {
                self.selected = self.selected.saturating_sub(1);
            }
            if input.key_pressed(Key::Enter) {
                if input.modifiers.alt {
                    web_search = true;
                } else {
                    launch_selected = true;
                }
            }
        });

        if launch_selected {
            if let Some(selected) = self.cached_results.get(self.selected) {
                self.launcher.launch_app(&selected.app);
                self.request_close = true;
            }
        }

        if web_search {
            self.launcher.web_search(&self.query);
            self.request_close = true;
        }
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("seekX")
                    .size(28.0)
                    .color(Color32::from_rgb(38, 66, 99))
                    .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Rust app launcher")
                    .size(14.0)
                    .color(Color32::from_rgb(90, 100, 120)),
            );
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new(&self.status_line)
                .size(13.0)
                .color(Color32::from_rgb(100, 106, 122)),
        );
    }

    fn draw_search_box(&mut self, ui: &mut egui::Ui) {
        let search = ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .desired_width(f32::INFINITY)
                .hint_text("Type an app name or keyword..."),
        );

        search.request_focus();
    }

    fn draw_results(&mut self, ui: &mut egui::Ui) {
        if self.cached_results.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new("No app matches. Press Alt+Enter to search on the web.")
                    .size(14.0)
                    .color(Color32::from_rgb(132, 138, 150)),
            );
            return;
        }

        ui.add_space(8.0);
        for (idx, item) in self.cached_results.iter().enumerate() {
            let is_selected = idx == self.selected;
            let name_color = if is_selected {
                Color32::from_rgb(20, 30, 45)
            } else {
                Color32::from_rgb(46, 60, 82)
            };

            let bg = if is_selected {
                Color32::from_rgb(201, 224, 248)
            } else {
                Color32::from_rgb(243, 247, 252)
            };

            egui::Frame::default()
                .fill(bg)
                .stroke(Stroke::new(1.0, Color32::from_rgb(216, 225, 236)))
                .inner_margin(egui::Margin::same(10.0))
                .rounding(egui::Rounding::same(8.0))
                .show(ui, |ui| {
                    let text = if let Some(comment) = &item.app.comment {
                        format!("{}\n{}", item.app.name, comment)
                    } else {
                        item.app.name.clone()
                    };

                    let response = ui.selectable_label(
                        is_selected,
                        RichText::new(text).size(16.0).color(name_color),
                    );

                    if response.clicked() {
                        self.selected = idx;
                    }

                    if response.double_clicked() {
                        self.launcher.launch_app(&item.app);
                        self.request_close = true;
                    }
                });

            ui.add_space(6.0);
        }
    }
}

impl eframe::App for SeekXApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_results();
        self.handle_keys(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(Color32::from_rgb(236, 242, 250))
                    .inner_margin(egui::Margin::same(20.0))
                    .rounding(egui::Rounding::same(12.0)),
            )
            .show(ctx, |ui| {
                self.draw_header(ui);
                ui.add_space(12.0);
                self.draw_search_box(ui);
                self.draw_results(ui);
            });

        if self.request_close {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}
