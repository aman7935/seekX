use std::sync::{Arc, RwLock};
use std::thread;

use crate::domain::{DesktopApp, score};
use crate::infrastructure::browser;
use crate::infrastructure::desktop;
use crate::infrastructure::fs_index::FileIndex;
use crate::settings;
use crate::ui::ResultItem;

#[derive(Clone)]
pub struct Launcher {
    apps: Arc<RwLock<Vec<DesktopApp>>>,
    file_index: FileIndex,
    search_template: Option<String>,
}

#[derive(Clone)]
pub struct RankedApp {
    pub app: DesktopApp,
    pub score: i64,
    pub match_idx: usize,
}

impl Launcher {
    pub fn new(apps: Vec<DesktopApp>) -> Self {
        let file_index = FileIndex::new();
        let search_template = settings::search_template_from_env().ok().flatten();
        let apps = Arc::new(RwLock::new(apps));

        let l_apps = apps.clone();
        thread::spawn(move || {
            if let Err(e) = desktop::watch_apps(l_apps) {
                eprintln!("App watcher error: {:?}", e);
            }
        });

        Self {
            apps,
            file_index,
            search_template,
        }
    }

    pub fn search_template(&self) -> Option<&str> {
        self.search_template.as_deref()
    }

    pub fn rank_files(&self, query: &str, limit: usize) -> Vec<ResultItem> {
        self.file_index.rank_files(query, limit)
    }

    pub fn rank_folders(&self, query: &str, limit: usize) -> Vec<ResultItem> {
        self.file_index.rank_folders(query, limit)
    }

    pub fn rank(&self, query: &str, limit: usize) -> Vec<RankedApp> {
        let q = query.trim();

        let apps_lock = if let Ok(a) = self.apps.read() {
            a
        } else {
            return Vec::new();
        };

        if q.is_empty() {
            return apps_lock
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

        let mut ranked: Vec<RankedApp> = apps_lock
            .iter()
            .filter_map(|app| {
                let score = score(q, &app.search_terms, &app.normalized_terms)?;
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
        let parts = browser::parse_exec(&app.exec);
        if parts.is_empty() {
            return;
        }

        let mut cmd = std::process::Command::new(&parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        let _ = cmd
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    pub fn web_search(&self, query: &str) -> bool {
        let q = query.trim();
        if q.is_empty() {
            return false;
        }

        let url = if browser::looks_like_url(q) {
            browser::normalize_url(q)
        } else {
            browser::build_search_url(q, self.search_template.as_deref())
        };

        browser::open_default_browser(&url)
    }

    pub fn get_suggestions(&self, query: &str) -> Vec<ResultItem> {
        crate::infrastructure::suggestions::get_suggestions(query)
            .into_iter()
            .map(|res| {
                if let Some(url) = res.url {
                    ResultItem::QuickLink {
                        name: res.text,
                        url,
                        icon: "applications-internet".to_string(),
                    }
                } else {
                    ResultItem::Suggestion { text: res.text }
                }
            })
            .collect()
    }

    pub fn get_quicklinks(&self, _query: &str) -> Vec<ResultItem> {
        // QuickLinks are now dynamic and handled within get_suggestions
        Vec::new()
    }
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
            std::env::remove_var("SEEKX_SEARCH_URL_TEMPLATE");
        }
        let launcher = Launcher::new(vec![]);
        assert!(launcher.search_template().is_none());
        assert_eq!(
            browser::build_search_url("hello world", None),
            "https://duckduckgo.com/?q=hello%20world"
        );
    }

    #[test]
    fn env_template_overrides_default() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            std::env::set_var(
                "SEEKX_SEARCH_URL_TEMPLATE",
                "https://example.com/search?q={query}",
            );
        }
        let launcher = Launcher::new(vec![]);
        assert_eq!(
            browser::build_search_url("rust lang", launcher.search_template()),
            "https://example.com/search?q=rust%20lang"
        );
        unsafe {
            std::env::remove_var("SEEKX_SEARCH_URL_TEMPLATE");
        }
    }
}
