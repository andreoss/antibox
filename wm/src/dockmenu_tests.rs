    use super::*;
    use antibox_core::point::Point;
    #[test]
    fn test_new() {
        let m = DockMenu::new();
        assert!(!m.visible);
        assert!(m.items.is_empty());
    }
    #[test]
    fn test_show_empty() {
        use antibox_core::backend::DisplayBackend;
        use antibox_core::mock::MockDisplay;
        let b = MockDisplay::new(1280, 720, 24);
        let mut m = dock_menu(vec![]);
        m.show(&b as &dyn DisplayBackend, Point::new(100, 100));
        assert!(!m.visible);
    }
