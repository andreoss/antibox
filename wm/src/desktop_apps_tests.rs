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
