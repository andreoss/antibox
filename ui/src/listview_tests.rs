use super::*;
use antibox_core::mock::MockDisplay;

fn sample_nodes() -> Vec<MenuNode<u32>> {
    let mut nodes: Vec<MenuNode<u32>> = (0..25)
        .map(|i| MenuNode::leaf(format!("alpha{}", i), i))
        .collect();
    nodes.extend((0..5).map(|i| MenuNode::leaf(format!("zeta{}", i), 100 + i)));
    nodes
}

fn shown(conn: &MockDisplay) -> ListView<u32> {
    let mut view = ListView::new();
    view.show(conn, sample_nodes(), Place::Centre, 200, 0);
    view
}

fn with_bar(conn: &MockDisplay, view: &mut ListView<u32>) {
    let rb: Arc<dyn RenderBackend> = Arc::new(MockDisplay::new(1280, 800, 24));
    let bar = crate::searchbar::SearchBar::new(&rb, view.window_id(), 0, 0, 100, 20).unwrap();
    view.set_bar(bar);
    let _ = conn;
}

fn mapping_of(keysyms: &[u32]) -> KeyboardMapping {
    KeyboardMapping {
        keysyms_per_keycode: 1,
        keysyms: keysyms.to_vec(),
    }
}

fn type_keys(conn: &MockDisplay, view: &mut ListView<u32>, keysyms: &[u32]) -> Vec<ListNav<u32>> {
    let mapping = mapping_of(keysyms);
    keysyms
        .iter()
        .enumerate()
        .map(|(i, ks)| view.handle_key_input(conn, 8 + i as u32, 0, &mapping, *ks))
        .collect()
}

#[test]
fn refilter_with_narrowing_needle_configures_window_to_new_height() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = shown(&conn);
    with_bar(&conn, &mut view);
    let before = view.h;
    let _ = type_keys(&conn, &mut view, &['z' as u32]);
    assert_eq!(view.items.len(), 5);
    assert!(view.h < before);
    let win = conn.get_window(view.window_id()).unwrap();
    let (_, gh) = win.get_geometry().unwrap();
    assert_eq!(gh, view.h);
}

#[test]
fn typing_through_the_keymap_path_fills_the_bar_and_filters() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = ListView::new();
    let nodes = vec![
        MenuNode::leaf("abc", 1u32),
        MenuNode::leaf("abd", 2),
        MenuNode::leaf("zzz", 3),
    ];
    view.show(&conn, nodes, Place::Centre, 200, 0);
    with_bar(&conn, &mut view);
    let _ = type_keys(&conn, &mut view, &['a' as u32, 'b' as u32, 'c' as u32]);
    assert_eq!(view.bar().unwrap().text(), "abc");
    assert_eq!(view.items.len(), 1);
    assert_eq!(view.items[0].title, "abc");
    let win = conn.get_window(view.window_id()).unwrap();
    let (gw, gh) = win.get_geometry().unwrap();
    assert_eq!((gw, gh), (view.w, view.h));
}

#[test]
fn escape_through_the_keymap_path_reports_close() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = shown(&conn);
    with_bar(&conn, &mut view);
    let navs = type_keys(&conn, &mut view, &[0xFF1B]);
    assert!(matches!(navs[0], ListNav::Close));
}

#[test]
fn escape_without_a_bar_reports_close() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = shown(&conn);
    assert!(matches!(view.handle_key(&conn, 0xFF1B, false), ListNav::Close));
    view.hide(&conn);
    assert!(!view.visible);
    assert!(view.window.is_none());
    assert!(view.items.is_empty());
}

#[test]
fn set_size_issues_a_configure() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = shown(&conn);
    view.set_size(320, 240);
    let win = conn.get_window(view.window_id()).unwrap();
    assert_eq!(win.get_geometry().unwrap(), (320, 240));
}

#[test]
fn sync_geometry_aligns_pixmap_window_and_field_heights() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = shown(&conn);
    view.h = 60;
    view.sync_geometry(&conn);
    let win = conn.get_window(view.window_id()).unwrap();
    let (_, wh) = win.get_geometry().unwrap();
    let pm = conn
        .create_pixmap(view.w.max(1), view.h.max(1), conn.screen_depth())
        .unwrap();
    let (_, ph) = conn.get_window(pm).unwrap().get_geometry().unwrap();
    assert_eq!(ph, wh);
    assert_eq!(wh, view.h);
    let _ = conn.free_pixmap(pm);
}

fn grouped_nodes() -> Vec<MenuNode<u32>> {
    vec![
        MenuNode::group(
            "Tools",
            vec![
                MenuNode::leaf("Hammer", 1),
                MenuNode::group("Saws", vec![MenuNode::leaf("Hacksaw", 2)]),
            ],
        ),
        MenuNode::leaf("Exit", 3),
    ]
}

#[test]
fn enter_on_a_group_expands_and_collapses_in_place() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = ListView::new();
    view.show(&conn, grouped_nodes(), Place::Centre, 200, 0);
    assert_eq!(view.items.len(), 2);
    view.selected = Some(0);
    assert!(matches!(
        view.handle_key(&conn, 0xFF0D, false),
        ListNav::Handled
    ));
    assert_eq!(view.items.len(), 4);
    assert_eq!(view.items[1].title, "Hammer");
    assert_eq!(view.items[1].depth, 1);
    assert_eq!(view.selected, Some(0));
    assert!(matches!(
        view.handle_key(&conn, 0xFF0D, false),
        ListNav::Handled
    ));
    assert_eq!(view.items.len(), 2);
}

#[test]
fn filtering_keeps_structure_and_expands_matching_branches() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = ListView::new();
    view.show(&conn, grouped_nodes(), Place::Centre, 200, 0);
    with_bar(&conn, &mut view);
    let _ = type_keys(&conn, &mut view, &['h' as u32, 'a' as u32, 'c' as u32]);
    let titles: Vec<(&str, u16)> = view
        .items
        .iter()
        .map(|r| (r.title.as_str(), r.depth))
        .collect();
    assert_eq!(titles, vec![("Tools", 0), ("Saws", 1), ("Hacksaw", 2)]);
    assert!(view.items[0].is_group());
    assert_eq!(view.items[2].payload(), Some(&2));
}

fn expanded_nodes() -> Vec<MenuNode<u32>> {
    vec![
        MenuNode::group_expanded(
            "Apps",
            (0..12).map(|i| MenuNode::leaf(format!("app{}", i), i)).collect(),
        ),
        MenuNode::leaf("Exit", 99),
    ]
}

#[test]
fn collapsing_a_group_shrinks_the_configured_window() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = ListView::new();
    view.show(&conn, expanded_nodes(), Place::Centre, 200, 0);
    assert_eq!(view.items.len(), 14);
    let before = view.h;
    view.selected = Some(0);
    assert!(matches!(view.handle_key(&conn, 0xFF0D, false), ListNav::Handled));
    assert_eq!(view.items.len(), 2);
    assert!(view.h < before);
    let win = conn.get_window(view.window_id()).unwrap();
    let (gw, gh) = win.get_geometry().unwrap();
    assert_eq!((gw, gh), (view.w, view.h));
}

#[test]
fn collapsing_an_upward_anchored_group_reanchors_the_window() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = ListView::new();
    let anchor = Point::new(40, 700);
    view.show(&conn, expanded_nodes(), Place::Anchor(anchor), 200, 0);
    assert_eq!(view.pos().y, anchor.y - view.h as i32);
    view.selected = Some(0);
    let _ = view.handle_key(&conn, 0xFF0D, false);
    assert_eq!(view.pos().y, anchor.y - view.h as i32);
    let win = conn.get_window(view.window_id()).unwrap();
    let (_, gh) = win.get_geometry().unwrap();
    assert_eq!(gh, view.h);
}

#[test]
fn set_size_keeps_the_search_bar_spanning_the_window() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = shown(&conn);
    with_bar(&conn, &mut view);
    view.set_size(340, 200);
    let bar = view.bar().unwrap();
    assert_eq!(bar.w, (340i16 - pad() * 2).max(1) as u16);
    assert_eq!((bar.x, bar.y), (pad(), pad()));
    view.set_size(180, 150);
    let bar = view.bar().unwrap();
    assert_eq!(bar.w, (180i16 - pad() * 2).max(1) as u16);
}

#[test]
fn enter_on_a_leaf_activates_its_payload() {
    let conn = MockDisplay::new(1280, 800, 24);
    let mut view = ListView::new();
    view.show(&conn, grouped_nodes(), Place::Centre, 200, 0);
    view.selected = Some(1);
    match view.handle_key(&conn, 0xFF0D, false) {
        ListNav::Activate(p) => assert_eq!(p, 3),
        _ => panic!("expected activation"),
    }
}
