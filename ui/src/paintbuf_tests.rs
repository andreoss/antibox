    use super::*;
    use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
    use antibox_core::mock::MockDisplay;
    use antibox_core::rect::Rect;

    #[test]
    fn buffered_paints_once_and_frees_pixmap() {
        let d = MockDisplay::new(640, 480, 24);
        let win = d
            .create_window(
                1,
                Rect::new(0, 0, 100, 50),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap();
        let before = d.windows.lock().unwrap().len();
        let mut calls = 0;
        buffered(&d, win.id(), 100, 50, |g| {
            calls += 1;
            g.set_foreground(0x123456).unwrap();
            g.fill_rect(0, 0, 100, 50).unwrap();
        });
        assert_eq!(calls, 1);
        assert_eq!(d.windows.lock().unwrap().len(), before);
    }

    #[test]
    fn buffered_paints_zero_size_without_leak() {
        let d = MockDisplay::new(640, 480, 24);
        let win = d
            .create_window(
                1,
                Rect::new(0, 0, 10, 10),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap();
        let before = d.windows.lock().unwrap().len();
        let mut calls = 0;
        buffered(&d, win.id(), 0, 0, |_| {
            calls += 1;
        });
        assert_eq!(calls, 1);
        assert_eq!(d.windows.lock().unwrap().len(), before);
    }
