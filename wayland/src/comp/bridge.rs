use antibox_core::backend::BackendEvent;
use antibox_core::rect::Rect;
use smithay::input::keyboard::Keycode;
use smithay::utils::{Point, Rectangle, Size};

use crate::shared::{Intent, WinKind, WinRec, ROOT_WINDOW};

use super::state::{window_wl_surface, Compositor};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

pub(crate) const MIN_KEYCODE: u8 = 8;
pub(crate) const MAX_KEYCODE: u8 = 255;

impl Compositor {
    pub(crate) fn build_keymap(&mut self) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        let mut rows: Vec<Vec<u32>> = Vec::new();
        keyboard.with_xkb_state(self, |ctx| {
            let xkb = ctx.xkb().lock().unwrap();
            let layout = xkb.active_layout();
            for kc in MIN_KEYCODE..=MAX_KEYCODE {
                let syms = xkb.raw_syms_for_key_in_layout(Keycode::new(kc as u32), layout);
                rows.push(syms.iter().map(|k| k.raw()).collect());
            }
        });
        let kpc = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
        let mut flat = Vec::with_capacity(rows.len() * kpc);
        for r in &rows {
            for i in 0..kpc {
                flat.push(r.get(i).copied().unwrap_or(0));
            }
        }
        let mut s = self.shared.lock();
        s.keymap = flat;
        s.keysyms_per_keycode = kpc as u8;
    }

    pub(crate) fn synth(&self, event: BackendEvent) {
        self.shared.lock().events.push(event);
    }

    pub(crate) fn register_client(&mut self, window: smithay::desktop::Window) -> u32 {
        let rect = window
            .x11_surface()
            .map(|x| {
                let g = x.geometry();
                Rect::new(g.loc.x, g.loc.y, g.size.w, g.size.h)
            })
            .unwrap_or_else(|| {
                let g = window.geometry();
                Rect::new(g.loc.x, g.loc.y, g.size.w.max(1), g.size.h.max(1))
            });
        let id = {
            let mut s = self.shared.lock();
            let id = s.alloc_id();
            s.windows.insert(
                id,
                WinRec {
                    kind: WinKind::Client,
                    rect,
                    mapped: false,
                    override_redirect: false,
                    depth: 32,
                    parent: ROOT_WINDOW,
                    event_mask: 0,
                },
            );
            id
        };
        let ready = window.x11_surface().is_some() || (rect.w > 1 && rect.h > 1);
        self.clients.insert(id, window);
        if ready {
            self.synth(BackendEvent::MapRequest { window: id });
        } else {
            self.pending_map.insert(id);
        }
        id
    }

    pub(crate) fn unregister_client(&mut self, id: u32) {
        if let Some(window) = self.clients.remove(&id) {
            self.space.unmap_elem(&window);
        }
        self.shared.lock().windows.remove(&id);
        self.synth(BackendEvent::UnmapNotify { window: id });
        self.synth(BackendEvent::DestroyNotify { window: id });
    }

    fn client_origin(&self, id: u32) -> Point<i32, smithay::utils::Logical> {
        let s = self.shared.lock();
        if let Some(rec) = s.windows.get(&id) {
            let (px, py) = s
                .windows
                .get(&rec.parent)
                .map(|p| (p.rect.x, p.rect.y))
                .unwrap_or((0, 0));
            Point::from((px + rec.rect.x, py + rec.rect.y))
        } else {
            Point::from((0, 0))
        }
    }

    fn place_client(&mut self, id: u32, size: Option<(u16, u16)>) {
        let Some(window) = self.clients.get(&id).cloned() else {
            return;
        };
        let loc = self.client_origin(id);
        if let Some(toplevel) = window.toplevel() {
            if let Some((w, h)) = size {
                toplevel.with_pending_state(|st| {
                    st.size = Some(Size::from((w as i32, h as i32)));
                });
                toplevel.send_pending_configure();
            }
        } else if let Some(x11) = window.x11_surface() {
            let (w, h) = size.map(|(w, h)| (w as i32, h as i32)).unwrap_or_else(|| {
                let g = x11.geometry();
                (g.size.w, g.size.h)
            });
            let _ = x11.configure(Some(Rectangle::new(loc, Size::from((w, h)))));
        }
        self.space.map_element(window, loc, false);
    }

    pub fn apply_intents(&mut self) {
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
                    if let Some(window) = self.clients.get(&id).cloned() {
                        self.space.unmap_elem(&window);
                    }
                }
                Intent::Destroy(id) => {
                    if let Some(window) = self.clients.remove(&id) {
                        self.space.unmap_elem(&window);
                    }
                }
                Intent::Kill(id) => {
                    if let Some(window) = self.clients.get(&id) {
                        if let Some(t) = window.toplevel() {
                            t.send_close();
                        } else if let Some(x11) = window.x11_surface() {
                            let _ = x11.close();
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
                                .filter(|cid| s.windows.get(cid).map(|r| r.parent) == Some(win))
                                .collect()
                        };
                        for cid in children {
                            self.place_client(cid, None);
                        }
                    }
                }
                Intent::Raise(id) => {
                    if let Some(window) = self.clients.get(&id).cloned() {
                        self.space.raise_element(&window, true);
                    }
                }
                Intent::Restack(ids) => {
                    for id in &ids {
                        if let Some(window) = self.clients.get(id).cloned() {
                            self.space.raise_element(&window, false);
                        }
                    }
                    self.shared.lock().stack = ids;
                }
                Intent::Focus(_) => focus_changed = true,
                Intent::Lower(id) => {
                    let mut s = self.shared.lock();
                    if let Some(pos) = s.stack.iter().position(|&w| w == id) {
                        let w = s.stack.remove(pos);
                        s.stack.insert(0, w);
                    }
                }
                Intent::Warp { x, y } => {
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

    fn apply_focus(&mut self) {
        let focus = self.shared.lock().focus;
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        let serial = smithay::utils::SERIAL_COUNTER.next_serial();
        let target = self
            .clients
            .get(&focus)
            .and_then(super::state::KeyboardFocusTarget::for_window);
        keyboard.set_focus(self, target, serial);
    }
}

impl Compositor {
    pub(crate) fn sync_client_geometry(&mut self, surface: &WlSurface) {
        let Some(id) = self.clients.iter().find_map(|(id, w)| {
            (window_wl_surface(w).as_ref() == Some(surface)).then_some(*id)
        }) else {
            return;
        };
        let size = {
            let Some(w) = self.clients.get(&id) else {
                return;
            };
            w.on_commit();
            let g = w.geometry();
            (g.size.w, g.size.h)
        };
        if size.0 <= 0 || size.1 <= 0 {
            return;
        }
        let announce = self.pending_map.remove(&id);
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&id) {
                if announce {
                    rec.rect.w = size.0;
                    rec.rect.h = size.1;
                }
            }
        }
        self.sync_toplevel_props(id, surface);
        if announce {
            self.synth(BackendEvent::MapRequest { window: id });
        }
    }
}

impl Compositor {
    pub(crate) fn sync_toplevel_props(&mut self, id: u32, surface: &WlSurface) {
        use smithay::wayland::compositor::with_states;
        use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;
        let (title, app_id) = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|d| d.lock().ok().map(|g| (g.title.clone(), g.app_id.clone())))
                .unwrap_or((None, None))
        });
        if let Some(title) = title {
            self.put_prop(id, "_NET_WM_NAME", title.as_bytes().to_vec());
            self.put_prop(id, "WM_NAME", title.into_bytes());
        }
        if let Some(app_id) = app_id {
            let mut data = app_id.clone().into_bytes();
            data.push(0);
            data.extend_from_slice(app_id.as_bytes());
            data.push(0);
            self.put_prop(id, "WM_CLASS", data);
        }
    }
}
