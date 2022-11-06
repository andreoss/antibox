use super::*;
use antibox_core::mock::{MockDisplay, MockWindow};

fn make_applet() -> KeyboardApplet {
    let tc = crate::render::ThemeColors::default();
    KeyboardApplet {
        window: Box::new(MockWindow::new(42)) as Box<dyn WindowHandle>,
        conn: Arc::new(MockDisplay::new(800, 600, 24)),
        layout: "US".to_string(),
        tooltip_text: "layout: US".to_string(),
        layouts: vec!["us".to_string(), "de".to_string(), "fr".to_string()],
        xkb_groups: true,
        index: 0,
        menu_window: None,
        menu_selected: None,
        applet_abs: (0, 0),
        menu_abs: (0, 0),
        opened_at: None,
        width: 32,
        height: 28,
        face: tc.task_bar_colour,
        text: tc.button_fg,
        sel_bg: tc.workspace_active_bg,
        sel_fg: tc.workspace_active_fg,
        tooltip: None,
    }
}

#[test]
fn test_new_default_state() {
    let app = make_applet();
    assert_eq!(app.layout, "US");
    assert!(!app.tooltip_text.is_empty());
    assert_eq!(app.index, 0);
}

#[test]
fn test_new_zero_width_without_layouts() {
    let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(800, 600, 24));
    let tc = crate::render::ThemeColors::default();
    let app = KeyboardApplet::new(&conn, 1, Vec::new(), &tc).unwrap();
    assert_eq!(app.preferred_width(), 0);
}

#[test]
fn test_becomes_visible_when_layouts_appear() {
    let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(800, 600, 24));
    let tc = crate::render::ThemeColors::default();
    let mut app = KeyboardApplet::new(&conn, 1, Vec::new(), &tc).unwrap();
    assert_eq!(app.preferred_width(), 0);
    app.layouts = vec!["us".to_string(), "ru".to_string()];
    assert!(app.preferred_width() > 0);
}

#[test]
fn test_new_with_configured_layouts() {
    let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(800, 600, 24));
    let tc = crate::render::ThemeColors::default();
    let layouts = vec!["us".to_string(), "ru".to_string()];
    let app = KeyboardApplet::new(&conn, 1, layouts, &tc).unwrap();
    assert_eq!(app.layouts.len(), 2);
    assert!(!app.xkb_groups);
    assert!(app.preferred_width() > 0);
}

#[test]
fn test_next_layout_cycles() {
    let mut app = make_applet();
    assert_eq!(app.index, 0);
    app.next_layout();
    assert_eq!(app.index, 1);
    app.next_layout();
    assert_eq!(app.index, 2);
    app.next_layout();
    assert_eq!(app.index, 0);
}

#[test]
fn test_next_layout_empty_list_no_crash() {
    let mut app = make_applet();
    app.layouts.clear();
    app.next_layout();
    assert_eq!(app.index, 0);
}

#[test]
fn test_switch_to_valid() {
    let mut app = make_applet();
    app.switch_to(2);
    assert_eq!(app.index, 2);
    assert_eq!(app.layout, "FR");
    assert!(app.menu_window.is_none());
}

#[test]
fn test_switch_to_out_of_range() {
    let mut app = make_applet();
    app.switch_to(99);
    assert_eq!(app.index, 0);
}

#[test]
fn test_update_returns_true_when_changed() {
    let mut app = make_applet();
    app.layout = "XX".to_string();
    let changed = app.update();
    assert!(changed);
}

#[test]
fn test_update_returns_false_when_unchanged() {
    let mut app = make_applet();
    let (layout, tooltip) = detect_layout_info();
    app.layout = layout;
    app.tooltip_text = tooltip;
    let changed = app.update();
    assert!(!changed);
}

#[test]
fn test_update_keeps_layouts_when_backend_has_none() {
    let mut app = make_applet();
    let before = app.layouts.clone();
    let _ = app.update();
    assert_eq!(app.layouts, before);
}

#[test]
fn test_tooltip_not_empty() {
    let app = make_applet();
    let tt = app.tooltip();
    assert!(!tt.is_empty());
    assert!(tt.to_uppercase().contains("US"));
}

#[test]
fn test_click_left_opens_and_toggles_menu() {
    let mut app = make_applet();
    assert!(app.menu_window.is_none());
    app.handle_click(0, 0, 1);
    assert_eq!(app.index, 0);
    assert!(app.menu_window.is_some());
    app.handle_click(0, 0, 1);
    assert!(app.menu_window.is_none());
}

#[test]
fn test_click_right_shows_menu() {
    let mut app = make_applet();
    assert!(app.menu_window.is_none());
    app.handle_click(0, 0, 3);
    assert!(app.menu_window.is_some());
}

#[test]
fn test_click_middle_does_nothing() {
    let mut app = make_applet();
    app.handle_click(0, 0, 2);
    assert_eq!(app.index, 0);
    assert!(app.menu_window.is_none());
}

#[test]
fn test_menu_index_at_bounds() {
    let app = make_applet();
    let pad = antibox_ui::metrics::pad();
    assert_eq!(app.menu_index_at(Point::new(-1, pad)), None);
    assert_eq!(app.menu_index_at(Point::new(0, pad)), Some(0));
    let below = pad + menu_item_h() as i32 * 3;
    assert_eq!(app.menu_index_at(Point::new(0, below)), None);
}

#[test]
fn test_estimate_width_nonzero() {
    assert!(estimate_width() > 0);
}

#[test]
fn test_detect_layout_info_fallback() {
    let (layout, _) = detect_layout_info();
    assert!(!layout.is_empty());
}

#[test]
fn test_format_xkb_tooltip() {
    let info = KeyboardInfo {
        rules: "evdev".to_string(),
        model: "pc105".to_string(),
        layouts: "us,ru".to_string(),
        variants: ",".to_string(),
        options: "grp:alt_shift_toggle".to_string(),
        group: 1,
    };
    let tip = format_xkb_tooltip(&info);
    assert!(tip.contains("Layout: RU"), "{}", tip);
    assert!(tip.contains("Layouts: us,ru"));
    assert!(tip.contains("Options: grp:alt_shift_toggle"));
    assert!(!tip.contains("Model"), "{}", tip);
    assert!(!tip.contains("Variants"), "{}", tip);
}

#[test]
fn test_owns_window_own() {
    let app = make_applet();
    assert!(app.owns_window(42));
}

#[test]
fn test_owns_window_not_own() {
    let app = make_applet();
    assert!(!app.owns_window(99));
}

#[test]
fn test_preferred_sizes() {
    let app = make_applet();
    assert!(app.preferred_width() >= 28);
    assert_eq!(app.preferred_height(), 28);
}

#[test]
fn test_set_geometry_updates_size() {
    let mut app = make_applet();
    app.set_geometry(10, 20, 48, 32);
    assert_eq!(app.width, 48);
    assert_eq!(app.height, 32);
}

#[test]
fn test_shutdown_closes_menu() {
    let mut app = make_applet();
    app.handle_click(0, 0, 1);
    assert!(app.menu_window.is_some());
    app.shutdown();
    assert!(app.menu_window.is_none());
}
