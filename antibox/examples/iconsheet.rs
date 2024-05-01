#![deny(
    elided_lifetimes_in_paths,
    meta_variable_misuse,
    unreachable_pub,
    unused_lifetimes,
    unused_qualifications
)]
#![deny(
    clippy::cloned_instead_of_copied,
    clippy::dbg_macro,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    clippy::todo,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unimplemented,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    clippy::use_self
)]
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_ui::theme;
use antibox_wm::audio::AudioState;
use antibox_wm::audio_view::{AudioView, MicView};
use antibox_wm::battery_view::BatteryView;
use antibox_wm::power::{BatteryInfo, PowerStatus};
use std::sync::Arc;

fn bat(percent: i32, charging: bool, ac: bool) -> BatteryView {
    BatteryView::from_status(
        PowerStatus {
            ac_online: ac,
            batteries: vec![BatteryInfo {
                percent,
                charging,
                discharging: !charging,
                energy_now: None,
                energy_full: None,
                power_now: None,
                minutes_left: None,
            }],
        },
        true,
    )
}

fn audio(vol: i32, muted: bool, mic: bool, rec: bool) -> AudioView {
    AudioView::new(Some(AudioState {
        sink_muted: muted,
        volume: vol,
        source_muted: mic,
        readers: if rec {
            vec!["cap".to_string()]
        } else {
            Vec::new()
        },
    }))
}

fn mic(muted: bool) -> MicView {
    MicView::new(Some(AudioState {
        sink_muted: false,
        volume: 50,
        source_muted: muted,
        readers: vec!["cap".to_string()],
    }))
}

enum Slot {
    Bat(BatteryView),
    Aud(AudioView),
    Mic(MicView),
}

fn main() {
    let (conn, _render, _ev, _tray) = antibox_x11::xcb::build_backend(None).unwrap();
    let root = conn.root().as_parent();
    let sizes: [u16; 4] = [18, 22, 28, 40];
    let gap: i32 = 10;

    let sheet_w = 760;
    let sheet_h = 240;
    let win = conn
        .create_window(
            root,
            Rect::new(20, 20, sheet_w, sheet_h),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE,
        )
        .unwrap();
    let _ = win.map();
    let _ = conn.flush();

    let mut slots: Vec<(Box<dyn WindowHandle>, u16, Slot)> = Vec::new();
    let mut y = gap;
    for &h in &sizes {
        let mut x = gap;
        let make: Vec<Slot> = vec![
            Slot::Bat(bat(95, false, false)),
            Slot::Bat(bat(60, false, false)),
            Slot::Bat(bat(30, false, false)),
            Slot::Bat(bat(8, false, false)),
            Slot::Bat(bat(50, true, true)),
            Slot::Aud(audio(90, false, false, false)),
            Slot::Aud(audio(50, false, false, false)),
            Slot::Aud(audio(10, false, false, false)),
            Slot::Aud(audio(0, false, false, false)),
            Slot::Aud(audio(70, true, false, false)),
            Slot::Mic(mic(false)),
            Slot::Mic(mic(true)),
        ];
        for s in make {
            let w = conn
                .create_window(
                    win.id(),
                    Rect::new(x, y, h as i32, h as i32),
                    WmWindowClass::InputOutput,
                    true,
                    EventMask::EXPOSURE,
                )
                .unwrap();
            let _ = w.map();
            slots.push((w, h, s));
            x += h as i32 + gap;
        }
        y += h as i32 + gap;
    }
    let _ = conn.flush();

    for _ in 0..120 {
        if let Ok(g) = conn.create_graphics(win.id()) {
            let _ = g.set_foreground(theme::tray_face());
            let _ = g.fill_rect(0, 0, sheet_w as u16, sheet_h as u16);
        }
        for (w, h, s) in &slots {
            if let Ok(g) = conn.create_graphics(w.id()) {
                let _ = g.set_foreground(theme::tray_face());
                let _ = g.fill_rect(0, 0, *h, *h);
                theme::well(&*g, 0, 0, *h, *h);
                match s {
                    Slot::Bat(v) => v.draw(&*g, 0, 0, *h, *h),
                    Slot::Aud(v) => v.draw(&*g, 0, *h),
                    Slot::Mic(v) => v.draw(&*g, 0, *h),
                }
            }
        }
        let _ = conn.flush();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    drop(slots);
    let _ = Arc::strong_count(&conn);
}
