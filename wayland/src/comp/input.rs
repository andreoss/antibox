use antibox_core::backend::{BackendEvent, EventMask};
use antibox_core::point::Point as WmPoint;

use super::state::Server;
use crate::ffi::wlr::*;

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

const fn x11_mods(depressed: u32) -> u16 {
    let mut x = 0u16;
    if depressed & 1 != 0 {
        x |= 1;
    }
    if depressed & 2 != 0 {
        x |= 2;
    }
    if depressed & 4 != 0 {
        x |= 4;
    }
    if depressed & 8 != 0 {
        x |= 8;
    }
    if depressed & 64 != 0 {
        x |= 64;
    }
    x
}

impl Server {
    fn local_point(&self, win: u32, x: i32, y: i32) -> WmPoint {
        let (ox, oy) = self.shared.lock().absolute_origin(win);
        WmPoint::new(x - ox, y - oy)
    }

    unsafe fn surface_at(&self, x: f64, y: f64) -> Option<(*mut wlr_surface, f64, f64)> {
        let mut nx = 0.0;
        let mut ny = 0.0;
        let node = wlr_scene_node_at(
            std::ptr::addr_of_mut!((*self.client_tree).node),
            x,
            y,
            std::ptr::addr_of_mut!(nx),
            std::ptr::addr_of_mut!(ny),
        );
        if node.is_null() {
            return None;
        }
        let buffer = wlr_scene_buffer_from_node(node);
        if buffer.is_null() {
            return None;
        }
        let scene_surface = wlr_scene_surface_try_from_buffer(buffer);
        if scene_surface.is_null() {
            return None;
        }
        Some(((*scene_surface).surface, nx, ny))
    }

    pub(crate) unsafe fn on_modifiers(&mut self) {
        if self.keyboard.is_null() {
            return;
        }
        let kb = self.keyboard;
        wlr_seat_set_keyboard(self.seat, kb);
        wlr_seat_keyboard_notify_modifiers(self.seat, std::ptr::addr_of!((*kb).modifiers));
        let mods = x11_mods((*kb).modifiers.depressed);
        self.shared.lock().key_mods = mods;
    }

    pub(crate) unsafe fn on_key(&mut self, event: *mut wlr_keyboard_key_event) {
        if event.is_null() {
            return;
        }
        let time = (*event).time_msec;
        let pressed = (*event).state != WL_KEYBOARD_KEY_STATE_RELEASED;
        let kc = (*event).keycode + 8;
        let target = {
            let mut s = self.shared.lock();
            s.last_time = time;
            let xmods = s.key_mods;
            let want = xmods & 0xFF & !LOCK_MASK;
            if let Some(gw) = s.keyboard_grab {
                Some(gw)
            } else {
                s.key_grabs
                    .iter()
                    .find(|g| {
                        u32::from(g.keycode) == kc && (g.modifiers & 0xFF & !LOCK_MASK) == want
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
            }
        };
        let xmods = self.shared.lock().key_mods;
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
                self.synth(ev);
            }
            None => {
                let state = u32::from(pressed);
                wlr_seat_keyboard_notify_key(self.seat, time, (*event).keycode, state);
            }
        }
    }

    pub(crate) unsafe fn on_motion(&mut self, event: *mut wlr_pointer_motion_event) {
        if event.is_null() {
            return;
        }
        wlr_cursor_move(
            self.cursor,
            std::ptr::null_mut(),
            (*event).delta_x,
            (*event).delta_y,
        );
        self.pointer_moved((*event).time_msec);
    }

    pub(crate) unsafe fn on_motion_absolute(
        &mut self,
        event: *mut wlr_pointer_motion_absolute_event,
    ) {
        if event.is_null() {
            return;
        }
        wlr_cursor_warp_absolute(
            self.cursor,
            std::ptr::null_mut(),
            (*event).x,
            (*event).y,
        );
        self.pointer_moved((*event).time_msec);
    }

    unsafe fn pointer_moved(&mut self, time: u32) {
        let (px, py) = self.cursor_xy();
        let grab = {
            let mut s = self.shared.lock();
            s.pointer_x = px as i16;
            s.pointer_y = py as i16;
            s.last_time = time;
            s.pointer_grab.or(s.implicit_grab)
        };
        let client_under = if grab.is_some() {
            None
        } else {
            self.surface_at(f64::from(px), f64::from(py))
        };
        match client_under {
            Some((surface, sx, sy)) if !surface.is_null() => {
                wlr_seat_pointer_notify_enter(self.seat, surface, sx, sy);
                wlr_seat_pointer_notify_motion(self.seat, time, sx, sy);
                wlr_seat_pointer_notify_frame(self.seat);
            }
            _ => {
                wlr_seat_pointer_notify_clear_focus(self.seat);
                self.set_default_cursor();
            }
        }
        if grab.is_none() {
            let raw = if client_under.is_some() {
                None
            } else {
                self.shared.lock().window_at(px, py)
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
            let s = self.shared.lock();
            s.window_at(px, py).and_then(|w| {
                s.propagate_event_target(
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

    pub(crate) unsafe fn on_button(&mut self, event: *mut wlr_pointer_button_event) {
        if event.is_null() {
            return;
        }
        let time = (*event).time_msec;
        let pressed = (*event).state != WL_POINTER_BUTTON_STATE_RELEASED;
        let btn = x11_button((*event).button);
        let (px, py) = self.cursor_xy();
        let (grab, mask) = {
            let mut s = self.shared.lock();
            if (1..=5).contains(&btn) {
                let bit = 1u16 << (7 + u16::from(btn));
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
        let client_under = self.surface_at(f64::from(px), f64::from(py));
        let wm_target = if let Some(g) = grab {
            Some(g)
        } else if let Some((surface, _, _)) = client_under {
            self.client_id_for_surface(surface)
        } else {
            let bit = if pressed {
                EventMask::BUTTON_PRESS
            } else {
                EventMask::BUTTON_RELEASE
            };
            let s = self.shared.lock();
            s.window_at(px, py)
                .and_then(|w| s.propagate_event_target(w, bit.bits()))
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
        if grab.is_none() && client_under.is_some() {
            wlr_seat_pointer_notify_button(
                self.seat,
                time,
                (*event).button,
                u32::from(pressed),
            );
            wlr_seat_pointer_notify_frame(self.seat);
        }
    }

    pub(crate) unsafe fn on_axis(&mut self, event: *mut wlr_pointer_axis_event) {
        if event.is_null() {
            return;
        }
        let delta = (*event).delta;
        let discrete = (*event).delta_discrete;
        let horizontal = (*event).orientation == 1;
        let (px, py) = self.cursor_xy();
        let client_under = self.surface_at(f64::from(px), f64::from(py));
        if client_under.is_some() {
            wlr_seat_pointer_notify_axis(
                self.seat,
                (*event).time_msec,
                (*event).orientation,
                delta,
                discrete,
                (*event).source,
                (*event).relative_direction,
            );
            wlr_seat_pointer_notify_frame(self.seat);
        }
        if delta == 0.0 {
            return;
        }
        let mask = {
            let s = self.shared.lock();
            s.pointer_mask | s.key_mods
        };
        let wm_target = if let Some((surface, _, _)) = client_under {
            self.client_id_for_surface(surface)
        } else {
            let s = self.shared.lock();
            s.window_at(px, py)
                .and_then(|w| s.propagate_event_target(w, EventMask::BUTTON_PRESS.bits()))
        };
        let Some(win) = wm_target else {
            return;
        };
        let button = if horizontal {
            if delta < 0.0 {
                6
            } else {
                7
            }
        } else if delta < 0.0 {
            4
        } else {
            5
        };
        let steps = if discrete == 0 {
            1
        } else {
            ((discrete.abs() as f64 / 120.0).round() as usize).max(1)
        };
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
    }
}

impl Server {
    pub(crate) unsafe fn cursor_xy(&self) -> (i32, i32) {
        if self.cursor.is_null() {
            return (0, 0);
        }
        ((*self.cursor).x as i32, (*self.cursor).y as i32)
    }
}

impl Server {
    pub(crate) unsafe fn set_default_cursor(&self) {
        if self.cursor.is_null() || self.cursor_mgr.is_null() {
            return;
        }
        let Ok(name) = std::ffi::CString::new("default") else {
            return;
        };
        wlr_cursor_set_xcursor(self.cursor, self.cursor_mgr, name.as_ptr());
    }
}
