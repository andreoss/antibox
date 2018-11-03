use antibox_core::backend::*;
use antibox_core::mock::MockDisplay;
use antibox_wm::taskpane::TaskPane;
use std::sync::Arc;

fn make_taskpane() -> TaskPane {
    let d = MockDisplay::new(1280, 720, 24);
    let ad: Arc<dyn DisplayBackend> = Arc::new(d);
    let colours = antibox_wm::render::ThemeColors::default();
    TaskPane::new(&ad, 1, colours).unwrap()
}

#[test]
fn test_add_button() {
    let mut tp = make_taskpane();
    tp.add_button(42, "Test Window");
    assert_eq!(tp.find_by_pos(10), Some(42));
    assert_eq!(tp.find_by_pos(200), None);
}

#[test]
fn test_add_multiple_buttons() {
    let mut tp = make_taskpane();
    tp.add_button(1, "First");
    tp.add_button(2, "Second");
    assert_eq!(tp.find_by_pos(10), Some(1));
    assert_eq!(tp.find_by_pos(130), Some(2));
}

