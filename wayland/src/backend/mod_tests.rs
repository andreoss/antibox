use super::*;

fn backend() -> WaylandCompositor {
    WaylandCompositor::new(Shared::new(1280, 720), BufferStore::new())
}

#[test]
fn screen_and_root_basics() {
    let b = backend();
    assert_eq!(b.root().read_id(), ROOT_WINDOW);
    assert_eq!(b.screen_width(), 1280);
    assert_eq!(b.screen_depth(), 32);
    assert!(b.composite_supported());
}

#[test]
fn atoms_roundtrip_through_backend() {
    let b = backend();
    let a = b.intern_atom("_NET_ACTIVE_WINDOW").unwrap();
    assert_eq!(b.intern_atom("_NET_ACTIVE_WINDOW").unwrap(), a);
    assert_eq!(b.get_atom_name(a).unwrap(), "_NET_ACTIVE_WINDOW");
}

#[test]
fn properties_store_and_load() {
    let b = backend();
    let a = b.intern_atom("FOO").unwrap();
    b.change_property8(PropMode::Replace, 5, a, 0, &[1, 2, 3])
        .unwrap();
    assert_eq!(b.get_property(5, a, 0, 0, 99).unwrap(), Some(vec![1, 2, 3]));
    b.delete_property(5, a).unwrap();
    assert_eq!(b.get_property(5, a, 0, 0, 99).unwrap(), None);
}

#[test]
fn create_window_queues_and_backs_a_buffer() {
    let b = backend();
    let win = b
        .create_window(
            ROOT_WINDOW,
            Rect::new(0, 0, 100, 40),
            WmWindowClass::InputOutput,
            false,
            EventMask::EXPOSURE,
        )
        .unwrap();
    let id = win.id();
    let g = b.create_graphics(id).unwrap();
    assert_eq!(g.drawable(), id);
    win.map().unwrap();
    let s = b.shared().lock();
    assert!(s.intents.contains(&Intent::Map(id)));
    assert!(s.windows[&id].mapped);
}

#[test]
fn configure_updates_rect_and_intent() {
    let b = backend();
    b.configure_window(7, &[10, 20, 300, 200]).ok();
    assert!(b
        .shared()
        .lock()
        .intents
        .iter()
        .any(|i| matches!(i, Intent::Configure { win: 7, .. })));
}

#[test]
fn grabs_record_and_release() {
    let b = backend();
    b.grab_key(false, ROOT_WINDOW, 8, 24, GrabMode::Async, GrabMode::Async)
        .unwrap();
    assert_eq!(b.shared().lock().key_grabs.len(), 1);
    b.ungrab_key(24, 8, ROOT_WINDOW).unwrap();
    assert_eq!(b.shared().lock().key_grabs.len(), 0);
}
