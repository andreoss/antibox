    use super::*;
    
    

    #[test]
    fn test_bar_height_double_derives_from_panel_height() {
        let edge = antibox_ui::theme::panel_edge_height();
        let panel = antibox_ui::metrics::panel_height() as u16;
        assert_eq!(TaskBar::bar_height_for(false), panel + edge);
        assert_eq!(TaskBar::bar_height_for(true), panel * 2 + edge);
    }

    #[test]
    fn effective_slots_default_when_unset() {
        let (left, right) = effective_slots(None);
        assert_eq!(left, DEFAULT_LEFT.to_vec());
        assert_eq!(right, DEFAULT_RIGHT.to_vec());
    }

    mod taskbar_menu_tests {

    }

    use crate::layout_preferences::Widget;

    #[test]
    fn qa_probe_clock_moved_left_of_tasks() {
        let (l, r) = effective_slots(Some(vec![
            Widget::Clock,
            Widget::Workspaces,
            Widget::Windows,
        ]));
        assert!(l.contains(&PanelSlot::Clock), "left={l:?} right={r:?}");
        assert!(!r.contains(&PanelSlot::Clock));
    }

    #[test]
    fn qa_probe_clock_after_tasks_is_right() {
        let (_l, r) = effective_slots(Some(vec![
            Widget::Workspaces,
            Widget::Windows,
            Widget::Clock,
        ]));
        assert_eq!(r, vec![PanelSlot::Clock]);
    }

    #[test]
    fn qa_probe_layout_without_tasks_all_left() {
        let (l, r) = effective_slots(Some(vec![Widget::Workspaces, Widget::Clock]));
        assert_eq!(l, vec![PanelSlot::Workspaces, PanelSlot::Clock]);
        assert!(r.is_empty());
    }



    #[test]
    fn qa_probe_mpris_shares_the_power_slot() {
        let (_l, r) = effective_slots(Some(vec![
            Widget::Windows,
            Widget::PowerAudio,
            Widget::Mpris,
            Widget::Clock,
        ]));
        assert_eq!(r, vec![PanelSlot::PowerAudio, PanelSlot::Clock]);
        let (_l, r) = effective_slots(Some(vec![Widget::Windows, Widget::Mpris]));
        assert_eq!(r, vec![PanelSlot::PowerAudio]);
    }

    #[test]
    fn qa_probe_mpris_and_power_share_one_slot() {
        assert!(shares_slot(Widget::Mpris, Widget::PowerAudio));
        assert!(shares_slot(Widget::PowerAudio, Widget::Mpris));
        assert!(!shares_slot(Widget::Mpris, Widget::Clock));
    }

#[test]
fn test_position_parses_every_mode() {
    use TaskBarPosition as P;
    assert_eq!(P::parse("top"), Some(P::Top));
    assert_eq!(P::parse("Bottom"), Some(P::Bottom));
    assert_eq!(P::parse("left"), Some(P::Left));
    assert_eq!(P::parse("right"), Some(P::Right));
    assert_eq!(P::parse("left-vertical"), Some(P::LeftVertical));
    assert_eq!(P::parse(" right-vertical "), Some(P::RightVertical));
    assert_eq!(P::parse("disabled"), Some(P::Disabled));
    assert_eq!(P::parse("sideways"), None);
    assert!(P::Left.is_rotated() && P::Right.is_rotated());
    assert!(!P::LeftVertical.is_rotated() && !P::Top.is_rotated());
    assert!(P::LeftVertical.is_side() && P::Right.is_side() && !P::Bottom.is_side());
}

#[test]
fn test_place_on_screen_covers_each_edge() {
    use TaskBarPosition as P;
    assert_eq!(TaskBar::place_on_screen(P::Top, 1280, 800, 30), (1280, (0, 0, 1280, 30)));
    assert_eq!(
        TaskBar::place_on_screen(P::Bottom, 1280, 800, 30),
        (1280, (0, 770, 1280, 30))
    );
    assert_eq!(TaskBar::place_on_screen(P::Left, 1280, 800, 30), (800, (0, 0, 30, 800)));
    assert_eq!(
        TaskBar::place_on_screen(P::Right, 1280, 800, 30),
        (800, (1250, 0, 30, 800))
    );
    assert_eq!(
        TaskBar::place_on_screen(P::RightVertical, 1280, 800, 160),
        (800, (1120, 0, 160, 800))
    );
}

#[test]
fn test_rotation_turns_the_strip_and_back() {
    let (w, h) = (3u16, 2u16);
    let mut src = vec![0u8; w as usize * h as usize * 4];
    for i in 0..(w as usize * h as usize) {
        src[i * 4] = i as u8;
    }
    let cw = rotate_cw(&src, w, h);
    assert_eq!(cw.len(), src.len());
    assert_eq!(cw[0], 3);
    assert_eq!(cw[4], 0);
    let ccw = rotate_ccw(&src, w, h);
    assert_eq!(ccw[0], 2);
    assert_eq!(ccw[4], 5);
}

#[test]
fn test_unrotate_point_inverts_the_turn() {
    use TaskBarPosition as P;
    let (len, thick) = (100, 28);
    for (x, y) in [(0, 0), (17, 5), (99, 27)] {
        let left = (thick - 1 - y, x);
        assert_eq!(unrotate_point(P::Left, left, len, thick), (x, y));
        let right = (y, len - 1 - x);
        assert_eq!(unrotate_point(P::Right, right, len, thick), (x, y));
    }
    assert_eq!(unrotate_point(P::Bottom, (4, 9), len, thick), (4, 9));
}

#[test]
fn test_logical_rect_lands_where_the_bar_drew_it() {
    use TaskBarPosition as P;
    let (len, thick) = (600, 22);
    let rect = (0, 2, 180, 18);
    assert_eq!(logical_to_window(P::Left, rect, len, thick), (2, 0, 18, 180));
    assert_eq!(logical_to_window(P::Right, rect, len, thick), (2, 420, 18, 180));
    assert_eq!(logical_to_window(P::Bottom, rect, len, thick), rect);
    assert_eq!(logical_to_window(P::LeftVertical, rect, len, thick), rect);
}

#[test]
fn test_mapped_rect_holds_the_points_it_answers_for() {
    use TaskBarPosition as P;
    let (len, thick) = (600, 22);
    let rect = (30, 0, 90, 22);
    for position in [P::Left, P::Right] {
        let (wx, wy, ww, wh) = logical_to_window(position, rect, len, thick);
        for (px, py) in [(wx, wy), (wx + ww - 1, wy + wh - 1), (wx + ww / 2, wy + wh / 2)] {
            let (lx, ly) = unrotate_point(position, (px, py), len, thick);
            assert!(lx >= rect.0 && lx < rect.0 + rect.2, "{position:?} x {lx}");
            assert!(ly >= rect.1 && ly < rect.1 + rect.3, "{position:?} y {ly}");
        }
    }
}

#[test]
fn test_position_survives_the_round_trip_through_a_code() {
    use TaskBarPosition as P;
    for position in [P::Top, P::Bottom, P::Left, P::Right, P::LeftVertical, P::RightVertical, P::Disabled] {
        assert_eq!(P::from_code(position.code()), position);
    }
    assert_eq!(P::from_code(200), P::Bottom);
}

fn bar_on_mock(position: TaskBarPosition) -> TaskBar {
    let conn: Arc<dyn DisplayBackend> =
        Arc::new(antibox_core::mock::MockDisplay::new(1280, 800, 24));
    TaskBar::new(&conn, position, 0).expect("the mock backend makes windows")
}

#[test]
fn test_an_upright_bar_keeps_its_length_through_a_reload() {
    let mut tb = bar_on_mock(TaskBarPosition::RightVertical);
    let thick = TaskBar::thickness_for(TaskBarPosition::RightVertical);
    assert_eq!(tb.thickness(), thick);
    assert_eq!(tb.height, 800);
    assert!(!tb.update_height(), "an upright bar has no bar height to chase");
    assert_eq!(tb.thickness(), thick);
    assert_eq!(tb.height, 800, "the bar length must survive a reload");
    assert_eq!(tb.strut().right, u32::from(thick));
}

#[test]
fn test_fitting_a_rotated_bar_keeps_its_thickness() {
    let mut tb = bar_on_mock(TaskBarPosition::Left);
    let thick = tb.thickness();
    tb.fit_to_screen();
    assert_eq!(tb.thickness(), thick);
    assert_eq!(tb.width, 800, "the length runs down the screen");
    assert_eq!(tb.strut().left, u32::from(thick));
}

#[test]
fn test_closing_a_bar_drops_its_applets() {
    let mut tb = bar_on_mock(TaskBarPosition::Bottom);
    let id = tb.window.id();
    tb.close();
    assert!(tb.applets.is_empty());
    assert!(tb.menu.is_none());
    assert_eq!(id, tb.window.id());
}

#[test]
fn test_flow_rows_wraps_when_the_row_is_full() {
    let sizes = [(40, 18), (40, 18), (40, 18), (64, 22)];
    let (spots, height) = flow_rows(&sizes, 148, 3);
    assert_eq!(spots[0], (0, 0));
    assert_eq!(spots[1], (43, 0));
    assert_eq!(spots[2], (86, 0));
    assert_eq!(spots[3], (0, 21), "the fourth widget starts a new row");
    assert_eq!(height, 21 + 22);
}

#[test]
fn test_flow_rows_never_runs_past_the_bar() {
    let sizes = [(200, 20), (30, 20)];
    let (spots, height) = flow_rows(&sizes, 100, 4);
    assert_eq!(spots[0], (0, 0), "an oversized widget is clamped, not shoved out");
    assert_eq!(spots[1], (0, 24));
    assert_eq!(height, 44);
}

#[test]
fn test_flow_rows_keeps_one_row_when_everything_fits() {
    let sizes = [(20, 10), (20, 14)];
    let (spots, height) = flow_rows(&sizes, 100, 2);
    assert_eq!(spots, vec![(0, 0), (22, 0)]);
    assert_eq!(height, 14, "the row is as tall as its tallest widget");
}

#[test]
fn test_fit_row_keeps_widgets_on_one_line() {
    let fitted = fit_row(&[90, 90, 90], 148, 3, 16).expect("three graphs share the line");
    assert_eq!(fitted.len(), 3);
    let span: i16 = fitted.iter().sum::<i16>() + 3 * 2;
    assert!(span <= 148, "span {span}");
    assert!(fitted.iter().all(|w| *w >= 16));
}

#[test]
fn test_fit_row_leaves_room_alone_when_it_fits() {
    assert_eq!(fit_row(&[40, 40], 148, 3, 16), Some(vec![40, 40]));
}

#[test]
fn test_fit_row_gives_up_when_the_floor_does_not_fit() {
    assert_eq!(fit_row(&[40, 40, 40, 40, 40, 40], 60, 3, 16), None);
}

#[test]
fn test_bands_keep_the_tray_and_clock_on_their_own_lines() {
    use PanelSlot as S;
    let bands = bands_of(&[S::Cpu, S::Mem, S::Net, S::PowerAudio, S::Keyboard, S::Tray, S::Clock]);
    assert_eq!(bands.len(), 4);
    assert_eq!(bands[0], vec![S::Cpu, S::Mem, S::Net]);
    assert_eq!(bands[1], vec![S::PowerAudio, S::Keyboard]);
    assert_eq!(bands[2], vec![S::Tray], "the tray owns its line");
    assert_eq!(bands[3], vec![S::Clock], "the clock owns its line");
}

#[test]
fn test_bands_keep_other_widgets_apart() {
    use PanelSlot as S;
    let bands = bands_of(&[S::Workspaces, S::Menu, S::Cpu]);
    assert_eq!(bands.len(), 3);
}

#[test]
fn test_wrapped_rows_are_centred() {
    let sizes = [(60, 20), (60, 20), (60, 20)];
    let (spots, _) = flow_rows(&sizes, 130, 4);
    let offsets = row_offsets(&sizes, &spots, 130, 4);
    let full_row = offsets[&spots[0].1];
    let short_row = offsets[&spots[2].1];
    assert_eq!(full_row, (130 - 124) / 2, "a full row keeps its slack halved");
    assert!(short_row > full_row, "a lone widget is pushed to the middle");
    assert_eq!(short_row, (130 - 60) / 2);
}

#[test]
fn test_the_upright_width_never_drops_below_the_minimum() {
    let min = TaskBar::upright_min_thickness();
    assert!(min > 0);
    crate::layout_preferences::set_taskbar_width_px(0);
    assert_eq!(TaskBar::upright_thickness(), min, "no width set means the minimum");
    crate::layout_preferences::set_taskbar_width_px(10);
    assert_eq!(TaskBar::upright_thickness(), min, "a tiny width is forced up");
    let wide = i64::from(min) * 2;
    crate::layout_preferences::set_taskbar_width_px(wide);
    assert!(TaskBar::upright_thickness() >= min * 2, "a real width is honoured");
    crate::layout_preferences::set_taskbar_width_px(0);
}
