use std::path::PathBuf;

pub struct DesktopApp {
    pub name: String,
    pub command: Vec<String>,
    pub categories: Vec<String>,
}

fn xdg_app_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    match std::env::var_os("XDG_DATA_HOME").filter(|s| !s.is_empty()) {
        Some(home) => dirs.push(PathBuf::from(home).join("applications")),
        None => {
            if let Some(home) = std::env::var_os("HOME") {
                dirs.push(PathBuf::from(home).join(".local/share/applications"));
            }
        }
    }
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
    for d in data_dirs.split(':').filter(|s| !s.is_empty()) {
        dirs.push(PathBuf::from(d).join("applications"));
    }
    dirs
}

fn expand_field_codes(tok: &str, name: &str, icon: &str) -> String {
    if !tok.contains('%') {
        return tok.to_string();
    }
    let mut out = String::with_capacity(tok.len());
    let mut chars = tok.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('%') => out.push('%'),
            Some('c') => out.push_str(name),
            Some('i') => out.push_str(icon),
            Some(_) | None => {}
        }
    }
    out
}

fn shell_split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut had = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quote = !in_quote;
                had = true;
            }
            '\\' if in_quote => {
                if let Some(&next) = chars.peek() {
                    cur.push(next);
                    chars.next();
                }
            }
            c if c.is_whitespace() && !in_quote => {
                if had {
                    out.push(std::mem::take(&mut cur));
                    had = false;
                }
            }
            c => {
                cur.push(c);
                had = true;
            }
        }
    }
    if had {
        out.push(cur);
    }
    out
}

fn clean_exec(exec: &str, name: &str, icon: &str) -> Vec<String> {
    let mut tokens = shell_split(exec);
    for tok in &mut tokens {
        *tok = expand_field_codes(tok, name, icon);
    }
    tokens.retain(|t| !t.is_empty());
    tokens
}

fn parse_desktop(content: &str) -> Option<DesktopApp> {
    let mut in_entry = false;
    let (mut name, mut exec, mut icon, mut categories) = (
        String::new(),
        String::new(),
        String::new(),
        Vec::new(),
    );
    let mut typ = String::new();
    let (mut terminal, mut hidden) = (false, false);
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            if !in_entry && !name.is_empty() {
                break;
            }
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "Type" => typ = value.to_string(),
            "Name" => name = value.to_string(),
            "Exec" => exec = value.to_string(),
            "Icon" => icon = value.to_string(),
            "Categories" => {
                categories = value
                    .split(';')
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect();
            }
            "Terminal" => terminal = value.eq_ignore_ascii_case("true"),
            "NoDisplay" | "Hidden" if value.eq_ignore_ascii_case("true") => {
                hidden = true;
            }
            "OnlyShowIn" => hidden = true,
            _ => {}
        }
    }
    if hidden || name.is_empty() || exec.is_empty() {
        return None;
    }
    if !typ.is_empty() && typ != "Application" {
        return None;
    }
    let mut tokens = clean_exec(&exec, &name, &icon);
    if tokens.is_empty() {
        return None;
    }
    if terminal {
        let mut wrapped = vec!["xterm".to_string(), "-e".to_string()];
        wrapped.append(&mut tokens);
        tokens = wrapped;
    }
    Some(DesktopApp {
        name,
        command: tokens,
        categories,
    })
}

pub const OTHER_SECTION: &str = "Other";

pub fn section_for(app: &DesktopApp) -> &str {
    match app.categories.first() {
        Some(c) => c.as_str(),
        None => OTHER_SECTION,
    }
}

pub fn grouped(apps: &[DesktopApp]) -> Vec<(String, Vec<&DesktopApp>)> {
    let mut out: Vec<(String, Vec<&DesktopApp>)> = Vec::new();
    for a in apps {
        let key = section_for(a).to_string();
        match out.iter_mut().find(|(k, _)| *k == key) {
            Some(e) => e.1.push(a),
            None => out.push((key, vec![a])),
        }
    }
    out.sort_by(|a, b| match (a.0 == OTHER_SECTION, b.0 == OTHER_SECTION) {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) => a.0.cmp(&b.0),
    });
    out
}

pub fn scan() -> Vec<DesktopApp> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut apps: Vec<DesktopApp> = Vec::new();
    for dir in xdg_app_dirs() {
        let Ok(read) = std::fs::read_dir(&dir) else { continue };
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            let id = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if !seen.insert(id) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            if let Some(app) = parse_desktop(&content) {
                apps.push(app);
            }
        }
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

#[cfg(test)]
#[path = "desktop_apps_tests.rs"]
mod tests;
