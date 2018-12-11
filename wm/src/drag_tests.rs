    use super::*;
    use antibox_core::backend::RenderBackend;
    use antibox_core::rect::Rect;

    #[test]
    fn test_focus_frame_raises_already_focused() {
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
        let c1 = mk(100, 100);
        let c1_xid = c1.id();
        let f1 = mk(120, 140);
        let fid1 = f1.id();
        let fw1 = FrameWindow::new(ClientWindow::new(c1), f1);
        let cid1 = fw1.client_id();
        wm.xid_index.insert(cid1, c1_xid, fid1);
        wm.frames.insert(cid1, fw1);
        wm.insertion_order.push(cid1);
        let c2 = mk(100, 100);
        let c2_xid = c2.id();
        let f2 = mk(120, 140);
        let f2_id = f2.id();
        let fw2 = FrameWindow::new(ClientWindow::new(c2), f2);
        let cid2 = fw2.client_id();
        wm.xid_index.insert(cid2, c2_xid, f2_id);
        wm.frames.insert(cid2, fw2);
        wm.insertion_order.push(cid2);

        wm.focused_window = Some(cid1);
        assert_eq!(wm.insertion_order.last(), Some(&cid2));

        focus_frame(&mut wm, cid1, Some(FrameId(fid1)));
        assert_eq!(wm.insertion_order.last(), Some(&cid1));
    }

    #[test]
    fn test_drag_to_left_edge_snaps_on_release() {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use crate::manager::WindowManager;
        use crate::wmstate::ResizeEdge;
        use antibox_core::backend::{EventMask, WmWindowClass};
        use antibox_core::mock::MockDisplay;
        use antibox_core::point::Point;
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
        let client = mk(100, 100);
        let client_xid = client.id();
        let frame = mk(400, 300);
        let frame_xid = frame.id();
        let mut fw = FrameWindow::new(ClientWindow::new(client), frame);
        fw.set_frame_rect(Rect::new(300, 200, 400, 300));
        let cid = fw.client_id();
        let fid = FrameId(frame_xid);
        wm.xid_index.insert(cid, client_xid, frame_xid);
        wm.frames.insert(cid, fw);
        wm.insertion_order.push(cid);

        let init = Rect::new(300, 200, 400, 300);
        wm.drag_state = Some((fid, Point::new(500, 210), ResizeEdge::None, init));
        motion_notify(&mut wm, frame_xid, Point::new(0, 0), Point::new(2, 350));
        assert!(wm.snap_preview.is_some(), "preview should be shown at edge");
        button_release(&mut wm, frame_xid, Point::new(2, 350));
        let fr = wm.frames[&cid].frame_rect();
        assert_eq!(fr, Rect::new(0, 0, 640, 720));
        assert_eq!(
            wm.frames[&cid].snap_zone.map(|z| z.h),
            Some(Some(crate::snap::Horz::Left))
        );
        assert_eq!(wm.frames[&cid].snap_saved, Some(init));
        assert!(wm.snap_preview.is_none());
    }

    #[test]
    fn test_apply_frame_rect_resizes_client() {
        use crate::client::ClientWindow;
        use crate::frame::{border_width, title_bar_height, FrameWindow};
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
        let client = mk(100, 100);
        let client_xid = client.id();
        let frame = mk(120, 140);
        let frame_xid = frame.id();
        let fw = FrameWindow::new(ClientWindow::new(client), frame);
        let cid = fw.client_id();
        let fid = FrameId(frame_xid);
        wm.xid_index.insert(cid, client_xid, frame_xid);
        wm.frames.insert(cid, fw);
        wm.insertion_order.push(cid);

        let new = Rect::new(50, 60, 400, 300);
        apply_frame_rect(&mut wm, fid, new);
        let (cw, ch) = wm.frames[&cid].client().xwindow.get_geometry().unwrap();
        assert_eq!(cw, (400 - border_width() * 2) as u16);
        assert_eq!(ch, (300 - title_bar_height() - border_width() * 2) as u16);

        let moved = Rect::new(200, 200, 400, 300);
        apply_frame_rect(&mut wm, fid, moved);
        let (cw2, ch2) = wm.frames[&cid].client().xwindow.get_geometry().unwrap();
        assert_eq!((cw2, ch2), (cw, ch));
    }

