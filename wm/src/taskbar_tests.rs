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
        assert!(l.contains(&PanelSlot::Clock), "left={:?} right={:?}", l, r);
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


