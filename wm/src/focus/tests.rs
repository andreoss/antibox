use super::*;
use crate::client::ClientWindow;
use crate::frame::FrameWindow;
use antibox_core::backend::hints::wm_hints_flags::INPUT_HINT;
use antibox_core::backend::RenderBackend;
use antibox_core::backend::WmHints;
use antibox_core::mock::MockDisplay;
use antibox_core::rect::Rect;
use std::sync::Arc;

fn hints(input: bool) -> WmHints {
    WmHints {
        flags: INPUT_HINT,
        input,
        ..Default::default()
    }
}

#[test]
fn test_icccm_focus_model() {
    assert_eq!(icccm_focus_actions(None, false), (true, false));
    assert_eq!(
        icccm_focus_actions(Some(&hints(true)), false),
        (true, false)
    );
    assert_eq!(icccm_focus_actions(Some(&hints(true)), true), (true, true));
    assert_eq!(
        icccm_focus_actions(Some(&hints(false)), true),
        (false, true)
    );
    assert_eq!(
        icccm_focus_actions(Some(&hints(false)), false),
        (false, false)
    );
}

fn make_wm() -> (Arc<MockDisplay>, WindowManager<MockDisplay>, u32, u32) {
    let d = Arc::new(MockDisplay::new(1280, 720, 24));
    let mut wm = WindowManager::<MockDisplay>::new_test();
    wm.backend = Some(Arc::clone(&d));
    let ch = d
        .create_window(
            1,
            Rect::new(0, 0, 100, 100),
            antibox_core::backend::WmWindowClass::InputOutput,
            false,
            antibox_core::backend::EventMask::NO_EVENT,
        )
        .unwrap();
    let cid = ch.id();
    let fh = d
        .create_window(
            1,
            Rect::new(0, 0, 120, 140),
            antibox_core::backend::WmWindowClass::InputOutput,
            false,
            antibox_core::backend::EventMask::NO_EVENT,
        )
        .unwrap();
    let fw = FrameWindow::new(ClientWindow::new(ch), fh);
    let fid = fw.frame().id();
    let client_id = fw.client_id();
    wm.xid_index.insert(client_id, cid, fid);
    wm.frames.insert(client_id, fw);
    (d, wm, cid, fid)
}

#[test]
fn test_mouse_follows_focus_warps_pointer() {
    let (d, mut wm, _cid, _fid) = make_wm();
    wm.config.mouse_follows_focus = true;
    let id = wm.frames.keys().next().unwrap();
    focus_window(&mut wm, id);
    assert!(!d.warps.lock().unwrap().is_empty());
}

