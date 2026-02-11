use antibox_core::backend::BackendEvent;

use super::state::Server;
use crate::ffi::wlr::*;
use crate::shared::Intent;

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
        let mut s = self.shared.lock();
        if s.keymap.is_empty() {
            s.keymap = vec![0; 256];
            s.keysyms_per_keycode = 1;
        }
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
        if let Some((w, h)) = size {
            if !toplevel.is_null() {
                wlr_xdg_toplevel_set_size(toplevel, i32::from(w), i32::from(h));
            }
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
                Intent::Unmap(id) => {
                    if let Some(c) = self.clients.get(&id) {
                        if !c.tree.is_null() {
                            wlr_scene_node_set_enabled(
                                std::ptr::addr_of_mut!((*c.tree).node),
                                false,
                            );
                        }
                    }
                }
                Intent::Destroy(id) => {
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
                }
            }
            _ => wlr_seat_keyboard_notify_clear_focus(self.seat),
        }
    }
}
