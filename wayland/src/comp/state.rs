use crate::buffers::BufferStore;
use crate::ffi::buffer::{wlr_buffer, wlr_buffer_drop, PixBuffer};
use crate::ffi::wl::*;
use crate::ffi::wlr::*;
use crate::shared::Shared;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tag {
    NewOutput,
    NewInput,
    NewToplevel,
    OutputFrame,
    OutputDestroy,
    SurfaceMap,
    SurfaceUnmap,
    SurfaceCommit,
    ToplevelDestroy,
    SetTitle,
    SetAppId,
    KeyboardKey,
    KeyboardModifiers,
    CursorMotion,
    CursorMotionAbsolute,
    CursorButton,
    CursorAxis,
    CursorFrame,
    XwaylandReady,
    XwaylandNewSurface,
    XwaylandAssociate,
    XwaylandDissociate,
    XwaylandMap,
    XwaylandUnmap,
    XwaylandDestroy,
    XwaylandConfigure,
    XwaylandSetTitle,
}

#[repr(C)]
pub(crate) struct Hook {
    pub listener: wl_listener,
    pub server: *mut Server,
    pub tag: Tag,
    pub id: u32,
}

unsafe extern "C" fn trampoline(listener: *mut wl_listener, data: *mut c_void) {
    let hook = listener.cast::<Hook>();
    let server = (*hook).server;
    if server.is_null() {
        return;
    }
    let tag = (*hook).tag;
    let id = (*hook).id;
    (*server).dispatch(tag, id, data);
}

pub(crate) struct Client {
    pub toplevel: *mut wlr_xdg_toplevel,
    pub xsurface: *mut crate::ffi::xwayland::wlr_xwayland_surface,
    pub surface: *mut wlr_surface,
    pub tree: *mut wlr_scene_tree,
    pub mapped: bool,
    pub announced: bool,
}

pub(crate) struct Decoration {
    pub buffer_node: *mut wlr_scene_buffer,
    pub buffer: *mut wlr_buffer,
    pub width: u16,
    pub height: u16,
}

pub struct Server {
    pub(crate) display: *mut wl_display,
    pub(crate) event_loop: *mut wl_event_loop,
    pub(crate) backend: *mut wlr_backend,
    pub(crate) renderer: *mut wlr_renderer,
    pub(crate) allocator: *mut wlr_allocator,
    pub(crate) scene: *mut wlr_scene,
    pub(crate) layout: *mut wlr_output_layout,
    pub(crate) client_tree: *mut wlr_scene_tree,
    pub(crate) decor_tree: *mut wlr_scene_tree,
    pub(crate) compositor: *mut wlr_compositor,
    pub(crate) xdg_shell: *mut wlr_xdg_shell,
    pub(crate) seat: *mut wlr_seat,
    pub(crate) cursor: *mut wlr_cursor,
    pub(crate) cursor_mgr: *mut wlr_xcursor_manager,
    pub(crate) keyboard: *mut wlr_keyboard,
    pub(crate) outputs: Vec<*mut wlr_output>,
    pub(crate) scene_outputs: Vec<*mut wlr_scene_output>,
    pub(crate) clients: HashMap<u32, Client>,
    pub(crate) decorations: HashMap<u32, Decoration>,
    pub(crate) hooks: Vec<Box<Hook>>,
    pub(crate) shared: Shared,
    pub(crate) buffers: Arc<BufferStore>,
    pub(crate) socket: Option<String>,
    pub(crate) xwayland: *mut crate::ffi::xwayland::wlr_xwayland,
}

impl Server {
    pub(crate) fn hook(&mut self, signal: *mut wl_signal, tag: Tag, id: u32) {
        let mut hook = Box::new(Hook {
            listener: wl_listener::new(),
            server: std::ptr::from_mut(self),
            tag,
            id,
        });
        hook.listener.notify = Some(trampoline);
        unsafe {
            wl_list_init(&mut hook.listener.link);
            signal_add(signal, &mut hook.listener);
        }
        self.hooks.push(hook);
    }

    pub(crate) fn retarget_hooks(&mut self) {
        let me = std::ptr::from_mut(self);
        for hook in &mut self.hooks {
            hook.server = me;
        }
    }

    pub(crate) fn client_id_for_surface(&self, surface: *mut wlr_surface) -> Option<u32> {
        self.clients
            .iter()
            .find(|(_, c)| c.surface == surface)
            .map(|(id, _)| *id)
    }

    pub(crate) fn client_id_for_toplevel(
        &self,
        toplevel: *mut wlr_xdg_toplevel,
    ) -> Option<u32> {
        self.clients
            .iter()
            .find(|(_, c)| c.toplevel == toplevel)
            .map(|(id, _)| *id)
    }

    pub(crate) fn drop_decoration(&mut self, id: u32) {
        if let Some(d) = self.decorations.remove(&id) {
            unsafe {
                if !d.buffer_node.is_null() {
                    wlr_scene_node_destroy(std::ptr::addr_of_mut!((*d.buffer_node).node));
                }
                if !d.buffer.is_null() {
                    wlr_buffer_drop(d.buffer);
                }
            }
        }
    }

    pub(crate) fn sync_decoration(&mut self, id: u32) {
        let (ax, ay, viewable, kind) = {
            let s = self.shared.lock();
            let Some(rec) = s.windows.get(&id) else {
                return;
            };
            let (x, y) = s.absolute_origin(id);
            (x, y, s.viewable(id), rec.kind)
        };
        if !matches!(kind, crate::shared::WinKind::Server) || !viewable {
            self.drop_decoration(id);
            return;
        }
        let Some(pix) = self.buffers.snapshot(id) else {
            self.drop_decoration(id);
            return;
        };
        if pix.width == 0 || pix.height == 0 {
            return;
        }
        let pixels = rgba_to_argb(&pix.data);
        let buffer = PixBuffer::create(pix.width, pix.height, pixels);
        unsafe {
            let existing = self.decorations.get(&id).map(|d| d.buffer_node);
            let node = match existing {
                Some(n) if !n.is_null() => {
                    if let Some(d) = self.decorations.get(&id) {
                        if !d.buffer.is_null() {
                            wlr_buffer_drop(d.buffer);
                        }
                    }
                    wlr_scene_buffer_set_buffer(n, buffer);
                    n
                }
                _ => wlr_scene_buffer_create(self.decor_tree, buffer),
            };
            if node.is_null() {
                wlr_buffer_drop(buffer);
                return;
            }
            wlr_scene_node_set_position(std::ptr::addr_of_mut!((*node).node), ax, ay);
            wlr_scene_node_set_enabled(std::ptr::addr_of_mut!((*node).node), true);
            self.decorations.insert(
                id,
                Decoration {
                    buffer_node: node,
                    buffer,
                    width: pix.width,
                    height: pix.height,
                },
            );
        }
    }

    pub(crate) fn sync_all_decorations(&mut self) {
        let ids: Vec<u32> = {
            let s = self.shared.lock();
            s.windows
                .iter()
                .filter(|(_, r)| matches!(r.kind, crate::shared::WinKind::Server))
                .map(|(id, _)| *id)
                .collect()
        };
        for id in &ids {
            self.sync_decoration(*id);
        }
        let stale: Vec<u32> = self
            .decorations
            .keys()
            .copied()
            .filter(|id| !ids.contains(id))
            .collect();
        for id in stale {
            self.drop_decoration(id);
        }
        self.restack();
    }

    pub(crate) fn restack(&mut self) {
        let stack = self.shared.lock().stack.clone();
        for id in stack {
            unsafe {
                if let Some(c) = self.clients.get(&id) {
                    if !c.tree.is_null() {
                        wlr_scene_node_raise_to_top(std::ptr::addr_of_mut!((*c.tree).node));
                    }
                }
                if let Some(d) = self.decorations.get(&id) {
                    if !d.buffer_node.is_null() {
                        wlr_scene_node_raise_to_top(std::ptr::addr_of_mut!(
                            (*d.buffer_node).node
                        ));
                    }
                }
            }
        }
    }
}

fn rgba_to_argb(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|p| {
            u32::from(p[3]) << 24
                | u32::from(p[0]) << 16
                | u32::from(p[1]) << 8
                | u32::from(p[2])
        })
        .collect()
}
