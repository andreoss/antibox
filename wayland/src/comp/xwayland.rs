use antibox_core::backend::BackendEvent;
use antibox_core::rect::Rect;

use super::state::{Client, Server, Tag};
use crate::ffi::cstr_to_string;
use crate::ffi::wlr::*;
use crate::ffi::xwayland::*;
use crate::shared::{WinKind, WinRec, ROOT_WINDOW};

impl Server {
    pub(crate) unsafe fn start_xwayland(&mut self) {
        let compositor = self.compositor;
        if compositor.is_null() {
            return;
        }
        let xwayland = wlr_xwayland_create(self.display, compositor, false);
        if xwayland.is_null() {
            return;
        }
        self.xwayland = xwayland;
        wlr_xwayland_set_seat(xwayland, self.seat);
        self.hook(
            std::ptr::addr_of_mut!((*xwayland).events.ready),
            Tag::XwaylandReady,
            0,
        );
        self.hook(
            std::ptr::addr_of_mut!((*xwayland).events.new_surface),
            Tag::XwaylandNewSurface,
            0,
        );
    }

    pub(crate) unsafe fn on_xwayland_ready(&mut self) {
        if self.xwayland.is_null() {
            return;
        }
        if let Some(name) = cstr_to_string((*self.xwayland).display_name) {
            std::env::set_var("DISPLAY", name);
        }
    }

    pub(crate) unsafe fn on_xwayland_new_surface(
        &mut self,
        xsurface: *mut wlr_xwayland_surface,
    ) {
        if xsurface.is_null() {
            return;
        }
        let id = {
            let mut s = self.shared.lock();
            let id = s.alloc_id();
            s.windows.insert(
                id,
                WinRec {
                    kind: WinKind::Client,
                    rect: Rect::new(
                        i32::from((*xsurface).x),
                        i32::from((*xsurface).y),
                        i32::from((*xsurface).width).max(1),
                        i32::from((*xsurface).height).max(1),
                    ),
                    mapped: false,
                    override_redirect: (*xsurface).override_redirect,
                    depth: 32,
                    parent: ROOT_WINDOW,
                    event_mask: 0,
                },
            );
            id
        };
        self.clients.insert(
            id,
            Client {
                toplevel: std::ptr::null_mut(),
                xsurface,
                surface: std::ptr::null_mut(),
                tree: std::ptr::null_mut(),
                mapped: false,
                announced: false,
            },
        );
        self.hook(
            std::ptr::addr_of_mut!((*xsurface).events.associate),
            Tag::XwaylandAssociate,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*xsurface).events.dissociate),
            Tag::XwaylandDissociate,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*xsurface).events.destroy),
            Tag::XwaylandDestroy,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*xsurface).events.request_configure),
            Tag::XwaylandConfigure,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*xsurface).events.set_title),
            Tag::XwaylandSetTitle,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*xsurface).events.set_hints),
            Tag::XwaylandSetHints,
            id,
        );
    }

    pub(crate) unsafe fn on_xwayland_associate(&mut self, id: u32) {
        let Some(client) = self.clients.get(&id) else {
            return;
        };
        let xsurface = client.xsurface;
        if xsurface.is_null() {
            return;
        }
        let surface = (*xsurface).surface;
        if surface.is_null() {
            return;
        }
        let tree = wlr_scene_subsurface_tree_create(self.client_tree, surface);
        if let Some(c) = self.clients.get_mut(&id) {
            c.surface = surface;
            c.tree = tree;
        }
        let events = surface_events(surface);
        self.hook(
            std::ptr::addr_of_mut!((*events).map),
            Tag::XwaylandMap,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*events).unmap),
            Tag::XwaylandUnmap,
            id,
        );
    }

    pub(crate) unsafe fn on_xwayland_dissociate(&mut self, id: u32) {
        if let Some(c) = self.clients.get_mut(&id) {
            if !c.tree.is_null() {
                wlr_scene_node_destroy(std::ptr::addr_of_mut!((*c.tree).node));
            }
            c.tree = std::ptr::null_mut();
            c.surface = std::ptr::null_mut();
        }
    }

    pub(crate) unsafe fn on_xwayland_map(&mut self, id: u32) {
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        client.mapped = true;
        let xsurface = client.xsurface;
        let announced = client.announced;
        let surface = (*xsurface).surface;
        let (sw, sh) = if surface.is_null() {
            (0, 0)
        } else {
            surface_size(surface)
        };
        self.sync_xwayland_hints(id);
        let hinted = self.hinted_size(xsurface);
        let raw = (i32::from((*xsurface).width), i32::from((*xsurface).height));
        let (w, h) = if raw.0 > 1 && raw.1 > 1 {
            raw
        } else if sw > 1 && sh > 1 {
            (sw, sh)
        } else {
            hinted.unwrap_or((640, 480))
        };
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&id) {
                rec.rect.w = w;
                rec.rect.h = h;
                rec.override_redirect = (*xsurface).override_redirect;
            }
        }
        self.sync_xwayland_title(id);
        if (*xsurface).override_redirect {
            let tree = self.clients.get(&id).map_or(std::ptr::null_mut(), |c| c.tree);
            if !tree.is_null() {
                wlr_scene_node_set_position(
                    std::ptr::addr_of_mut!((*tree).node),
                    i32::from((*xsurface).x),
                    i32::from((*xsurface).y),
                );
                wlr_scene_node_set_enabled(std::ptr::addr_of_mut!((*tree).node), true);
                wlr_scene_node_raise_to_top(std::ptr::addr_of_mut!((*tree).node));
            }
            return;
        }
        if !announced {
            if let Some(c) = self.clients.get_mut(&id) {
                c.announced = true;
            }
            self.synth(BackendEvent::MapRequest { window: id });
        }
    }

    pub(crate) unsafe fn on_xwayland_unmap(&mut self, id: u32) {
        if let Some(c) = self.clients.get_mut(&id) {
            c.mapped = false;
        }
        self.synth(BackendEvent::UnmapNotify { window: id });
    }

    pub(crate) unsafe fn on_xwayland_destroy(&mut self, id: u32) {
        self.clients.remove(&id);
        self.shared.lock().windows.remove(&id);
        self.synth(BackendEvent::UnmapNotify { window: id });
        self.synth(BackendEvent::DestroyNotify { window: id });
    }

    pub(crate) unsafe fn on_xwayland_configure(&mut self, id: u32) {
        let Some(client) = self.clients.get(&id) else {
            return;
        };
        let xsurface = client.xsurface;
        if xsurface.is_null() {
            return;
        }
        let (x, y, w, h) = {
            let s = self.shared.lock();
            let Some(rec) = s.windows.get(&id) else {
                return;
            };
            let (ax, ay) = s.absolute_origin(id);
            (ax, ay, rec.rect.w, rec.rect.h)
        };
        wlr_xwayland_surface_configure(
            xsurface,
            x as i16,
            y as i16,
            u16::try_from(w.max(1)).unwrap_or(u16::MAX),
            u16::try_from(h.max(1)).unwrap_or(u16::MAX),
        );
    }

    pub(crate) unsafe fn sync_xwayland_title(&mut self, id: u32) {
        let Some(client) = self.clients.get(&id) else {
            return;
        };
        let xsurface = client.xsurface;
        if xsurface.is_null() {
            return;
        }
        if let Some(title) = cstr_to_string((*xsurface).title) {
            self.put_prop(id, "_NET_WM_NAME", title.as_bytes().to_vec());
            self.put_prop(id, "WM_NAME", title.into_bytes());
        }
        if let Some(class) = cstr_to_string((*xsurface).class) {
            let instance = cstr_to_string((*xsurface).instance).unwrap_or_else(|| class.clone());
            let mut data = instance.into_bytes();
            data.push(0);
            data.extend_from_slice(class.as_bytes());
            data.push(0);
            self.put_prop(id, "WM_CLASS", data);
        }
    }
}

impl Server {
    pub(crate) unsafe fn sync_xwayland_hints(&mut self, id: u32) {
        let Some(client) = self.clients.get(&id) else {
            return;
        };
        let xsurface = client.xsurface;
        if xsurface.is_null() {
            return;
        }
        let size_hints = (*xsurface).size_hints;
        if !size_hints.is_null() {
            let words = std::slice::from_raw_parts(size_hints.cast::<u32>(), 18);
            self.put_prop(id, "WM_NORMAL_HINTS", words_to_bytes(words));
        }
        let hints = (*xsurface).hints;
        if !hints.is_null() {
            let words = std::slice::from_raw_parts(hints.cast::<u32>(), 9);
            self.put_prop(id, "WM_HINTS", words_to_bytes(words));
        }
    }

    unsafe fn hinted_size(&self, xsurface: *mut wlr_xwayland_surface) -> Option<(i32, i32)> {
        let size_hints = (*xsurface).size_hints;
        if size_hints.is_null() {
            return None;
        }
        let words = std::slice::from_raw_parts(size_hints.cast::<u32>(), 18);
        let flags = words[0];
        if flags & (US_SIZE | P_SIZE) != 0 && words[3] > 0 && words[4] > 0 {
            return Some((words[3] as i32, words[4] as i32));
        }
        if flags & P_BASE_SIZE != 0 && words[15] > 0 && words[16] > 0 {
            return Some((words[15] as i32, words[16] as i32));
        }
        if flags & P_MIN_SIZE != 0 && words[5] > 0 && words[6] > 0 {
            return Some((words[5] as i32, words[6] as i32));
        }
        None
    }
}

const US_SIZE: u32 = 2;
const P_SIZE: u32 = 8;
const P_MIN_SIZE: u32 = 16;
const P_BASE_SIZE: u32 = 256;

fn words_to_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_ne_bytes()).collect()
}
