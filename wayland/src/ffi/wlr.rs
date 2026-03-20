use super::buffer::wlr_buffer;
use super::wl::{wl_display, wl_event_loop, wl_list, wl_listener, wl_resource, wl_signal};
use std::os::raw::{c_char, c_int, c_void};

pub const WLR_INPUT_DEVICE_KEYBOARD: c_int = 0;
pub const WLR_INPUT_DEVICE_POINTER: c_int = 1;

pub const WL_KEYBOARD_KEY_STATE_RELEASED: c_int = 0;
pub const WL_POINTER_BUTTON_STATE_RELEASED: c_int = 0;

pub const WLR_LED_COUNT: usize = 3;
pub const WLR_MODIFIER_COUNT: usize = 8;
pub const WLR_KEYBOARD_KEYS_CAP: usize = 32;

#[repr(C)]
pub struct wlr_backend {
    pub impl_: *const c_void,
    pub buffer_caps: u32,
    pub features: wlr_backend_features,
    pub events: wlr_backend_events,
}

#[repr(C)]
pub struct wlr_backend_features {
    pub timeline: bool,
}

#[repr(C)]
pub struct wlr_renderer {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wlr_allocator {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wlr_compositor {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wlr_surface {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wlr_output_layout {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wlr_seat {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wlr_cursor {
    pub state: *mut c_void,
    pub x: f64,
    pub y: f64,
    pub events: wlr_cursor_events,
}

#[repr(C)]
pub struct wlr_cursor_events {
    pub motion: wl_signal,
    pub motion_absolute: wl_signal,
    pub button: wl_signal,
    pub axis: wl_signal,
    pub frame: wl_signal,
    pub swipe_begin: wl_signal,
    pub swipe_update: wl_signal,
    pub swipe_end: wl_signal,
    pub pinch_begin: wl_signal,
    pub pinch_update: wl_signal,
    pub pinch_end: wl_signal,
    pub hold_begin: wl_signal,
    pub hold_end: wl_signal,
    pub touch_up: wl_signal,
    pub touch_down: wl_signal,
    pub touch_motion: wl_signal,
    pub touch_cancel: wl_signal,
    pub touch_frame: wl_signal,
    pub tablet_tool_axis: wl_signal,
    pub tablet_tool_proximity: wl_signal,
    pub tablet_tool_tip: wl_signal,
    pub tablet_tool_button: wl_signal,
}

#[repr(C)]
pub struct wlr_xcursor_manager {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct wlr_texture {
    _opaque: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct wlr_box {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
}

#[repr(C)]
pub struct wlr_output {
    pub impl_: *const c_void,
    pub backend: *mut wlr_backend,
    pub event_loop: *mut wl_event_loop,
    pub global: *mut c_void,
    pub resources: wl_list,
    pub name: *mut c_char,
    pub description: *mut c_char,
    pub make: *mut c_char,
    pub model: *mut c_char,
    pub serial: *mut c_char,
    pub phys_width: i32,
    pub phys_height: i32,
    pub modes: wl_list,
    pub current_mode: *mut c_void,
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
    pub enabled: bool,
    pub scale: f32,
    pub subpixel: c_int,
    pub transform: c_int,
    pub adaptive_sync_status: c_int,
    pub render_format: u32,
    pub adaptive_sync_supported: bool,
    pub needs_frame: bool,
    pub frame_pending: bool,
    pub non_desktop: bool,
    pub commit_seq: u32,
    pub events: wlr_output_events,
}

#[repr(C)]
pub struct wlr_output_events {
    pub frame: wl_signal,
    pub damage: wl_signal,
    pub needs_frame: wl_signal,
    pub precommit: wl_signal,
    pub commit: wl_signal,
    pub present: wl_signal,
    pub bind: wl_signal,
    pub description: wl_signal,
    pub request_state: wl_signal,
    pub destroy: wl_signal,
}

#[repr(C)]
pub struct wlr_backend_events {
    pub destroy: wl_signal,
    pub new_input: wl_signal,
    pub new_output: wl_signal,
}

#[repr(C)]
pub struct wlr_input_device {
    pub type_: c_int,
    pub name: *mut c_char,
    pub events: wlr_input_device_events,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct wlr_input_device_events {
    pub destroy: wl_signal,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct wlr_keyboard_modifiers {
    pub depressed: u32,
    pub latched: u32,
    pub locked: u32,
    pub group: u32,
}

#[repr(C)]
pub struct wlr_keyboard {
    pub base: wlr_input_device,
    pub impl_: *const c_void,
    pub group: *mut c_void,
    pub keymap_string: *mut c_char,
    pub keymap_size: usize,
    pub keymap_fd: c_int,
    pub keymap: *mut c_void,
    pub xkb_state: *mut c_void,
    pub led_indexes: [u32; WLR_LED_COUNT],
    pub mod_indexes: [u32; WLR_MODIFIER_COUNT],
    pub leds: u32,
    pub keycodes: [u32; WLR_KEYBOARD_KEYS_CAP],
    pub num_keycodes: usize,
    pub modifiers: wlr_keyboard_modifiers,
    pub repeat_info: wlr_keyboard_repeat_info,
    pub events: wlr_keyboard_events,
}

#[repr(C)]
pub struct wlr_keyboard_repeat_info {
    pub rate: i32,
    pub delay: i32,
}

#[repr(C)]
pub struct wlr_keyboard_events {
    pub key: wl_signal,
    pub modifiers: wl_signal,
    pub keymap: wl_signal,
    pub repeat_info: wl_signal,
}

#[repr(C)]
pub struct wlr_keyboard_key_event {
    pub time_msec: u32,
    pub keycode: u32,
    pub update_state: bool,
    pub state: c_int,
}

#[repr(C)]
pub struct wlr_pointer_motion_event {
    pub pointer: *mut c_void,
    pub time_msec: u32,
    pub delta_x: f64,
    pub delta_y: f64,
    pub unaccel_dx: f64,
    pub unaccel_dy: f64,
}

#[repr(C)]
pub struct wlr_pointer_motion_absolute_event {
    pub pointer: *mut c_void,
    pub time_msec: u32,
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
pub struct wlr_pointer_button_event {
    pub pointer: *mut c_void,
    pub time_msec: u32,
    pub button: u32,
    pub state: c_int,
}

#[repr(C)]
pub struct wlr_pointer_axis_event {
    pub pointer: *mut c_void,
    pub time_msec: u32,
    pub source: c_int,
    pub orientation: c_int,
    pub relative_direction: c_int,
    pub delta: f64,
    pub delta_discrete: i32,
}

#[repr(C)]
pub struct wlr_xdg_shell {
    pub global: *mut c_void,
    pub version: u32,
    pub clients: wl_list,
    pub popup_grabs: wl_list,
    pub ping_timeout: u32,
    pub events: wlr_xdg_shell_events,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct wlr_xdg_shell_events {
    pub new_surface: wl_signal,
    pub new_toplevel: wl_signal,
    pub new_popup: wl_signal,
    pub destroy: wl_signal,
}

#[repr(C)]
pub struct wlr_xdg_surface_state {
    pub committed: u32,
    pub geometry: wlr_box,
    pub configure_serial: u32,
}

#[repr(C)]
pub struct wlr_xdg_surface {
    pub client: *mut c_void,
    pub resource: *mut wl_resource,
    pub surface: *mut wlr_surface,
    pub link: wl_list,
    pub role: c_int,
    pub role_resource: *mut wl_resource,
    pub role_object: *mut c_void,
    pub popups: wl_list,
    pub configured: bool,
    pub configure_idle: *mut c_void,
    pub scheduled_serial: u32,
    pub configure_list: wl_list,
    pub current: wlr_xdg_surface_state,
    pub pending: wlr_xdg_surface_state,
    pub initialized: bool,
    pub initial_commit: bool,
    pub geometry: wlr_box,
    pub events: wlr_xdg_surface_events,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct wlr_xdg_surface_events {
    pub destroy: wl_signal,
    pub ping_timeout: wl_signal,
    pub new_popup: wl_signal,
    pub configure: wl_signal,
    pub ack_configure: wl_signal,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct wlr_xdg_toplevel_state {
    pub maximized: bool,
    pub fullscreen: bool,
    pub resizing: bool,
    pub activated: bool,
    pub suspended: bool,
    pub tiled: u32,
    pub width: i32,
    pub height: i32,
    pub max_width: i32,
    pub max_height: i32,
    pub min_width: i32,
    pub min_height: i32,
}

#[repr(C)]
pub struct wlr_xdg_toplevel_configure {
    pub fields: u32,
    pub maximized: bool,
    pub fullscreen: bool,
    pub resizing: bool,
    pub activated: bool,
    pub suspended: bool,
    pub tiled: u32,
    pub width: i32,
    pub height: i32,
    pub bounds_width: i32,
    pub bounds_height: i32,
    pub wm_capabilities: u32,
}

#[repr(C)]
pub struct wlr_xdg_toplevel_requested {
    pub maximized: bool,
    pub minimized: bool,
    pub fullscreen: bool,
    pub fullscreen_output: *mut wlr_output,
    pub fullscreen_output_destroy: wl_listener,
}

#[repr(C)]
pub struct wlr_xdg_toplevel {
    pub resource: *mut wl_resource,
    pub base: *mut wlr_xdg_surface,
    pub parent: *mut wlr_xdg_toplevel,
    pub current: wlr_xdg_toplevel_state,
    pub pending: wlr_xdg_toplevel_state,
    pub scheduled: wlr_xdg_toplevel_configure,
    pub requested: wlr_xdg_toplevel_requested,
    pub title: *mut c_char,
    pub app_id: *mut c_char,
    pub events: wlr_xdg_toplevel_events,
}

#[repr(C)]
pub struct wlr_xdg_toplevel_events {
    pub destroy: wl_signal,
    pub request_maximize: wl_signal,
    pub request_fullscreen: wl_signal,
    pub request_minimize: wl_signal,
    pub request_move: wl_signal,
    pub request_resize: wl_signal,
    pub request_show_window_menu: wl_signal,
    pub set_parent: wl_signal,
    pub set_title: wl_signal,
    pub set_app_id: wl_signal,
}

#[repr(C)]
pub struct wlr_scene_node {
    pub type_: c_int,
    pub parent: *mut wlr_scene_tree,
    pub link: wl_list,
    pub enabled: bool,
    pub x: c_int,
    pub y: c_int,
    pub events: wlr_scene_node_events,
    pub data: *mut c_void,
    pub addons: wl_list,
    pub visible: pixman_region32,
}

#[repr(C)]
pub struct wlr_scene_node_events {
    pub destroy: wl_signal,
}

#[repr(C)]
pub struct wlr_scene_tree {
    pub node: wlr_scene_node,
    pub children: wl_list,
}

#[repr(C)]
pub struct wlr_scene {
    pub tree: wlr_scene_tree,
    pub outputs: wl_list,
}

#[repr(C)]
pub struct wlr_scene_output {
    pub output: *mut wlr_output,
    pub link: wl_list,
    pub scene: *mut wlr_scene,
}

#[repr(C)]
pub struct wlr_scene_buffer {
    pub node: wlr_scene_node,
    pub buffer: *mut wlr_buffer,
}

#[repr(C)]
pub struct wlr_scene_surface {
    pub buffer: *mut wlr_scene_buffer,
    pub surface: *mut wlr_surface,
}

#[repr(C)]
pub struct wlr_scene_rect {
    pub node: wlr_scene_node,
    pub width: c_int,
    pub height: c_int,
    pub color: [f32; 4],
}

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_log_init(verbosity: c_int, callback: *mut c_void);

    pub fn wlr_backend_autocreate(
        loop_: *mut wl_event_loop,
        session: *mut *mut c_void,
    ) -> *mut wlr_backend;
    pub fn wlr_backend_start(backend: *mut wlr_backend) -> bool;
    pub fn wlr_backend_destroy(backend: *mut wlr_backend);

    pub fn wlr_renderer_autocreate(backend: *mut wlr_backend) -> *mut wlr_renderer;
    pub fn wlr_renderer_init_wl_display(
        renderer: *mut wlr_renderer,
        display: *mut wl_display,
    ) -> bool;
    pub fn wlr_renderer_destroy(renderer: *mut wlr_renderer);

    pub fn wlr_allocator_autocreate(
        backend: *mut wlr_backend,
        renderer: *mut wlr_renderer,
    ) -> *mut wlr_allocator;
    pub fn wlr_allocator_destroy(alloc: *mut wlr_allocator);

    pub fn wlr_compositor_create(
        display: *mut wl_display,
        version: u32,
        renderer: *mut wlr_renderer,
    ) -> *mut wlr_compositor;
    pub fn wlr_subcompositor_create(display: *mut wl_display) -> *mut c_void;
    pub fn wlr_data_device_manager_create(display: *mut wl_display) -> *mut c_void;

    pub fn wlr_output_layout_create(display: *mut wl_display) -> *mut wlr_output_layout;
    pub fn wlr_output_layout_add_auto(
        layout: *mut wlr_output_layout,
        output: *mut wlr_output,
    ) -> *mut c_void;

    pub fn wlr_output_init_render(
        output: *mut wlr_output,
        alloc: *mut wlr_allocator,
        renderer: *mut wlr_renderer,
    ) -> bool;

    pub fn wlr_scene_create() -> *mut wlr_scene;
    pub fn wlr_scene_attach_output_layout(
        scene: *mut wlr_scene,
        layout: *mut wlr_output_layout,
    ) -> *mut c_void;
    pub fn wlr_scene_output_create(
        scene: *mut wlr_scene,
        output: *mut wlr_output,
    ) -> *mut wlr_scene_output;
    pub fn wlr_scene_output_commit(
        scene_output: *mut wlr_scene_output,
        options: *const c_void,
    ) -> bool;
    pub fn wlr_scene_output_send_frame_done(
        scene_output: *mut wlr_scene_output,
        now: *const c_void,
    );
    pub fn wlr_scene_tree_create(parent: *mut wlr_scene_tree) -> *mut wlr_scene_tree;
    pub fn wlr_scene_subsurface_tree_create(
        parent: *mut wlr_scene_tree,
        surface: *mut wlr_surface,
    ) -> *mut wlr_scene_tree;
    pub fn wlr_scene_xdg_surface_create(
        parent: *mut wlr_scene_tree,
        xdg_surface: *mut wlr_xdg_surface,
    ) -> *mut wlr_scene_tree;
    pub fn wlr_scene_rect_create(
        parent: *mut wlr_scene_tree,
        width: c_int,
        height: c_int,
        color: *const f32,
    ) -> *mut wlr_scene_rect;
    pub fn wlr_scene_rect_set_size(rect: *mut wlr_scene_rect, width: c_int, height: c_int);
    pub fn wlr_scene_rect_set_color(rect: *mut wlr_scene_rect, color: *const f32);
    pub fn wlr_scene_buffer_create(
        parent: *mut wlr_scene_tree,
        buffer: *mut wlr_buffer,
    ) -> *mut wlr_scene_buffer;
    pub fn wlr_scene_buffer_set_buffer(
        scene_buffer: *mut wlr_scene_buffer,
        buffer: *mut wlr_buffer,
    );
    pub fn wlr_scene_node_set_position(node: *mut wlr_scene_node, x: c_int, y: c_int);
    pub fn wlr_scene_node_set_enabled(node: *mut wlr_scene_node, enabled: bool);
    pub fn wlr_scene_node_raise_to_top(node: *mut wlr_scene_node);
    pub fn wlr_scene_node_lower_to_bottom(node: *mut wlr_scene_node);
    pub fn wlr_scene_node_reparent(node: *mut wlr_scene_node, parent: *mut wlr_scene_tree);
    pub fn wlr_scene_node_destroy(node: *mut wlr_scene_node);

    pub fn wlr_xdg_shell_create(display: *mut wl_display, version: u32) -> *mut wlr_xdg_shell;
    pub fn wlr_xdg_toplevel_set_size(toplevel: *mut wlr_xdg_toplevel, width: i32, height: i32)
        -> u32;
    pub fn wlr_xdg_toplevel_set_activated(toplevel: *mut wlr_xdg_toplevel, activated: bool) -> u32;
    pub fn wlr_xdg_toplevel_send_close(toplevel: *mut wlr_xdg_toplevel);
    pub fn wlr_xdg_surface_schedule_configure(surface: *mut wlr_xdg_surface) -> u32;

    pub fn wlr_seat_create(display: *mut wl_display, name: *const c_char) -> *mut wlr_seat;
    pub fn wlr_seat_set_capabilities(seat: *mut wlr_seat, capabilities: u32);
    pub fn wlr_seat_set_keyboard(seat: *mut wlr_seat, keyboard: *mut wlr_keyboard);
    pub fn wlr_seat_keyboard_notify_key(
        seat: *mut wlr_seat,
        time_msec: u32,
        key: u32,
        state: u32,
    );
    pub fn wlr_seat_keyboard_notify_modifiers(
        seat: *mut wlr_seat,
        modifiers: *const wlr_keyboard_modifiers,
    );
    pub fn wlr_seat_keyboard_notify_enter(
        seat: *mut wlr_seat,
        surface: *mut wlr_surface,
        keycodes: *const u32,
        num_keycodes: usize,
        modifiers: *const wlr_keyboard_modifiers,
    );
    pub fn wlr_seat_keyboard_notify_clear_focus(seat: *mut wlr_seat);
    pub fn wlr_seat_pointer_notify_enter(
        seat: *mut wlr_seat,
        surface: *mut wlr_surface,
        sx: f64,
        sy: f64,
    );
    pub fn wlr_seat_pointer_notify_clear_focus(seat: *mut wlr_seat);
    pub fn wlr_seat_pointer_notify_motion(seat: *mut wlr_seat, time_msec: u32, sx: f64, sy: f64);
    pub fn wlr_seat_pointer_notify_button(
        seat: *mut wlr_seat,
        time_msec: u32,
        button: u32,
        state: u32,
    ) -> u32;
    pub fn wlr_seat_pointer_notify_axis(
        seat: *mut wlr_seat,
        time_msec: u32,
        orientation: c_int,
        value: f64,
        value_discrete: i32,
        source: c_int,
        relative_direction: c_int,
    );
    pub fn wlr_seat_pointer_notify_frame(seat: *mut wlr_seat);

    pub fn wlr_cursor_create() -> *mut wlr_cursor;
    pub fn wlr_cursor_attach_output_layout(cursor: *mut wlr_cursor, layout: *mut wlr_output_layout);
    pub fn wlr_cursor_attach_input_device(cursor: *mut wlr_cursor, dev: *mut wlr_input_device);
    pub fn wlr_cursor_move(
        cursor: *mut wlr_cursor,
        dev: *mut wlr_input_device,
        delta_x: f64,
        delta_y: f64,
    );
    pub fn wlr_cursor_warp_absolute(
        cursor: *mut wlr_cursor,
        dev: *mut wlr_input_device,
        x: f64,
        y: f64,
    );
    pub fn wlr_cursor_set_xcursor(
        cursor: *mut wlr_cursor,
        manager: *mut wlr_xcursor_manager,
        name: *const c_char,
    );
    pub fn wlr_cursor_destroy(cursor: *mut wlr_cursor);

    pub fn wlr_xcursor_manager_create(name: *const c_char, size: u32)
        -> *mut wlr_xcursor_manager;
    pub fn wlr_xcursor_manager_load(manager: *mut wlr_xcursor_manager, scale: f32) -> bool;
    pub fn wlr_xcursor_manager_destroy(manager: *mut wlr_xcursor_manager);

    pub fn wlr_keyboard_set_keymap(keyboard: *mut wlr_keyboard, keymap: *mut c_void) -> bool;
    pub fn wlr_keyboard_set_repeat_info(keyboard: *mut wlr_keyboard, rate: i32, delay: i32);

    pub fn wlr_surface_get_texture(surface: *mut wlr_surface) -> *mut wlr_texture;
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct pixman_box32 {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct pixman_region32 {
    pub extents: pixman_box32,
    pub data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct wl_array {
    pub size: usize,
    pub alloc: usize,
    pub data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct wlr_fbox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
pub struct wlr_surface_viewport {
    pub has_src: bool,
    pub has_dst: bool,
    pub src: wlr_fbox,
    pub dst_width: c_int,
    pub dst_height: c_int,
}

#[repr(C)]
pub struct wlr_surface_state {
    pub committed: u32,
    pub seq: u32,
    pub buffer: *mut c_void,
    pub dx: i32,
    pub dy: i32,
    pub surface_damage: pixman_region32,
    pub buffer_damage: pixman_region32,
    pub opaque: pixman_region32,
    pub input: pixman_region32,
    pub transform: c_int,
    pub scale: i32,
    pub frame_callback_list: wl_list,
    pub width: c_int,
    pub height: c_int,
    pub buffer_width: c_int,
    pub buffer_height: c_int,
    pub subsurfaces_below: wl_list,
    pub subsurfaces_above: wl_list,
    pub viewport: wlr_surface_viewport,
    pub cached_state_locks: usize,
    pub cached_state_link: wl_list,
    pub synced: wl_array,
}

#[repr(C)]
pub struct wlr_surface_events {
    pub client_commit: wl_signal,
    pub commit: wl_signal,
    pub map: wl_signal,
    pub unmap: wl_signal,
    pub new_subsurface: wl_signal,
    pub destroy: wl_signal,
}

#[repr(C)]
pub struct wlr_surface_full {
    pub resource: *mut wl_resource,
    pub compositor: *mut wlr_compositor,
    pub buffer: *mut c_void,
    pub buffer_damage: pixman_region32,
    pub opaque_region: pixman_region32,
    pub input_region: pixman_region32,
    pub current: wlr_surface_state,
    pub pending: wlr_surface_state,
    pub cached: wl_list,
    pub mapped: bool,
    pub role: *const c_void,
    pub role_resource: *mut wl_resource,
    pub events: wlr_surface_events,
}

pub const WLR_XDG_SURFACE_ROLE_TOPLEVEL: c_int = 1;

pub unsafe fn surface_events(surface: *mut wlr_surface) -> *mut wlr_surface_events {
    std::ptr::addr_of_mut!((*surface.cast::<wlr_surface_full>()).events)
}

pub unsafe fn surface_mapped(surface: *mut wlr_surface) -> bool {
    (*surface.cast::<wlr_surface_full>()).mapped
}

pub unsafe fn surface_size(surface: *mut wlr_surface) -> (i32, i32) {
    let s = &*surface.cast::<wlr_surface_full>();
    (s.current.width, s.current.height)
}

pub const WL_SEAT_CAPABILITY_POINTER: u32 = 1;
pub const WL_SEAT_CAPABILITY_KEYBOARD: u32 = 2;

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_scene_node_at(
        node: *mut wlr_scene_node,
        lx: f64,
        ly: f64,
        nx: *mut f64,
        ny: *mut f64,
    ) -> *mut wlr_scene_node;
    pub fn wlr_scene_buffer_from_node(node: *mut wlr_scene_node) -> *mut wlr_scene_buffer;
    pub fn wlr_scene_surface_try_from_buffer(
        buffer: *mut wlr_scene_buffer,
    ) -> *mut wlr_scene_surface;
    pub fn wlr_xdg_toplevel_try_from_wlr_surface(
        surface: *mut wlr_surface,
    ) -> *mut wlr_xdg_toplevel;
    pub fn wlr_keyboard_from_input_device(dev: *mut wlr_input_device) -> *mut wlr_keyboard;
    pub fn wlr_output_layout_output_coords(
        layout: *mut wlr_output_layout,
        output: *mut wlr_output,
        x: *mut f64,
        y: *mut f64,
    );
}

#[repr(C)]
pub struct wlr_output_custom_mode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

#[repr(C)]
pub struct wlr_output_state {
    pub committed: u32,
    pub allow_reconfiguration: bool,
    pub damage: pixman_region32,
    pub enabled: bool,
    pub scale: f32,
    pub transform: c_int,
    pub adaptive_sync_enabled: bool,
    pub render_format: u32,
    pub subpixel: c_int,
    pub buffer: *mut wlr_buffer,
    pub buffer_src_box: wlr_fbox,
    pub buffer_dst_box: wlr_box,
    pub tearing_page_flip: bool,
    pub mode_type: c_int,
    pub mode: *mut c_void,
    pub custom_mode: wlr_output_custom_mode,
    pub gamma_lut: *mut u16,
    pub gamma_lut_size: usize,
    pub layers: *mut c_void,
    pub layers_len: usize,
    pub wait_timeline: *mut c_void,
    pub wait_point: u64,
    pub signal_timeline: *mut c_void,
    pub signal_point: u64,
}

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_output_state_init(state: *mut wlr_output_state);
    pub fn wlr_output_state_finish(state: *mut wlr_output_state);
    pub fn wlr_output_state_set_enabled(state: *mut wlr_output_state, enabled: bool);
    pub fn wlr_output_state_set_mode(state: *mut wlr_output_state, mode: *mut c_void);
    pub fn wlr_output_preferred_mode(output: *mut wlr_output) -> *mut c_void;
    pub fn wlr_output_commit_state(output: *mut wlr_output, state: *const wlr_output_state)
        -> bool;
}

pub const WLR_XDG_TOPLEVEL_DECORATION_V1_MODE_CLIENT_SIDE: u32 = 1;
pub const WLR_XDG_TOPLEVEL_DECORATION_V1_MODE_SERVER_SIDE: u32 = 2;

#[repr(C)]
pub struct wlr_xdg_decoration_manager_v1 {
    pub global: *mut c_void,
    pub decorations: wl_list,
    pub events: wlr_xdg_decoration_manager_v1_events,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct wlr_xdg_decoration_manager_v1_events {
    pub new_toplevel_decoration: wl_signal,
    pub destroy: wl_signal,
}

#[repr(C)]
pub struct wlr_xdg_toplevel_decoration_v1_state {
    pub mode: c_int,
}

#[repr(C)]
pub struct wlr_xdg_toplevel_decoration_v1 {
    pub resource: *mut wl_resource,
    pub toplevel: *mut wlr_xdg_toplevel,
    pub manager: *mut wlr_xdg_decoration_manager_v1,
    pub link: wl_list,
    pub current: wlr_xdg_toplevel_decoration_v1_state,
    pub pending: wlr_xdg_toplevel_decoration_v1_state,
    pub scheduled_mode: c_int,
    pub requested_mode: c_int,
    pub configure_list: wl_list,
    pub events: wlr_xdg_toplevel_decoration_v1_events,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct wlr_xdg_toplevel_decoration_v1_events {
    pub destroy: wl_signal,
    pub request_mode: wl_signal,
}

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_xdg_decoration_manager_v1_create(
        display: *mut wl_display,
    ) -> *mut wlr_xdg_decoration_manager_v1;
    pub fn wlr_xdg_toplevel_decoration_v1_set_mode(
        decoration: *mut wlr_xdg_toplevel_decoration_v1,
        mode: u32,
    ) -> u32;
}

pub const WLR_SERVER_DECORATION_MANAGER_MODE_SERVER: u32 = 2;

#[repr(C)]
pub struct wlr_server_decoration_manager {
    pub global: *mut c_void,
    pub resources: wl_list,
    pub decorations: wl_list,
    pub default_mode: u32,
    pub events: wlr_server_decoration_manager_events,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct wlr_server_decoration_manager_events {
    pub new_decoration: wl_signal,
    pub destroy: wl_signal,
}

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_server_decoration_manager_create(
        display: *mut wl_display,
    ) -> *mut wlr_server_decoration_manager;
    pub fn wlr_server_decoration_manager_set_default_mode(
        manager: *mut wlr_server_decoration_manager,
        default_mode: u32,
    );
}

pub const WLR_SERVER_DECORATION_MANAGER_MODE_CLIENT: u32 = 1;

#[repr(C)]
pub struct wlr_server_decoration {
    pub resource: *mut wl_resource,
    pub surface: *mut wlr_surface,
    pub link: wl_list,
    pub mode: u32,
    pub events: wlr_server_decoration_events,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct wlr_server_decoration_events {
    pub destroy: wl_signal,
    pub mode: wl_signal,
}

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_primary_selection_v1_device_manager_create(
        display: *mut wl_display,
    ) -> *mut c_void;
    pub fn wlr_xdg_output_manager_v1_create(
        display: *mut wl_display,
        layout: *mut wlr_output_layout,
    ) -> *mut c_void;
    pub fn wlr_screencopy_manager_v1_create(display: *mut wl_display) -> *mut c_void;
    pub fn wlr_viewporter_create(display: *mut wl_display) -> *mut c_void;
    pub fn wlr_presentation_create(
        display: *mut wl_display,
        backend: *mut wlr_backend,
        version: u32,
    ) -> *mut c_void;
    pub fn wlr_fractional_scale_manager_v1_create(
        display: *mut wl_display,
        version: u32,
    ) -> *mut c_void;
    pub fn wlr_single_pixel_buffer_manager_v1_create(
        display: *mut wl_display,
    ) -> *mut c_void;
    pub fn wlr_gamma_control_manager_v1_create(display: *mut wl_display) -> *mut c_void;
    pub fn wlr_idle_notifier_v1_create(display: *mut wl_display) -> *mut c_void;
}
