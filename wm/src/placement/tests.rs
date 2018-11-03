    use super::*;
    
    
    

    #[test]
    fn requested_position_ignores_bogus_program_origin() {
        use antibox_core::backend::hints::size_hints_flags::{P_POSITION, US_POSITION};
        use antibox_core::backend::SizeHints;
        let hints = |flags, x, y| SizeHints {
            flags,
            x,
            y,
            ..Default::default()
        };
        assert_eq!(requested_position(Some(&hints(P_POSITION, 0, 0))), None);
        assert_eq!(
            requested_position(Some(&hints(P_POSITION, 40, 30))),
            Some(Point::new(40, 30))
        );
        assert_eq!(
            requested_position(Some(&hints(US_POSITION, 0, 0))),
            Some(Point::new(0, 0))
        );
        assert_eq!(requested_position(Some(&hints(0, 5, 5))), None);
        assert_eq!(requested_position(None), None);
    }

    #[test]
    fn test_clamp_window_y_keeps_frame_below_top_panel() {
        let wa = Rect::new(0, 26, 1280, 774);
        let dec = crate::frame::title_block_height();
        assert_eq!(clamp_window_y(0, 300, wa), wa.y + dec);
        assert_eq!(clamp_window_y(10, 300, wa), wa.y + dec);
        assert_eq!(clamp_window_y(400, 300, wa), 400);
        assert_eq!(clamp_window_y(5, 2000, wa), wa.y + dec);
    }


    #[test]
    fn test_fit_size_shrinks_a_window_that_would_cover_the_panel() {
        let wa = Rect::new(0, 0, 1280, 776);
        assert_eq!(fit_size(1280, 800, wa, (8, 26), None, false), (1272, 750));
    }

    #[test]
    fn test_fit_size_leaves_a_window_that_already_fits() {
        let wa = Rect::new(0, 0, 1280, 776);
        assert_eq!(fit_size(640, 480, wa, (8, 26), None, false), (640, 480));
    }

    #[test]
    fn test_fit_size_honours_a_minimum_size_hint() {
        use antibox_core::backend::hints::{size_hints_flags::P_MIN_SIZE, SizeHints};
        let wa = Rect::new(0, 0, 400, 300);
        let mut h = SizeHints::default();
        h.flags = P_MIN_SIZE;
        h.min_width = 800;
        h.min_height = 600;
        assert_eq!(
            fit_size(1280, 800, wa, (8, 26), Some(&h), false),
            (800, 600),
            "a client that cannot shrink keeps its minimum"
        );
    }

    #[test]
    fn test_a_dialog_is_lifted_to_its_owners_layer() {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use crate::wmstate::WinLayer;
        use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
        use antibox_core::mock::MockDisplay;

        let d = MockDisplay::new(1280, 720, 24);
        let win = |w: i32, h: i32| {
            d.create_window(
                1,
                Rect::new(0, 0, w, h),
                WmWindowClass::InputOutput,
                false,
                EventMask::NO_EVENT,
            )
            .unwrap()
        };
        let mut m = WindowManager::<MockDisplay>::new_test();
        let mut add = |m: &mut WindowManager<MockDisplay>| {
            let (c, f) = (win(100, 100), win(120, 140));
            let (cx, fx) = (c.id(), f.id());
            let fw = FrameWindow::new(ClientWindow::new(c), f);
            let cid = fw.client_id();
            m.xid_index.insert(cid, cx, fx);
            m.frames.insert(cid, fw);
            (cid, cx)
        };
        let (owner, owner_xid) = add(&mut m);
        let (dialog, _) = add(&mut m);
        m.frames.get_mut(&dialog).unwrap().client.transient_for = Some(owner_xid);

        m.frames.get_mut(&owner).unwrap().set_layer(WinLayer::Fullscreen);
        assert_eq!(
            effective_layer(&m, dialog),
            WinLayer::Fullscreen,
            "a dialog must not be left below its own fullscreen parent"
        );
        assert_eq!(effective_layer(&m, owner), WinLayer::Fullscreen);

        m.frames.get_mut(&owner).unwrap().set_layer(WinLayer::Normal);
        m.frames.get_mut(&dialog).unwrap().set_layer(WinLayer::OnTop);
        assert_eq!(
            effective_layer(&m, dialog),
            WinLayer::OnTop,
            "a dialog keeps its own higher layer"
        );
    }

    #[test]
    fn test_effective_layer_survives_a_transient_cycle() {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use crate::wmstate::WinLayer;
        use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
        use antibox_core::mock::MockDisplay;

        let d = MockDisplay::new(1280, 720, 24);
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
        let mut m = WindowManager::<MockDisplay>::new_test();
        let mut add = |m: &mut WindowManager<MockDisplay>| {
            let (c, f) = (win(), win());
            let (cx, fx) = (c.id(), f.id());
            let fw = FrameWindow::new(ClientWindow::new(c), f);
            let cid = fw.client_id();
            m.xid_index.insert(cid, cx, fx);
            m.frames.insert(cid, fw);
            (cid, cx)
        };
        let (a, ax) = add(&mut m);
        let (b, bx) = add(&mut m);
        m.frames.get_mut(&a).unwrap().client.transient_for = Some(bx);
        m.frames.get_mut(&b).unwrap().client.transient_for = Some(ax);
        assert_eq!(effective_layer(&m, a), WinLayer::Normal, "a cycle must terminate");
    }

    #[test]
    fn test_transient_descendants_walks_nested_dialogs_and_stops_on_a_cycle() {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
        use antibox_core::mock::MockDisplay;

        let d = MockDisplay::new(1280, 720, 24);
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
        let mut frames = FrameStore::new();
        let mut add = |frames: &mut FrameStore| {
            let fw = FrameWindow::new(ClientWindow::new(win()), win());
            let cid = fw.client_id();
            frames.insert(cid, fw);
            cid
        };
        let owner = add(&mut frames);
        let child = add(&mut frames);
        let grandchild = add(&mut frames);
        add_to_transient_chain(&mut frames, child, owner);
        add_to_transient_chain(&mut frames, grandchild, child);
        assert_eq!(
            transient_descendants(&frames, owner),
            vec![child, grandchild],
            "nested dialogs are reached"
        );

        add_to_transient_chain(&mut frames, owner, grandchild);
        let walked = transient_descendants(&frames, owner);
        assert!(walked.len() <= 3, "a cycle must not run away: {walked:?}");
    }

    #[test]
    fn test_a_dialog_is_capped_at_half_the_screen() {
        let wa = Rect::new(0, 0, 1920, 1080);
        assert_eq!(
            fit_size(1800, 1000, wa, (8, 26), None, true),
            (960, 540),
            "an oversized Open/Save dialog is cut down to half the screen"
        );
        assert_eq!(
            fit_size(1800, 1000, wa, (8, 26), None, false),
            (1800, 1000),
            "an ordinary window is only clamped to the work area"
        );
        assert_eq!(
            fit_size(500, 400, wa, (8, 26), None, true),
            (500, 400),
            "a modest dialog is left alone"
        );
    }

    #[test]
    fn test_a_dialog_that_cannot_shrink_keeps_its_minimum() {
        use antibox_core::backend::hints::{size_hints_flags::P_MIN_SIZE, SizeHints};
        let wa = Rect::new(0, 0, 1024, 700);
        let mut h = SizeHints::default();
        h.flags = P_MIN_SIZE;
        h.min_width = 700;
        h.min_height = 500;
        assert_eq!(
            fit_size(1000, 680, wa, (8, 26), Some(&h), true),
            (700, 500),
            "the half-screen cap never violates a minimum size hint"
        );
    }

    #[test]
    fn test_a_dialog_is_centred_on_its_owner_not_the_screen() {
        let wa = Rect::new(0, 0, 1024, 700);
        let owner = Rect::new(40, 60, 400, 300);
        assert_eq!(
            centre_on(Some(owner), wa, 200, 100, [0; 4]),
            Point::new(140, 160),
            "centred over the owner"
        );
        assert_eq!(
            centre_on(None, wa, 200, 100, [0; 4]),
            Point::new(412, 300),
            "with no owner it falls back to the work area centre"
        );
    }

    #[test]
    fn test_a_dialog_of_an_offscreen_owner_is_pulled_into_the_work_area() {
        let wa = Rect::new(0, 24, 1024, 676);
        let owner = Rect::new(900, 600, 400, 300);
        let p = centre_on(Some(owner), wa, 300, 200, [4, 22, 4, 4]);
        assert!(p.x >= wa.x && p.x + 300 <= wa.x + wa.w, "inside horizontally: {p:?}");
        assert!(p.y >= wa.y && p.y + 200 <= wa.y + wa.h, "inside vertically: {p:?}");
    }

    #[test]
    fn test_a_corner_dialog_keeps_its_title_bar_on_screen() {
        let wa = Rect::new(0, 0, 1024, 676);
        let dec = [4, 22, 4, 4];
        let owner = Rect::new(30, 40, 360, 260);
        let p = centre_on(Some(owner), wa, 512, 338, dec);
        assert!(p.y - dec[1] >= wa.y, "the title bar stays below the top edge: {p:?}");
        assert!(p.x - dec[0] >= wa.x, "the left border stays on screen: {p:?}");
    }


    #[test]
    fn test_a_transient_mapped_after_an_unrelated_window_still_outranks_it() {
        let owner = ClientId(1);
        let unrelated = ClientId(2);
        let dialog = ClientId(3);
        let bottom_to_top = [owner, unrelated, dialog];
        let parent_of = |w: ClientId| if w == dialog { Some(owner) } else { None };
        assert_eq!(
            stack_transients_above(&bottom_to_top, parent_of),
            vec![unrelated, owner, dialog]
        );
    }
