use super::*;

#[test]
fn us_layout() {
    let data = b"us\0pc104\0\0\0\0";
    assert_eq!(parse_xkb_layout(data, 0), Some("us".to_string()));
}

#[test]
fn multi_layout() {
    let data = b"us\0de\0pc104\0\0\0\0";
    assert_eq!(parse_xkb_layout(data, 1), Some("de".to_string()));
}

#[test]
fn empty_input() {
    assert_eq!(parse_xkb_layout(&[], 0), None);
}

#[test]
fn rules_full() {
    let data = b"evdev\0pc105\0us,ru\0,\0grp:alt_shift_toggle";
    let info = KeyboardInfo {
        rules: "evdev".to_string(),
        model: "pc105".to_string(),
        layouts: "us,ru".to_string(),
        variants: ",".to_string(),
        options: "grp:alt_shift_toggle".to_string(),
        group: 1,
    };
    assert_eq!(parse_xkb_rules(data, 1), Some(info));
}

#[test]
fn rules_active_layout() {
    let info = parse_xkb_rules(b"evdev\0pc105\0us,ru\0\0", 1);
    assert_eq!(
        info.and_then(|i| i.active_layout().map(str::to_string)),
        Some("ru".to_string())
    );
}

#[test]
fn rules_empty() {
    assert_eq!(parse_xkb_rules(&[], 0), None);
}

#[test]
fn rules_no_layouts() {
    assert_eq!(parse_xkb_rules(b"evdev\0pc105\0\0\0", 0), None);
}
