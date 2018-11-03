    use super::*;
    use antibox_core::backend::RenderBackend;

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

