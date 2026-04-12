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
