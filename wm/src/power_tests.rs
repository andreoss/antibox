use super::*;

#[test]
fn test_read_status_does_not_crash() {
    let st = read_status();
    let _ = st.ac_online;
    let _ = st.batteries.len();
}

#[test]
fn test_read_status_caps_batteries() {
    assert!(read_status().batteries.len() <= MAX_BATTERIES);
}

#[test]
fn test_power_status_default_is_empty() {
    let st = PowerStatus::default();
    assert!(!st.ac_online);
    assert!(st.batteries.is_empty());
}
