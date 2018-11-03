    use super::*;

    #[test]
    fn test_new() {
        let d = DockManager::new();
        assert_eq!(d.iter().count(), 0);
    }

    #[test]
    fn test_dock_adds_window() {
        let mut dock = DockManager::new();
        assert!(dock.dock(42));
        assert_eq!(dock.iter().count(), 1);
        assert!(dock.is_dock_app(42));
    }

