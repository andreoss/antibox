    use super::*;
    use antibox_core::mock::MockDisplay;
    use std::sync::Arc;

    #[test]
    fn screen_dims_and_rect_follow_backend() {
        let mut wm = WindowManager::<MockDisplay>::new_test();
        assert_eq!(screen_dims(&wm), (0, 0));
        wm.backend = Some(Arc::new(MockDisplay::new(1280, 720, 24)));
        assert_eq!(screen_dims(&wm), (1280, 720));
        assert_eq!(screen_rect(&wm), Rect::new(0, 0, 1280, 720));
    }

    #[test]
    fn workarea_prefers_first_entry_and_falls_back_to_screen() {
        let mut wm = WindowManager::<MockDisplay>::new_test();
        wm.backend = Some(Arc::new(MockDisplay::new(1280, 720, 24)));
        assert_eq!(workarea(&wm), Rect::new(0, 0, 1280, 720));
        wm.workareas = vec![Rect::new(0, 0, 1280, 688), Rect::new(0, 0, 640, 480)];
        assert_eq!(workarea(&wm), Rect::new(0, 0, 1280, 688));
    }

