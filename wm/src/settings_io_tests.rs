use super::*;

fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("antibox-settings-{}-{}", std::process::id(), name))
}

#[test]
fn test_save_creates_sections() {
    let p = tmp("create.toml");
    let _ = std::fs::remove_file(&p);
    save_to(
        &p,
        &[
            ("workspace", "count", SettingValue::Int(6)),
            ("pointer", "warp", SettingValue::Bool(true)),
            ("font", "name", SettingValue::Text("fixed".into())),
        ],
    )
    .unwrap();
    let v: toml::Value = std::fs::read_to_string(&p).unwrap().parse().unwrap();
    assert_eq!(v["workspace"]["count"].as_integer(), Some(6));
    assert_eq!(v["pointer"]["warp"].as_bool(), Some(true));
    assert_eq!(v["font"]["name"].as_str(), Some("fixed"));
    let _ = std::fs::remove_file(&p);
}

#[test]
fn test_save_preserves_existing_keys() {
    let p = tmp("preserve.toml");
    std::fs::write(&p, "[keys]\n\"Alt+Z\" = \"Close\"\n[workspace]\ncount = 2\n").unwrap();
    save_to(&p, &[("workspace", "count", SettingValue::Int(8))]).unwrap();
    let v: toml::Value = std::fs::read_to_string(&p).unwrap().parse().unwrap();
    assert_eq!(v["keys"]["Alt+Z"].as_str(), Some("Close"));
    assert_eq!(v["workspace"]["count"].as_integer(), Some(8));
    let _ = std::fs::remove_file(&p);
}

#[test]
fn test_saved_file_survives_apply_prefs() {
    let p = tmp("apply.toml");
    let _ = std::fs::remove_file(&p);
    save_to(
        &p,
        &[
            ("workspace", "count", SettingValue::Int(5)),
            ("winlist", "position", SettingValue::Text("pointer".into())),
            ("ticker", "enabled", SettingValue::Bool(false)),
        ],
    )
    .unwrap();
    let text = std::fs::read_to_string(&p).unwrap();
    let mut prefs = crate::wmconfig::default_prefs();
    crate::wmconfig::apply_prefs(&mut prefs, &text);
    assert_eq!(prefs.workspace.count, 5);
    assert_eq!(prefs.winlist.position, "pointer");
    assert!(!prefs.ticker.enabled);
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_fresh_home_saves_under_xdg() {
    let dir = std::env::temp_dir().join(format!("antibox-fresh-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(".config/antibox/config.toml");
    save_to(&path, &[("theme", "name", SettingValue::Text("kde".into()))]).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("kde"), "settings must land in the xdg path");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_saving_never_targets_a_system_dir() {
    let user = crate::wmconfig::Config::user_config_dirs();
    let system = antibox_core::paths::system_config_dirs();
    for d in &user {
        assert!(
            !system.contains(d),
            "writable user dir {d:?} must not be a read-only system dir"
        );
    }
    let all = crate::wmconfig::Config::search_dirs();
    for d in &system {
        assert!(all.contains(d), "search still falls back to system dirs for reading");
    }
}
