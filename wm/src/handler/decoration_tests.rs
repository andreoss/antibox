    use super::{hide_from_taskbar_on_map, window_type_decorated};
    use crate::client::{ClientWindow, WindowType};
    use antibox_core::backend::hints::{mwm_hints_flags, MwmHints};
    use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
    use antibox_core::mock::MockDisplay;
    use antibox_core::rect::Rect;

    #[test]
    fn test_window_type_decorated() {
        assert!(!window_type_decorated(WindowType::Dock));
        assert!(!window_type_decorated(WindowType::Desktop));
        assert!(!window_type_decorated(WindowType::Splash));
        assert!(window_type_decorated(WindowType::Normal));
        assert!(window_type_decorated(WindowType::Dialog));
        assert!(window_type_decorated(WindowType::Utility));
        assert!(window_type_decorated(WindowType::Toolbar));
        assert!(window_type_decorated(WindowType::Menu));
    }

    fn client_of_type(wt: WindowType, undecorated: bool) -> ClientWindow {
        let d = MockDisplay::new(1280, 720, 24);
        let w = d
            .create_window(
                1,
                Rect::new(0, 0, 100, 100),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap();
        let mut c = ClientWindow::new(w);
        c.window_type = wt;
        if undecorated {
            c.mwm_hints = Some(MwmHints {
                flags: mwm_hints_flags::DECORATIONS,
                functions: 0,
                decorations: 0,
                input_mode: 0,
            });
        }
        c
    }

    #[test]
    fn undecorated_normal_window_stays_in_taskbar() {
        assert!(!hide_from_taskbar_on_map(&client_of_type(
            WindowType::Normal,
            true
        )));
        assert!(!hide_from_taskbar_on_map(&client_of_type(
            WindowType::Dialog,
            true
        )));
        assert!(hide_from_taskbar_on_map(&client_of_type(
            WindowType::Splash,
            false
        )));
        assert!(hide_from_taskbar_on_map(&client_of_type(
            WindowType::Desktop,
            false
        )));
        assert!(hide_from_taskbar_on_map(&client_of_type(
            WindowType::Dock,
            false
        )));
    }
