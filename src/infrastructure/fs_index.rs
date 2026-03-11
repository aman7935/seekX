use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::thread;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use walkdir::WalkDir;

use crate::ui::ResultItem;

const IGNORED_DIRS: [&str; 8] = [
    "node_modules",
    "target",
    ".cache",
    ".git",
    ".local",
    ".npm",
    "venv",
    ".venv",
];

#[derive(Clone, Default)]
pub struct FileIndex {
    files: Arc<RwLock<Vec<(String, String)>>>,
}

impl FileIndex {
    pub fn new() -> Self {
        let files = Arc::new(RwLock::new(index_home_files()));
        let index = Self {
            files: files.clone(),
        };

        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            let files_for_watcher = files.clone();
            thread::spawn(move || {
                if let Err(e) = watch_home(home, files_for_watcher) {
                    eprintln!("Watcher error: {:?}", e);
                }
            });
        }

        index
    }

    pub fn rank_files(&self, query: &str, limit: usize) -> Vec<ResultItem> {
        let search = query.trim_start_matches("//").to_lowercase();
        let mut results = Vec::new();

        let files = if let Ok(f) = self.files.read() {
            f
        } else {
            return results;
        };

        for (name, path) in files.iter() {
            if name.to_lowercase().contains(&search) {
                results.push(ResultItem::File {
                    name: name.clone(),
                    path: path.clone(),
                });

                if results.len() >= limit {
                    break;
                }
            }
        }

        results
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
}

fn index_home_files() -> Vec<(String, String)> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut files = Vec::new();

    for entry in WalkDir::new(home)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();

            if name.starts_with('.') {
                return false;
            }

            !IGNORED_DIRS.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let path = entry.path().display().to_string();
            let name = entry.file_name().to_string_lossy().to_string();

            files.push((name, path));
        }
    }

    files
}

fn watch_home(home: String, files: Arc<RwLock<Vec<(String, String)>>>) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(Path::new(&home), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => handle_event(event, &files),
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn handle_event(event: Event, files: &Arc<RwLock<Vec<(String, String)>>>) {
    match event.kind {
        EventKind::Create(_) => {
            for path in event.paths {
                if is_valid_file(&path) {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let path_str = path.to_string_lossy().to_string();
                        if let Ok(mut f) = files.write() {
                            if !f.iter().any(|(_, p)| p == &path_str) {
                                f.push((name.to_string(), path_str));
                            }
                        }
                    }
                }
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                let path_str = path.to_string_lossy().to_string();
                if let Ok(mut f) = files.write() {
                    f.retain(|(_, p)| p != &path_str);
                }
            }
        }
        EventKind::Modify(modify_kind) => {
            if let notify::event::ModifyKind::Name(_) = modify_kind {
                // If it's a rename, we get two paths if it's within the same watch,
                // or we get two events. notify usually provides paths for both.
                // To keep it simple, if it's a rename and we have TWO paths, it's (from, to).
                // If we have ONE path, it's either the "from" or the "to" depending on the implementation.
                // However, most reliable way is Create/Remove.
                // Actually, notify 6.x Rename event often yields two paths.
                if event.paths.len() == 2 {
                    let from = &event.paths[0];
                    let to = &event.paths[1];

                    let from_str = from.to_string_lossy().to_string();
                    let to_str = to.to_string_lossy().to_string();

                    if let Ok(mut f) = files.write() {
                        f.retain(|(_, p)| p != &from_str);
                        if is_valid_file(to) {
                            if let Some(name) = to.file_name().and_then(|n| n.to_str()) {
                                f.push((name.to_string(), to_str));
                            }
                        }
                    }
                } else {
                    // Fallback: check if path exists. If not, remove. If yes, add/update.
                    for path in event.paths {
                        let path_str = path.to_string_lossy().to_string();
                        if path.exists() {
                            if is_valid_file(&path) {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    if let Ok(mut f) = files.write() {
                                        if !f.iter().any(|(_, p)| p == &path_str) {
                                            f.push((name.to_string(), path_str));
                                        }
                                    }
                                }
                            } else if let Ok(mut f) = files.write() {
                                f.retain(|(_, p)| p != &path_str);
                            }
                        } else if let Ok(mut f) = files.write() {
                            f.retain(|(_, p)| p != &path_str);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn is_valid_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            return false;
        }

        // Check if any parent component is ignored
        for component in path.components() {
            if let Some(c) = component.as_os_str().to_str() {
                if IGNORED_DIRS.contains(&c) {
                    return false;
                }
            }
        }
        true
    } else {
        false
    }
}
