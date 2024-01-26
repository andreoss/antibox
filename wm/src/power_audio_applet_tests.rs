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

struct FakeAudio {
    calls: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl crate::audio::AudioSystem for FakeAudio {
    fn read(&self) -> Option<AudioState> {
        Some(AudioState {
            sink_muted: false,
            volume: 50,
            source_muted: false,
            readers: Vec::new(),
        })
    }
    fn toggle_sink_mute(&self) {
        self.calls.lock().unwrap().push("sink_mute".into());
    }
    fn toggle_source_mute(&self) {
        self.calls.lock().unwrap().push("source_mute".into());
    }
    fn nudge_sink_volume(&self, delta_pct: i32) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("sink{:+}", delta_pct));
    }
    fn nudge_source_volume(&self, delta_pct: i32) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("source{:+}", delta_pct));
    }
}

fn applet_with_fake_audio() -> (
    PowerAudioApplet,
    std::sync::Arc<std::sync::Mutex<Vec<String>>>,
) {
    let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    applet.audio_system = Box::new(FakeAudio {
        calls: std::sync::Arc::clone(&calls),
    });
    (applet, calls)
}

#[test]
fn test_wheel_over_audio_half_nudges_volume() {
    let (mut applet, calls) = applet_with_fake_audio();
    let ax = applet.audio_x() as i32;
    applet.handle_click(ax + 2, 0, 4);
    let ax = applet.audio_x() as i32;
    applet.handle_click(ax + 2, 0, 5);
    assert_eq!(calls.lock().unwrap().as_slice(), ["sink+5", "sink-5"]);
}

#[test]
fn test_left_click_audio_half_toggles_sink_mute() {
    let (mut applet, calls) = applet_with_fake_audio();
    let ax = applet.audio_x() as i32;
    applet.handle_click(ax + 2, 0, 1);
    assert_eq!(calls.lock().unwrap().as_slice(), ["sink_mute"]);
}

#[test]
fn test_middle_click_audio_half_toggles_source_mute() {
    let (mut applet, calls) = applet_with_fake_audio();
    let ax = applet.audio_x() as i32;
    applet.handle_click(ax + 2, 0, 2);
    assert_eq!(calls.lock().unwrap().as_slice(), ["source_mute"]);
}

#[test]
fn test_clicks_on_battery_half_do_not_touch_audio() {
    let (mut applet, calls) = applet_with_fake_audio();
    for button in [1, 2, 4, 5] {
        assert_eq!(applet.handle_click(0, 0, button), None);
    }
    assert!(calls.lock().unwrap().is_empty());
}

#[test]
fn test_clicks_ignored_when_audio_absent() {
    let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_absent();
    applet.audio_system = Box::new(FakeAudio {
        calls: std::sync::Arc::clone(&calls),
    });
    applet.handle_click(5, 0, 4);
    assert!(calls.lock().unwrap().is_empty());
}

#[test]
fn test_click_rereads_audio_state() {
    let (mut applet, _calls) = applet_with_fake_audio();
    applet.audio.state = Some(AudioState {
        sink_muted: true,
        volume: 0,
        source_muted: true,
        readers: Vec::new(),
    });
    let ax = applet.audio_x() as i32;
    applet.handle_click(ax + 2, 0, 4);
    let st = applet.audio.state.as_ref().unwrap();
    assert_eq!(st.volume, 50);
    assert!(!st.sink_muted);
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
    assert_eq!(applet.hovered, Some(0));
}

#[test]
fn test_hover_audio_half_tracks_audio_not_battery() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    let audio_x = applet.audio_x() as i32;
    applet.handle_motion(audio_x + 1, 0);
    assert_eq!(applet.hovered, Some(1));
}

#[test]
fn test_hover_switches_when_crossing_into_the_other_half() {
    let mut applet = new_applet();
    applet.battery = battery_present();
    applet.audio = audio_present();
    applet.handle_motion(0, 0);
    assert_eq!(applet.hovered, Some(0));
    let audio_x = applet.audio_x() as i32;
    applet.handle_motion(audio_x + 1, 0);
    assert_eq!(applet.hovered, Some(1));
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
