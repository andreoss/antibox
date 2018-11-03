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
