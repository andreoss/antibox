use super::*;

#[test]
fn test_window_option_default() {
    let o = WindowOption::default();
    assert_eq!(o.placement.workspace, None);
    assert_eq!(o.placement.layer, None);
}

#[test]
fn test_window_option_new() {
    let o = WindowOption::new("xterm.XTerm");
    assert_eq!(o.class_instance, "xterm.XTerm");
}

