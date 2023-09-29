use super::*;
use crate::client::ClientWindow;
use antibox_core::mock::MockDisplay;
use antibox_core::rect::Rect;
fn cw(d: &MockDisplay, w: i32, h: i32) -> Box<dyn WindowHandle> {
    d.create_window(
        1,
        Rect::new(0, 0, w, h),
        WmWindowClass::InputOutput,
        false,
        EventMask::NO_EVENT,
    )
    .unwrap()
}
fn make_cid(m: &mut WindowManager<MockDisplay>) -> ClientId {
    let d = MockDisplay::new(1280, 720, 24);
    let client_win = cw(&d, 100, 100);
    let frame_win = cw(&d, 120, 140);
    let client_xid = client_win.id();
    let frame_xid = frame_win.id();
    let fw = FrameWindow::new(ClientWindow::new(client_win), frame_win);
    let cid = fw.client_id();
    m.xid_index.insert(cid, client_xid, frame_xid);
    m.frames.insert(cid, fw);
    m.insertion_order.push(cid);
    m.map_order.push(cid);
    cid
}
fn insert_test<H: DisplayBackend + 'static + ?Sized>(m: &mut WindowManager<H>) -> ClientId {
    let d = MockDisplay::new(1280, 720, 24);
    let client_win = cw(&d, 100, 100);
    let frame_win = cw(&d, 120, 140);
    let client_xid = client_win.id();
    let frame_xid = frame_win.id();
    let fw = FrameWindow::new(ClientWindow::new(client_win), frame_win);
    let cid = fw.client_id();
    m.xid_index.insert(cid, client_xid, frame_xid);
    m.frames.insert(cid, fw);
    m.insertion_order.push(cid);
    m.map_order.push(cid);
    cid
}
#[test]
fn test_menu_actions_reach_pending_action() {
    let mut m = WindowManager::<MockDisplay>::new_test();
    for op in [MenuOp::WindowPickerList, MenuOp::Pager, MenuOp::Omni] {
        m.pending_action = None;
        m.handle_action(&Action::Menu(op.clone()));
        assert_eq!(
            m.pending_action,
            Some(Action::Menu(op)),
            "menu action must be forwarded to the app layer"
        );
    }
}

#[test]
fn test_new() {
    let m = WindowManager::<MockDisplay>::new_test();
    assert_eq!(m.workspace_count(), 4);
    assert_eq!(m.active_workspace(), 0);
}
#[test]
fn test_frame_ops() {
    let mut m = WindowManager::<MockDisplay>::new_test();
    let cid100 = make_cid(&mut m);
    assert_eq!(m.frame_count(), 1);
    m.frames.remove(&cid100);
    m.xid_index.remove(cid100);
    assert_eq!(m.frame_count(), 0);
    let cid = insert_test(&mut m);
    let xid = m.xid_index.xid_of(cid);
    m.handle_destroy(xid);
    assert_eq!(m.frame_count(), 0);
}

#[test]
fn test_expected_unmap_counts_self_window_twice() {
    let mut m = WindowManager::<MockDisplay>::new_test();
    m.expect_client_unmap(300);
    assert!(m.consume_expected_unmap(300));
    assert!(!m.consume_expected_unmap(300));
    m.self_windows.insert(400);
    m.expect_client_unmap(400);
    assert!(m.consume_expected_unmap(400));
    assert!(m.consume_expected_unmap(400));
    assert!(!m.consume_expected_unmap(400));
}
