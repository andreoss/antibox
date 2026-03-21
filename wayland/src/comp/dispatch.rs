use antibox_core::backend::BackendEvent;
use antibox_core::rect::Rect;
use std::os::raw::{c_int, c_void};

use super::state::{Client, Server, Tag};
use antibox_core::backend::hints::{mwm_decor, mwm_hints_flags};
use crate::ffi::cstr_to_string;
use super::xwayland::words_to_bytes;

use crate::ffi::wlr::*;
use crate::shared::{WinKind, WinRec, ROOT_WINDOW};

impl Server {
    pub(crate) unsafe fn dispatch(&mut self, tag: Tag, id: u32, data: *mut c_void) {
        match tag {
            Tag::NewOutput => self.on_new_output(data.cast::<wlr_output>()),
            Tag::OutputFrame => self.on_output_frame(id),
            Tag::OutputDestroy => self.on_output_destroy(data.cast::<wlr_output>()),
            Tag::NewInput => self.on_new_input(data.cast::<wlr_input_device>()),
            Tag::NewToplevel => self.on_new_toplevel(data.cast::<wlr_xdg_toplevel>()),
            Tag::SurfaceMap => self.on_surface_map(id),
            Tag::SurfaceUnmap => self.on_surface_unmap(id),
            Tag::SurfaceCommit => self.on_surface_commit(id),
            Tag::ToplevelDestroy => self.on_toplevel_destroy(id),
            Tag::SetTitle => self.on_set_title(id),
            Tag::SetAppId => self.on_set_app_id(id),
            Tag::KeyboardKey => self.on_key(data.cast::<wlr_keyboard_key_event>()),
            Tag::KeyboardModifiers => self.on_modifiers(),
            Tag::CursorMotion => self.on_motion(data.cast::<wlr_pointer_motion_event>()),
            Tag::CursorMotionAbsolute => {
                self.on_motion_absolute(data.cast::<wlr_pointer_motion_absolute_event>());
            }
            Tag::CursorButton => self.on_button(data.cast::<wlr_pointer_button_event>()),
            Tag::CursorAxis => self.on_axis(data.cast::<wlr_pointer_axis_event>()),
            Tag::CursorFrame => wlr_seat_pointer_notify_frame(self.seat),
            Tag::XwaylandReady => self.on_xwayland_ready(),
            Tag::XwaylandNewSurface => self.on_xwayland_new_surface(data.cast()),
            Tag::XwaylandAssociate => self.on_xwayland_associate(id),
            Tag::XwaylandDissociate => self.on_xwayland_dissociate(id),
            Tag::XwaylandMap => self.on_xwayland_map(id),
            Tag::XwaylandUnmap => self.on_xwayland_unmap(id),
            Tag::XwaylandDestroy => self.on_xwayland_destroy(id),
            Tag::XwaylandConfigure => self.on_xwayland_configure(id, data.cast()),
            Tag::XwaylandSetTitle => self.sync_xwayland_title(id),
            Tag::XwaylandSetGeometry => self.on_xwayland_geometry(id),
            Tag::NewDecoration => self.on_new_decoration(data.cast()),
            Tag::DecorationRequestMode => self.settle_decoration(data.cast()),
            Tag::DecorationDestroy => self.on_decoration_destroy(data.cast()),
            Tag::NewKdeDecoration => self.on_new_kde_decoration(data.cast()),
            Tag::KdeDecorationMode => self.apply_kde_decoration(data.cast()),
            Tag::KdeDecorationDestroy => self.on_kde_decoration_destroy(data.cast()),
            Tag::XwaylandSetHints => self.sync_xwayland_hints(id),
        }
    }

    unsafe fn on_new_output(&mut self, output: *mut wlr_output) {
        if output.is_null() {
            return;
        }
        wlr_output_init_render(output, self.allocator, self.renderer);
        let mut state: wlr_output_state = std::mem::zeroed();
        wlr_output_state_init(std::ptr::addr_of_mut!(state));
        wlr_output_state_set_enabled(std::ptr::addr_of_mut!(state), true);
        let mode = wlr_output_preferred_mode(output);
        if !mode.is_null() {
            wlr_output_state_set_mode(std::ptr::addr_of_mut!(state), mode);
        }
        wlr_output_commit_state(output, std::ptr::addr_of!(state));
        wlr_output_state_finish(std::ptr::addr_of_mut!(state));
        wlr_output_layout_add_auto(self.layout, output);
        let scene_output = wlr_scene_output_create(self.scene, output);
        let index = self.outputs.len() as u32;
        self.outputs.push(output);
        self.scene_outputs.push(scene_output);
        self.hook(
            std::ptr::addr_of_mut!((*output).events.frame),
            Tag::OutputFrame,
            index,
        );
        self.hook(
            std::ptr::addr_of_mut!((*output).events.destroy),
            Tag::OutputDestroy,
            index,
        );
        let (w, h) = ((*output).width, (*output).height);
        if w > 0 && h > 0 {
            let mut s = self.shared.lock();
            s.screen_w = w as u16;
            s.screen_h = h as u16;
            if let Some(root) = s.windows.get_mut(&ROOT_WINDOW) {
                root.rect = Rect::new(0, 0, w, h);
            }
        }
    }

    unsafe fn on_output_frame(&mut self, index: u32) {
        let Some(&scene_output) = self.scene_outputs.get(index as usize) else {
            return;
        };
        if scene_output.is_null() {
            return;
        }
        wlr_scene_output_commit(scene_output, std::ptr::null());
        let ts = crate::ffi::wl::now();
        wlr_scene_output_send_frame_done(scene_output, std::ptr::addr_of!(ts).cast());
    }

    unsafe fn on_output_destroy(&mut self, output: *mut wlr_output) {
        if let Some(pos) = self.outputs.iter().position(|o| *o == output) {
            self.outputs.remove(pos);
            self.scene_outputs.remove(pos);
        }
    }

    unsafe fn on_new_input(&mut self, dev: *mut wlr_input_device) {
        if dev.is_null() {
            return;
        }
        match (*dev).type_ {
            WLR_INPUT_DEVICE_KEYBOARD => {
                let kb = wlr_keyboard_from_input_device(dev);
                if kb.is_null() {
                    return;
                }
                self.install_keymap(kb);
                wlr_keyboard_group_add_keyboard(self.keyboard_group, kb);
            }
            WLR_INPUT_DEVICE_POINTER => {
                wlr_cursor_attach_input_device(self.cursor, dev);
                self.set_default_cursor();
            }
            _ => {}
        }
        let mut caps = WL_SEAT_CAPABILITY_POINTER;
        if !self.keyboard.is_null() {
            caps |= WL_SEAT_CAPABILITY_KEYBOARD;
        }
        wlr_seat_set_capabilities(self.seat, caps);
    }

    unsafe fn on_new_toplevel(&mut self, toplevel: *mut wlr_xdg_toplevel) {
        if toplevel.is_null() {
            return;
        }
        let base = (*toplevel).base;
        if base.is_null() {
            return;
        }
        let surface = (*base).surface;
        let tree = wlr_scene_xdg_surface_create(self.client_tree, base);
        let id = {
            let mut s = self.shared.lock();
            let id = s.alloc_id();
            s.windows.insert(
                id,
                WinRec {
                    kind: WinKind::Client,
                    rect: Rect::new(0, 0, 1, 1),
                    mapped: false,
                    override_redirect: false,
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
                toplevel,
                decoration: std::ptr::null_mut(),
                xsurface: std::ptr::null_mut(),
                surface,
                tree,
                mapped: false,
                announced: false,
            },
        );
        self.publish_decor_hint(id, false);
        self.adopt_kde_decoration(surface);
        let events = surface_events(surface);
        self.hook(std::ptr::addr_of_mut!((*events).map), Tag::SurfaceMap, id);
        self.hook(
            std::ptr::addr_of_mut!((*events).unmap),
            Tag::SurfaceUnmap,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*events).commit),
            Tag::SurfaceCommit,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*toplevel).events.destroy),
            Tag::ToplevelDestroy,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*toplevel).events.set_title),
            Tag::SetTitle,
            id,
        );
        self.hook(
            std::ptr::addr_of_mut!((*toplevel).events.set_app_id),
            Tag::SetAppId,
            id,
        );
    }

    unsafe fn on_surface_map(&mut self, id: u32) {
        let Some(client) = self.clients.get_mut(&id) else {
            return;
        };
        client.mapped = true;
        let toplevel = client.toplevel;
        let announced = client.announced;
        let (w, h) = geometry_of(toplevel);
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&id) {
                rec.rect.w = w.max(1);
                rec.rect.h = h.max(1);
            }
        }
        self.sync_title(id);
        if !announced {
            if let Some(c) = self.clients.get_mut(&id) {
                c.announced = true;
            }
            self.synth(BackendEvent::MapRequest { window: id });
        }
    }

    unsafe fn on_surface_unmap(&mut self, id: u32) {
        if let Some(client) = self.clients.get_mut(&id) {
            client.mapped = false;
        }
        self.synth(BackendEvent::UnmapNotify { window: id });
    }

    unsafe fn on_surface_commit(&mut self, id: u32) {
        let Some(client) = self.clients.get(&id) else {
            return;
        };
        let toplevel = client.toplevel;
        if !toplevel.is_null() {
            let base = (*toplevel).base;
            if !base.is_null() && (*base).initial_commit {
                let decoration = client.decoration;
                self.settle_decoration(decoration);
                wlr_xdg_toplevel_set_size(toplevel, 0, 0);
                return;
            }
        }
        let announced = client.announced;
        let (w, h) = geometry_of(toplevel);
        if w <= 0 || h <= 0 {
            return;
        }
        if !announced {
            {
                let mut s = self.shared.lock();
                if let Some(rec) = s.windows.get_mut(&id) {
                    rec.rect.w = w;
                    rec.rect.h = h;
                }
            }
            if let Some(c) = self.clients.get_mut(&id) {
                c.announced = true;
            }
            self.sync_title(id);
            self.synth(BackendEvent::MapRequest { window: id });
        }
    }

    unsafe fn on_toplevel_destroy(&mut self, id: u32) {
        self.drop_hooks(id);
        self.clients.remove(&id);
        self.shared.lock().windows.remove(&id);
        self.synth(BackendEvent::UnmapNotify { window: id });
        self.synth(BackendEvent::DestroyNotify { window: id });
    }

    unsafe fn on_set_title(&mut self, id: u32) {
        self.sync_title(id);
    }

    unsafe fn on_set_app_id(&mut self, id: u32) {
        self.sync_title(id);
    }

    unsafe fn sync_title(&mut self, id: u32) {
        let Some(client) = self.clients.get(&id) else {
            return;
        };
        let toplevel = client.toplevel;
        if toplevel.is_null() {
            return;
        }
        if let Some(title) = cstr_to_string((*toplevel).title) {
            self.put_prop(id, "_NET_WM_NAME", title.as_bytes().to_vec());
            self.put_prop(id, "WM_NAME", title.into_bytes());
        }
        if let Some(app_id) = cstr_to_string((*toplevel).app_id) {
            let mut data = app_id.clone().into_bytes();
            data.push(0);
            data.extend_from_slice(app_id.as_bytes());
            data.push(0);
            self.put_prop(id, "WM_CLASS", data);
        }
    }
}

unsafe fn geometry_of(toplevel: *mut wlr_xdg_toplevel) -> (i32, i32) {
    if toplevel.is_null() {
        return (0, 0);
    }
    let base = (*toplevel).base;
    if base.is_null() {
        return (0, 0);
    }
    let g = (*base).geometry;
    if g.width > 0 && g.height > 0 {
        return (g.width, g.height);
    }
    let surface = (*base).surface;
    if surface.is_null() {
        return (0, 0);
    }
    surface_size(surface)
}

impl Server {
    unsafe fn on_new_decoration(&mut self, decoration: *mut wlr_xdg_toplevel_decoration_v1) {
        if decoration.is_null() {
            return;
        }
        let toplevel = (*decoration).toplevel;
        if let Some(client) = self
            .clients
            .values_mut()
            .find(|c| c.toplevel == toplevel && !toplevel.is_null())
        {
            client.decoration = decoration;
        }
        self.hook_on(
            std::ptr::addr_of_mut!((*decoration).events.request_mode),
            Tag::DecorationRequestMode,
            0,
            decoration.cast(),
        );
        self.hook_on(
            std::ptr::addr_of_mut!((*decoration).events.destroy),
            Tag::DecorationDestroy,
            0,
            decoration.cast(),
        );
    }

    unsafe fn on_decoration_destroy(
        &mut self,
        decoration: *mut wlr_xdg_toplevel_decoration_v1,
    ) {
        for client in self.clients.values_mut() {
            if client.decoration == decoration {
                client.decoration = std::ptr::null_mut();
            }
        }
        self.drop_hooks_on(decoration.cast());
    }

    unsafe fn settle_decoration(&mut self, decoration: *mut wlr_xdg_toplevel_decoration_v1) {
        if decoration.is_null() {
            return;
        }
        let toplevel = (*decoration).toplevel;
        if toplevel.is_null() {
            return;
        }
        let base = (*toplevel).base;
        if base.is_null() {
            return;
        }
        let client_side = (*decoration).requested_mode
            == WLR_XDG_TOPLEVEL_DECORATION_V1_MODE_CLIENT_SIDE as c_int;
        let mode = if client_side {
            WLR_XDG_TOPLEVEL_DECORATION_V1_MODE_CLIENT_SIDE
        } else {
            WLR_XDG_TOPLEVEL_DECORATION_V1_MODE_SERVER_SIDE
        };
        if (*base).initialized {
            wlr_xdg_toplevel_decoration_v1_set_mode(decoration, mode);
        }
        if let Some(id) = self.client_id_for_surface((*base).surface) {
            self.publish_decor_hint(id, !client_side);
        }
    }
}

impl Server {
    unsafe fn on_new_kde_decoration(&mut self, decoration: *mut wlr_server_decoration) {
        if decoration.is_null() {
            return;
        }
        self.kde_decorations.push(decoration);
        self.hook_on(
            std::ptr::addr_of_mut!((*decoration).events.mode),
            Tag::KdeDecorationMode,
            0,
            decoration.cast(),
        );
        self.hook_on(
            std::ptr::addr_of_mut!((*decoration).events.destroy),
            Tag::KdeDecorationDestroy,
            0,
            decoration.cast(),
        );
        self.apply_kde_decoration(decoration);
    }

    pub(crate) unsafe fn apply_kde_decoration(
        &mut self,
        decoration: *mut wlr_server_decoration,
    ) {
        if decoration.is_null() {
            return;
        }
        let surface = (*decoration).surface;
        let Some(id) = self.client_id_for_surface(surface) else {
            return;
        };
        let client_draws = (*decoration).mode == WLR_SERVER_DECORATION_MANAGER_MODE_CLIENT
            || (*decoration).mode == 0;
        self.publish_decor_hint(id, !client_draws);
    }

    unsafe fn on_kde_decoration_destroy(&mut self, decoration: *mut wlr_server_decoration) {
        self.kde_decorations.retain(|d| *d != decoration);
        self.drop_hooks_on(decoration.cast());
    }

    pub(crate) unsafe fn adopt_kde_decoration(&mut self, surface: *mut wlr_surface) {
        let pending: Vec<*mut wlr_server_decoration> = self
            .kde_decorations
            .iter()
            .copied()
            .filter(|d| !d.is_null() && (**d).surface == surface)
            .collect();
        for decoration in pending {
            self.apply_kde_decoration(decoration);
        }
    }

    pub(crate) fn publish_decor_hint(&self, id: u32, decorate: bool) {
        let decorations = if decorate { mwm_decor::ALL } else { 0 };
        let words = [mwm_hints_flags::DECORATIONS, 0, decorations, 0];
        self.put_prop(id, "_MOTIF_WM_HINTS", words_to_bytes(&words));
    }
}
