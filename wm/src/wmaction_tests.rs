    use super::*;
    use antibox_core::backend::RenderBackend;

    fn tab_fixture() -> (
        WindowManager<antibox_core::mock::MockDisplay>,
        Vec<ClientId>,
    ) {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use crate::manager::WindowManager;
        use antibox_core::backend::{EventMask, WmWindowClass};
        use antibox_core::mock::MockDisplay;
        use std::sync::Arc;

        let d = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut wm = WindowManager::<MockDisplay>::new_test();
        wm.backend = Some(Arc::clone(&d));
        let mut ids = Vec::new();
        for _ in 0..2 {
            let client = d
                .create_window(
                    1,
                    Rect::new(0, 0, 400, 300),
                    WmWindowClass::InputOutput,
                    false,
                    EventMask::NO_EVENT,
                )
                .unwrap();
            let client_xid = client.id();
            let frame = d
                .create_window(
                    1,
                    Rect::new(0, 0, 406, 322),
                    WmWindowClass::InputOutput,
                    false,
                    EventMask::NO_EVENT,
                )
                .unwrap();
            let frame_xid = frame.id();
            let mut fw = FrameWindow::new(ClientWindow::new(client), frame);
            let cid = fw.client_id();
            wm.xid_index.insert(cid, client_xid, frame_xid);
            fw.set_frame_rect(Rect::new(100, 100, 406, 322));
            wm.frames.insert(cid, fw);
            wm.insertion_order.push(cid);
            wm.map_order.push(cid);
            ids.push(cid);
        }
        wm.focused_window = Some(ids[1]);
        (wm, ids)
    }

    #[test]
    fn test_tab_window_merges_frames() {
        let (mut wm, ids) = tab_fixture();
        let src_xid = wm.xid_index.xid_of(ids[1]);
        tab_window(&mut wm, ids[1], ids[0]);
        assert!(!wm.frames.contains_key(&ids[1]));
        let fw = wm.frames.get(&ids[0]).unwrap();
        assert_eq!(fw.tabbed_clients, vec![src_xid]);
        assert!(fw.tab_strip_h() > 0);
        let [_, it, _, _] = fw.client_insets();
        assert!(it >= crate::frame::title_bar_height() * 2);
        assert_eq!(wm.focused_window, Some(ids[0]));
        assert_eq!(fw.tab_display().len(), 2);
    }

    #[test]
    fn test_tab_select_swaps_active() {
        let (mut wm, ids) = tab_fixture();
        let a_xid = wm.xid_index.xid_of(ids[0]);
        let b_xid = wm.xid_index.xid_of(ids[1]);
        tab_window(&mut wm, ids[1], ids[0]);
        tab_select(&mut wm, ids[0], b_xid);
        let fw = wm.frames.get(&ids[1]).unwrap();
        assert_eq!(fw.client_xid(), b_xid);
        assert_eq!(fw.tabbed_clients, vec![a_xid]);
        assert_eq!(wm.focused_window, Some(ids[1]));
        assert!(!wm.frames.contains_key(&ids[0]));
    }

    #[test]
    fn test_untab_restores_frames() {
        let (mut wm, ids) = tab_fixture();
        tab_window(&mut wm, ids[1], ids[0]);
        wm.focused_window = Some(ids[0]);
        detach_tab(
            &mut wm,
            ids[0],
            ids[0],
            Point::new(50, 50),
        );
        assert_eq!(wm.frames.len(), 2);
        let a = wm.frames.get(&ids[0]).unwrap();
        let b = wm.frames.get(&ids[1]).unwrap();
        assert!(a.tabbed_clients.is_empty());
        assert!(b.tabbed_clients.is_empty());
        assert_eq!(a.tab_strip_h(), 0);
    }

    #[test]
    fn test_cascade_action_applies_layout_and_unmaximizes() {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use crate::manager::WindowManager;
        use antibox_core::backend::{EventMask, WmWindowClass};
        use antibox_core::mock::MockDisplay;
        use std::sync::Arc;

        let d = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut wm = WindowManager::<MockDisplay>::new_test();
        wm.backend = Some(Arc::clone(&d));
        let mk = |w: i32, h: i32| {
            d.create_window(
                1,
                Rect::new(0, 0, w, h),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap()
        };
        let mut ids = Vec::new();
        for _ in 0..2 {
            let client = mk(400, 300);
            let client_xid = client.id();
            let frame = mk(406, 322);
            let frame_xid = frame.id();
            let mut fw = FrameWindow::new(ClientWindow::new(client), frame);
            let cid = fw.client_id();
            wm.xid_index.insert(cid, client_xid, frame_xid);
            fw.set_frame_rect(Rect::new(600, 500, 406, 322));
            fw.state_mut().maximized = true;
            fw.state_mut().max_vert = true;
            fw.state_mut().max_horz = true;
            wm.frames.insert(cid, fw);
            wm.insertion_order.push(cid);
            ids.push(cid);
        }

        cascade(&mut wm);

        let step = crate::frame::title_block_height();
        let f0 = wm.frames[&ids[0]].frame_rect();
        let f1 = wm.frames[&ids[1]].frame_rect();
        assert_eq!((f0.x, f0.y), (0, 0));
        assert_eq!((f1.x, f1.y), (step, step));
        assert_eq!((f0.w, f0.h), (406, 322));
        assert!(!wm.frames[&ids[0]].state().maximized);
    }

    #[test]
    fn test_compute_tile_directional_left() {
        let r = compute_tile_directional_rect(0, 0, 0, 800, 600).unwrap();
        assert_eq!(r, Rect::new(0, 0, 400, 600));
    }

