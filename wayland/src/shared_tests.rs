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

#[test]
fn unmapped_parent_hides_children() {
    let shared = Shared::new(800, 600);
    let (parent, child) = {
        let mut s = shared.lock();
        let parent = s.alloc_id();
        s.windows.insert(
            parent,
            WinRec {
                kind: WinKind::Server,
                rect: Rect::new(10, 10, 200, 100),
                mapped: true,
                override_redirect: false,
                depth: 24,
                parent: ROOT_WINDOW,
                event_mask: 0,
            },
        );
        let child = s.alloc_id();
        s.windows.insert(
            child,
            WinRec {
                kind: WinKind::Server,
                rect: Rect::new(5, 5, 50, 20),
                mapped: true,
                override_redirect: false,
                depth: 24,
                parent,
                event_mask: 0,
            },
        );
        (parent, child)
    };
    {
        let s = shared.lock();
        assert!(s.viewable(child));
        assert_eq!(s.window_at(20, 20), Some(child));
    }
    shared.lock().windows.get_mut(&parent).unwrap().mapped = false;
    {
        let s = shared.lock();
        assert!(!s.viewable(child), "child of unmapped parent is not viewable");
        assert_eq!(s.window_at(20, 20), None);
    }
}

#[test]
fn subtree_collects_descendants() {
    let shared = Shared::new(800, 600);
    let mut s = shared.lock();
    let a = s.alloc_id();
    let b = s.alloc_id();
    let c = s.alloc_id();
    let rec = |parent| WinRec {
        kind: WinKind::Server,
        rect: Rect::new(0, 0, 10, 10),
        mapped: true,
        override_redirect: false,
        depth: 24,
        parent,
        event_mask: 0,
    };
    s.windows.insert(a, rec(ROOT_WINDOW));
    s.windows.insert(b, rec(a));
    s.windows.insert(c, rec(b));
    let mut tree = s.subtree(a);
    tree.sort_unstable();
    assert_eq!(tree, vec![a, b, c]);
    assert_eq!(s.subtree(c), vec![c]);
}
