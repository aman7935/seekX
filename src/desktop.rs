use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct DesktopApp {
    pub name: String,
    pub exec: String,
    pub comment: Option<String>,
    pub search_text: String,
}

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
            if !entry.file_type().is_file() {
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

fn app_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    dirs
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

    let comment = section
        .get("Comment")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut search_parts = vec![name.clone(), exec.clone()];
    if let Some(comment) = &comment {
        search_parts.push(comment.clone());
    }
    if let Some(categories) = section.get("Categories") {
        search_parts.push(categories.clone());
    }
    if let Some(keywords) = section.get("Keywords") {
        search_parts.push(keywords.clone());
    }

    let search_text = search_parts.join(" ").to_lowercase();

    Some(DesktopApp {
        name,
        exec,
        comment,
        search_text,
    })
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

    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}
