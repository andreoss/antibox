    use super::*;
    use crate::client::ClientWindow;
    use crate::frame::FrameWindow;
    use antibox_core::backend::RenderBackend;
    use antibox_core::mock::MockDisplay;
    use antibox_core::rect::Rect;
    fn setup_wm() -> (WindowManager<MockDisplay>, u32, u32) {
        let d = MockDisplay::new(1280, 720, 24);
        let backend = std::sync::Arc::new(d);
        let mut wm = WindowManager::<MockDisplay>::new_test();
        let cw = backend
            .create_window(
                1,
                Rect::new(0, 0, 100, 100),
                antibox_core::backend::WmWindowClass::InputOutput,
                false,
                antibox_core::backend::EventMask::NO_EVENT,
            )
            .unwrap();
        let fw = backend
            .create_window(
                1,
                Rect::new(0, 0, 120, 140),
                antibox_core::backend::WmWindowClass::InputOutput,
                false,
                antibox_core::backend::EventMask::NO_EVENT,
            )
            .unwrap();
        let cid = cw.id();
        let fid = fw.id();
        let frame = FrameWindow::new(ClientWindow::new(cw), fw);
        let client_id = frame.client_id();
        wm.xid_index.insert(client_id, cid, fid);
        wm.frames.insert(client_id, frame);
        (wm, cid, fid)
    }
    #[test]
    fn test_client_click_focuses() {
        let (mut wm, cid, _fid) = setup_wm();
        handle_click(&mut wm, cid);
        assert_eq!(wm.focused_window, wm.cid_for_xid(cid));
    }
    #[test]
    fn test_frame_click_ignored() {
        let (mut wm, _cid, fid) = setup_wm();
        handle_click(&mut wm, fid);
    }
