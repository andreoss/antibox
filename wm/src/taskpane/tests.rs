use super::*;
use crate::taskpane::sync::task_columns;
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

#[test]
fn test_button_bg_differs_between_active_and_inactive() {
    let face = antibox_ui::theme::tray_face();
    assert_ne!(
        TaskPane::button_bg(face, true),
        TaskPane::button_bg(face, false),
        "icon blend background must follow the active button surface"
    );
}

#[test]
fn test_active_button_bg_matches_pressed_surface() {
    let face = antibox_ui::theme::tray_face();
    let active = TaskPane::button_bg(face, true);
    let expected = if antibox_ui::theme::pressed_dither(true) {
        antibox_core::colour::lerp(antibox_ui::theme::face(), antibox_ui::theme::light(), 0.5)
    } else {
        antibox_ui::theme::pressed_face(face)
    };
    assert_eq!(active, expected);
}

#[test]
fn test_task_columns_follow_the_room() {
    let gap = antibox_ui::metrics::item_gap();
    let min = TaskPane::min_task_w();
    assert_eq!(task_columns(min - 1, gap), 1);
    assert_eq!(task_columns(min, gap), 1);
    assert_eq!(task_columns(min * 2 + gap, gap), 2);
    assert!(task_columns(min * 5 + gap * 4, gap) >= 5);
}

#[test]
fn test_vertical_buttons_flow_into_a_grid() {
    let (mut pane, _d) = make_pane();
    pane.set_vertical(true);
    for i in 0..5u32 {
        pane.add_button(100 + i, &format!("w{i}"));
    }
    let wide = (TaskPane::min_task_w() * 2
        + antibox_ui::metrics::item_gap()
        + antibox_ui::metrics::button_inset() * 2) as u16;
    pane.layout_buttons(wide, 600);
    let first_row_y = pane.buttons[0].rect.1;
    let cols = pane
        .buttons
        .iter()
        .filter(|b| b.rect.1 == first_row_y)
        .count();
    assert!(cols >= 2, "buttons should share a row, got {cols}");
    assert_eq!(pane.buttons[cols].rect.0, pane.buttons[0].rect.0);
    assert!(pane.buttons[cols].rect.1 > first_row_y);
    assert!(
        pane.buttons[0].rect.2 < wide,
        "a button must not take the whole line"
    );
}

#[test]
fn test_a_vertical_click_finds_the_button_on_its_own_row() {
    let (mut pane, _d) = make_pane();
    pane.set_vertical(true);
    for i in 0..4u32 {
        pane.add_button(200 + i, &format!("w{i}"));
    }
    let wide = (TaskPane::min_task_w() * 2
        + antibox_ui::metrics::item_gap()
        + antibox_ui::metrics::button_inset() * 2) as u16;
    pane.layout_buttons(wide, 600);
    for (i, btn) in pane.buttons.iter().enumerate() {
        let (x, y) = (
            i32::from(btn.rect.0) + i32::from(btn.rect.2) / 2,
            i32::from(btn.rect.1) + i32::from(btn.rect.3) / 2,
        );
        assert_eq!(pane.button_at(x, y), Some(i), "button {i}");
    }
}

#[test]
fn test_a_short_row_of_buttons_is_centred() {
    let (mut pane, _d) = make_pane();
    pane.set_vertical(true);
    for i in 0..3u32 {
        pane.add_button(300 + i, &format!("w{i}"));
    }
    let wide = (TaskPane::min_task_w() * 2
        + antibox_ui::metrics::item_gap()
        + antibox_ui::metrics::button_inset() * 2) as u16;
    pane.layout_buttons(wide, 600);
    let lone = &pane.buttons[2];
    let left = lone.rect.0;
    let right = wide as i16 - (lone.rect.0 + lone.rect.2 as i16);
    assert!((left - right).abs() <= 1, "left {left} right {right}");
    assert!(left > pane.buttons[0].rect.0, "a lone button moves inwards");
}
