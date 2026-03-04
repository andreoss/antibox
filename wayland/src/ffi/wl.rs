use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
pub struct wl_display {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wl_event_loop {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wl_event_source {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wl_client {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wl_resource {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wl_global {
    _opaque: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct wl_list {
    pub prev: *mut wl_list,
    pub next: *mut wl_list,
}

impl wl_list {
    pub const fn empty() -> Self {
        Self {
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        }
    }
}

pub type wl_notify_func_t = unsafe extern "C" fn(*mut wl_listener, *mut c_void);

#[repr(C)]
pub struct wl_listener {
    pub link: wl_list,
    pub notify: Option<wl_notify_func_t>,
}

impl Default for wl_listener {
    fn default() -> Self {
        Self::new()
    }
}

impl wl_listener {
    pub const fn new() -> Self {
        Self {
            link: wl_list::empty(),
            notify: None,
        }
    }
}

#[repr(C)]
pub struct wl_signal {
    pub listener_list: wl_list,
}

pub const WL_EVENT_READABLE: u32 = 0x01;

pub type wl_event_loop_fd_func_t = unsafe extern "C" fn(c_int, u32, *mut c_void) -> c_int;

#[link(name = "wayland-server")]
extern "C" {
    pub fn wl_display_create() -> *mut wl_display;
    pub fn wl_display_destroy(display: *mut wl_display);
    pub fn wl_display_destroy_clients(display: *mut wl_display);
    pub fn wl_display_get_event_loop(display: *mut wl_display) -> *mut wl_event_loop;
    pub fn wl_display_add_socket_auto(display: *mut wl_display) -> *const c_char;
    pub fn wl_display_terminate(display: *mut wl_display);
    pub fn wl_display_flush_clients(display: *mut wl_display);
    pub fn wl_display_init_shm(display: *mut wl_display) -> c_int;

    pub fn wl_event_loop_dispatch(loop_: *mut wl_event_loop, timeout: c_int) -> c_int;
    pub fn wl_event_loop_dispatch_idle(loop_: *mut wl_event_loop);
    pub fn wl_event_loop_get_fd(loop_: *mut wl_event_loop) -> c_int;
    pub fn wl_event_loop_add_fd(
        loop_: *mut wl_event_loop,
        fd: c_int,
        mask: u32,
        func: wl_event_loop_fd_func_t,
        data: *mut c_void,
    ) -> *mut wl_event_source;
    pub fn wl_event_source_remove(source: *mut wl_event_source) -> c_int;

    pub fn wl_list_init(list: *mut wl_list);
    pub fn wl_list_insert(list: *mut wl_list, elm: *mut wl_list);
    pub fn wl_list_remove(elm: *mut wl_list);
}

pub unsafe fn signal_add(signal: *mut wl_signal, listener: *mut wl_listener) {
    wl_list_insert((*signal).listener_list.prev, &mut (*listener).link);
}

pub unsafe fn listener_detach(listener: *mut wl_listener) {
    if !(*listener).link.next.is_null() {
        wl_list_remove(&mut (*listener).link);
        wl_list_init(&mut (*listener).link);
    }
}

#[repr(C)]
pub struct timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

#[cfg(target_os = "openbsd")]
pub const CLOCK_MONOTONIC: c_int = 3;
#[cfg(not(target_os = "openbsd"))]
pub const CLOCK_MONOTONIC: c_int = 1;

extern "C" {
    pub fn clock_gettime(clk_id: c_int, tp: *mut timespec) -> c_int;
}

pub fn now() -> timespec {
    let mut ts = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        clock_gettime(CLOCK_MONOTONIC, std::ptr::addr_of_mut!(ts));
    }
    ts
}
