    use super::*;

    fn tc() -> antibox_core::colour::Colour {
        crate::theme::menu_bg()
    }

    fn children_of<'a>(nodes: &'a [MenuNode<Action>], title: &str) -> &'a [MenuNode<Action>] {
        nodes
            .iter()
            .find_map(|n| match n {
                MenuNode::Group {
                    title: t, children, ..
                } if t == title => Some(children.as_slice()),
                _ => None,
            })
            .expect("group present")
    }

    #[test]
    fn test_new() {
        let m = WindowActionMenu::new();
        assert!(!m.visible());
        assert!(m.nodes.is_empty());
    }

    #[test]
    fn test_for_focused_client() {
        let m = WindowActionMenu::for_focused_client(4, tc(), false);
        assert!(!m.visible());
        assert!(!m.nodes.is_empty());
        assert_eq!(m.nodes[0].title(), "_Restore");
        assert_eq!(crate::menurender::mnemonic_key(m.nodes[0].title()), Some('R'));
        assert!(matches!(m.nodes[4], MenuNode::Group { .. }));
        assert_eq!(children_of(&m.nodes, "_Tile").len(), 8);
        assert!(children_of(&m.nodes, "Ma_ximize")
            .iter()
            .all(|n| matches!(n, MenuNode::Leaf { .. })));
        assert_eq!(children_of(&m.nodes, "_Workspace").len(), 4);
    }

    #[test]
    fn test_groups_expand_in_place_when_toggled() {
        let m = WindowActionMenu::for_focused_client(2, tc(), false);
        let mut nodes = m.nodes.clone();
        let flat = crate::menu_tree::flatten_nodes(&nodes);
        let idx = flat
            .iter()
            .position(|r| r.title == "_Workspace")
            .expect("workspace row");
        let path = match &flat[idx].entry {
            crate::menu_tree::FlatEntry::Group { path, .. } => path.clone(),
            _ => panic!("expected a group row"),
        };
        assert!(crate::menu_tree::toggle_at(&mut nodes, &path));
        let flat = crate::menu_tree::flatten_nodes(&nodes);
        let ws1 = flat
            .iter()
            .find(|r| r.title == "Workspace 1")
            .expect("expanded child");
        assert_eq!(ws1.depth, 1);
    }

    #[test]
    fn test_wheel_does_not_activate_or_close() {
        use antibox_core::mock::MockDisplay;
        use antibox_core::point::Point;
        let conn = MockDisplay::new(1280, 800, 24);
        let mut m = WindowActionMenu::for_focused_client(4, tc(), false);
        m.show(&conn, Point::new(10, 10));
        assert!(m.visible());
        for button in [4u8, 5u8] {
            let ev = BackendEvent::ButtonPress {
                window: 1,
                event: 1,
                point: Point::new(5, 5),
                root: Point::new(15, 20),
                button,
                state: 0,
            };
            assert!(matches!(m.handle_event(&conn, &ev), MenuNav::Handled));
            assert!(m.visible());
        }
    }

    #[test]
    fn test_press_outside_closes() {
        use antibox_core::mock::MockDisplay;
        use antibox_core::point::Point;
        let conn = MockDisplay::new(1280, 800, 24);
        let mut m = WindowActionMenu::for_focused_client(4, tc(), false);
        m.show(&conn, Point::new(10, 10));
        let ev = BackendEvent::ButtonPress {
            window: 1,
            event: 1,
            point: Point::new(5, 5),
            root: Point::new(1200, 700),
            button: 1,
            state: 0,
        };
        assert!(matches!(m.handle_event(&conn, &ev), MenuNav::Close));
    }

    #[test]
    fn test_rollup_entry_follows_shade_state() {
        let count = |nodes: &[MenuNode<Action>], t: &str| {
            nodes
                .iter()
                .filter(|n| matches!(n, MenuNode::Leaf { title, .. } if title == t))
                .count()
        };
        let plain = WindowActionMenu::action_nodes(2, &[], false);
        assert_eq!(count(&plain, "Roll_up"), 1);
        assert_eq!(count(&plain, "Un_roll"), 0);
        let rolled = WindowActionMenu::action_nodes(2, &[], true);
        assert_eq!(count(&rolled, "Roll_up"), 0);
        assert_eq!(count(&rolled, "Un_roll"), 1);
        assert!(rolled.iter().all(
            |n| !matches!(n, MenuNode::Group { title, .. } if title == "Roll_up")
        ));
    }
