use crate::option::WindowOptions;
use std::path::PathBuf;

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
