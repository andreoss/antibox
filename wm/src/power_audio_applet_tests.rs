use super::*;
use crate::audio::AudioState;
use crate::power::BatteryInfo;
use antibox_core::mock::{MockDisplay, MockGraphics};

fn make_conn() -> Arc<dyn DisplayBackend> {
    Arc::new(MockDisplay::new(1280, 720, 24)) as Arc<dyn DisplayBackend>
}

fn new_applet() -> PowerAudioApplet {
    let conn = make_conn();
    PowerAudioApplet::new(&conn, conn.root().read_id()).unwrap()
}

fn battery_present() -> BatteryView {
    let mut v = BatteryView::new(true);
    v.batteries = vec![BatteryInfo {
        percent: 50,
        charging: false,
        discharging: true,
        energy_now: None,
        energy_full: None,
        power_now: None,
        minutes_left: None,
    }];
    v.ac_online = false;
    v
}

fn battery_absent() -> BatteryView {
    let mut v = BatteryView::new(true);
    v.batteries = Vec::new();
    v.ac_online = false;
    v
}

fn audio_present() -> AudioView {
    AudioView::new(Some(AudioState {
        sink_muted: false,
        volume: 50,
        source_muted: false,
        readers: Vec::new(),
    }))
}

fn audio_absent() -> AudioView {
    AudioView::new(None)
}

#[test]
fn test_new_applet_creates_window() {
    let applet = new_applet();
    assert_ne!(applet.window.id(), 0);
}

#[test]
fn test_hidden_when_neither_present() {
    let mut applet = new_applet();
    applet.battery = battery_absent();
    applet.audio = audio_absent();
    assert!(!applet.present());
    assert_eq!(applet.preferred_width(), 0);
}

#[test]
fn test_shows_only_battery_slot() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_absent();
    assert!(applet.present());
    let slot = PowerAudioApplet::slot_w(applet.h) as u32;
    assert_eq!(applet.preferred_width(), slot);
}

#[test]
fn test_shows_only_audio_slot() {
    let mut applet = new_applet();
    applet.battery = battery_absent();
    applet.audio = audio_present();
    assert!(applet.present());
    let slot = PowerAudioApplet::slot_w(applet.h) as u32;
    assert_eq!(applet.preferred_width(), slot);
}

#[test]
fn test_shows_shared_box_when_both_present() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    let slot = PowerAudioApplet::slot_w(applet.h) as u32;
    assert_eq!(applet.preferred_width(), slot * 2);
    assert_eq!(slot, applet.h as u32, "each half must be a square NxN slot");
}

#[test]
fn test_tooltip_combines_both_when_present() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    let tt = applet.tooltip();
    assert!(tt.contains("Battery"));
    assert!(tt.contains("Volume"));
}

#[test]
fn test_tooltip_only_audio_when_battery_absent() {
    let mut applet = new_applet();
    applet.battery = battery_absent();
    applet.audio = audio_present();
    let tt = applet.tooltip();
    assert!(!tt.contains("Battery"));
    assert!(tt.contains("Volume"));
}

#[test]
fn test_draw_both_present_no_crash() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    applet.w = applet.preferred_width() as u16;
    let g = MockGraphics::new(1);
    applet.paint(&g);
}

#[test]
fn test_preferred_height_matches_pref_h() {
    let applet = new_applet();
    assert_eq!(applet.preferred_height(), crate::status_graph::pref_h());
}

#[test]
fn test_hover_battery_half_tracks_battery_not_audio() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    applet.handle_motion(0, 0);
    assert_eq!(applet.hovered, Some(false));
}

#[test]
fn test_hover_audio_half_tracks_audio_not_battery() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    let audio_x = applet.audio_x() as i32;
    applet.handle_motion(audio_x + 1, 0);
    assert_eq!(applet.hovered, Some(true));
}

#[test]
fn test_hover_switches_when_crossing_into_the_other_half() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    applet.handle_motion(0, 0);
    assert_eq!(applet.hovered, Some(false));
    let audio_x = applet.audio_x() as i32;
    applet.handle_motion(audio_x + 1, 0);
    assert_eq!(applet.hovered, Some(true));
}

#[test]
fn test_leave_clears_hover() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    applet.handle_motion(0, 0);
    applet.handle_leave();
    assert_eq!(applet.hovered, None);
}
