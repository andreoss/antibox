use super::*;

#[test]
fn test_clean_exec_strips_field_codes() {
    assert_eq!(clean_exec("firefox %u", "Firefox", ""), vec!["firefox"]);
    assert_eq!(clean_exec("gimp-2.10 %U", "GIMP", ""), vec!["gimp-2.10"]);
    assert_eq!(
        clean_exec("app --flag %F --name %c", "My App", ""),
        vec!["app", "--flag", "--name", "My App"]
    );
    assert_eq!(clean_exec("foo 100%%", "n", ""), vec!["foo", "100%"]);
}

#[test]
fn test_shell_split_honours_quotes() {
    assert_eq!(
        shell_split("sh -c \"echo hi there\""),
        vec!["sh", "-c", "echo hi there"]
    );
}

#[test]
fn test_parse_desktop_basic() {
    let app = parse_desktop(
        "[Desktop Entry]\nType=Application\nName=Alacritty\nExec=alacritty\nIcon=Alacritty\n",
    )
    .unwrap();
    assert_eq!(app.name, "Alacritty");
    assert_eq!(app.command, vec!["alacritty"]);
}

#[test]
fn test_parse_desktop_hidden_and_nondisplay_skipped() {
    assert!(
        parse_desktop("[Desktop Entry]\nType=Application\nName=X\nExec=x\nNoDisplay=true\n")
            .is_none()
    );
    assert!(
        parse_desktop("[Desktop Entry]\nType=Application\nName=X\nExec=x\nHidden=true\n").is_none()
    );
    assert!(
        parse_desktop("[Desktop Entry]\nType=Application\nName=X\nExec=x\nOnlyShowIn=KDE;\n")
            .is_none()
    );
    assert!(parse_desktop("[Desktop Entry]\nType=Link\nName=X\nURL=http://x\n").is_none());
    assert!(parse_desktop("[Desktop Entry]\nType=Application\nName=X\n").is_none());
}

#[test]
fn test_parse_desktop_terminal_wrapped() {
    let app =
        parse_desktop("[Desktop Entry]\nType=Application\nName=Vim\nExec=vim\nTerminal=true\n")
            .unwrap();
    assert_eq!(app.command, vec!["xterm", "-e", "vim"]);
}

#[test]
fn test_parse_desktop_stops_at_second_group() {
    let app = parse_desktop(
        "[Desktop Entry]\nType=Application\nName=Term\nExec=alacritty\n\n[Desktop Action New]\nName=New\nExec=alacritty --new\n",
    )
    .unwrap();
    assert_eq!(app.command, vec!["alacritty"]);
}

#[test]
fn test_parse_desktop_reads_categories() {
    let app = parse_desktop(
        "[Desktop Entry]\nType=Application\nName=Browser\nExec=firefox\nCategories=Network;WebBrowser;\n",
    )
    .unwrap();
    assert_eq!(app.categories, vec!["Network", "WebBrowser"]);
}

#[test]
fn test_section_for_uses_the_first_category() {
    let app = |cats: &[&str]| DesktopApp {
        name: "x".to_string(),
        command: vec!["x".to_string()],
        categories: cats.iter().map(|c| c.to_string()).collect(),
    };
    assert_eq!(section_for(&app(&["Network", "WebBrowser"])), "Network");
    assert_eq!(section_for(&app(&["Game"])), "Game");
    assert_eq!(section_for(&app(&[])), OTHER_SECTION);
}

#[test]
fn test_grouped_buckets_by_category_and_sorts_sections() {
    let a = |name: &str, cats: &[&str]| DesktopApp {
        name: name.to_string(),
        command: vec![name.to_string()],
        categories: cats.iter().map(|c| c.to_string()).collect(),
    };
    let apps = vec![
        a("Zed", &["Utility"]),
        a("Firefox", &["Network"]),
        a("Mines", &["Game"]),
        a("NoCat", &[]),
        a("Alacritty", &["Utility"]),
    ];
    let g = grouped(&apps);
    let got: Vec<(&str, Vec<&str>)> = g
        .iter()
        .map(|(k, v)| (k.as_str(), v.iter().map(|x| x.name.as_str()).collect()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("Game", vec!["Mines"]),
            ("Network", vec!["Firefox"]),
            ("Utility", vec!["Zed", "Alacritty"]),
            (OTHER_SECTION, vec!["NoCat"]),
        ]
    );
}
