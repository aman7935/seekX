use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::desktop::DesktopApp;
use crate::search;
use crate::ui::ResultItem;

const FIELD_CODES: [&str; 14] = [
    "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m", "%",
];

const SEARCH_URL_TEMPLATE_ENV: &str = "SEEKX_SEARCH_URL_TEMPLATE";
const DEFAULT_SEARCH_URL_TEMPLATE: &str = "https://duckduckgo.com/?q={query}";

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

    pub fn rank_folders(&self, query: &str, limit: usize) -> Vec<ResultItem> {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = PathBuf::from(home);

        let search = query.trim_start_matches('/').to_lowercase();

        let mut results = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.to_lowercase().contains(&search) {
                            results.push(ResultItem::Folder {
                                name: name.to_string(),
                                path: path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }

                if results.len() >= limit {
                    break;
                }
            }
        }

        results
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
                let score = search::score(q, &app.search_terms, &app.normalized_terms)?;
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

    let url = if looks_like_url(q) {
        normalize_url(q)
    } else {
        build_search_url(q)
    };

    if open_in_default_browser_new_window(&url) {
        focus_browser_window();
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

fn open_in_default_browser_new_window(url: &str) -> bool {
    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output();

    let Ok(output) = output else { return false };

    let browser = String::from_utf8_lossy(&output.stdout)
        .trim()
        .replace(".desktop", "");

    if browser.is_empty() {
        return false;
    }

    Command::new(browser)
        .args(["--new-window", url])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

fn build_search_url_from_env(query: &str) -> Option<String> {
    let template = std::env::var(SEARCH_URL_TEMPLATE_ENV).ok()?;
    let encoded = urlencoding::encode(query).into_owned();

    if template.contains("{query}") {
        return Some(template.replace("{query}", &encoded));
    }

    if template.contains("%s") {
        return Some(template.replace("%s", &encoded));
    }

    Some(format!("{template}{encoded}"))
}

fn build_search_url(query: &str) -> String {
    if let Ok(template) = std::env::var(SEARCH_URL_TEMPLATE_ENV) {
        if !template.trim().is_empty() {
            if let Some(url) = build_search_url_from_env(query) {
                return url;
            }
        }
    }

    let encoded = urlencoding::encode(query).into_owned();
    DEFAULT_SEARCH_URL_TEMPLATE.replace("{query}", &encoded)
}

fn looks_like_url(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    if input.contains(char::is_whitespace) {
        return false;
    }

    let lower = input.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return true;
    }

    if lower.starts_with("www.") {
        return true;
    }

    // Basic heuristic: host-like strings (domain, localhost, or IP) without spaces.
    if lower == "localhost" || lower.starts_with("localhost:") {
        return true;
    }

    let is_ipv4ish = lower
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ':');
    if is_ipv4ish && (lower.contains('.') || lower.contains(':')) {
        return true;
    }

    // domain.tld[/...]
    lower.contains('.') && !lower.starts_with('.')
}




fn focus_browser_window() {
    let output = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output();

    let Ok(output) = output else { return };

    let browser = String::from_utf8_lossy(&output.stdout)
        .trim()
        .replace(".desktop", "");

    if browser.is_empty() {
        return;
    }

    let _ = Command::new("niri")
        .args([
            "msg",
            "action",
            "focus-window",
            "--app-id",
            &browser,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}






fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return trimmed.to_string();
    }

    format!("https://{trimmed}")
}

#[allow(dead_code)]
fn executable_name(executable: &str) -> String {
    Path::new(executable)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(executable)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn default_web_search_url_is_used_when_env_missing() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            std::env::remove_var(SEARCH_URL_TEMPLATE_ENV);
        }
        let url = build_search_url("hello world");
        assert_eq!(url, "https://duckduckgo.com/?q=hello%20world");
    }

    #[test]
    fn env_template_overrides_default() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            std::env::set_var(
                SEARCH_URL_TEMPLATE_ENV,
                "https://example.com/search?q={query}",
            );
        }
        let url = build_search_url("rust lang");
        assert_eq!(url, "https://example.com/search?q=rust%20lang");
        unsafe {
            std::env::remove_var(SEARCH_URL_TEMPLATE_ENV);
        }
    }
}
