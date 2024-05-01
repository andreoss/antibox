    use super::*;

    #[test]
    fn parse_accepts_aliases_and_rejects_unknown() {
        assert_eq!(Layout::parse("floating"), Some(Layout::Floating));
        assert_eq!(Layout::parse(" Float "), Some(Layout::Floating));
        assert_eq!(Layout::parse("tall"), Some(Layout::Tall));
        assert_eq!(Layout::parse("TILED"), Some(Layout::Tall));
        assert_eq!(Layout::parse(""), Some(Layout::Floating));
        assert_eq!(Layout::parse("mosaic"), None);
    }

    #[test]
    fn parse_list_pads_and_truncates() {
        let l = Layout::parse_list("tall, bogus", 4);
        assert_eq!(
            l,
            vec![
                Layout::Tall,
                Layout::Floating,
                Layout::Floating,
                Layout::Floating
            ]
        );
        let l = Layout::parse_list("tall,tall,tall", 2);
        assert_eq!(l, vec![Layout::Tall, Layout::Tall]);
    }

    #[test]
    fn next_cycles_through_all() {
        assert_eq!(Layout::Floating.next(), Layout::Tall);
        assert_eq!(Layout::Tall.next(), Layout::Wide);
        assert_eq!(Layout::Wide.next(), Layout::Floating);
    }

    #[test]
    fn wide_is_tall_rotated() {
        let r = Rect::new(0, 0, 800, 600);
        let wide = tile_wide(50, r, 1, 3);
        assert_eq!(wide.len(), 3);
        assert_eq!(wide[0], Rect::new(0, 0, 800, 300));
        assert_eq!(wide[1].y, 300);
        assert_eq!(wide[1].h, 300);
        assert_eq!(wide[1].x, 0);
        assert_eq!(wide[2].x, wide[1].x + wide[1].w);
        assert_eq!(wide[2].x + wide[2].w, 800);
    }

    #[test]
    fn wide_parses_and_titles() {
        assert_eq!(Layout::parse("wide"), Some(Layout::Wide));
        assert_eq!(Layout::parse("mirror"), Some(Layout::Wide));
        assert!(Layout::Wide.is_tiled());
    }

    #[test]
    fn split_vertically_covers_exactly() {
        let r = Rect::new(0, 0, 100, 101);
        let rows = split_vertically(3, r);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].y, 0);
        assert_eq!(rows[1].y, rows[0].y + rows[0].h);
        assert_eq!(rows[2].y, rows[1].y + rows[1].h);
        assert_eq!(rows[2].y + rows[2].h, 101);
    }

    #[test]
    fn split_vertically_single_is_full() {
        let r = Rect::new(5, 6, 40, 30);
        assert_eq!(split_vertically(1, r), vec![r]);
    }

    #[test]
    fn tile_one_window_fills_area() {
        let r = Rect::new(0, 0, 800, 600);
        let t = tile(50, r, 1, 1);
        assert_eq!(t, vec![r], "a lone window ignores the master split");
    }

    #[test]
    fn tile_master_and_stack_columns() {
        let r = Rect::new(0, 0, 800, 600);
        let t = tile(50, r, 1, 3);
        assert_eq!(t.len(), 3);
        assert_eq!(t[0], Rect::new(0, 0, 400, 600));
        assert_eq!(t[1].x, 400);
        assert_eq!(t[1].w, 400);
        assert_eq!(t[1].y, 0);
        assert_eq!(t[2].y, t[1].y + t[1].h);
        assert_eq!(t[2].y + t[2].h, 600);
    }

    #[test]
    fn tile_all_master_when_few() {
        let r = Rect::new(0, 0, 800, 600);
        let t = tile(50, r, 2, 2);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].w, 800);
        assert_eq!(t[1].w, 800);
    }

    fn wm_with_maximized_frame() -> (
        WindowManager<antibox_core::mock::MockDisplay>,
        ClientId,
    ) {
        use crate::client::ClientWindow;
        use crate::frame::FrameWindow;
        use antibox_core::backend::{EventMask, RenderBackend, WmWindowClass};
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
        let client = mk(400, 300);
        let client_xid = client.id();
        let frame = mk(406, 322);
        let frame_xid = frame.id();
        let mut fw = FrameWindow::new(ClientWindow::new(client), frame);
        let cid = fw.client_id();
        wm.xid_index.insert(cid, client_xid, frame_xid);
        fw.set_frame_rect(Rect::new(0, 0, 1280, 720));
        fw.state_mut().maximized = true;
        fw.state_mut().max_vert = true;
        fw.state_mut().max_horz = true;
        wm.frames.insert(cid, fw);
        wm.insertion_order.push(cid);
        wm.map_order.push(cid);
        (wm, cid)
    }

    #[test]
    fn arrange_clears_maximized_state() {
        let (mut wm, cid) = wm_with_maximized_frame();
        Layout::Tall.arrange(&mut wm, 0);
        let s = wm.frames[&cid].state();
        assert!(!s.maximized, "tiled frames must not stay maximized");
        assert!(!s.max_vert);
        assert!(!s.max_horz);
    }

    #[test]
    fn maximize_refused_on_tiled_workspace() {
        let (mut wm, cid) = wm_with_maximized_frame();
        wm.set_layout(0, Layout::Tall);
        assert!(!wm.frames[&cid].state().maximized);
        crate::wmaction::set_max_state(&mut wm, cid, true, true);
        let s = wm.frames[&cid].state();
        assert!(!s.maximized, "maximize must be ignored while tiled");
        assert!(!s.max_vert && !s.max_horz);
    }

    #[test]
    fn maximize_allowed_on_floating_workspace() {
        let (mut wm, cid) = wm_with_maximized_frame();
        wm.set_layout(0, Layout::Floating);
        crate::wmaction::set_max_state(&mut wm, cid, false, false);
        assert!(!wm.frames[&cid].state().maximized);
        crate::wmaction::set_max_state(&mut wm, cid, true, true);
        assert!(wm.frames[&cid].state().maximized);
    }

    #[test]
    fn tile_directional_clears_maximized() {
        let (mut wm, cid) = wm_with_maximized_frame();
        wm.focused_window = Some(cid);
        wm.handle_action(&crate::action::Action::Tile(crate::action::TileOp::TileLeft));
        let s = wm.frames[&cid].state();
        assert!(!s.maximized, "half-tiled frames must not stay maximized");
        assert!(!s.max_vert && !s.max_horz);
    }

    #[test]
    fn keyboard_snap_clears_maximized() {
        let (mut wm, cid) = wm_with_maximized_frame();
        wm.focused_window = Some(cid);
        crate::snap::keyboard_snap(&mut wm, crate::snap::SnapDir::Left);
        let s = wm.frames[&cid].state();
        assert!(!s.maximized, "snapped frames must not stay maximized");
        assert!(!s.max_vert && !s.max_horz);
    }

    #[test]
    fn tile_all_clears_maximized() {
        let (mut wm, cid) = wm_with_maximized_frame();
        wm.focused_window = Some(cid);
        wm.handle_action(&crate::action::Action::Tile(crate::action::TileOp::TileVertical));
        assert!(!wm.frames[&cid].state().maximized);
    }
