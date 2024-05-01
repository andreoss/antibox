    use super::*;
    use antibox_core::rect::Rect;
    use crate::client::ClientWindow;
    use crate::frame::FrameWindow;
    use antibox_core::mock::MockDisplay;

    fn insert_client(wm: &mut WindowManager<dyn DisplayBackend>) -> crate::id::ClientId {
        let d = MockDisplay::new(1280, 720, 24);
        let client_win = d
            .create_window(
                1,
                Rect::new(0, 0, 100, 100),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap();
        let frame_win = d
            .create_window(
                1,
                Rect::new(0, 0, 120, 140),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap();
        let client_xid = client_win.id();
        let frame_xid = frame_win.id();
        let fw = FrameWindow::new(ClientWindow::new(client_win), frame_win);
        let cid = fw.client_id();
        wm.xid_index.insert(cid, client_xid, frame_xid);
        wm.frames.insert(cid, fw);
        cid
    }

    #[test]
    fn test_switcher_goes_to_previous_window_first() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let _a = insert_client(&mut wm);
        let b = insert_client(&mut wm);
        let c = insert_client(&mut wm);
        wm.focused_window = Some(c);
        wm.last_focused_window = Some(b);
        let mut m = WinListMenu::new();
        m.rebuild(&wm);
        let sel = m.selection_from_focus(&wm, true).unwrap();
        let got = m.view.items[sel].payload().unwrap().client_id;
        assert_eq!(got, wm.xid_index.xid_of(b));
        wm.focused_window = Some(b);
        wm.last_focused_window = Some(c);
        m.rebuild(&wm);
        let sel = m.selection_from_focus(&wm, true).unwrap();
        let got = m.view.items[sel].payload().unwrap().client_id;
        assert_eq!(got, wm.xid_index.xid_of(c));
    }

    fn list_with(n: usize) -> WinListMenu {
        let mut m = WinListMenu::new();
        let leaves: Vec<MenuNode<WinListItem>> = (0..n)
            .map(|i| {
                let it = WinListItem {
                    title: format!("win{i}"),
                    client_id: i as u32 + 1,
                    workspace: 0,
                    icon: None,
                };
                MenuNode::leaf(it.title.clone(), it)
            })
            .collect();
        m.view
            .set_tree(vec![MenuNode::group_expanded("Workspace 1", leaves)]);
        m.view.set_size(scaled(280) as u16, scaled(340) as u16);
        m
    }

    #[test]
    fn test_new() {
        let m = WinListMenu::new();
        assert!(!m.visible);
        assert!(m.view.window.is_none());
        assert!(!m.owns_window(5));
    }

    #[test]
    fn rebuild_groups_windows_under_their_workspace() {
        let m = list_with(2);
        assert_eq!(m.view.items.len(), 3);
        assert!(m.view.items[0].is_group());
        assert_eq!(m.view.items[0].title, "Workspace 1");
        assert_eq!(m.view.items[1].depth, 1);
        assert_eq!(m.view.items[1].payload().unwrap().client_id, 1);
    }

    fn shown_list(conn: &Arc<dyn DisplayBackend>, n: usize) -> WinListMenu {
        let mut m = WinListMenu::new();
        let leaves: Vec<MenuNode<WinListItem>> = (0..n)
            .map(|i| {
                let it = WinListItem {
                    title: format!("win{i}"),
                    client_id: i as u32 + 1,
                    workspace: 0,
                    icon: None,
                };
                MenuNode::leaf(it.title.clone(), it)
            })
            .collect();
        m.view.show(
            conn.as_ref(),
            vec![MenuNode::group_expanded("Workspace 1", leaves)],
            crate::listview::Place::Centre,
            scaled(280) as u16,
            0,
        );
        m.visible = true;
        m
    }

    #[test]
    fn escape_through_the_key_input_path_closes_and_clears_the_list() {
        let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let mut m = shown_list(&conn, 3);
        let mapping = KeyboardMapping {
            keysyms_per_keycode: 1,
            keysyms: vec![0xFF1B],
        };
        assert!(m.handle_key_input(&conn, &mut wm, 8, 0, &mapping, 0xFF1B));
        m.hide(&conn);
        assert!(!m.visible);
        assert!(m.view.window.is_none());
        assert!(m.view.items.is_empty());
    }

    #[test]
    fn escape_keysym_alone_reports_close() {
        let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let mut m = shown_list(&conn, 2);
        assert!(m.handle_key(&conn, &mut wm, 0xFF1B));
        assert!(!m.handle_key(&conn, &mut wm, 0xFF54));
    }

    #[test]
    fn window_at_survives_rows_referencing_missing_items() {
        let mut m = list_with(3);
        m.visible = true;
        m.view.items.truncate(1);
        for vr in 0..(m.view.items.len() as i16 + 2) {
            let py = 3 + vr * row_h() + 2;
            let _ = m.window_at(Point::new(10, py as i32));
        }
    }
