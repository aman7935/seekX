use std::path::Path;
use std::process::{Command, Stdio};

use crate::desktop::DesktopApp;
use crate::search;

const FIELD_CODES: [&str; 14] = [
    "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m", "%",
];

#[derive(Clone)]
pub struct Launcher {
    apps: Vec<DesktopApp>,
}

#[derive(Clone)]
pub struct RankedApp {
    pub app: DesktopApp,
    pub score: i64,
    pub match_idx: usize,
}

impl Launcher {
    pub fn new(apps: Vec<DesktopApp>) -> Self {
        Self { apps }
    }

    pub fn app_count(&self) -> usize {
        self.apps.len()
    }

    pub fn rank(&self, query: &str, limit: usize) -> Vec<RankedApp> {
        let q = query.trim();

        if q.is_empty() {
            return self
                .apps
                .iter()
                .take(limit)
                .cloned()
                .map(|app| RankedApp {
                    app,
                    score: 0,
                    match_idx: 0,
                })
                .collect();
        }

        let mut ranked: Vec<RankedApp> = self
            .apps
            .iter()
            .filter_map(|app| {
                let score = search::score(q, &app.search_text)?;
                Some(RankedApp {
                    app: app.clone(),
                    score: score.score,
                    match_idx: score.start_idx,
                })
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.match_idx.cmp(&b.match_idx))
                .then_with(|| a.app.name.to_lowercase().cmp(&b.app.name.to_lowercase()))
        });

        ranked.truncate(limit);
        ranked
    }

    pub fn launch_app(&self, app: &DesktopApp) {
        let parts = parse_exec(&app.exec);
        if parts.is_empty() {
            return;
        }

        let mut cmd = Command::new(&parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        let _ = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    pub fn web_search(&self, query: &str) -> bool {
        let q = query.trim();
        if q.is_empty() {
            return false;
        }

        let url = format!(
            "https://duckduckgo.com/?q={}",
            urlencoding::encode(q).into_owned()
        );

        if try_spawn("firefox", &["--new-tab", &url]) {
            return true;
        }

        if try_spawn("google-chrome", &[&url]) {
            return true;
        }

        if try_spawn("chromium", &[&url]) {
            return true;
        }

        if try_spawn("chromium-browser", &[&url]) {
            return true;
        }

        if try_spawn("brave-browser", &[&url]) {
            return true;
        }

        if webbrowser::open(&url).is_ok() {
            return true;
        }

        if try_spawn("gio", &["open", &url]) {
            return true;
        }

        if try_spawn("xdg-open", &[&url]) {
            return true;
        }

        if try_spawn("sensible-browser", &[&url]) {
            return true;
        }

        false
    }
}

fn try_spawn(command: &str, args: &[&str]) -> bool {
    if which::which(command).is_err() {
        return false;
    }

    Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

fn parse_exec(exec_line: &str) -> Vec<String> {
    let parts = match shlex::split(exec_line) {
        Some(parts) => parts,
        None => return Vec::new(),
    };

    parts
        .into_iter()
        .filter(|part| !FIELD_CODES.contains(&part.as_str()) && !part.starts_with('%'))
        .collect()
}

#[allow(dead_code)]
fn executable_name(executable: &str) -> String {
    Path::new(executable)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(executable)
        .to_string()
}
