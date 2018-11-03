    use super::*;

    #[test]
    fn test_button_height_fits_within_panel() {
        assert_eq!(
            button_height(),
            (panel_height() - button_inset() * 2).max(1)
        );
        assert!(button_height() < panel_height());
    }

    #[test]
    fn test_text_w_is_linear() {
        assert_eq!(text_w(0), 0);
        assert_eq!(text_w(4), 4 * text_w(1));
        assert!(text_w(2) > text_w(1));
    }

