use antibox_core::backend::BackendEvent;
use antibox_core::rect::Rect;
use std::os::raw::c_void;

use super::state::{Client, Server, Tag};
use crate::ffi::cstr_to_string;

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
        wlr_scene_output_send_frame_done(scene_output, std::ptr::null());
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
                wlr_keyboard_set_repeat_info(kb, 25, 600);
                self.keyboard = kb;
                self.hook(
                    std::ptr::addr_of_mut!((*kb).events.key),
                    Tag::KeyboardKey,
                    0,
                );
                self.hook(
                    std::ptr::addr_of_mut!((*kb).events.modifiers),
                    Tag::KeyboardModifiers,
                    0,
                );
                wlr_seat_set_keyboard(self.seat, kb);
                self.build_keymap();
            }
            WLR_INPUT_DEVICE_POINTER => wlr_cursor_attach_input_device(self.cursor, dev),
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
        let tree = wlr_scene_subsurface_tree_create(self.client_tree, surface);
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
                surface,
                tree,
                mapped: false,
                announced: false,
            },
        );
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
        wlr_xdg_surface_schedule_configure(base);
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
