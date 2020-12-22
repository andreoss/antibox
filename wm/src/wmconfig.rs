use crate::option::WindowOptions;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prefs {
    pub font: FontPrefs,
    pub workspace: WorkspacePrefs,
    pub keyboard: KeyboardPrefs,
    pub cpu: GraphPrefs,
    pub mem: GraphPrefs,
    pub net: NetPrefs,
    pub keys: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontPrefs {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePrefs {
    pub count: u32,
    pub layouts: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardPrefs {
    pub layouts: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphPrefs {
    pub width: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetPrefs {
    pub width: u16,
    pub device: String,
}

impl Default for Prefs {
    fn default() -> Prefs {
        Prefs {
            font: FontPrefs {
                name: "-misc-fixed-medium-r-semicondensed--13-*-*-*-*-*-iso10646-1".to_string(),
            },
            workspace: WorkspacePrefs {
                count: 4,
                layouts: "floating".to_string(),
            },
            keyboard: KeyboardPrefs {
                layouts: String::new(),
            },
            cpu: GraphPrefs { width: 40 },
            mem: GraphPrefs { width: 40 },
            net: NetPrefs {
                width: 40,
                device: "*".to_string(),
            },
            keys: Vec::new(),
        }
    }
}

const DEFAULTS_INI: &str = include_str!("../../defaults.ini");

pub fn default_prefs() -> Prefs {
    parse_prefs(DEFAULTS_INI)
}

pub fn parse_prefs(text: &str) -> Prefs {
    let mut p = Prefs::default();
    apply_prefs(&mut p, text);
    p
}

pub fn apply_prefs(p: &mut Prefs, text: &str) {
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
        let raw_key = line[..eq].trim();
        let key = raw_key.to_ascii_lowercase();
        let raw = line[eq + 1..].trim();
        let value = if raw.starts_with('"') {
            match raw[1..].find('"') {
                Some(end) => &raw[1..1 + end],
                None => raw.trim_matches('"'),
            }
        } else {
            raw.split(|c| c == '#' || c == ';')
                .next()
                .unwrap_or("")
                .trim()
        };
        if section == "keys" {
            let combo = raw_key.to_string();
            let action = value.to_string();
            match p
                .keys
                .iter_mut()
                .find(|(c, _)| c.eq_ignore_ascii_case(&combo))
            {
                Some(entry) => entry.1 = action,
                None => p.keys.push((combo, action)),
            }
            continue;
        }
        match (section.as_str(), key.as_str()) {
            ("font", "name") => p.font.name = value.to_string(),
            ("workspace", "count") => {
                if let Ok(v) = value.parse::<u32>() {
                    if v >= 1 && v <= 32 {
                        p.workspace.count = v;
                    }
                }
            }
            ("workspace", "layouts") => p.workspace.layouts = value.to_string(),
            ("keyboard", "layouts") => p.keyboard.layouts = value.to_string(),
            ("cpu", "width") => {
                if let Ok(v) = value.parse::<u16>() {
                    if v >= 8 && v <= 220 {
                        p.cpu.width = v;
                    }
                }
            }
            ("mem", "width") => {
                if let Ok(v) = value.parse::<u16>() {
                    if v >= 8 && v <= 220 {
                        p.mem.width = v;
                    }
                }
            }
            ("net", "width") => {
                if let Ok(v) = value.parse::<u16>() {
                    if v >= 8 && v <= 220 {
                        p.net.width = v;
                    }
                }
            }
            ("net", "device") => p.net.device = value.to_string(),
            _ => {}
        }
    }
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
        let mut p = default_prefs();
        for d in Self::search_dirs() {
            if let Ok(s) = std::fs::read_to_string(d.join("config.ini")) {
                apply_prefs(&mut p, &s);
                break;
            }
        }
        p
    }

    pub fn keyboard_layouts() -> Vec<String> {
        Self::load_prefs()
            .keyboard
            .layouts
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
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
