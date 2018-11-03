    use super::*;
    
    
    

    #[test]
    fn requested_position_ignores_bogus_program_origin() {
        use antibox_core::backend::hints::size_hints_flags::{P_POSITION, US_POSITION};
        use antibox_core::backend::SizeHints;
        let hints = |flags, x, y| SizeHints {
            flags,
            x,
            y,
            ..Default::default()
        };
        assert_eq!(requested_position(Some(&hints(P_POSITION, 0, 0))), None);
        assert_eq!(
            requested_position(Some(&hints(P_POSITION, 40, 30))),
            Some(Point::new(40, 30))
        );
        assert_eq!(
            requested_position(Some(&hints(US_POSITION, 0, 0))),
            Some(Point::new(0, 0))
        );
        assert_eq!(requested_position(Some(&hints(0, 5, 5))), None);
        assert_eq!(requested_position(None), None);
    }

    #[test]
    fn test_clamp_window_y_keeps_frame_below_top_panel() {
        let wa = Rect::new(0, 26, 1280, 774);
        let dec = crate::frame::title_block_height();
        assert_eq!(clamp_window_y(0, 300, wa), wa.y + dec);
        assert_eq!(clamp_window_y(10, 300, wa), wa.y + dec);
        assert_eq!(clamp_window_y(400, 300, wa), 400);
        assert_eq!(clamp_window_y(5, 2000, wa), wa.y + dec);
    }

