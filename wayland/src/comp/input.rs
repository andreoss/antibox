use smithay::backend::input::{
    AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
    KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
};
use smithay::input::keyboard::{FilterResult, ModifiersState};
use smithay::input::pointer::{AxisFrame, ButtonEvent, MotionEvent};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, SERIAL_COUNTER};

use antibox_core::backend::{BackendEvent, EventMask};
use antibox_core::point::Point as WmPoint;

use super::state::Compositor;

const fn x11_mods(m: &ModifiersState) -> u16 {
    let mut x = 0u16;
    if m.shift {
        x |= 1;
    }
    if m.caps_lock {
        x |= 2;
    }
    if m.ctrl {
        x |= 4;
    }
    if m.alt {
        x |= 8;
    }
    if m.logo {
        x |= 64;
    }
    x
}

const LOCK_MASK: u16 = 2;

const fn x11_button(code: u32) -> u8 {
    match code {
        0x110 => 1,
        0x112 => 2,
        0x111 => 3,
        0x113 => 8,
        0x114 => 9,
        _ => 0,
    }
}

impl Compositor {
    fn wm_window_at(&self, x: i32, y: i32) -> Option<u32> {
        self.shared.lock().window_at(x, y)
    }

    fn local_point(&self, win: u32, x: i32, y: i32) -> WmPoint {
        let (ox, oy) = self.shared.lock().absolute_origin(win);
        WmPoint::new(x - ox, y - oy)
    }

    pub fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event, .. } => self.on_key::<I>(&event),
            InputEvent::PointerMotionAbsolute { event, .. } => self.on_motion::<I>(&event),
            InputEvent::PointerButton { event, .. } => self.on_button::<I>(&event),
            InputEvent::PointerAxis { event, .. } => self.on_axis::<I>(&event),
            _ => {}
        }
    }

    fn on_key<I: InputBackend>(&mut self, event: &I::KeyboardKeyEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let time = Event::time_msec(event);
        let pressed = event.state() == KeyState::Pressed;
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        keyboard.input::<(), _>(
            self,
            event.key_code(),
            event.state(),
            serial,
            time,
            |data, mods, keysym| {
                let kc = keysym.raw_code().raw();
                let xmods = x11_mods(mods);
                let mut s = data.shared.lock();
                s.last_time = time;
                s.key_mods = xmods;
                let want = xmods & 0xFF & !LOCK_MASK;
                let target = if let Some(gw) = s.keyboard_grab {
                    Some(gw)
                } else {
                    s.key_grabs
                        .iter()
                        .find(|g| {
                            g.keycode as u32 == kc && (g.modifiers & 0xFF & !LOCK_MASK) == want
                        })
                        .map(|g| g.window)
                        .or_else(|| {
                            let focus = s.focus;
                            s.windows
                                .get(&focus)
                                .filter(|r| {
                                    matches!(r.kind, crate::shared::WinKind::Server)
                                        && r.event_mask & EventMask::KEY_PRESS.bits() != 0
                                })
                                .map(|_| focus)
                        })
                };
                match target {
                    Some(win) => {
                        let ev = if pressed {
                            BackendEvent::KeyPress {
                                window: win,
                                event: win,
                                keycode: kc,
                                state: xmods,
                            }
                        } else {
                            BackendEvent::KeyRelease {
                                window: win,
                                keycode: kc,
                                state: xmods,
                            }
                        };
                        s.events.push(ev);
                        FilterResult::Intercept(())
                    }
                    None => FilterResult::Forward,
                }
            },
        );
    }

    fn on_motion<I: InputBackend>(&mut self, event: &I::PointerMotionAbsoluteEvent) {
        let Some(output) = self.space.outputs().next() else {
            return;
        };
        let Some(output_geo) = self.space.output_geometry(output) else {
            return;
        };
        let pos = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();
        let (px, py) = (pos.x as i32, pos.y as i32);
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time_msec();

        let grab = {
            let mut s = self.shared.lock();
            s.pointer_x = px as i16;
            s.pointer_y = py as i16;
            s.last_time = time;
            s.pointer_grab.or(s.implicit_grab)
        };

        let client_under = self.surface_under(pos);
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let focus: Option<(WlSurface, Point<f64, Logical>)> = if grab.is_some() {
            None
        } else {
            client_under.clone()
        };
        pointer.motion(
            self,
            focus,
            &MotionEvent {
                location: pos,
                serial,
                time,
            },
        );
        pointer.frame(self);

        if grab.is_none() {
            let raw = if client_under.is_some() {
                None
            } else {
                self.wm_window_at(px, py)
            };
            let (leave, enter) = self.shared.lock().pointer_crossing(raw);
            if let Some(w) = leave {
                self.synth(BackendEvent::LeaveNotify { window: w, mode: 0 });
            }
            if let Some(w) = enter {
                self.synth(BackendEvent::EnterNotify { window: w, mode: 0 });
            }
        }
        let wm_target = if let Some(g) = grab {
            Some(g)
        } else if client_under.is_some() {
            None
        } else {
            self.wm_window_at(px, py).and_then(|w| {
                self.shared.lock().propagate_event_target(
                    w,
                    (EventMask::POINTER_MOTION | EventMask::BUTTON_MOTION).bits(),
                )
            })
        };
        if let Some(win) = wm_target {
            let mask = {
                let s = self.shared.lock();
                s.pointer_mask | s.key_mods
            };
            let point = self.local_point(win, px, py);
            self.synth(BackendEvent::MotionNotify {
                window: win,
                point,
                root: WmPoint::new(px, py),
                state: mask,
            });
        }
    }

    fn on_button<I: InputBackend>(&mut self, event: &I::PointerButtonEvent) {
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time_msec();
        let pressed = event.state() == ButtonState::Pressed;
        let btn = x11_button(event.button_code());

        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let pos = pointer.current_location();
        let (px, py) = (pos.x as i32, pos.y as i32);

        let (grab, mask) = {
            let mut s = self.shared.lock();
            if (1..=5).contains(&btn) {
                let bit = 1u16 << (7 + btn as u16);
                if pressed {
                    s.pointer_mask |= bit;
                } else {
                    s.pointer_mask &= !bit;
                }
            }
            s.last_time = time;
            let g = s.pointer_grab.or(s.implicit_grab);
            if !pressed && s.pointer_mask & 0x1F00 == 0 {
                s.implicit_grab = None;
            }
            (g, s.pointer_mask | s.key_mods)
        };

        let client_under = self.surface_under(pos);
        let wm_target = if let Some(g) = grab {
            Some(g)
        } else if let Some((surface, _)) = client_under.as_ref() {
            self.client_id_for_surface(surface)
        } else {
            let bit = if pressed {
                EventMask::BUTTON_PRESS
            } else {
                EventMask::BUTTON_RELEASE
            };
            self.wm_window_at(px, py)
                .and_then(|w| self.shared.lock().propagate_event_target(w, bit.bits()))
        };

        if pressed && grab.is_none() && client_under.is_none() {
            if let Some(win) = wm_target {
                self.shared.lock().implicit_grab = Some(win);
            }
        }

        if let Some(win) = wm_target {
            let point = self.local_point(win, px, py);
            let ev = if pressed {
                BackendEvent::ButtonPress {
                    window: win,
                    event: win,
                    point,
                    root: WmPoint::new(px, py),
                    button: btn,
                    state: mask,
                }
            } else {
                BackendEvent::ButtonRelease {
                    window: win,
                    point,
                    button: btn,
                }
            };
            self.synth(ev);
        }

        if grab.is_none() {
            pointer.button(
                self,
                &ButtonEvent {
                    button: event.button_code(),
                    state: event.state(),
                    serial,
                    time,
                },
            );
            pointer.frame(self);
        }
    }

    fn on_axis<I: InputBackend>(&mut self, event: &I::PointerAxisEvent) {
        let source = event.source();
        let horizontal = event
            .amount(Axis::Horizontal)
            .unwrap_or_else(|| event.amount_v120(Axis::Horizontal).unwrap_or(0.0) * 15.0 / 120.);
        let vertical = event
            .amount(Axis::Vertical)
            .unwrap_or_else(|| event.amount_v120(Axis::Vertical).unwrap_or(0.0) * 15.0 / 120.);
        let h_discrete = event.amount_v120(Axis::Horizontal);
        let v_discrete = event.amount_v120(Axis::Vertical);

        let mut frame = AxisFrame::new(event.time_msec()).source(source);
        if horizontal != 0.0 {
            frame = frame.value(Axis::Horizontal, horizontal);
            if let Some(d) = h_discrete {
                frame = frame.v120(Axis::Horizontal, d as i32);
            }
        }
        if vertical != 0.0 {
            frame = frame.value(Axis::Vertical, vertical);
            if let Some(d) = v_discrete {
                frame = frame.v120(Axis::Vertical, d as i32);
            }
        }
        if source == AxisSource::Finger {
            if event.amount(Axis::Horizontal) == Some(0.0) {
                frame = frame.stop(Axis::Horizontal);
            }
            if event.amount(Axis::Vertical) == Some(0.0) {
                frame = frame.stop(Axis::Vertical);
            }
        }
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        pointer.axis(self, frame);
        pointer.frame(self);
        self.synth_wheel_buttons(vertical, v_discrete, horizontal, h_discrete);
    }

    fn synth_wheel_buttons(
        &mut self,
        vertical: f64,
        v_discrete: Option<f64>,
        horizontal: f64,
        h_discrete: Option<f64>,
    ) {
        let (px, py, mask) = {
            let s = self.shared.lock();
            (
                i32::from(s.pointer_x),
                i32::from(s.pointer_y),
                s.pointer_mask | s.key_mods,
            )
        };
        let pos = Point::from((px as f64, py as f64));
        let client_under = self.surface_under(pos);
        let wm_target = if let Some((surface, _)) = client_under.as_ref() {
            self.client_id_for_surface(surface)
        } else {
            self.wm_window_at(px, py).and_then(|w| {
                self.shared
                    .lock()
                    .propagate_event_target(w, EventMask::BUTTON_PRESS.bits())
            })
        };
        let Some(win) = wm_target else {
            return;
        };
        let emit = |button: u8, steps: usize| {
            let point = self.local_point(win, px, py);
            for _ in 0..steps {
                self.synth(BackendEvent::ButtonPress {
                    window: win,
                    event: win,
                    point,
                    root: WmPoint::new(px, py),
                    button,
                    state: mask,
                });
                self.synth(BackendEvent::ButtonRelease {
                    window: win,
                    point,
                    button,
                });
            }
        };
        let steps = |amount: f64, discrete: Option<f64>| {
            discrete.map_or(usize::from(amount != 0.0), |d| {
                ((d.abs() / 120.0).round() as usize).max(1)
            })
        };
        if vertical != 0.0 {
            emit(if vertical < 0.0 { 4 } else { 5 }, steps(vertical, v_discrete));
        }
        if horizontal != 0.0 {
            emit(if horizontal < 0.0 { 6 } else { 7 }, steps(horizontal, h_discrete));
        }
    }
}
