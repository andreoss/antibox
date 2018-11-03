use super::*;
use crate::render::ThemeColors;
use antibox_core::backend::DisplayBackend;
use antibox_core::mock::MockDisplay;
use std::sync::Arc;

fn make_pane() -> (TaskPane, Arc<MockDisplay>) {
    let d = Arc::new(MockDisplay::new(1280, 720, 24));
    let conn = Arc::clone(&d) as Arc<dyn DisplayBackend>;
    let colours = ThemeColors::default();
    let pane = TaskPane::new(&conn, conn.root().read_id(), colours).unwrap();
    (pane, d)
}

#[test]
fn test_add_button() {
    let (mut pane, _d) = make_pane();
    pane.add_button(42, "Terminal");
    assert_eq!(pane.buttons.len(), 1);
    assert_eq!(pane.buttons[0].window_id, 42);
    assert_eq!(pane.buttons[0].label, "Terminal");
    assert!(!pane.buttons[0].active);
    assert!(!pane.buttons[0].minimized);
    assert_eq!(pane.buttons[0].rect, (2, 2, 116, 24));
}

#[test]
fn test_layout_buttons_use_theme_item_gap() {
    let (mut pane, _d) = make_pane();
    pane.add_button(1, "A");
    pane.add_button(2, "B");
    pane.add_button(3, "C");
    pane.layout_buttons(600, 28);
    let gap = antibox_ui::metrics::item_gap() as i16;
    for w in pane.buttons.windows(2) {
        let (x0, _, w0, _) = w[0].rect;
        let (x1, ..) = w[1].rect;
        assert_eq!(x1 - (x0 + w0 as i16), gap);
    }
}

