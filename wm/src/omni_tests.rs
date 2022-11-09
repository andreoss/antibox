use super::*;

#[test]
fn test_panel_size_scales_with_the_monitor() {
    assert!(panel_w_for(3000) >= 1000);
    assert!(panel_w_for(200) <= 200);
    assert!(rows_for(600) >= DEFAULT_ROWS);
    assert!(rows_for(30000) > rows_for(600));
}
