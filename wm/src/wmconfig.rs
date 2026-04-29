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
    pub pager: PagerPrefs,
    pub tabs: TabsPrefs,
    pub ticker: TickerPrefs,
    pub taskbar: TaskbarPrefs,
    pub omni: OmniPrefs,
    pub launch: LaunchPrefs,
    pub expose: ExposePrefs,
    pub theme: ThemePrefs,
    pub keys: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePrefs {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPrefs {
    pub items: String,
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
    pub preview: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagerPrefs {
    pub preview: bool,
    pub style: String,
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
    pub position: String,
    pub preview: bool,
    pub width: i64,
    pub rows: i64,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmniPrefs {
    pub preview: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExposePrefs {
    pub modes: String,
    pub mouse1: String,
    pub mouse2: String,
    pub mouse3: String,
    pub opacity: u8,
    pub tint: String,
    pub tint_opacity: u8,
    pub highlight_opacity: u8,
    pub highlight_tint: String,
    pub highlight_tint_opacity: u8,
    pub tooltip: bool,
    pub tooltip_follows_mouse: bool,
    pub tooltip_offset_x: i32,
    pub tooltip_offset_y: i32,
    pub tooltip_align: String,
    pub all_monitors: bool,
    pub live_thumbnails: bool,
    pub warp_on_show: bool,
    pub warp_on_select: bool,
    pub warp_on_activate: bool,
    pub distance: i32,
    pub icon_size: u16,
    pub allow_upscale: bool,
    pub all_desktops: bool,
    pub show_minimized: bool,
    pub show_skip_taskbar: bool,
    pub include_frame: bool,
}

impl Default for ExposePrefs {
    fn default() -> Self {
        Self {
            modes: DEFAULT_EXPOSE_MODES.to_string(),
            mouse1: "focus".to_string(),
            mouse2: "close".to_string(),
            mouse3: "iconify".to_string(),
            opacity: 150,
            tint: "000000".to_string(),
            tint_opacity: 0,
            highlight_opacity: 255,
            highlight_tint: "000000".to_string(),
            highlight_tint_opacity: 0,
            tooltip: true,
            tooltip_follows_mouse: true,
            tooltip_offset_x: 20,
            tooltip_offset_y: 20,
            tooltip_align: "left".to_string(),
            all_monitors: true,
            live_thumbnails: true,
            warp_on_show: false,
            warp_on_select: false,
            warp_on_activate: false,
            distance: 50,
            icon_size: 48,
            allow_upscale: false,
            all_desktops: true,
            show_minimized: true,
            show_skip_taskbar: false,
            include_frame: true,
        }
    }
}

pub const DEFAULT_EXPOSE_MODES: &str = "thumbnail icon filled none";

pub const DEFAULT_TASKBAR_LAYOUT: &str =
    "menu launch pager tasks cpu mem net power mpris keyboard tray clock showdesktop";

impl Default for Prefs {
    fn default() -> Self {
        Self {
            font: FontPrefs {
                name: "sans-serif".to_string(),
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
                preview: true,
            },
            pager: PagerPrefs {
                preview: true,
                style: "buttons".to_string(),
            },
            tabs: TabsPrefs {
                position: "top".to_string(),
            },
            ticker: TickerPrefs { enabled: false },
            theme: ThemePrefs {
                name: "nt".to_string(),
            },
            taskbar: TaskbarPrefs {
                layout: DEFAULT_TASKBAR_LAYOUT.to_string(),
                position: "bottom".to_string(),
                preview: true,
                width: 120,
                rows: 1,
                locked: true,
            },
            omni: OmniPrefs { preview: true },
            launch: LaunchPrefs { items: String::new() },
            expose: ExposePrefs::default(),
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

fn as_bounded(value: &toml::Value, min: i32, max: i32) -> Option<i32> {
    let v = i32::try_from(value.as_integer()?).ok()?;
    (min..=max).contains(&v).then_some(v)
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
                ("taskbar", "layout") => {
                    if let Some(v) = value.as_str() {
                        if !v.trim().is_empty() {
                            p.taskbar.layout = v.to_string();
                        }
                    }
                }
                ("expose", "modes") => {
                    if let Some(v) = value.as_str() {
                        if !v.trim().is_empty() {
                            p.expose.modes = v.to_string();
                        }
                    }
                }
                ("expose", "mouse1") => {
                    if let Some(v) = value.as_str() {
                        p.expose.mouse1 = v.to_string();
                    }
                }
                ("expose", "mouse2") => {
                    if let Some(v) = value.as_str() {
                        p.expose.mouse2 = v.to_string();
                    }
                }
                ("expose", "mouse3") => {
                    if let Some(v) = value.as_str() {
                        p.expose.mouse3 = v.to_string();
                    }
                }
                ("expose", "opacity") => {
                    if let Some(v) = as_bounded(value, 0, 255) {
                        p.expose.opacity = v as u8;
                    }
                }
                ("expose", "tint") => {
                    if let Some(v) = value.as_str() {
                        if parse_colour(v).is_some() {
                            p.expose.tint = v.to_string();
                        }
                    }
                }
                ("expose", "tint_opacity") => {
                    if let Some(v) = as_bounded(value, 0, 255) {
                        p.expose.tint_opacity = v as u8;
                    }
                }
                ("expose", "highlight_opacity") => {
                    if let Some(v) = as_bounded(value, 0, 255) {
                        p.expose.highlight_opacity = v as u8;
                    }
                }
                ("expose", "highlight_tint") => {
                    if let Some(v) = value.as_str() {
                        if parse_colour(v).is_some() {
                            p.expose.highlight_tint = v.to_string();
                        }
                    }
                }
                ("expose", "highlight_tint_opacity") => {
                    if let Some(v) = as_bounded(value, 0, 255) {
                        p.expose.highlight_tint_opacity = v as u8;
                    }
                }
                ("expose", "tooltip") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.tooltip = v;
                    }
                }
                ("expose", "tooltip_follows_mouse") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.tooltip_follows_mouse = v;
                    }
                }
                ("expose", "tooltip_offset_x") => {
                    if let Some(v) = as_bounded(value, -400, 400) {
                        p.expose.tooltip_offset_x = v;
                    }
                }
                ("expose", "tooltip_offset_y") => {
                    if let Some(v) = as_bounded(value, -400, 400) {
                        p.expose.tooltip_offset_y = v;
                    }
                }
                ("expose", "tooltip_align") => {
                    if let Some(v) = value.as_str() {
                        match v {
                            "left" | "mid" | "right" => p.expose.tooltip_align = v.to_string(),
                            _ => {}
                        }
                    }
                }
                ("expose", "live_thumbnails") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.live_thumbnails = v;
                    }
                }
                ("expose", "all_monitors") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.all_monitors = v;
                    }
                }
                ("expose", "warp_on_show") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.warp_on_show = v;
                    }
                }
                ("expose", "warp_on_select") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.warp_on_select = v;
                    }
                }
                ("expose", "warp_on_activate") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.warp_on_activate = v;
                    }
                }
                ("expose", "distance") => {
                    if let Some(v) = as_bounded(value, 1, 400) {
                        p.expose.distance = v;
                    }
                }
                ("expose", "icon_size") => {
                    if let Some(v) = as_bounded(value, 8, 256) {
                        p.expose.icon_size = v as u16;
                    }
                }
                ("expose", "allow_upscale") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.allow_upscale = v;
                    }
                }
                ("expose", "all_desktops") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.all_desktops = v;
                    }
                }
                ("expose", "show_minimized") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.show_minimized = v;
                    }
                }
                ("expose", "show_skip_taskbar") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.show_skip_taskbar = v;
                    }
                }
                ("expose", "include_frame") => {
                    if let Some(v) = value.as_bool() {
                        p.expose.include_frame = v;
                    }
                }
                ("taskbar", "position") => {
                    if let Some(v) = value.as_str() {
                        if crate::taskbar::TaskBarPosition::parse(v).is_some() {
                            p.taskbar.position = v.trim().to_ascii_lowercase();
                        }
                    }
                }
                ("taskbar", "preview") => {
                    if let Some(v) = value.as_bool() {
                        p.taskbar.preview = v;
                    }
                }
                ("taskbar", "width") => {
                    if let Some(v) = value.as_integer() {
                        p.taskbar.width = v.clamp(0, 1000);
                    }
                }
                ("taskbar", "rows") => {
                    if let Some(v) = value.as_integer() {
                        p.taskbar.rows = v.clamp(1, 4);
                    }
                }
                ("taskbar", "locked") => {
                    if let Some(v) = value.as_bool() {
                        p.taskbar.locked = v;
                    }
                }
                ("winlist", "preview") => {
                    if let Some(v) = value.as_bool() {
                        p.winlist.preview = v;
                    }
                }
                ("pager", "preview") => {
                    if let Some(v) = value.as_bool() {
                        p.pager.preview = v;
                    }
                }
                ("pager", "style") => {
                    if let Some(v) = value.as_str() {
                        p.pager.style = v.trim().to_ascii_lowercase();
                    }
                }
                ("omni", "preview") => {
                    if let Some(v) = value.as_bool() {
                        p.omni.preview = v;
                    }
                }
                ("launch", "items") => {
                    if let Some(v) = value.as_str() {
                        p.launch.items = v.to_string();
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

    pub fn preferred_dir() -> Option<PathBuf> {
        if let Some(priv_cfg) = std::env::var_os("ANTIBOX_PRIVCFG") {
            return Some(PathBuf::from(priv_cfg));
        }
        if let Some(xdg_home) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg_home).join("antibox"));
        }
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".config/antibox"))
    }

    pub fn user_config_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(priv_cfg) = std::env::var_os("ANTIBOX_PRIVCFG") {
            dirs.push(PathBuf::from(priv_cfg));
        } else if let Some(xdg_home) = std::env::var_os("XDG_CONFIG_HOME") {
            dirs.push(PathBuf::from(xdg_home).join("antibox"));
        } else if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(&home).join(".config/antibox"));
            dirs.push(PathBuf::from(home).join(".antibox"));
        }

        dirs
    }

    pub fn search_dirs() -> Vec<PathBuf> {
        let mut dirs = Self::user_config_dirs();
        dirs.extend(antibox_core::paths::system_config_dirs());
        dirs
    }

    pub fn load_prefs() -> Prefs {
        let mut p = default_prefs();
        let system = antibox_core::paths::system_config_dirs();
        for d in Self::search_dirs() {
            if !system.contains(&d) {
                migrate_legacy_config(&d);
            }
            if let Ok(s) = std::fs::read_to_string(d.join(crate::config_watch::CONFIG_FILE)) {
                apply_prefs(&mut p, &s);
                break;
            }
        }
        p
    }
}

pub(crate) fn migrate_legacy_config(dir: &std::path::Path) {
    let current = dir.join(crate::config_watch::CONFIG_FILE);
    let legacy = dir.join(crate::config_watch::LEGACY_CONFIG_FILE);
    if !current.is_file() && legacy.is_file() {
        let _ = std::fs::rename(&legacy, &current);
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
