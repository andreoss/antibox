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
    pub clock: ClockPrefs,
    pub pointer: PointerPrefs,
    pub graph: GraphColourPrefs,
    pub winlist: WinlistPrefs,
    pub tabs: TabsPrefs,
    pub ticker: TickerPrefs,
    pub taskbar: TaskbarPrefs,
    pub theme: ThemePrefs,
    pub keys: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePrefs {
    pub name: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockPrefs {
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerPrefs {
    pub warp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphColourPrefs {
    pub series: String,
    pub heat: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WinlistPrefs {
    pub position: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsPrefs {
    pub position: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickerPrefs {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskbarPrefs {
    pub layout: String,
    pub menu_on_super_tap: bool,
}

pub const DEFAULT_TASKBAR_LAYOUT: &str = "pager tasks cpu mem net power keyboard tray clock";

impl Default for Prefs {
    fn default() -> Self {
        Self {
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
            clock: ClockPrefs {
                format: "%H:%M:%S".to_string(),
            },
            pointer: PointerPrefs { warp: false },
            graph: GraphColourPrefs {
                series: "000080,000080,7F7FBF,808080".to_string(),
                heat: "C82020".to_string(),
            },
            winlist: WinlistPrefs {
                position: "centre".to_string(),
            },
            tabs: TabsPrefs {
                position: "top".to_string(),
            },
            ticker: TickerPrefs { enabled: true },
            theme: ThemePrefs {
                name: "nt".to_string(),
            },
            taskbar: TaskbarPrefs {
                layout: DEFAULT_TASKBAR_LAYOUT.to_string(),
                menu_on_super_tap: true,
            },
            keys: Vec::new(),
        }
    }
}

pub const WINOPTIONS_FILE: &str = "winoptions";

const DEFAULTS_TOML: &str = include_str!("../../defaults.toml");

pub fn default_prefs() -> Prefs {
    parse_prefs(DEFAULTS_TOML)
}

pub fn parse_prefs(text: &str) -> Prefs {
    let mut p = Prefs::default();
    apply_prefs(&mut p, text);
    p
}

pub fn split_layout_list(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub fn workspaces_from(prefs: &Prefs) -> (u32, Vec<String>) {
    let count = prefs.workspace.count.max(1);
    (count, parse_workspace_names("", count as usize))
}

pub fn parse_colour(value: &str) -> Option<u32> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()
}

pub fn parse_colour_list(value: &str) -> Vec<u32> {
    value
        .split([',', ' '].as_ref())
        .filter(|s| !s.is_empty())
        .filter_map(parse_colour)
        .collect()
}

fn parse_width(value: &toml::Value) -> Option<u16> {
    let width = u16::try_from(value.as_integer()?).ok()?;
    if (8..=220).contains(&width) {
        Some(width)
    } else {
        None
    }
}

fn as_count(value: &toml::Value) -> Option<u32> {
    u32::try_from(value.as_integer()?).ok()
}

pub fn apply_prefs(p: &mut Prefs, text: &str) {
    let Ok(toml::Value::Table(doc)) = toml::from_str::<toml::Value>(text) else { return };
    for (section, entries) in &doc {
        let toml::Value::Table(entries) = entries else { continue };
        if section.eq_ignore_ascii_case("keys") {
            for (combo, value) in entries {
                let action = match value.as_str() {
                    Some(action) => action.to_string(),
                    None => continue,
                };
                match p
                    .keys
                    .iter_mut()
                    .find(|(c, _)| c.eq_ignore_ascii_case(combo))
                {
                    Some(entry) => entry.1 = action,
                    None => p.keys.push((combo.clone(), action)),
                }
            }
            continue;
        }
        let section = section.to_ascii_lowercase();
        for (key, value) in entries {
            let key = key.to_ascii_lowercase();
            match (section.as_str(), key.as_str()) {
                ("font", "name") => {
                    if let Some(v) = value.as_str() {
                        p.font.name = v.to_string();
                    }
                }
                ("workspace", "count") => {
                    if let Some(v) = as_count(value) {
                        if (1..=32).contains(&v) {
                            p.workspace.count = v;
                        }
                    }
                }
                ("workspace", "layouts") => {
                    if let Some(v) = value.as_str() {
                        p.workspace.layouts = v.to_string();
                    }
                }
                ("keyboard", "layouts") => {
                    if let Some(v) = value.as_str() {
                        p.keyboard.layouts = v.to_string();
                    }
                }
                ("cpu", "width") => {
                    if let Some(v) = parse_width(value) {
                        p.cpu.width = v;
                    }
                }
                ("mem", "width") => {
                    if let Some(v) = parse_width(value) {
                        p.mem.width = v;
                    }
                }
                ("net", "width") => {
                    if let Some(v) = parse_width(value) {
                        p.net.width = v;
                    }
                }
                ("net", "device") => {
                    if let Some(v) = value.as_str() {
                        p.net.device = v.to_string();
                    }
                }
                ("clock", "format") => {
                    if let Some(v) = value.as_str() {
                        if !v.is_empty() {
                            p.clock.format = v.to_string();
                        }
                    }
                }
                ("graph", "series") => {
                    if let Some(v) = value.as_str() {
                        if !parse_colour_list(v).is_empty() {
                            p.graph.series = v.to_string();
                        }
                    }
                }
                ("graph", "heat") => {
                    if let Some(v) = value.as_str() {
                        if parse_colour(v).is_some() {
                            p.graph.heat = v.to_string();
                        }
                    }
                }
                ("pointer", "warp") => {
                    if let Some(v) = value.as_bool() {
                        p.pointer.warp = v;
                    }
                }
                ("winlist", "position") => {
                    if let Some(v) = value.as_str() {
                        match v {
                            "centre" | "pointer" => p.winlist.position = v.to_string(),
                            _ => {}
                        }
                    }
                }
                ("tabs", "position") => {
                    if let Some(v) = value.as_str() {
                        match v {
                            "top" | "bottom" => p.tabs.position = v.to_string(),
                            _ => {}
                        }
                    }
                }
                ("taskbar", "menu_on_super_tap") => {
                    if let Some(v) = value.as_bool() {
                        p.taskbar.menu_on_super_tap = v;
                    }
                }
                ("taskbar", "layout") => {
                    if let Some(v) = value.as_str() {
                        if !v.trim().is_empty() {
                            p.taskbar.layout = v.to_string();
                        }
                    }
                }
                ("ticker", "enabled") => {
                    if let Some(v) = value.as_bool() {
                        p.ticker.enabled = v;
                    }
                }
                ("theme", "name") => {
                    if let Some(v) = value.as_str() {
                        if !v.trim().is_empty() {
                            p.theme.name = v.trim().to_string();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub struct Config;

impl Config {
    pub fn load_winoptions() -> WindowOptions {
        let mut opts = WindowOptions::new();
        for d in Self::search_dirs() {
            if let Ok(s) = std::fs::read_to_string(d.join(WINOPTIONS_FILE)) {
                apply_winoptions(&mut opts, &s);
                break;
            }
        }
        opts
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
            if let Ok(s) = std::fs::read_to_string(d.join(crate::config_watch::CONFIG_FILE)) {
                apply_prefs(&mut p, &s);
                break;
            }
        }
        p
    }
}

pub fn apply_winoptions(opts: &mut WindowOptions, text: &str) {
    for line in text.lines() {
        let line = match line.find('#') {
            Some(i) => &line[..i],
            None => line,
        }
        .trim();
        if line.is_empty() {
            continue;
        }
        let (lhs, value) = match line.find(':') {
            Some(i) => (line[..i].trim(), line[i + 1..].trim()),
            None => continue,
        };
        let (class_instance, opt) = match lhs.rfind('.') {
            Some(i) => (lhs[..i].trim(), lhs[i + 1..].trim()),
            None => continue,
        };
        if class_instance.is_empty() || opt.is_empty() {
            continue;
        }
        opts.set_win_option(class_instance, opt, value);
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
