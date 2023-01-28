use super::*;
use antibox_core::backend::RenderBackend;
use antibox_core::backend::{AtomManager, PropMode};
use antibox_core::mock::MockDisplay;

fn make_display_and_window() -> (MockDisplay, Box<dyn WindowHandle>) {
    let display = MockDisplay::new(1280, 720, 24);
    let window = display
        .create_window(
            1,
            antibox_core::rect::Rect::new(0, 0, 100, 100),
            antibox_core::backend::WmWindowClass::InputOutput,
            false,
            antibox_core::backend::EventMask::NO_EVENT,
        )
        .unwrap();
    (display, window)
}

#[test]
fn test_client_new() {
    let (_display, window) = make_display_and_window();
    let client = ClientWindow::new(window);
    assert!(client.id().raw() > 0);
    let (_display2, window2) = make_display_and_window();
    let client2 = ClientWindow::new(window2);
    assert!(client2.id().raw() > client.id().raw());
    assert_eq!(client.window_type(), WindowType::Normal);
    assert!(client.protocols().is_empty());
    assert!(client.wm_hints().is_none());
    assert!(client.size_hints().is_none());
    assert!(client.mwm_hints().is_none());
    assert_eq!(client.user_time(), 0);
}

fn setup_atoms(display: &MockDisplay, atoms: &mut AtomManager) {
    for name in &[
        "_NET_WM_NAME",
        "WM_NAME",
        "_NET_WM_WINDOW_TYPE",
        "_NET_WM_PID",
        "WM_PROTOCOLS",
        "WM_DELETE_WINDOW",
        "WM_TAKE_FOCUS",
        "WM_HINTS",
        "WM_NORMAL_HINTS",
        "_MOTIF_WM_HINTS",
        "_GTK_FRAME_EXTENTS",
        "_NET_WM_XAPP_PROGRESS",
        "_NET_WM_USER_TIME",
        "WM_TRANSIENT_FOR",
        "_NET_WM_STATE",
        "_NET_WM_STATE_MAXIMIZED_VERT",
        "_NET_WM_STATE_MAXIMIZED_HORZ",
        "_NET_WM_STATE_SHADED",
        "_NET_WM_STATE_FULLSCREEN",
        "_NET_WM_STATE_ABOVE",
        "_NET_WM_STATE_BELOW",
        "_NET_WM_STATE_STICKY",
        "_NET_WM_STATE_SKIP_TASKBAR",
        "_NET_WM_STATE_SKIP_PAGER",
        "_NET_WM_STATE_DEMANDS_ATTENTION",
    ] {
        atoms.intern(display, name).unwrap();
    }
}

#[test]
fn test_read_protocols() {
    let (display, window) = make_display_and_window();
    let mut atom_mgr = AtomManager::new();
    setup_atoms(&display, &mut atom_mgr);

    let delete_atom = atom_mgr.get("WM_DELETE_WINDOW").unwrap();
    let focus_atom = atom_mgr.get("WM_TAKE_FOCUS").unwrap();

    let proto_atom = atom_mgr.get("WM_PROTOCOLS").unwrap();
    display
        .change_property32(
            PropMode::Replace,
            window.id(),
            proto_atom,
            proto_atom,
            &[delete_atom, focus_atom],
        )
        .unwrap();

    let mut client = ClientWindow::new(window);
    client.read_initial_properties(&display, &atom_mgr);

    assert_eq!(client.protocols(), &[delete_atom, focus_atom]);
    assert!(client.has_protocol(delete_atom));
    assert!(client.has_protocol(focus_atom));
}


#[test]
fn test_is_xpra_reports_cached_flag() {
    let (_d, window) = make_display_and_window();
    let mut client = ClientWindow::new(window);
    assert!(!client.is_xpra());
    client.is_xpra = true;
    assert!(client.is_xpra());
}

#[test]
fn test_xpra_resolved_guards_recomputation() {
    let (display, window) = make_display_and_window();
    let mut atom_mgr = AtomManager::new();
    let _ = atom_mgr.intern_all(&display);
    let mut client = ClientWindow::new(window);
    assert!(!client.xpra_resolved);
    client.read_initial_properties(&display, &atom_mgr);
    let resolved_once = client.xpra_resolved;
    client.is_xpra = true;
    client.read_initial_properties(&display, &atom_mgr);
    if resolved_once {
        assert!(
            client.is_xpra,
            "once resolved, a second property read must not recompute is_xpra"
        );
    }
    assert_eq!(client.xpra_resolved, resolved_once);
}
