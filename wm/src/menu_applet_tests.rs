    use super::*;
    use std::sync::Arc;

    fn sample() -> MenuApplet {
        let display: Arc<dyn DisplayBackend> = Arc::new(antibox_core::mock::MockDisplay::new(1280, 720, 24));
        let a = MenuApplet::new(&display, 1).unwrap();
        a
    }

    #[test]
    fn label_is_stable() {
        assert!(!LABEL.is_empty());
    }

    #[test]
    fn preferred_size_is_positive() {
        let a = sample();
        assert!(a.preferred_width() > 0);
        assert!(a.preferred_height() > 0);
    }

    #[test]
    fn click_then_release_toggles_press_state() {
        let mut a = sample();
        assert!(!a.pressed);
        assert!(a.handle_click(0, 0, 1).is_some());
        assert!(a.pressed);
        assert!(a.handle_release(0, 0, 1).is_some());
        assert!(!a.pressed);
    }

    #[test]
    fn action_is_emitted_once_per_click() {
        let mut a = sample();
        assert!(a.take_action().is_none());
        let _ = a.handle_click(0, 0, 1);
        assert!(matches!(
            a.take_action(),
            Some(crate::action::Action::Menu(crate::action::MenuOp::RootMenu))
        ));
        assert!(a.take_action().is_none());
    }

    #[test]
    fn geometry_is_applied() {
        let mut a = sample();
        a.set_geometry(4, 6, 40, 20);
        assert_eq!(a.w, 40);
        assert_eq!(a.h, 20);
    }
