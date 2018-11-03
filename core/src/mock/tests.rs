    use super::*;
    use crate::backend::*;
    use crate::rect::Rect;

    #[test]
    fn test_mock_display_creation() {
        let display = MockDisplay::new(1280, 720, 24);
        assert_eq!(display.screen_width(), 1280);
        assert_eq!(display.screen_height(), 720);
        assert_eq!(display.screen_depth(), 24);
    }

    #[test]
    fn test_mock_display_create_window() {
        let display = MockDisplay::new(1280, 720, 24);
        let window = display
            .create_window(
                1,
                Rect::new(0, 0, 100, 100),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap();
        assert_eq!(window.id(), 2);
    }

