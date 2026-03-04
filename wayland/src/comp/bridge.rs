use antibox_core::backend::BackendEvent;

use super::state::Server;
use crate::ffi::wlr::*;
use crate::shared::Intent;

pub(crate) const MIN_KEYCODE: u8 = 8;
pub(crate) const MAX_KEYCODE: u8 = 255;

impl Server {
    pub(crate) fn synth(&self, event: BackendEvent) {
        self.shared.lock().events.push(event);
    }

    pub(crate) fn put_prop(&self, window: u32, name: &str, data: Vec<u8>) {
        let mut s = self.shared.lock();
        let atom = s.atoms.intern(name);
        s.props.insert((window, atom), data);
        s.events.push(BackendEvent::PropertyNotify {
            window,
            atom,
            state: 0,
        });
    }

    pub(crate) fn build_keymap(&mut self) {
        let rows = unsafe { self.read_keymap() };
        let kpc = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
        let mut flat = Vec::with_capacity(rows.len() * kpc);
        for row in &rows {
            for i in 0..kpc {
                flat.push(row.get(i).copied().unwrap_or(0));
            }
        }
        let mut s = self.shared.lock();
        s.keymap = flat;
        s.keysyms_per_keycode = u8::try_from(kpc).unwrap_or(1);
    }

    unsafe fn read_keymap(&self) -> Vec<Vec<u32>> {
        let mut rows = Vec::new();
        if self.keyboard.is_null() {
            return vec![Vec::new(); usize::from(MAX_KEYCODE - MIN_KEYCODE) + 1];
        }
        let keymap = (*self.keyboard).keymap;
        let state = (*self.keyboard).xkb_state;
        if keymap.is_null() {
            return vec![Vec::new(); usize::from(MAX_KEYCODE - MIN_KEYCODE) + 1];
        }
        let layout = if state.is_null() {
            0
        } else {
            crate::ffi::xkb::xkb_state_serialize_layout(
                state,
                crate::ffi::xkb::XKB_STATE_LAYOUT_EFFECTIVE,
            )
        };
        for kc in MIN_KEYCODE..=MAX_KEYCODE {
            let levels =
                crate::ffi::xkb::xkb_keymap_num_levels_for_key(keymap, u32::from(kc), layout);
            let mut syms = Vec::new();
            for level in 0..levels {
                let mut out: *const u32 = std::ptr::null();
                let n = crate::ffi::xkb::xkb_keymap_key_get_syms_by_level(
                    keymap,
                    u32::from(kc),
                    layout,
                    level,
                    std::ptr::addr_of_mut!(out),
                );
                if n > 0 && !out.is_null() {
                    syms.push(*out);
                } else {
                    syms.push(0);
                }
            }
            rows.push(syms);
        }
        rows
    }

    pub(crate) unsafe fn place_client(&mut self, id: u32, size: Option<(u16, u16)>) {
        let Some(client) = self.clients.get(&id) else {
            return;
        };
        let (toplevel, tree) = (client.toplevel, client.tree);
        let (ax, ay) = {
            let s = self.shared.lock();
            s.absolute_origin(id)
        };
        let xsurface = client.xsurface;
        let (cw, ch) = {
            let s = self.shared.lock();
            s.windows
                .get(&id)
                .map_or((0, 0), |r| (r.rect.w, r.rect.h))
        };
        let (w, h) = size.map_or((cw, ch), |(w, h)| (i32::from(w), i32::from(h)));
        if !toplevel.is_null() && size.is_some() {
            wlr_xdg_toplevel_set_size(toplevel, w, h);
        }
        if !xsurface.is_null() {
            crate::ffi::xwayland::wlr_xwayland_surface_configure(
                xsurface,
                ax as i16,
                ay as i16,
                u16::try_from(w.max(1)).unwrap_or(u16::MAX),
                u16::try_from(h.max(1)).unwrap_or(u16::MAX),
            );
        }
        if !tree.is_null() {
            wlr_scene_node_set_position(std::ptr::addr_of_mut!((*tree).node), ax, ay);
            wlr_scene_node_set_enabled(std::ptr::addr_of_mut!((*tree).node), true);
        }
    }

    pub(crate) unsafe fn apply_intents(&mut self) {
        let intents = {
            let mut s = self.shared.lock();
            std::mem::take(&mut s.intents)
        };
        let mut focus_changed = false;
        for intent in intents {
            match intent {
                Intent::Map(id) => {
                    if self.clients.contains_key(&id) {
                        self.place_client(id, None);
                    }
                }
                Intent::Unmap(id) | Intent::Destroy(id) => {
                    if let Some(c) = self.clients.get(&id) {
                        if !c.tree.is_null() {
                            wlr_scene_node_set_enabled(
                                std::ptr::addr_of_mut!((*c.tree).node),
                                false,
                            );
                        }
                    }
                }
                Intent::Kill(id) => {
                    if let Some(c) = self.clients.get(&id) {
                        if !c.toplevel.is_null() {
                            wlr_xdg_toplevel_send_close(c.toplevel);
                        }
                        if !c.xsurface.is_null() {
                            crate::ffi::xwayland::wlr_xwayland_surface_close(c.xsurface);
                        }
                    }
                }
                Intent::Configure { win, w, h, .. } => {
                    if self.clients.contains_key(&win) {
                        let size = match (w, h) {
                            (Some(w), Some(h)) => Some((w, h)),
                            _ => None,
                        };
                        self.place_client(win, size);
                    } else {
                        let children: Vec<u32> = {
                            let s = self.shared.lock();
                            self.clients
                                .keys()
                                .copied()
                                .filter(|cid| {
                                    s.windows.get(cid).map(|r| r.parent) == Some(win)
                                })
                                .collect()
                        };
                        for cid in children {
                            self.place_client(cid, None);
                        }
                    }
                }
                Intent::Raise(id) => {
                    if let Some(c) = self.clients.get(&id) {
                        if !c.tree.is_null() {
                            wlr_scene_node_raise_to_top(std::ptr::addr_of_mut!(
                                (*c.tree).node
                            ));
                        }
                    }
                }
                Intent::Lower(id) => {
                    if let Some(c) = self.clients.get(&id) {
                        if !c.tree.is_null() {
                            wlr_scene_node_lower_to_bottom(std::ptr::addr_of_mut!(
                                (*c.tree).node
                            ));
                        }
                    }
                    let mut s = self.shared.lock();
                    if let Some(pos) = s.stack.iter().position(|&w| w == id) {
                        let w = s.stack.remove(pos);
                        s.stack.insert(0, w);
                    }
                }
                Intent::Restack(ids) => {
                    self.shared.lock().stack = ids;
                    self.restack();
                }
                Intent::Focus(_) => focus_changed = true,
                Intent::Warp { x, y } => {
                    wlr_cursor_warp_absolute(
                        self.cursor,
                        std::ptr::null_mut(),
                        f64::from(x),
                        f64::from(y),
                    );
                    let mut s = self.shared.lock();
                    s.pointer_x = x;
                    s.pointer_y = y;
                }
            }
        }
        if focus_changed {
            self.apply_focus();
        }
    }

    unsafe fn apply_focus(&mut self) {
        let focus = self.shared.lock().focus;
        let surface = self.clients.get(&focus).map(|c| c.surface);
        match surface {
            Some(surface) if !surface.is_null() && !self.keyboard.is_null() => {
                let kb = self.keyboard;
                wlr_seat_keyboard_notify_enter(
                    self.seat,
                    surface,
                    (*kb).keycodes.as_ptr(),
                    (*kb).num_keycodes,
                    std::ptr::addr_of!((*kb).modifiers),
                );
                if let Some(c) = self.clients.get(&focus) {
                    if !c.toplevel.is_null() {
                        wlr_xdg_toplevel_set_activated(c.toplevel, true);
                    }
                    if !c.xsurface.is_null() {
                        crate::ffi::xwayland::wlr_xwayland_surface_activate(c.xsurface, true);
                    }
                }
            }
            _ => wlr_seat_keyboard_notify_clear_focus(self.seat),
        }
    }
}

impl Server {
    pub(crate) unsafe fn install_keymap(&mut self, keyboard: *mut wlr_keyboard) {
        use crate::ffi::xkb::{
            xkb_context_new, xkb_context_unref, xkb_keymap_new_from_names, xkb_keymap_unref,
            xkb_rule_names,
        };
        let context = xkb_context_new(0);
        if context.is_null() {
            return;
        }
        let names = xkb_rule_names {
            rules: std::ptr::null(),
            model: std::ptr::null(),
            layout: std::ptr::null(),
            variant: std::ptr::null(),
            options: std::ptr::null(),
        };
        let keymap = xkb_keymap_new_from_names(context, std::ptr::addr_of!(names), 0);
        if !keymap.is_null() {
            wlr_keyboard_set_keymap(keyboard, keymap);
            xkb_keymap_unref(keymap);
        }
        xkb_context_unref(context);
    }
}
