use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use walkdir::WalkDir;

use crate::domain::DesktopApp;

pub fn load_installed_apps() -> Vec<DesktopApp> {
    let mut apps = Vec::new();
    let mut seen = HashSet::new();

    for dir in app_dirs() {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir)
            .min_depth(1)
            .max_depth(3)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() && !entry.file_type().is_symlink() {
                continue;
            }
            if entry.path().extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }

            let Some(app) = parse_desktop_file(entry.path()) else {
                continue;
            };

            let dedupe_key = format!("{}:{}", app.name.to_lowercase(), app.exec.to_lowercase());
            if seen.insert(dedupe_key) {
                apps.push(app);
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

pub fn app_dirs() -> Vec<PathBuf> {
    let mut out = Vec::<PathBuf>::new();
    let mut seen = HashSet::<PathBuf>::new();

    if let Some(home) = dirs::home_dir() {
        for path in [
            home.join(".local/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            home.join(".local/share/flatpak/exports/share/applications"),
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        ] {
            if seen.insert(path.clone()) {
                out.push(path);
            }
        }
    } else {
        for path in [
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        ] {
            if seen.insert(path.clone()) {
                out.push(path);
            }
        }
    }

    // Include XDG dirs as extras, but don't rely on them.
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        let path = PathBuf::from(data_home).join("applications");
        if seen.insert(path.clone()) {
            out.push(path);
        }
    }
    if let Some(data_dirs) = std::env::var_os("XDG_DATA_DIRS").filter(|v| !v.is_empty()) {
        for base in std::env::split_paths(&data_dirs) {
            let path = base.join("applications");
            if seen.insert(path.clone()) {
                out.push(path);
            }
        }
    }

    out
}

pub fn watch_apps(apps: Arc<RwLock<Vec<DesktopApp>>>) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    for dir in app_dirs() {
        if dir.exists() {
            let _ = watcher.watch(&dir, RecursiveMode::Recursive);
        }
    }

    for res in rx {
        match res {
            Ok(event) => handle_app_event(event, &apps),
            Err(e) => eprintln!("app watch error: {:?}", e),
        }
    }

    Ok(())
}

fn handle_app_event(event: Event, apps: &Arc<RwLock<Vec<DesktopApp>>>) {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some(app) = parse_desktop_file(&path) {
                        if let Ok(mut a) = apps.write() {
                            // Deduplicate by name and exec
                            let dedupe_key =
                                format!("{}:{}", app.name.to_lowercase(), app.exec.to_lowercase());

                            // Remove existing if any (to update)
                            a.retain(|existing| {
                                let key = format!(
                                    "{}:{}",
                                    existing.name.to_lowercase(),
                                    existing.exec.to_lowercase()
                                );
                                key != dedupe_key
                            });

                            a.push(app);
                            a.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                        }
                    }
                }
            }
        }
        EventKind::Remove(_) => {
            // Removing accurately is tricky because we don't have the file anymore to parse it.
            // We'd need a mapping of path -> app.
            // For now, let's just trigger a full reload on removal to be safe,
            // or we could store the path in DesktopApp.
            // Let's improve DesktopApp later. For now, full reload is safer and apps are few.
            if let Ok(mut a) = apps.write() {
                *a = load_installed_apps();
            }
        }
        _ => {}
    }
}

fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
    let contents = fs::read_to_string(path).ok()?;
    let section = parse_desktop_entry_section(&contents)?;

    if !section
        .get("Type")
        .map(|v| v.eq_ignore_ascii_case("Application"))
        .unwrap_or(false)
    {
        return None;
    }

    if section
        .get("Hidden")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return None;
    }

    if section
        .get("NoDisplay")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return None;
    }

    let name = section.get("Name")?.trim().to_string();
    let exec = section.get("Exec")?.trim().to_string();
    if name.is_empty() || exec.is_empty() {
        return None;
    }

    let generic_name = section
        .get("GenericName")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let icon = section
        .get("Icon")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let wm_class = section
        .get("StartupWMClass")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut search_parts = vec![name.clone()];
    if let Some(generic_name) = &generic_name {
        search_parts.push(generic_name.clone());
    }
    search_parts.push(exec.clone());
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if !stem.is_empty() {
            search_parts.push(stem.to_string());
        }
    }
    if let Some(wm_class) = &wm_class {
        search_parts.push(wm_class.clone());
    }
    if let Some(categories) = section.get("Categories") {
        search_parts.extend(
            categories
                .split(';')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
        );
    }
    if let Some(keywords) = section.get("Keywords") {
        search_parts.extend(
            keywords
                .split(';')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
        );
    }

    let mut normalized_parts = Vec::new();
    for term in &search_parts {
        let compact = compact_alnum(term);
        if !compact.is_empty() {
            normalized_parts.push(compact);
        }
    }

    let search_terms = search_parts
        .iter()
        .map(|v| v.to_lowercase())
        .collect::<Vec<_>>();

    Some(DesktopApp {
        name,
        exec,
        icon,
        search_terms,
        normalized_terms: normalized_parts,
    })
}

fn compact_alnum(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

fn parse_desktop_entry_section(contents: &str) -> Option<HashMap<String, String>> {
    let mut in_desktop_entry = false;
    let mut map = HashMap::new();

    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        map.insert(key.trim().to_string(), value.trim().to_string());
    }

    if map.is_empty() { None } else { Some(map) }
}
