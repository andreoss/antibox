use crate::option::WindowOptions;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prefs {
    pub font: FontPrefs,
    pub workspace: WorkspacePrefs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontPrefs {
    pub name: String,
    pub size: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePrefs {
    pub count: u32,
}

impl Default for Prefs {
    fn default() -> Prefs {
        Prefs {
            font: FontPrefs {
                name: String::new(),
                size: 0,
            },
            workspace: WorkspacePrefs { count: 4 },
        }
    }
}

pub fn parse_prefs(text: &str) -> Prefs {
    let mut p = Prefs::default();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            continue;
        }
        let eq = match line.find('=') {
            Some(v) => v,
            None => continue,
        };
        let key = line[..eq].trim().to_ascii_lowercase();
        let raw = line[eq + 1..].trim();
        let value = if raw.starts_with('"') {
            match raw[1..].find('"') {
                Some(end) => &raw[1..1 + end],
                None => raw.trim_matches('"'),
            }
        } else {
            raw.split(|c| c == '#' || c == ';').next().unwrap_or("").trim()
        };
        match (section.as_str(), key.as_str()) {
            ("font", "name") => p.font.name = value.to_string(),
            ("font", "size") => {
                if let Ok(v) = value.parse::<u16>() {
                    p.font.size = v;
                }
            }
            ("workspace", "count") => {
                if let Ok(v) = value.parse::<u32>() {
                    if v >= 1 && v <= 32 {
                        p.workspace.count = v;
                    }
                }
            }
            _ => {}
        }
    }
    p
}

pub struct Config;

impl Config {
    pub fn load_winoptions() -> WindowOptions {
        WindowOptions::default()
    }

    pub fn search_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(priv_cfg) = std::env::var_os("ANTIBOX_PRIVCFG") {
            dirs.push(PathBuf::from(priv_cfg));
        } else if let Some(xdg_home) = std::env::var_os("XDG_CONFIG_HOME") {
            dirs.push(PathBuf::from(xdg_home).join("antibox"));
        } else if let Ok(home) = std::env::var("HOME") {
            let xdg_config = PathBuf::from(&home).join(".config/antibox");
            if xdg_config.is_dir() {
                dirs.push(xdg_config);
            }
            dirs.push(PathBuf::from(home).join(".antibox"));
        }

        dirs.extend(antibox_core::paths::system_config_dirs());
        dirs
    }

    pub fn load_prefs() -> Prefs {
        for d in Self::search_dirs() {
            if let Ok(s) = std::fs::read_to_string(d.join("config.toml")) {
                return parse_prefs(&s);
            }
        }
        Prefs::default()
    }

    pub fn workspace_layouts_pref() -> String {
        for d in Self::search_dirs() {
            if let Ok(s) = std::fs::read_to_string(d.join("workspace_layouts")) {
                let t = s.trim().to_string();
                if !t.is_empty() {
                    return t;
                }
            }
        }
        String::new()
    }

    pub fn keyboard_layouts() -> Vec<String> {
        for d in Self::search_dirs() {
            if let Ok(s) = std::fs::read_to_string(d.join("keyboard_layouts")) {
                let list: Vec<String> = s
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                if !list.is_empty() {
                    return list;
                }
            }
        }
        Vec::new()
    }
}

pub fn parse_workspace_name_list(pref: &str) -> Vec<String> {
    let sep = if pref.contains(',') { ',' } else { ':' };
    pref.split(sep)
        .map(|s| s.trim().trim_matches('"').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn parse_workspace_names(pref: &str, count: usize) -> Vec<String> {
    let mut names = parse_workspace_name_list(pref);
    names.truncate(count);
    while names.len() < count {
        names.push(format!("Workspace {}", names.len() + 1));
    }
    names
}

#[cfg(test)]
#[path = "wmconfig_tests.rs"]
mod tests;
