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
    XwaylandSetHints,
    XwaylandSetGeometry,
    NewDecoration,
    DecorationRequestMode,
    DecorationDestroy,
    NewKdeDecoration,
    KdeDecorationMode,
    KdeDecorationDestroy,
}

#[repr(C)]
pub(crate) struct Hook {
    pub listener: wl_listener,
    pub server: *mut Server,
    pub tag: Tag,
    pub id: u32,
    pub obj: *mut c_void,
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
    pub decoration: *mut wlr_xdg_toplevel_decoration_v1,
    pub xsurface: *mut crate::ffi::xwayland::wlr_xwayland_surface,
    pub surface: *mut wlr_surface,
    pub tree: *mut wlr_scene_tree,
    pub mapped: bool,
    pub announced: bool,
}

pub(crate) struct Decoration {
    pub buffer_node: *mut wlr_scene_buffer,
    pub buffer: *mut wlr_buffer,
    pub generation: u64,
    pub x: i32,
    pub y: i32,
}

pub struct Server {
    pub(crate) display: *mut wl_display,
    pub(crate) event_loop: *mut wl_event_loop,
    pub(crate) renderer: *mut wlr_renderer,
    pub(crate) allocator: *mut wlr_allocator,
    pub(crate) scene: *mut wlr_scene,
    pub(crate) layout: *mut wlr_output_layout,
    pub(crate) client_tree: *mut wlr_scene_tree,
    pub(crate) compositor: *mut wlr_compositor,
    pub(crate) seat: *mut wlr_seat,
    pub(crate) cursor: *mut wlr_cursor,
    pub(crate) cursor_mgr: *mut wlr_xcursor_manager,
    pub(crate) keyboard: *mut wlr_keyboard,
    pub(crate) outputs: Vec<*mut wlr_output>,
    pub(crate) scene_outputs: Vec<*mut wlr_scene_output>,
    pub(crate) clients: HashMap<u32, Client>,
    pub(crate) decorations: HashMap<u32, Decoration>,
    #[allow(clippy::vec_box)]
    pub(crate) hooks: Vec<Box<Hook>>,
    pub(crate) shared: Shared,
    pub(crate) buffers: Arc<BufferStore>,
    pub(crate) socket: Option<String>,
    pub(crate) xwayland: *mut crate::ffi::xwayland::wlr_xwayland,
    pub(crate) kde_decorations: Vec<*mut wlr_server_decoration>,
}

impl Server {
    pub(crate) fn hook(&mut self, signal: *mut wl_signal, tag: Tag, id: u32) {
        self.hook_on(signal, tag, id, std::ptr::null_mut());
    }

    pub(crate) fn hook_on(
        &mut self,
        signal: *mut wl_signal,
        tag: Tag,
        id: u32,
        obj: *mut c_void,
    ) {
        let mut hook = Box::new(Hook {
            listener: wl_listener::new(),
            server: std::ptr::from_mut(self),
            tag,
            id,
            obj,
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
        let generation = self.buffers.generation(id);
        if let Some(d) = self.decorations.get(&id) {
            if d.generation == generation {
                if d.x != ax || d.y != ay {
                    let node = d.buffer_node;
                    unsafe {
                        wlr_scene_node_set_position(
                            std::ptr::addr_of_mut!((*node).node),
                            ax,
                            ay,
                        );
                    }
                    if let Some(d) = self.decorations.get_mut(&id) {
                        d.x = ax;
                        d.y = ay;
                    }
                }
                return;
            }
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
                _ => wlr_scene_buffer_create(self.client_tree, buffer),
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
                    generation,
                    x: ax,
                    y: ay,
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
        let before = self.decorations.len();
        for id in &ids {
            self.sync_decoration(*id);
        }
        let stale: Vec<u32> = self
            .decorations
            .keys()
            .copied()
            .filter(|id| !ids.contains(id))
            .collect();
        let changed = !stale.is_empty() || self.decorations.len() != before;
        for id in stale {
            self.drop_decoration(id);
        }
        if changed {
            self.restack();
            self.raise_popups();
        }
    }

    pub(crate) fn restack(&mut self) {
        let (stack, ordered) = {
            let s = self.shared.lock();
            let mut ordered: Vec<(u32, usize)> = self
                .decorations
                .keys()
                .map(|id| (*id, window_depth(&s, *id)))
                .collect();
            ordered.sort_by_key(|(_, d)| *d);
            (s.stack.clone(), ordered)
        };
        for (id, _) in ordered {
            unsafe {
                if let Some(d) = self.decorations.get(&id) {
                    if !d.buffer_node.is_null() {
                        wlr_scene_node_raise_to_top(std::ptr::addr_of_mut!(
                            (*d.buffer_node).node
                        ));
                    }
                }
            }
        }
        for id in stack {
            let subtree = self.shared.lock().subtree(id);
            for kid in subtree {
                unsafe {
                    if let Some(d) = self.decorations.get(&kid) {
                        if !d.buffer_node.is_null() {
                            wlr_scene_node_raise_to_top(std::ptr::addr_of_mut!(
                                (*d.buffer_node).node
                            ));
                        }
                    }
                    if let Some(c) = self.clients.get(&kid) {
                        if !c.tree.is_null() {
                            wlr_scene_node_raise_to_top(std::ptr::addr_of_mut!(
                                (*c.tree).node
                            ));
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn raise_popups(&mut self) {
        let popups = {
            let s = self.shared.lock();
            let mut popups: Vec<(u32, usize)> = self
                .decorations
                .keys()
                .filter(|id| {
                    !s.stack.contains(id)
                        && s.windows.get(id).is_some_and(|r| {
                            r.override_redirect && r.parent == crate::shared::ROOT_WINDOW
                        })
                })
                .map(|id| (*id, window_depth(&s, *id)))
                .collect();
            popups.sort_by_key(|(_, d)| *d);
            popups
        };
        for (id, _) in popups {
            unsafe {
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

fn window_depth(s: &crate::shared::SharedState, id: u32) -> usize {
    let mut depth = 0;
    let mut cur = id;
    for _ in 0..32 {
        let Some(rec) = s.windows.get(&cur) else {
            break;
        };
        if rec.parent == crate::shared::ROOT_WINDOW || rec.parent == cur {
            break;
        }
        cur = rec.parent;
        depth += 1;
    }
    depth
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

impl Server {
    pub(crate) unsafe fn drop_hooks_on(&mut self, obj: *mut c_void) {
        let mut kept = Vec::with_capacity(self.hooks.len());
        for mut hook in std::mem::take(&mut self.hooks) {
            if hook.obj == obj {
                listener_detach(std::ptr::addr_of_mut!(hook.listener));
            } else {
                kept.push(hook);
            }
        }
        self.hooks = kept;
    }

    pub(crate) unsafe fn drop_hooks(&mut self, id: u32) {
        self.drop_hooks_where(id, |_| true);
    }

    pub(crate) unsafe fn drop_hooks_where(&mut self, id: u32, keep: impl Fn(Tag) -> bool) {
        if id == 0 {
            return;
        }
        let mut kept = Vec::with_capacity(self.hooks.len());
        for mut hook in std::mem::take(&mut self.hooks) {
            if hook.id == id && keep(hook.tag) {
                listener_detach(std::ptr::addr_of_mut!(hook.listener));
            } else {
                kept.push(hook);
            }
        }
        self.hooks = kept;
    }
}
