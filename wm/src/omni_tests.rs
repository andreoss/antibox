use super::*;
use antibox_core::mock::MockDisplay;
use antibox_ui::searchbar::SearchBar;

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

fn group_children<'a>(nodes: &'a [MenuNode<OmniAct>], title: &str) -> Option<&'a [MenuNode<OmniAct>]> {
    nodes.iter().find_map(|n| match n {
        MenuNode::Group {
            title: t, children, ..
        } if t == title => Some(children.as_slice()),
        _ => None,
    })
}

fn leaf_payload<T: Clone>(node: &MenuNode<T>) -> Option<&T> {
    match node {
        MenuNode::Leaf { payload, .. } => Some(payload),
        _ => None,
    }
}

#[test]
fn test_send_to_group_lists_every_workspace_with_name_fallback() {
    let mut o = Omni::new();
    o.ws_count = 3;
    o.ws_names = vec!["one".into(), String::new(), "three".into()];
    let nodes = o.ops_nodes();
    let ws = group_children(&nodes, "Send to").expect("send-to group");
    assert_eq!(ws.len(), 3);
    assert_eq!(ws[0].title(), "one");
    assert!(matches!(
        leaf_payload(&ws[0]),
        Some(OmniAct::Op(OmniWinOp::SendTo(0)))
    ));
    assert_eq!(ws[1].title(), "Workspace 2");
    assert!(matches!(
        leaf_payload(&ws[2]),
        Some(OmniAct::Op(OmniWinOp::SendTo(2)))
    ));
}

#[test]
fn test_join_group_excludes_the_target_window() {
    let mut o = Omni::new();
    o.ops_target = 10;
    o.win_list = vec![(10, "self".into()), (20, "b".into()), (30, "c".into())];
    let nodes = o.ops_nodes();
    let join = group_children(&nodes, "Join").expect("join group");
    assert_eq!(join.len(), 2);
    assert!(matches!(
        leaf_payload(&join[0]),
        Some(OmniAct::Op(OmniWinOp::Join(20)))
    ));
    assert!(matches!(
        leaf_payload(&join[1]),
        Some(OmniAct::Op(OmniWinOp::Join(30)))
    ));
}

#[test]
fn test_ops_menu_picks_ops_and_pops_back_to_windows() {
    let c = conn();
    let mut o = Omni::new();
    o.ops_target = 42;
    o.in_ops = true;
    let nodes = o.ops_nodes();
    o.view.set_tree(nodes);
    let idx = o
        .view
        .items
        .iter()
        .position(|r| r.title == "Kill")
        .expect("kill row");
    o.view.selected = Some(idx);
    match o.activate_selected(&c) {
        OmniOutcome::WindowOp { target, op } => {
            assert_eq!(target, 42);
            assert!(matches!(op, OmniWinOp::Kill));
        }
        _ => panic!("expected a WindowOp outcome"),
    }
    assert!(o.in_ops);
    o.pop_ops(&c);
    assert!(!o.in_ops);
}

#[test]
fn test_ops_separators_are_skipped_by_selection() {
    let mut o = Omni::new();
    o.view.set_tree(o.ops_nodes());
    assert!(o.view.items[1].is_separator());
    assert_eq!(o.view.next_selectable(Some(0), 1), Some(2));
    assert_eq!(o.view.items[2].title, "Close");
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
    o.in_ops = true;
    o.ops_target = 2;
    o.view.set_tree(o.ops_nodes());
    let idx = o
        .view
        .items
        .iter()
        .position(|r| r.title == "Sort: natural")
        .expect("sort row");
    o.view.selected = Some(idx);
    assert!(matches!(o.activate_selected(&c), OmniOutcome::Consumed));
    assert_eq!(o.sort, OmniSort::Title);
    assert!(o.view.items.iter().any(|r| r.title == "Sort: title"));
    assert_eq!(o.items[0].title, "alpha");
    assert_eq!(o.win_list[0].0, 1);
}

#[test]
fn test_windows_mode_groups_windows_applications_and_actions() {
    let mut o = Omni::new();
    o.items = vec![item("beta", "b", 0, 2, false)];
    o.apps = Some(vec![crate::desktop_apps::DesktopApp {
        name: "Editor".into(),
        command: vec!["ed".into()],
        categories: Vec::new(),
    }]);
    let nodes = o.window_nodes();
    assert_eq!(group_children(&nodes, "Windows").map(<[_]>::len), Some(1));
    assert_eq!(
        group_children(&nodes, "Applications").map(<[_]>::len),
        Some(1)
    );
    let actions = group_children(&nodes, "Actions").expect("actions group");
    assert!(matches!(leaf_payload(&actions[0]), Some(OmniAct::RunEntry)));
}

#[test]
fn test_command_line_unwraps_shell_history_and_joins_argv() {
    let mut hist = item("ls -la", "ls -la", !0, 0, false);
    hist.command = vec!["sh".into(), "-c".into(), "ls -la".into()];
    assert_eq!(Omni::command_line_argv(&hist.command), "ls -la");
    let mut app = item("Terminal", "xterm", !0, 0, false);
    app.command = vec!["xterm".into(), "-fg".into(), "grey".into()];
    assert_eq!(Omni::command_line_argv(&app.command), "xterm -fg grey");
}

#[test]
fn test_right_arrow_completion_puts_the_selected_command_into_the_bar() {
    let c = conn();
    let rconn: Arc<dyn RenderBackend> = Arc::new(MockDisplay::new(1280, 800, 24));
    let mut o = Omni::new();
    o.view.set_bar(SearchBar::new(&rconn, 1, 0, 0, 200, 24).unwrap());
    let mut app = item("Terminal", "xterm", !0, 0, false);
    app.command = vec!["xterm".into(), "-fg".into(), "grey".into()];
    o.run_items = vec![app];
    o.run_mode = true;
    o.view.set_tree(o.run_nodes());
    o.view.selected = o.view.next_leaf(None, 1);
    assert!(o.cursor_at_end());
    assert!(o.complete_selection(&c));
    assert_eq!(o.query(), "xterm -fg grey");
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
    o.view.set_bar(SearchBar::new(&rconn, 1, 0, 0, 200, 24).unwrap());
    o.run_mode = true;
    o.path_cmds = Some(Vec::new());
    o.run_history = vec!["xterm -fg grey -bg black".into(), "xclock".into()];
    o.set_query("xterm -f");
    let _ = o.complete_command(&c);
    assert_eq!(o.query(), "xterm -fg grey -bg black");
    o.set_query("xcl");
    let _ = o.complete_command(&c);
    assert_eq!(o.query(), "xclock");
}

#[test]
fn test_ops_menu_filter_narrows_rows_and_pop_restores_the_window_query() {
    let c = conn();
    let rconn: Arc<dyn RenderBackend> = Arc::new(MockDisplay::new(1280, 800, 24));
    let mut o = Omni::new();
    o.view.set_bar(SearchBar::new(&rconn, 1, 0, 0, 200, 24).unwrap());
    o.in_ops = true;
    o.saved_query = "alpha".into();
    o.view.set_tree(o.ops_nodes());
    let full = o.view.items.len();
    o.set_query("clo");
    o.view.refilter(c.as_ref());
    assert_eq!(o.view.items.len(), 1);
    assert_eq!(o.view.items[0].title, "Close");
    o.set_query("");
    o.view.refilter(c.as_ref());
    assert_eq!(o.view.items.len(), full);
    o.pop_ops(&c);
    assert!(!o.in_ops);
    assert_eq!(o.query(), "alpha");
}

#[test]
fn test_run_history_lives_under_the_cache_dir() {
    if let Some(p) = run_history_file() {
        let p = p.to_string_lossy();
        assert!(p.ends_with("antibox/run_history"));
    }
}
