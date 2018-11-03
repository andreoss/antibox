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


    #[test]
    fn test_clamp_to_keeps_a_window_out_of_the_panel() {
        let wa = Rect::new(0, 0, 1280, 776);
        let screen_sized = Rect::new(0, 0, 1280, 800);
        assert_eq!(clamp_to(screen_sized, wa), Rect::new(0, 0, 1280, 776));
    }

    #[test]
    fn test_clamp_to_leaves_a_fitting_rect_alone() {
        let wa = Rect::new(0, 0, 1280, 776);
        let normal = Rect::new(101, 101, 364, 264);
        assert_eq!(clamp_to(normal, wa), normal);
    }

    #[test]
    fn test_clamp_to_pulls_an_offscreen_rect_back() {
        let wa = Rect::new(0, 24, 1280, 776);
        assert_eq!(clamp_to(Rect::new(-40, 0, 400, 300), wa), Rect::new(0, 24, 400, 300));
        assert_eq!(
            clamp_to(Rect::new(1200, 700, 400, 300), wa),
            Rect::new(880, 500, 400, 300)
        );
    }

    fn transient_pair() -> (WindowManager<antibox_core::mock::MockDisplay>, ClientId, ClientId) {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
        let d = antibox_core::mock::MockDisplay::new(1280, 720, 24);
        let win = || {
            d.create_window(
                1,
                Rect::new(0, 0, 100, 100),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap()
        };
        let mut wm = WindowManager::<antibox_core::mock::MockDisplay>::new_test();
        let mut add = |wm: &mut WindowManager<antibox_core::mock::MockDisplay>| {
            let (c, f) = (win(), win());
            let (cx, fx) = (c.id(), f.id());
            let fw = FrameWindow::new(ClientWindow::new(c), f);
            let cid = fw.client_id();
            wm.xid_index.insert(cid, cx, fx);
            wm.frames.insert(cid, fw);
            cid
        };
        let owner = add(&mut wm);
        let dialog = add(&mut wm);
        placement::add_to_transient_chain(&mut wm.frames, dialog, owner);
        (wm, owner, dialog)
    }

    #[test]
    fn test_a_dialog_follows_its_owner_into_and_out_of_minimize() {
        let (mut wm, owner, dialog) = transient_pair();
        set_transients_minimized(&mut wm, owner, true);
        assert!(
            wm.frames.get(&dialog).unwrap().state().minimized,
            "the dialog hides with its owner"
        );
        set_transients_minimized(&mut wm, owner, false);
        assert!(
            !wm.frames.get(&dialog).unwrap().state().minimized,
            "and comes back with it"
        );
    }

    #[test]
    fn test_a_dialog_minimized_by_the_user_is_not_restored_with_its_owner() {
        let (mut wm, owner, dialog) = transient_pair();
        wm.frame_mut(dialog).unwrap().state_mut().minimized = true;

        set_transients_minimized(&mut wm, owner, true);
        set_transients_minimized(&mut wm, owner, false);
        assert!(
            wm.frames.get(&dialog).unwrap().state().minimized,
            "a dialog the user minimized stays minimized when the owner returns"
        );
        assert!(
            !wm.frames.get(&dialog).unwrap().was_minimized,
            "the remembered flag is cleared once it has been honoured"
        );

        set_transients_minimized(&mut wm, owner, true);
        set_transients_minimized(&mut wm, owner, false);
        assert!(
            wm.frames.get(&dialog).unwrap().state().minimized,
            "and it survives a second round trip"
        );
    }
