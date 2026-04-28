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
        let (x0, y0, w0, _) = w[0].rect;
        let (x1, y1, ..) = w[1].rect;
        if y0 != y1 {
            continue;
        }
        assert_eq!(x1 - (x0 + w0 as i16), gap);
    }
}

#[test]
fn test_button_bg_differs_between_active_and_inactive() {
    let face = antibox_ui::theme::tray_face();
    if antibox_ui::theme::pressed_dither(true) {
        assert_eq!(
            TaskPane::button_bg(face, true),
            TaskPane::button_bg(face, false),
            "dithered themes keep the face and let the surface dither"
        );
    } else {
        assert_ne!(
            TaskPane::button_bg(face, true),
            TaskPane::button_bg(face, false),
            "icon blend background must follow the active button surface"
        );
    }
}

#[test]
fn test_active_button_bg_matches_pressed_surface() {
    let face = antibox_ui::theme::tray_face();
    let active = TaskPane::button_bg(face, true);
    let expected = if antibox_ui::theme::pressed_dither(true) {
        face
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

#[test]
fn test_two_rows_split_the_buttons() {
    crate::layout_preferences::set_taskbar_rows(2);
    let (mut pane, _d) = make_pane();
    for i in 0..5u32 {
        pane.add_button(400 + i, &format!("w{i}"));
    }
    pane.layout_buttons(1000, 44);
    crate::layout_preferences::set_taskbar_rows(1);
    let first_y = pane.buttons[0].rect.1;
    let top: Vec<_> = pane.buttons.iter().filter(|b| b.rect.1 == first_y).collect();
    assert_eq!(top.len(), 3, "ceil(5/2) buttons on the first row");
    assert!(pane.buttons[3].rect.1 > first_y, "the rest drop to row two");
    assert_eq!(pane.buttons[3].rect.0, pane.buttons[0].rect.0);
}

#[test]
fn test_the_vertical_list_overflows_with_buttons() {
    let (mut pane, _d) = make_pane();
    pane.set_vertical(true);
    for i in 0..40u32 {
        pane.add_button(500 + i, &format!("w{i}"));
    }
    pane.layout_buttons(100, 200);
    assert!(
        pane.overflow.is_some(),
        "a short vertical bar overflows 40 windows"
    );
    assert!(pane.visible_count < pane.buttons.len());
    assert_eq!(
        pane.overflow_before_ids.len() + pane.overflow_ids.len(),
        pane.buttons.len() - pane.visible_count,
        "every hidden window sits before or after the visible strip"
    );
    let vin = antibox_ui::metrics::button_inset() as i16;
    for btn in &pane.buttons {
        if btn.rect.2 > 0 {
            assert_eq!(btn.rect.0, vin, "visible rows share the one column");
        }
    }
    pane.set_active(520);
    pane.layout_buttons(100, 200);
    assert!(
        pane.overflow_before.is_some() && pane.overflow.is_some(),
        "a mid-list active window hides rows both above and below"
    );
}

#[test]
fn test_overflow_splits_around_the_active_window() {
    let (mut pane, _d) = make_pane();
    for i in 1..=40u32 {
        pane.add_button(i, &format!("W{i}"));
    }
    pane.set_active(20);
    pane.layout_buttons(240, 28);
    assert!(
        pane.overflow_before.is_some(),
        "windows before the active one are hidden on the left"
    );
    assert!(
        pane.overflow.is_some(),
        "windows after the active one are hidden on the right"
    );
    assert!(!pane.overflow_before_ids.is_empty());
    assert!(!pane.overflow_ids.is_empty());
    assert!(
        pane.buttons.iter().any(|b| b.active && b.rect.2 > 0),
        "the active window stays within the visible strip"
    );
    assert_eq!(
        pane.overflow_before_ids.len() + pane.overflow_ids.len(),
        pane.buttons.len() - pane.visible_count,
        "every hidden window is accounted for on one side"
    );
}

#[test]
fn test_overflow_button_collects_the_windows_that_do_not_fit() {
    let (mut pane, _d) = make_pane();
    for i in 1..=40u32 {
        pane.add_button(i, &format!("W{i}"));
    }
    pane.layout_buttons(240, 28);
    assert!(pane.overflow.is_some(), "a narrow bar overflows 40 windows");
    assert!(pane.visible_count < pane.buttons.len());
    assert_eq!(
        pane.overflow_ids.len(),
        pane.buttons.len() - pane.visible_count,
        "every hidden window is collected"
    );
    let last_visible = &pane.buttons[pane.visible_count - 1];
    assert!(last_visible.rect.2 > 0, "visible buttons keep a real width");
    for hidden in &pane.buttons[pane.visible_count..] {
        assert_eq!(hidden.rect.2, 0, "hidden buttons are not drawn");
    }
    let (ox, oy, ow, _) = pane.overflow.unwrap();
    assert!(ow > 0 && ox > 0, "the overflow button has a real rect");
    let ids = pane
        .overflow_at(ox as i32 + 1, oy as i32 + 1)
        .expect("overflow is clickable");
    assert_eq!(ids, pane.overflow_ids);
}

#[test]
fn test_no_overflow_when_everything_fits() {
    let (mut pane, _d) = make_pane();
    pane.add_button(1, "A");
    pane.add_button(2, "B");
    pane.layout_buttons(800, 28);
    assert!(pane.overflow.is_none());
    assert_eq!(pane.visible_count, 2);
    assert!(pane.overflow_at(400, 4).is_none());
}
