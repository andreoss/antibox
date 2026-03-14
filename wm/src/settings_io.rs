use std::path::{Path, PathBuf};

pub enum SettingValue {
    Text(String),
    Int(i64),
    Bool(bool),
}

pub fn user_config_path() -> Option<PathBuf> {
    let file = crate::config_watch::CONFIG_FILE;
    for dir in crate::wmconfig::Config::search_dirs() {
        let path = dir.join(file);
        if path.is_file() {
            return Some(path);
        }
    }
    crate::wmconfig::Config::preferred_dir().map(|d| d.join(file))
}

pub fn save(values: &[(&str, &str, SettingValue)]) -> std::io::Result<PathBuf> {
    let path = user_config_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory")
    })?;
    save_to(&path, values)?;
    Ok(path)
}

pub fn save_to(path: &Path, values: &[(&str, &str, SettingValue)]) -> std::io::Result<()> {
    let mut root = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.parse::<toml::Value>().ok())
        .and_then(|v| match v {
            toml::Value::Table(t) => Some(t),
            _ => None,
        })
        .unwrap_or_default();
    for (section, key, value) in values {
        let entry = root
            .entry((*section).to_string())
            .or_insert_with(|| toml::Value::Table(toml::value::Table::default()));
        if !entry.is_table() {
            *entry = toml::Value::Table(toml::value::Table::default());
        }
        let Some(table) = entry.as_table_mut() else {
            continue;
        };
        let v = match value {
            SettingValue::Text(s) => toml::Value::String(s.clone()),
            SettingValue::Int(i) => toml::Value::Integer(*i),
            SettingValue::Bool(b) => toml::Value::Boolean(*b),
        };
        table.insert((*key).to_string(), v);
    }
    let text = toml::to_string(&toml::Value::Table(root))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, text)
}

#[cfg(test)]
#[path = "settings_io_tests.rs"]
mod tests;
