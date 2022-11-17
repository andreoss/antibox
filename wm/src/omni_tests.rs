use super::*;
use antibox_core::mock::MockDisplay;

fn conn() -> Arc<dyn DisplayBackend> {
    Arc::new(MockDisplay::new(1280, 800, 24))
}

fn item(title: &str, class: &str, ws: u32, id: u32, run: bool) -> OmniItem {
    OmniItem {
        title: title.to_string(),
        class: class.to_string(),
        client_id: id,
        workspace: ws,
        icon: PixmapData::new(0, 0, Vec::new()),
        run,
        marked: false,
        command: Vec::new(),
    }
}

#[test]
fn test_send_level_lists_every_workspace_with_name_fallback() {
    let mut o = Omni::new();
    o.ws_count = 3;
    o.ws_names = vec!["one".into(), String::new(), "three".into()];
    o.push_send_level();
    let lvl = o.menu_levels.last().unwrap();
    assert_eq!(lvl.rows.len(), 3);
    assert_eq!(lvl.rows[0].0, "one");
    assert!(matches!(lvl.rows[0].1, OpAction::Do(OmniWinOp::SendTo(0))));
    assert_eq!(lvl.rows[1].0, "Workspace 2");
    assert!(matches!(lvl.rows[2].1, OpAction::Do(OmniWinOp::SendTo(2))));
}

#[test]
fn test_join_level_excludes_the_target_window() {
    let mut o = Omni::new();
    o.menu_target = 10;
    o.win_list = vec![(10, "self".into()), (20, "b".into()), (30, "c".into())];
    o.push_join_level();
    let lvl = o.menu_levels.last().unwrap();
    assert_eq!(lvl.rows.len(), 2);
    assert!(matches!(lvl.rows[0].1, OpAction::Do(OmniWinOp::Join(20))));
    assert!(matches!(lvl.rows[1].1, OpAction::Do(OmniWinOp::Join(30))));
}

#[test]
fn test_submenu_navigates_picks_and_pops() {
    let c = conn();
    let mut o = Omni::new();
    o.menu_target = 42;
    o.menu_levels.push(OpLevel::new(vec![
        ("Close".into(), OpAction::Do(OmniWinOp::Close)),
        ("Kill".into(), OpAction::Do(OmniWinOp::Kill)),
    ]));
    assert!(matches!(
        o.submenu_key(antibox_core::keysyms::KEY_Down, &c),
        OmniOutcome::Consumed
    ));
    assert_eq!(o.menu_levels.last().unwrap().selected, 1);
    match o.submenu_key(antibox_core::keysyms::KEY_Return, &c) {
        OmniOutcome::WindowOp { target, op } => {
            assert_eq!(target, 42);
            assert!(matches!(op, OmniWinOp::Kill));
        }
        _ => panic!("expected a WindowOp outcome"),
    }
    assert!(o.in_submenu());
    assert!(matches!(
        o.submenu_key(antibox_core::keysyms::KEY_Left, &c),
        OmniOutcome::Consumed
    ));
    assert!(!o.in_submenu());
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
fn test_sort_modes_cycle_through_all_and_wrap() {
    let mut mode = OmniSort::default();
    let mut seen = vec![mode];
    for _ in 0..4 {
        mode = mode.next();
        assert!(!seen.contains(&mode));
        seen.push(mode);
    }
    assert_eq!(mode.next(), OmniSort::default());
}

#[test]
fn test_title_sort_is_case_insensitive_with_id_tiebreak() {
    use std::cmp::Ordering;
    let cmp = |a, b| compare_windows(OmniSort::Title, a, b);
    assert_eq!(
        cmp(("alpha", "x", 0, 1), ("Beta", "a", 9, 2)),
        Ordering::Less
    );
    assert_eq!(
        cmp(("ZZZ", "x", 0, 1), ("aaa", "x", 0, 2)),
        Ordering::Greater
    );
    assert_eq!(
        cmp(("Same", "x", 0, 2), ("same", "y", 1, 1)),
        Ordering::Greater
    );
}

#[test]
fn test_class_sort_groups_by_class_then_title() {
    use std::cmp::Ordering;
    let cmp = |a, b| compare_windows(OmniSort::Class, a, b);
    assert_eq!(
        cmp(("z", "emacs", 0, 9), ("a", "xterm", 0, 1)),
        Ordering::Less
    );
    assert_eq!(
        cmp(("b", "xterm", 0, 1), ("a", "xterm", 0, 2)),
        Ordering::Greater
    );
    assert_eq!(
        cmp(("a", "xterm", 0, 1), ("a", "xterm", 0, 2)),
        Ordering::Less
    );
}

#[test]
fn test_workspace_sort_orders_by_workspace_then_title() {
    use std::cmp::Ordering;
    let cmp = |a, b| compare_windows(OmniSort::Workspace, a, b);
    assert_eq!(cmp(("z", "x", 0, 9), ("a", "x", 1, 1)), Ordering::Less);
    assert_eq!(cmp(("b", "x", 2, 1), ("a", "x", 2, 2)), Ordering::Greater);
}

#[test]
fn test_window_sort_orders_by_client_id() {
    use std::cmp::Ordering;
    let cmp = |a, b| compare_windows(OmniSort::Window, a, b);
    assert_eq!(cmp(("z", "z", 9, 3), ("a", "a", 0, 7)), Ordering::Less);
    assert_eq!(cmp(("a", "a", 0, 7), ("z", "z", 9, 3)), Ordering::Greater);
}

#[test]
fn test_natural_sort_is_stable_and_keeps_run_entry_last() {
    let mut o = Omni::new();
    o.items = vec![
        item("beta", "b", 1, 20, false),
        item("Run", "run", !0, 0, true),
        item("alpha", "a", 0, 10, false),
    ];
    o.apply_sort();
    assert_eq!(o.items[0].client_id, 20);
    assert_eq!(o.items[1].client_id, 10);
    assert!(o.items[2].run);
    o.sort = OmniSort::Title;
    o.apply_sort();
    assert_eq!(o.items[0].title, "alpha");
    assert_eq!(o.items[1].title, "beta");
    assert!(o.items[2].run);
}

#[test]
fn test_grouping_override_wins_over_preference() {
    let mut o = Omni::new();
    o.group_override = Some(true);
    assert!(o.grouping_enabled());
    o.group_override = Some(false);
    assert!(!o.grouping_enabled());
    assert_eq!(o.group_row_label(), "Group by class: off");
}

#[test]
fn test_cycle_sort_updates_the_ops_menu_row_label() {
    let c = conn();
    let mut o = Omni::new();
    o.items = vec![
        item("beta", "b", 0, 2, false),
        item("alpha", "a", 0, 1, false),
    ];
    o.filtered = vec![0, 1];
    o.menu_target = 2;
    o.menu_levels.push(OpLevel::new(vec![(
        "Sort: natural".into(),
        OpAction::CycleSort,
    )]));
    assert!(matches!(o.activate_submenu(&c), OmniOutcome::Consumed));
    assert_eq!(o.sort, OmniSort::Title);
    assert_eq!(o.menu_levels.last().unwrap().rows[0].0, "Sort: title");
    assert_eq!(o.items[0].title, "alpha");
    let sel = o.filtered[o.selected];
    assert_eq!(o.items[sel].client_id, 2);
}

#[test]
fn test_command_line_unwraps_shell_history_and_joins_argv() {
    let mut hist = item("ls -la", "ls -la", !0, 0, false);
    hist.command = vec!["sh".into(), "-c".into(), "ls -la".into()];
    assert_eq!(Omni::command_line(&hist), "ls -la");
    let mut app = item("Terminal", "xterm", !0, 0, false);
    app.command = vec!["xterm".into(), "-fg".into(), "grey".into()];
    assert_eq!(Omni::command_line(&app), "xterm -fg grey");
}

#[test]
fn test_right_arrow_completion_puts_the_selected_command_into_the_bar() {
    let c = conn();
    let rconn: Arc<dyn RenderBackend> = Arc::new(MockDisplay::new(1280, 800, 24));
    let mut o = Omni::new();
    o.bar = SearchBar::new(&rconn, 1, 0, 0, 200, 24).ok();
    let mut app = item("Terminal", "xterm", !0, 0, false);
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
fn test_panel_size_scales_with_the_monitor() {
    assert!(panel_w_for(3000) >= 1000);
    assert!(panel_w_for(200) <= 200);
    assert!(rows_for(600) >= DEFAULT_ROWS);
    assert!(rows_for(30000) > rows_for(600));
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
fn test_ops_menu_filter_narrows_rows_and_pop_restores_the_window_query() {
    let c = conn();
    let rconn: Arc<dyn RenderBackend> = Arc::new(MockDisplay::new(1280, 800, 24));
    let mut o = Omni::new();
    o.bar = SearchBar::new(&rconn, 1, 0, 0, 200, 24).ok();
    o.saved_query = "alpha".into();
    o.menu_levels.push(OpLevel::new(vec![
        ("Mark".into(), OpAction::Do(OmniWinOp::ToggleMark)),
        (String::new(), OpAction::Do(OmniWinOp::Separator)),
        ("Close".into(), OpAction::Do(OmniWinOp::Close)),
        ("Kill".into(), OpAction::Do(OmniWinOp::Kill)),
    ]));
    o.bar.as_mut().unwrap().set_text("clo");
    o.refilter_ops(&c);
    let level = o.menu_levels.last().unwrap();
    assert_eq!(level.rows.len(), 1);
    assert_eq!(level.rows[0].0, "Close");
    assert_eq!(level.selected, 0);
    o.bar.as_mut().unwrap().set_text("");
    o.refilter_ops(&c);
    assert_eq!(o.menu_levels.last().unwrap().rows.len(), 4);
    o.pop_ops_level(&c);
    assert!(o.menu_levels.is_empty());
    assert_eq!(o.bar.as_ref().unwrap().text(), "alpha");
}

#[test]
fn test_menu_row_at_skips_separator_rows() {
    let mut o = Omni::new();
    o.menu_levels.push(OpLevel::new(vec![
        ("Mark".into(), OpAction::Do(OmniWinOp::ToggleMark)),
        (String::new(), OpAction::Do(OmniWinOp::Separator)),
        ("Close".into(), OpAction::Do(OmniWinOp::Close)),
    ]));
    let top = pad() as i32 * 2 + bar_h() as i32;
    let rh = row_h() as i32;
    assert_eq!(o.menu_row_at(top), Some(0));
    assert_eq!(o.menu_row_at(top + rh), None);
    assert_eq!(o.menu_row_at(top + rh * 2), Some(2));
}

#[test]
fn test_run_history_lives_under_the_cache_dir() {
    if let Some(p) = run_history_file() {
        let p = p.to_string_lossy();
        assert!(p.ends_with("antibox/run_history"));
    }
}
