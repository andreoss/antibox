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
fn test_vertical_buttons_take_the_full_width_one_per_line() {
    let (mut pane, _d) = make_pane();
    pane.set_vertical(true);
    for i in 0..3u32 {
        pane.add_button(100 + i, &format!("w{i}"));
    }
    let vin = antibox_ui::metrics::button_inset() as u16;
    pane.layout_buttons(120, 600);
    for btn in &pane.buttons {
        assert_eq!(btn.rect.0, vin as i16);
        assert_eq!(btn.rect.2, 120 - vin * 2, "a button spans the bar");
    }
    assert!(pane.buttons[1].rect.1 > pane.buttons[0].rect.1);
    for (i, btn) in pane.buttons.iter().enumerate() {
        let (x, y) = (
            i32::from(btn.rect.0) + i32::from(btn.rect.2) / 2,
            i32::from(btn.rect.1) + i32::from(btn.rect.3) / 2,
        );
        assert_eq!(pane.button_at(x, y), Some(i));
    }
}

#[test]
fn test_tight_rows_are_justified_with_the_leftover_pixels() {
    let widths = TaskPane::spread_widths(100, 3, 2, 10, 200);
    assert_eq!(widths.iter().sum::<i32>() + 2 * 2, 100, "the row is filled exactly");
    assert!(widths[0] >= widths[2]);
    assert!(widths[0] - widths[2] <= 1, "leftover spreads one pixel at a time");
}

#[test]
fn test_roomy_rows_keep_the_default_width() {
    assert_eq!(TaskPane::spread_widths(1000, 3, 2, 10, 120), vec![120, 120, 120]);
}

#[test]
fn test_crowded_rows_fall_back_to_half_the_default() {
    let widths = TaskPane::spread_widths(50, 4, 2, 30, 120);
    assert_eq!(widths, vec![60, 60, 60, 60]);
}
