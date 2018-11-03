use antibox_core::backend::*;
use antibox_core::mock::MockDisplay;
use antibox_core::rect::Rect;
use antibox_wm::client::ClientWindow;
use antibox_wm::frame::FrameWindow;
use antibox_wm::id::ClientId;
use antibox_wm::manager::WindowManager;
use std::sync::Arc;

fn make_wm() -> (Arc<MockDisplay>, WindowManager<MockDisplay>, u32, ClientId) {
    let d = Arc::new(MockDisplay::new(1280, 720, 24));
    let cw = d
        .create_window(
            1,
            Rect::new(0, 0, 100, 100),
            WmWindowClass::InputOutput,
            false,
            EventMask::NO_EVENT,
        )
        .unwrap();
    let cid = cw.id();
    let fw = d
        .create_window(
            1,
            Rect::new(0, 0, 120, 140),
            WmWindowClass::InputOutput,
            false,
            EventMask::NO_EVENT,
        )
        .unwrap();
    let frame_xid = fw.id();
    let mut wm = WindowManager::<MockDisplay>::new(&d);
    let frame_window = FrameWindow::new(ClientWindow::new(cw), fw);
    let client_id = frame_window.client_id();
    wm.xid_index.insert(client_id, cid, frame_xid);
    wm.frames.insert(client_id, frame_window);
    wm.focused_window = Some(client_id);
    (d, wm, cid, client_id)
}

#[test]
fn test_map_request_creates_frame() {
    let d = Arc::new(MockDisplay::new(1280, 720, 24));
    let d2 = Arc::clone(&d);
    let new_win = d
        .create_window(
            1,
            Rect::new(0, 0, 100, 100),
            WmWindowClass::InputOutput,
            false,
            EventMask::NO_EVENT,
        )
        .unwrap();
    let nid = new_win.id();
    let mut wm = WindowManager::<MockDisplay>::new(&d2);
    antibox_wm::handler::map_request(&mut wm, nid);
    let cid = wm.cid_for_xid(nid).unwrap();
    assert!(wm.frames.contains_key(&cid));
    assert_eq!(wm.focused_window, Some(cid));
}

#[test]
fn test_map_request_duplicate_ignored() {
    let (_d, mut wm, cid, _client_id) = make_wm();
    let before = wm.frame_count();
    antibox_wm::handler::map_request(&mut wm, cid);
    assert_eq!(wm.frame_count(), before);
}

