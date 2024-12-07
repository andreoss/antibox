use super::*;

#[test]
fn atoms_are_stable_and_reversible() {
    let mut t = AtomTable::default();
    let a = t.intern("_NET_WM_NAME");
    let b = t.intern("WM_CLASS");
    assert_eq!(t.intern("_NET_WM_NAME"), a);
    assert_ne!(a, b);
    assert_eq!(t.name(a).as_deref(), Some("_NET_WM_NAME"));
    assert_eq!(t.name(0), None);
}

#[test]
fn root_exists_and_ids_advance() {
    let mut s = SharedState::new(800, 600);
    assert_eq!(s.windows[&ROOT_WINDOW].kind, WinKind::Root);
    let a = s.alloc_id();
    let b = s.alloc_id();
    assert!(a >= FIRST_WINDOW_ID && b > a);
}

#[test]
fn shared_lock_roundtrips_intents() {
    let shared = Shared::new(1024, 768);
    shared.lock().intents.push(Intent::Map(42));
    assert_eq!(shared.lock().intents.len(), 1);
    assert_eq!(shared.lock().intents[0], Intent::Map(42));
}
