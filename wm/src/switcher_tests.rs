    use super::*;

    #[test]
    fn test_new() {
        let s = SwitcherWindow::new();
        assert!(s.items.is_empty());
        assert!(!s.visible);
    }

    #[test]
    fn test_cycle() {
        let mut s = SwitcherWindow::new();
        s.items.push(SwitcherItem {
            title: "Window A".into(),
            client_id: 1,
            members: vec![1],
            workspace: 0,
            class_instance: None,
        });
        s.items.push(SwitcherItem {
            title: "Window B".into(),
            client_id: 2,
            members: vec![2],
            workspace: 1,
            class_instance: None,
        });
        s.active_idx = 0;
        s.cycle(true);
        assert_eq!(s.active_idx, 1);
        s.cycle(true);
        assert_eq!(s.active_idx, 0);
        s.cycle(false);
        assert_eq!(s.active_idx, 1);
    }

