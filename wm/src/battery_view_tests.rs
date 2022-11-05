use super::*;

fn view_with(batteries: Vec<BatInfo>, ac_online: bool) -> BatteryView {
    BatteryView {
        batteries,
        ac_online,
        vertical: true,
    }
}

fn bat(percent: i32, charging: bool, discharging: bool) -> BatInfo {
    BatInfo {
        percent,
        charging,
        discharging,
        energy_now: None,
        energy_full: None,
        power_now: None,
        minutes_left: None,
    }
}

#[test]
fn test_default_state_empty() {
    let v = view_with(Vec::new(), false);
    assert!(!v.present());
    assert_eq!(v.combined_percent(), -1);
}

#[test]
fn test_present_battery_or_ac() {
    assert!(view_with(vec![bat(50, false, true)], false).present());
    assert!(view_with(Vec::new(), true).present());
    assert!(!view_with(Vec::new(), false).present());
}

#[test]
fn test_combined_percent_single() {
    let v = view_with(
        vec![BatInfo {
            percent: 75,
            charging: false,
            discharging: true,
            energy_now: Some(7500),
            energy_full: Some(10000),
            power_now: Some(5000),
            minutes_left: None,
        }],
        false,
    );
    assert_eq!(v.combined_percent(), 75);
}

#[test]
fn test_combined_percent_multi() {
    let v = view_with(
        vec![
            BatInfo {
                percent: 80,
                charging: false,
                discharging: true,
                energy_now: Some(8000),
                energy_full: Some(10000),
                power_now: Some(2000),
                minutes_left: None,
            },
            BatInfo {
                percent: 40,
                charging: false,
                discharging: true,
                energy_now: Some(2000),
                energy_full: Some(5000),
                power_now: Some(1000),
                minutes_left: None,
            },
        ],
        false,
    );
    assert!((v.combined_percent() - 66).abs() <= 1);
}

#[test]
fn test_time_remaining_from_energy() {
    let v = view_with(
        vec![BatInfo {
            percent: 75,
            charging: false,
            discharging: true,
            energy_now: Some(7500),
            energy_full: Some(10000),
            power_now: Some(5000),
            minutes_left: None,
        }],
        false,
    );
    assert_eq!(v.time_remaining_secs().unwrap(), 5400);
}

#[test]
fn test_time_remaining_from_minutes_left() {
    let v = view_with(
        vec![BatInfo {
            percent: 60,
            charging: false,
            discharging: true,
            energy_now: None,
            energy_full: None,
            power_now: None,
            minutes_left: Some(90),
        }],
        false,
    );
    assert_eq!(v.time_remaining_secs().unwrap(), 5400);
}

#[test]
fn test_level_colour_critical_even_when_charging() {
    let v = view_with(vec![bat(5, true, false)], true);
    assert_eq!(v.level_colour(), COLOR_CRITICAL);
}

#[test]
fn test_level_colour_critical() {
    let v = view_with(vec![bat(5, false, true)], false);
    assert_eq!(v.level_colour(), COLOR_CRITICAL);
    assert!(v.is_critical());
}

#[test]
fn test_is_critical_false_when_charging() {
    let v = view_with(vec![bat(5, true, false)], true);
    assert!(!v.is_critical());
}

#[test]
fn test_level_colour_low() {
    assert_eq!(
        view_with(vec![bat(20, false, true)], false).level_colour(),
        COLOR_LOW
    );
}

#[test]
fn test_level_colour_medium() {
    assert_eq!(
        view_with(vec![bat(35, false, true)], false).level_colour(),
        COLOR_MEDIUM
    );
}

#[test]
fn test_level_colour_full() {
    assert_eq!(
        view_with(vec![bat(80, false, true)], false).level_colour(),
        COLOR_FULL
    );
}

#[test]
fn test_tooltip_no_battery_ac() {
    assert!(view_with(Vec::new(), true).tooltip().contains("AC"));
}

#[test]
fn test_tooltip_ac_connected_disconnected() {
    assert!(view_with(vec![bat(60, false, true)], true)
        .tooltip()
        .contains("AC: connected"));
    assert!(view_with(vec![bat(60, false, true)], false)
        .tooltip()
        .contains("AC: disconnected"));
}

#[test]
fn test_tooltip_critical_warns() {
    assert!(view_with(vec![bat(5, false, true)], false)
        .tooltip()
        .contains("critically"));
}

#[test]
fn test_tooltip_with_battery_details() {
    let v = view_with(
        vec![BatInfo {
            percent: 75,
            charging: false,
            discharging: true,
            energy_now: Some(7500),
            energy_full: Some(10000),
            power_now: Some(5000),
            minutes_left: None,
        }],
        false,
    );
    let tt = v.tooltip();
    assert!(tt.contains("Battery"));
    assert!(tt.contains("BAT0"));
    assert!(tt.contains("75%"));
    assert!(tt.contains("Remaining"));
}

#[test]
fn test_outline_line_thickness_is_at_least_two_pixels() {
    assert!(
        line_thickness() >= 2,
        "battery outline strokes should be at least 2px thick"
    );
}

fn max_fill_rect_area(commands: &[antibox_core::mock::MockCommand]) -> u32 {
    use antibox_core::mock::MockCommand;
    commands
        .iter()
        .filter_map(|c| match c {
            MockCommand::FillRect(_, _, w, h) => Some(*w as u32 * *h as u32),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

#[test]
fn test_full_battery_visible_at_real_taskbar_slot_size() {
    let slot = crate::status_graph::pref_h() as u16;
    let v = view_with(vec![bat(90, false, false)], true);
    let g = antibox_core::mock::MockGraphics::new(1);
    v.draw(&g, 0, 0, slot, slot);
    assert!(
        max_fill_rect_area(&g.commands()) > 0,
        "battery at the real taskbar slot size ({slot}x{slot}) should still show a level fill"
    );
}

fn gauge_fill_heights(commands: &[antibox_core::mock::MockCommand], colour: u32) -> Vec<u16> {
    use antibox_core::mock::MockCommand;
    let mut fg = 0u32;
    let mut heights = Vec::new();
    for c in commands {
        match c {
            MockCommand::SetForeground(p) => fg = *p,
            MockCommand::FillRect(_, _, _, h) if fg == colour => heights.push(*h),
            _ => {}
        }
    }
    heights
}

#[test]
fn test_vertical_battery_fill_height_tracks_charge() {
    let low = antibox_core::mock::MockGraphics::new(1);
    view_with(vec![bat(60, false, true)], false).draw(&low, 0, 0, 18, 18);
    let high = antibox_core::mock::MockGraphics::new(1);
    view_with(vec![bat(95, false, true)], false).draw(&high, 0, 0, 18, 18);
    let low_h = *gauge_fill_heights(&low.commands(), COLOR_FULL)
        .iter()
        .max()
        .unwrap();
    let high_h = *gauge_fill_heights(&high.commands(), COLOR_FULL)
        .iter()
        .max()
        .unwrap();
    assert!(
        low_h < high_h,
        "battery interior fill must rise with charge ({} vs {})",
        low_h,
        high_h
    );
}

#[test]
fn test_full_battery_still_shows_a_visible_level_fill() {
    let v = view_with(vec![bat(90, false, false)], true);
    let vertical = antibox_core::mock::MockGraphics::new(1);
    v.draw(&vertical, 0, 0, 24, 20);
    assert!(
        max_fill_rect_area(&vertical.commands()) > 0,
        "vertical battery should still draw a visible level fill"
    );

    let mut h = view_with(vec![bat(90, false, false)], true);
    h.vertical = false;
    let horizontal = antibox_core::mock::MockGraphics::new(1);
    h.draw(&horizontal, 0, 0, 28, 18);
    assert!(
        max_fill_rect_area(&horizontal.commands()) > 0,
        "horizontal battery should still draw a visible level fill"
    );
}

#[test]
fn test_draw_all_states_no_crash() {
    let g = antibox_core::mock::MockGraphics::new(1);
    for (pct, chg, dis, ac) in [
        (90, true, false, true),
        (90, false, false, true),
        (5, false, true, false),
        (50, false, true, false),
    ] {
        let mut v = view_with(vec![bat(pct, chg, dis)], ac);
        v.draw(&g, 0, 0, 24, 20);
        v.vertical = false;
        v.draw(&g, 3, 3, 28, 18);
    }
}

#[test]
fn test_fmt_time() {
    assert_eq!(fmt_time(90), "1:30");
    assert_eq!(fmt_time(0), "0:00");
    assert_eq!(fmt_time(130), "2:10");
}

#[test]
fn test_fmt_power_watts() {
    assert_eq!(fmt_power(5_000_000), "5.0W");
}

#[test]
fn test_fmt_power_mw() {
    assert_eq!(fmt_power(500_000), "500mW");
}

#[test]
fn test_update_no_crash() {
    let mut v = view_with(Vec::new(), false);
    let _ = v.update();
}
