use super::*;
use antibox_core::mock::MockDisplay;

fn conn() -> Arc<dyn DisplayBackend> {
    Arc::new(MockDisplay::new(1280, 800, 24))
}

fn item(title: &str, class: &str, run: bool) -> OmniItem {
    OmniItem {
        title: title.to_string(),
        class: class.to_string(),
        client_id: 0,
        icon_normal: PixmapData::new(0, 0, Vec::new()),
        icon_selected: PixmapData::new(0, 0, Vec::new()),
        run,
        command: Vec::new(),
    }
}

#[test]
fn test_lcp_extends_to_shared_prefix() {
    assert_eq!(longest_common_prefix(&["firefox", "firejail"]), "fire");
    assert_eq!(longest_common_prefix(&["gimp"]), "gimp");
    assert_eq!(longest_common_prefix(&["cat", "dog"]), "");
    assert_eq!(longest_common_prefix(&["make", "makepkg"]), "make");
    assert_eq!(longest_common_prefix(&[]), "");
}

#[test]
fn test_command_line_unwraps_shell_history_and_joins_argv() {
    let mut hist = item("ls -la", "ls -la", false);
    hist.command = vec!["sh".into(), "-c".into(), "ls -la".into()];
    assert_eq!(Omni::command_line(&hist), "ls -la");
    let mut app = item("Terminal", "xterm", false);
    app.command = vec!["xterm".into(), "-fg".into(), "grey".into()];
    assert_eq!(Omni::command_line(&app), "xterm -fg grey");
}

#[test]
fn test_right_arrow_completion_puts_the_selected_command_into_the_bar() {
    let c = conn();
    let rconn: Arc<dyn RenderBackend> = Arc::new(MockDisplay::new(1280, 800, 24));
    let mut o = Omni::new();
    o.bar = SearchBar::new(&rconn, 1, 0, 0, 200, 24).ok();
    let mut app = item("Terminal", "xterm", false);
    app.command = vec!["xterm".into(), "-fg".into(), "grey".into()];
    o.run_items = vec![app];
    o.run_mode = true;
    o.filtered = vec![0];
    o.selected = 0;
    assert!(o.cursor_at_end());
    assert!(o.complete_selection(&c));
    assert_eq!(o.bar.as_ref().unwrap().text(), "xterm -fg grey");
    assert!(o.cursor_at_end());
    assert!(!o.complete_selection(&c));
}

#[test]
fn test_tab_completion_draws_from_run_history_including_multi_word_lines() {
    let c = conn();
    let rconn: Arc<dyn RenderBackend> = Arc::new(MockDisplay::new(1280, 800, 24));
    let mut o = Omni::new();
    o.bar = SearchBar::new(&rconn, 1, 0, 0, 200, 24).ok();
    o.run_mode = true;
    o.path_cmds = Some(Vec::new());
    o.run_history = vec!["xterm -fg grey -bg black".into(), "xclock".into()];
    o.bar.as_mut().unwrap().set_text("xterm -f");
    let _ = o.complete_command(&c);
    assert_eq!(o.bar.as_ref().unwrap().text(), "xterm -fg grey -bg black");
    o.bar.as_mut().unwrap().set_text("xcl");
    let _ = o.complete_command(&c);
    assert_eq!(o.bar.as_ref().unwrap().text(), "xclock");
}

#[test]
fn test_run_history_lives_under_the_cache_dir() {
    if let Some(p) = run_history_file() {
        let p = p.to_string_lossy();
        assert!(p.ends_with("antibox/run_history"));
    }
}

#[test]
fn test_panel_size_scales_with_the_monitor() {
    assert!(panel_w_for(3000) >= 1000);
    assert!(panel_w_for(200) <= 200);
    assert!(rows_for(600) >= DEFAULT_ROWS);
    assert!(rows_for(30000) > rows_for(600));
}
