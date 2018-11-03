    use super::client_list_ids;
    use crate::client::ClientWindow;
    use crate::frame::FrameWindow;
    use crate::manager::WindowManager;
    use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
    use antibox_core::mock::MockDisplay;
    use antibox_core::rect::Rect;
    use std::sync::Arc;

    #[test]
    fn client_list_follows_map_order_and_drops_nothing() {
        let d = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut wm = WindowManager::<MockDisplay>::new_test();
        wm.backend = Some(Arc::clone(&d));
        let mk = || {
            d.create_window(
                1,
                Rect::new(0, 0, 100, 100),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap()
        };
        let mut ids = vec![];
        for _ in 0..3 {
            let c = mk();
            let id = c.id();
            let fw = FrameWindow::new(ClientWindow::new(c), mk());
            let frame_xid = fw.frame().id();
            let cid = fw.client_id();
            wm.xid_index.insert(cid, id, frame_xid);
            wm.frames.insert(cid, fw);
            wm.map_order.push(cid);
            ids.push(id);
        }
        assert_eq!(client_list_ids(&wm), ids);

        let extra = mk();
        let eid = extra.id();
        let fw2 = FrameWindow::new(ClientWindow::new(extra), mk());
        let frame_xid2 = fw2.frame().id();
        let cid2 = fw2.client_id();
        wm.xid_index.insert(cid2, eid, frame_xid2);
        wm.frames.insert(cid2, fw2);
        let out = client_list_ids(&wm);
        assert_eq!(&out[..3], &ids[..]);
        assert!(out.contains(&eid));
        assert_eq!(out.len(), 4);
    }
