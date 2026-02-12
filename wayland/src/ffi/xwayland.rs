use super::wl::{wl_display, wl_list, wl_signal};
use super::wlr::{wlr_compositor, wlr_seat, wlr_surface};
use std::os::raw::{c_char, c_void};

#[repr(C)]
pub struct wlr_xwayland_events {
    pub destroy: wl_signal,
    pub ready: wl_signal,
    pub new_surface: wl_signal,
    pub remove_startup_info: wl_signal,
}

#[repr(C)]
pub struct wlr_xwayland {
    pub server: *mut c_void,
    pub own_server: bool,
    pub xwm: *mut c_void,
    pub shell_v1: *mut c_void,
    pub cursor: *mut c_void,
    pub display_name: *const c_char,
    pub wl_display: *mut wl_display,
    pub compositor: *mut wlr_compositor,
    pub seat: *mut wlr_seat,
    pub events: wlr_xwayland_events,
}

#[repr(C)]
pub struct wlr_addon {
    pub impl_: *const c_void,
    pub owner: *const c_void,
    pub link: wl_list,
}

#[repr(C)]
pub struct wlr_xwayland_surface_events {
    pub destroy: wl_signal,
    pub request_configure: wl_signal,
    pub request_move: wl_signal,
    pub request_resize: wl_signal,
    pub request_minimize: wl_signal,
    pub request_maximize: wl_signal,
    pub request_fullscreen: wl_signal,
    pub request_activate: wl_signal,
    pub request_close: wl_signal,
    pub request_sticky: wl_signal,
    pub request_shaded: wl_signal,
    pub request_skip_taskbar: wl_signal,
    pub request_skip_pager: wl_signal,
    pub request_above: wl_signal,
    pub request_below: wl_signal,
    pub request_demands_attention: wl_signal,
    pub associate: wl_signal,
    pub dissociate: wl_signal,
    pub set_title: wl_signal,
    pub set_class: wl_signal,
    pub set_role: wl_signal,
    pub set_parent: wl_signal,
    pub set_startup_id: wl_signal,
    pub set_window_type: wl_signal,
    pub set_hints: wl_signal,
    pub set_decorations: wl_signal,
    pub set_strut_partial: wl_signal,
    pub set_override_redirect: wl_signal,
    pub set_geometry: wl_signal,
    pub set_opacity: wl_signal,
    pub focus_in: wl_signal,
    pub grab_focus: wl_signal,
    pub map_request: wl_signal,
    pub ping_timeout: wl_signal,
}

#[repr(C)]
pub struct wlr_xwayland_surface {
    pub window_id: u32,
    pub xwm: *mut c_void,
    pub surface_id: u32,
    pub serial: u64,
    pub link: wl_list,
    pub stack_link: wl_list,
    pub unpaired_link: wl_list,
    pub surface: *mut wlr_surface,
    pub surface_addon: wlr_addon,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub override_redirect: bool,
    pub opacity: f32,
    pub title: *mut c_char,
    pub class: *mut c_char,
    pub instance: *mut c_char,
    pub role: *mut c_char,
    pub startup_id: *mut c_char,
    pub pid: i32,
    pub has_utf8_title: bool,
    pub children: wl_list,
    pub parent: *mut wlr_xwayland_surface,
    pub parent_link: wl_list,
    pub window_type: *mut u32,
    pub window_type_len: usize,
    pub protocols: *mut u32,
    pub protocols_len: usize,
    pub decorations: u32,
    pub hints: *mut c_void,
    pub size_hints: *mut c_void,
    pub strut_partial: *mut c_void,
    pub pinging: bool,
    pub ping_timer: *mut c_void,
    pub modal: bool,
    pub fullscreen: bool,
    pub maximized_vert: bool,
    pub maximized_horz: bool,
    pub minimized: bool,
    pub withdrawn: bool,
    pub sticky: bool,
    pub shaded: bool,
    pub skip_taskbar: bool,
    pub skip_pager: bool,
    pub above: bool,
    pub below: bool,
    pub demands_attention: bool,
    pub has_alpha: bool,
    pub events: wlr_xwayland_surface_events,
    pub data: *mut c_void,
    private: [u8; 88],
}

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_xwayland_create(
        display: *mut wl_display,
        compositor: *mut wlr_compositor,
        lazy: bool,
    ) -> *mut wlr_xwayland;
    pub fn wlr_xwayland_destroy(xwayland: *mut wlr_xwayland);
    pub fn wlr_xwayland_set_seat(xwayland: *mut wlr_xwayland, seat: *mut wlr_seat);
    pub fn wlr_xwayland_surface_activate(surface: *mut wlr_xwayland_surface, activated: bool);
    pub fn wlr_xwayland_surface_configure(
        surface: *mut wlr_xwayland_surface,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    );
    pub fn wlr_xwayland_surface_close(surface: *mut wlr_xwayland_surface);
}
