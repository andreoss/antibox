    use super::*;
    use crate::backend::{EventMask, RenderBackend, WindowHandle, WmWindowClass};
    use crate::rect::Rect;

    fn make_window(display: &MockDisplay) -> Box<dyn WindowHandle> {
        display
            .create_window(
                display.root_window,
                Rect::new(0, 0, 100, 100),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap()
    }

    #[test]
    fn lifecycle_records_create_map_unmap_destroy() {
        let display = MockDisplay::new(800, 600, 24);
        let window = make_window(&display);
        window.map().unwrap();
        window.unmap().unwrap();
        window.destroy().unwrap();
        let id = window.id();
        assert_eq!(
            display.lifecycle_events(),
            vec![
                (id, Lifecycle::Create),
                (id, Lifecycle::Map),
                (id, Lifecycle::Unmap),
                (id, Lifecycle::Destroy)
            ]
        );
    }

    #[test]
    fn lifecycle_interleaves_multiple_windows() {
        let display = MockDisplay::new(800, 600, 24);
        let a = make_window(&display);
        let b = make_window(&display);
        b.map().unwrap();
        a.map().unwrap();
        assert_eq!(
            display.lifecycle_events(),
            vec![
                (a.id(), Lifecycle::Create),
                (b.id(), Lifecycle::Create),
                (b.id(), Lifecycle::Map),
                (a.id(), Lifecycle::Map)
            ]
        );
    }

