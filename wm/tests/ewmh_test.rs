use antibox_core::backend::*;
use antibox_core::mock::MockDisplay;

fn make_backend() -> (MockDisplay, AtomManager) {
    let d = MockDisplay::new(1280, 720, 24);
    let mut am = AtomManager::new();
    am.intern_all(&d).unwrap();
    (d, am)
}

#[test]
fn test_update_client_list_smoke() {
    let (d, am) = make_backend();
    antibox_wm::ewmh::update_client_list(&d, &am, &[1, 2, 3]);
}

#[test]
fn test_update_client_list_empty() {
    let (d, am) = make_backend();
    antibox_wm::ewmh::update_client_list(&d, &am, &[]);
}

