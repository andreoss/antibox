    use super::*;
    use antibox_core::mock::MockDisplay;

    use std::sync::Arc;
    fn make_theme() -> ThemeColors {
        ThemeColors {
            workspace_active_bg: 0x5080D0,
            workspace_active_fg: 0xFFFFFF,
            workspace_normal_bg: 0x404040,
            workspace_normal_fg: 0xFFFFFF,
            ..ThemeColors::default()
        }
    }
    #[test]
    fn test_new() {
        let d = MockDisplay::new(1280, 720, 24);
        let conn = Arc::new(d) as Arc<dyn DisplayBackend>;
        let names = vec!["1".into(), "2".into(), "3".into()];
        let pane = WorkspacesPane::new(&conn, conn.root().read_id(), &names, make_theme()).unwrap();
        assert_eq!(pane.buttons.len(), 3);
        assert!(pane.buttons[0].active);
        assert!(!pane.buttons[1].active);
    }
    #[test]
    fn test_set_names_rebuilds_and_preserves_active() {
        let d = MockDisplay::new(1280, 720, 24);
        let conn = Arc::new(d) as Arc<dyn DisplayBackend>;
        let names = vec!["W1".into(), "W2".into(), "W3".into(), "W4".into()];
        let mut pane =
            WorkspacesPane::new(&conn, conn.root().read_id(), &names, make_theme()).unwrap();
        pane.set_active(2);
        pane.set_names(&["Main".into(), "Dev".into(), "Chat".into()]);
        assert_eq!(pane.buttons.len(), 3);
        assert_eq!(pane.buttons[0].label, "Main");
        assert_eq!(pane.buttons[2].label, "Chat");
        assert!(pane.buttons[2].active);
        assert!(!pane.buttons[0].active);
    }

