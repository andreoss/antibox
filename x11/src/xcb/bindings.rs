
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int, c_uint, c_void};

pub type xcb_window_t = u32;
pub type xcb_drawable_t = u32;
pub type xcb_fontable_t = u32;
pub type xcb_pixmap_t = u32;
pub type xcb_cursor_t = u32;
pub type xcb_gcontext_t = u32;
pub type xcb_colormap_t = u32;
pub type xcb_atom_t = u32;
pub type xcb_visualid_t = u32;
pub type xcb_keycode_t = u8;
pub type xcb_button_t = u8;
pub type xcb_timestamp_t = u32;
pub type xcb_bool32_t = u32;

pub type xcb_connection_t = c_void;

pub const XCB_EVENT_MASK_NO_EVENT: u32 = 0;
pub const XCB_EVENT_MASK_KEY_PRESS: u32 = 1 << 0;
pub const XCB_EVENT_MASK_KEY_RELEASE: u32 = 1 << 1;
pub const XCB_EVENT_MASK_BUTTON_PRESS: u32 = 1 << 2;
pub const XCB_EVENT_MASK_BUTTON_RELEASE: u32 = 1 << 3;
pub const XCB_EVENT_MASK_ENTER_WINDOW: u32 = 1 << 4;
pub const XCB_EVENT_MASK_LEAVE_WINDOW: u32 = 1 << 5;
pub const XCB_EVENT_MASK_POINTER_MOTION: u32 = 1 << 6;
pub const XCB_EVENT_MASK_EXPOSURE: u32 = 1 << 15;
pub const XCB_EVENT_MASK_STRUCTURE_NOTIFY: u32 = 1 << 17;
pub const XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY: u32 = 1 << 19;
pub const XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT: u32 = 1 << 20;
pub const XCB_EVENT_MASK_PROPERTY_CHANGE: u32 = 1 << 22;
pub const XCB_EVENT_MASK_FOCUS_CHANGE: u32 = 1 << 21;
pub const XCB_EVENT_MASK_COLOR_MAP_CHANGE: u32 = 1 << 13;

pub const XCB_CW_BACK_PIXMAP: u32 = 1 << 0;
pub const XCB_CW_BACK_PIXEL: u32 = 1 << 1;
pub const XCB_CW_BORDER_PIXMAP: u32 = 1 << 2;
pub const XCB_CW_BORDER_PIXEL: u32 = 1 << 3;
pub const XCB_CW_BIT_GRAVITY: u32 = 1 << 4;
pub const XCB_CW_WIN_GRAVITY: u32 = 1 << 5;
pub const XCB_CW_BACKING_STORE: u32 = 1 << 6;
pub const XCB_CW_BACKING_PLANES: u32 = 1 << 7;
pub const XCB_CW_BACKING_PIXEL: u32 = 1 << 8;
pub const XCB_CW_OVERRIDE_REDIRECT: u32 = 1 << 9;
pub const XCB_CW_SAVE_UNDER: u32 = 1 << 10;
pub const XCB_CW_EVENT_MASK: u32 = 1 << 11;
pub const XCB_CW_DO_NOT_PROPAGATE_MASK: u32 = 1 << 12;
pub const XCB_CW_COLORMAP: u32 = 1 << 13;
pub const XCB_CW_CURSOR: u32 = 1 << 14;

pub const XCB_CONFIG_WINDOW_X: u16 = 1 << 0;
pub const XCB_CONFIG_WINDOW_Y: u16 = 1 << 1;
pub const XCB_CONFIG_WINDOW_WIDTH: u16 = 1 << 2;
pub const XCB_CONFIG_WINDOW_HEIGHT: u16 = 1 << 3;
pub const XCB_CONFIG_WINDOW_BORDER_WIDTH: u16 = 1 << 4;
pub const XCB_CONFIG_WINDOW_SIBLING: u16 = 1 << 5;
pub const XCB_CONFIG_WINDOW_STACK_MODE: u16 = 1 << 6;

pub const XCB_STACK_MODE_ABOVE: u32 = 0;
pub const XCB_STACK_MODE_BELOW: u32 = 1;
pub const XCB_STACK_MODE_TOP_IF: u32 = 2;
pub const XCB_STACK_MODE_BOTTOM_IF: u32 = 3;
pub const XCB_STACK_MODE_OPPOSITE: u32 = 4;

pub const XCB_GC_FUNCTION: u32 = 1 << 0;
pub const XCB_GC_PLANE_MASK: u32 = 1 << 1;
pub const XCB_GC_FOREGROUND: u32 = 1 << 2;
pub const XCB_GC_BACKGROUND: u32 = 1 << 3;
pub const XCB_GC_LINE_WIDTH: u32 = 1 << 4;
pub const XCB_GC_LINE_STYLE: u32 = 1 << 5;
pub const XCB_GC_CAP_STYLE: u32 = 1 << 6;
pub const XCB_GC_JOIN_STYLE: u32 = 1 << 7;
pub const XCB_GC_FILL_STYLE: u32 = 1 << 8;
pub const XCB_GC_FILL_RULE: u32 = 1 << 9;
pub const XCB_GC_TILE: u32 = 1 << 10;
pub const XCB_GC_STIPPLE: u32 = 1 << 11;
pub const XCB_GC_TILE_STIPPLE_ORIGIN_X: u32 = 1 << 12;
pub const XCB_GC_TILE_STIPPLE_ORIGIN_Y: u32 = 1 << 13;
pub const XCB_GC_FONT: u32 = 1 << 14;
pub const XCB_GC_SUBWINDOW_MODE: u32 = 1 << 15;
pub const XCB_GC_GRAPHICS_EXPOSURES: u32 = 1 << 16;
pub const XCB_GC_CLIP_ORIGIN_X: u32 = 1 << 17;
pub const XCB_GC_CLIP_ORIGIN_Y: u32 = 1 << 18;
pub const XCB_GC_CLIP_MASK: u32 = 1 << 19;
pub const XCB_GC_DASH_OFFSET: u32 = 1 << 20;
pub const XCB_GC_DASH_LIST: u32 = 1 << 21;
pub const XCB_GC_ARC_MODE: u32 = 1 << 22;

pub const XCB_PROP_MODE_REPLACE: u8 = 0;
pub const XCB_PROP_MODE_PREPEND: u8 = 1;
pub const XCB_PROP_MODE_APPEND: u8 = 2;

pub const XCB_GRAB_MODE_SYNC: u8 = 0;
pub const XCB_GRAB_MODE_ASYNC: u8 = 1;

pub const XCB_SET_MODE_INSERT: u8 = 0;
pub const XCB_SET_MODE_DELETE: u8 = 1;

pub const XCB_INPUT_FOCUS_PARENT: u32 = 1;
pub const XCB_INPUT_FOCUS_POINTER_ROOT: u32 = 2;

pub const XCB_ALLOW_SYNC_POINTER: u8 = 0;
pub const XCB_ALLOW_SYNC_KEYBOARD: u8 = 1;

pub const XCB_WINDOW_CLASS_COPY_FROM_PARENT: u16 = 0;
pub const XCB_WINDOW_CLASS_INPUT_OUTPUT: u16 = 1;
pub const XCB_WINDOW_CLASS_INPUT_ONLY: u16 = 2;

pub const XCB_CURSOR_NONE: u32 = 0;

pub const XCB_IMAGE_FORMAT_XY_BITMAP: u8 = 0;
pub const XCB_IMAGE_FORMAT_XY_PIXMAP: u8 = 1;
pub const XCB_IMAGE_FORMAT_Z_PIXMAP: u8 = 2;

pub const XCB_FONT_DRAW_LEFT_TO_RIGHT: u32 = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_setup_t {
    pub status: u8,
    pub protocol_major_version: u8,
    pub protocol_minor_version: u16,
    pub length: u16,
    pub release_number: u32,
    pub resource_id_base: u32,
    pub resource_id_mask: u32,
    pub motion_buffer_size: u32,
    pub vendor_len: u16,
    pub maximum_request_length: u16,
    pub roots_len: u8,
    pub pixmap_formats_len: u8,
    pub image_byte_order: u8,
    pub bitmap_format_bit_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: xcb_keycode_t,
    pub max_keycode: xcb_keycode_t,
    pub pad2: [u8; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_screen_t {
    pub root: xcb_window_t,
    pub default_colormap: xcb_colormap_t,
    pub white_pixel: u32,
    pub black_pixel: u32,
    pub current_input_masks: u32,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_millimeters: u16,
    pub height_in_millimeters: u16,
    pub min_installed_maps: u16,
    pub max_installed_maps: u16,
    pub root_visual: xcb_visualid_t,
    pub backing_stores: u8,
    pub save_unders: u8,
    pub root_depth: u8,
    pub allowed_depths_len: u8,
}

#[repr(C)]
pub struct xcb_screen_iterator_t {
    pub data: *mut xcb_screen_t,
    pub rem: c_int,
    pub index: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_generic_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub pad: [u32; 7],
    pub full_sequence: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_map_request_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub parent: xcb_window_t,
    pub window: xcb_window_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_configure_request_event_t {
    pub response_type: u8,
    pub stack_mode: u8,
    pub sequence: u16,
    pub parent: xcb_window_t,
    pub window: xcb_window_t,
    pub sibling: xcb_window_t,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub value_mask: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_configure_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub event: xcb_window_t,
    pub window: xcb_window_t,
    pub above_sibling: xcb_window_t,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: u8,
    pub pad1: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_destroy_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub event: xcb_window_t,
    pub window: xcb_window_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_unmap_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub event: xcb_window_t,
    pub window: xcb_window_t,
    pub from_configure: u8,
    pub pad1: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_map_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub event: xcb_window_t,
    pub window: xcb_window_t,
    pub override_redirect: u8,
    pub pad1: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_reparent_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub event: xcb_window_t,
    pub window: xcb_window_t,
    pub parent: xcb_window_t,
    pub x: i16,
    pub y: i16,
    pub override_redirect: u8,
    pub pad1: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_expose_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub window: xcb_window_t,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub count: u16,
    pub pad1: [u8; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_property_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub window: xcb_window_t,
    pub atom: xcb_atom_t,
    pub time: xcb_timestamp_t,
    pub state: u8,
    pub pad1: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_key_press_event_t {
    pub response_type: u8,
    pub detail: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub root: xcb_window_t,
    pub event: xcb_window_t,
    pub child: xcb_window_t,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16,
    pub same_screen: u8,
    pub pad0: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_button_press_event_t {
    pub response_type: u8,
    pub detail: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub root: xcb_window_t,
    pub event: xcb_window_t,
    pub child: xcb_window_t,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16,
    pub same_screen: u8,
    pub pad0: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_motion_notify_event_t {
    pub response_type: u8,
    pub detail: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub root: xcb_window_t,
    pub event: xcb_window_t,
    pub child: xcb_window_t,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16,
    pub same_screen: u8,
    pub pad0: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_enter_notify_event_t {
    pub response_type: u8,
    pub detail: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub root: xcb_window_t,
    pub event: xcb_window_t,
    pub child: xcb_window_t,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16,
    pub mode: u8,
    pub same_screen_focus: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_leave_notify_event_t {
    pub response_type: u8,
    pub detail: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub root: xcb_window_t,
    pub event: xcb_window_t,
    pub child: xcb_window_t,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16,
    pub mode: u8,
    pub same_screen_focus: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_focus_in_event_t {
    pub response_type: u8,
    pub detail: u8,
    pub sequence: u16,
    pub event: xcb_window_t,
    pub mode: u8,
    pub pad0: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_focus_out_event_t {
    pub response_type: u8,
    pub detail: u8,
    pub sequence: u16,
    pub event: xcb_window_t,
    pub mode: u8,
    pub pad0: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_client_message_event_t {
    pub response_type: u8,
    pub format: u8,
    pub sequence: u16,
    pub window: xcb_window_t,
    pub type_: xcb_atom_t,
    pub data: [u32; 5],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_mapping_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub request: u8,
    pub first_keycode: xcb_keycode_t,
    pub count: u8,
    pub pad1: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_selection_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub requestor: xcb_window_t,
    pub selection: xcb_atom_t,
    pub target: xcb_atom_t,
    pub property: xcb_atom_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_selection_request_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub owner: xcb_window_t,
    pub requestor: xcb_window_t,
    pub selection: xcb_atom_t,
    pub target: xcb_atom_t,
    pub property: xcb_atom_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_selection_clear_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub owner: xcb_window_t,
    pub selection: xcb_atom_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_create_notify_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub parent: xcb_window_t,
    pub window: xcb_window_t,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: u8,
    pub pad1: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_window_attributes_reply_t {
    pub response_type: u8,
    pub backing_store: u8,
    pub sequence: u16,
    pub length: u32,
    pub visual: xcb_visualid_t,
    pub class: u16,
    pub bit_gravity: u8,
    pub win_gravity: u8,
    pub backing_planes: u32,
    pub backing_pixel: u32,
    pub save_under: u8,
    pub map_is_installed: u8,
    pub map_state: u8,
    pub override_redirect: u8,
    pub colormap: xcb_colormap_t,
    pub all_event_masks: u32,
    pub your_event_mask: u32,
    pub do_not_propagate_mask: u16,
    pub pad0: [u8; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_query_tree_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub root: xcb_window_t,
    pub parent: xcb_window_t,
    pub children_len: u16,
    pub pad1: [u8; 14],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_geometry_reply_t {
    pub response_type: u8,
    pub depth: u8,
    pub sequence: u16,
    pub length: u32,
    pub root: xcb_window_t,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub pad0: [u8; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_property_reply_t {
    pub response_type: u8,
    pub format: u8,
    pub sequence: u16,
    pub length: u32,
    pub type_: xcb_atom_t,
    pub bytes_after: u32,
    pub num_items: u32,
    pub pad0: [u8; 12],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_query_font_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub min_bounds: xcb_charinfo_t,
    pub pad1: [u8; 4],
    pub max_bounds: xcb_charinfo_t,
    pub pad2: [u8; 4],
    pub min_char_or_byte2: u16,
    pub max_char_or_byte2: u16,
    pub default_char: u16,
    pub properties_len: u16,
    pub draw_direction: u8,
    pub min_byte1: u8,
    pub max_byte1: u8,
    pub all_chars_exist: u8,
    pub font_ascent: i16,
    pub font_descent: i16,
    pub char_infos_len: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_charinfo_t {
    pub left_side_bearing: i16,
    pub right_side_bearing: i16,
    pub character_width: i16,
    pub ascent: i16,
    pub descent: i16,
    pub attributes: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_keyboard_mapping_reply_t {
    pub response_type: u8,
    pub keysyms_per_keycode: u8,
    pub sequence: u16,
    pub length: u32,
    pub pad0: [u8; 24],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_modifier_mapping_reply_t {
    pub response_type: u8,
    pub keycodes_per_modifier: u8,
    pub sequence: u16,
    pub length: u32,
    pub pad0: [u8; 24],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_query_pointer_reply_t {
    pub response_type: u8,
    pub same_screen: u8,
    pub sequence: u16,
    pub length: u32,
    pub root: xcb_window_t,
    pub child: xcb_window_t,
    pub root_x: i16,
    pub root_y: i16,
    pub win_x: i16,
    pub win_y: i16,
    pub mask: u16,
    pub pad0: [u8; 2],
}

#[link(name = "xcb")]
extern "C" {
    pub fn xcb_connect(displayname: *const c_char, screenp: *mut c_int) -> *mut xcb_connection_t;
    pub fn xcb_disconnect(c: *mut xcb_connection_t);
    pub fn xcb_connection_has_error(c: *mut xcb_connection_t) -> c_int;
    pub fn xcb_flush(c: *mut xcb_connection_t) -> c_int;
    pub fn xcb_get_setup(c: *mut xcb_connection_t) -> *const xcb_setup_t;
    pub fn xcb_generate_id(c: *mut xcb_connection_t) -> u32;
    pub fn xcb_setup_roots_iterator(setup: *const xcb_setup_t) -> xcb_screen_iterator_t;
    pub fn xcb_screen_next(i: *mut xcb_screen_iterator_t);
    pub fn xcb_get_file_descriptor(c: *mut xcb_connection_t) -> c_int;
    pub fn xcb_poll_for_event(c: *mut xcb_connection_t) -> *mut xcb_generic_event_t;

    pub fn xcb_create_window(
        c: *mut xcb_connection_t,
        depth: u8,
        wid: xcb_window_t,
        parent: xcb_window_t,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        class: u16,
        visual: xcb_visualid_t,
        value_mask: u32,
        value_list: *const u32,
    ) -> c_uint;
    pub fn xcb_destroy_window(c: *mut xcb_connection_t, window: xcb_window_t) -> c_uint;
    pub fn xcb_map_window(c: *mut xcb_connection_t, window: xcb_window_t) -> c_uint;
    pub fn xcb_unmap_window(c: *mut xcb_connection_t, window: xcb_window_t) -> c_uint;
    pub fn xcb_configure_window(
        c: *mut xcb_connection_t,
        window: xcb_window_t,
        value_mask: u16,
        value_list: *const u32,
    ) -> c_uint;
    pub fn xcb_change_window_attributes(
        c: *mut xcb_connection_t,
        window: xcb_window_t,
        value_mask: u32,
        value_list: *const u32,
    ) -> c_uint;
    pub fn xcb_reparent_window(
        c: *mut xcb_connection_t,
        window: xcb_window_t,
        parent: xcb_window_t,
        x: i16,
        y: i16,
    ) -> c_uint;
    pub fn xcb_clear_area(
        c: *mut xcb_connection_t,
        exposures: u8,
        window: xcb_window_t,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> c_uint;
    pub fn xcb_set_input_focus(
        c: *mut xcb_connection_t,
        revert_to: u8,
        focus: xcb_window_t,
        time: xcb_timestamp_t,
    ) -> c_uint;
    pub fn xcb_kill_client(c: *mut xcb_connection_t, resource: u32) -> c_uint;
    pub fn xcb_grab_key(
        c: *mut xcb_connection_t,
        owner_events: u8,
        grab_window: xcb_window_t,
        modifiers: u16,
        key: xcb_keycode_t,
        pointer_mode: u8,
        keyboard_mode: u8,
    ) -> c_uint;
    pub fn xcb_ungrab_key(
        c: *mut xcb_connection_t,
        key: xcb_keycode_t,
        grab_window: xcb_window_t,
        modifiers: u16,
    ) -> c_uint;
    pub fn xcb_grab_keyboard(
        c: *mut xcb_connection_t,
        owner_events: u8,
        grab_window: xcb_window_t,
        time: xcb_timestamp_t,
        pointer_mode: u8,
        keyboard_mode: u8,
    ) -> c_uint;
    pub fn xcb_ungrab_keyboard(c: *mut xcb_connection_t, time: xcb_timestamp_t) -> c_uint;
    pub fn xcb_grab_pointer(
        c: *mut xcb_connection_t,
        owner_events: u8,
        grab_window: xcb_window_t,
        event_mask: u16,
        pointer_mode: u8,
        keyboard_mode: u8,
        confine_to: xcb_window_t,
        cursor: xcb_cursor_t,
        time: xcb_timestamp_t,
    ) -> c_uint;
    pub fn xcb_ungrab_pointer(c: *mut xcb_connection_t, time: xcb_timestamp_t) -> c_uint;
    pub fn xcb_grab_button(
        c: *mut xcb_connection_t,
        owner_events: u8,
        grab_window: xcb_window_t,
        event_mask: u16,
        pointer_mode: u8,
        keyboard_mode: u8,
        confine_to: xcb_window_t,
        cursor: xcb_cursor_t,
        button: u8,
        modifiers: u16,
    ) -> c_uint;
    pub fn xcb_ungrab_button(
        c: *mut xcb_connection_t,
        button: u8,
        grab_window: xcb_window_t,
        modifiers: u16,
    ) -> c_uint;
    pub fn xcb_allow_events(c: *mut xcb_connection_t, mode: u8, time: xcb_timestamp_t) -> c_uint;
    pub fn xcb_warp_pointer(
        c: *mut xcb_connection_t,
        src_window: xcb_window_t,
        dst_window: xcb_window_t,
        src_x: i16,
        src_y: i16,
        src_width: u16,
        src_height: u16,
        dst_x: i16,
        dst_y: i16,
    ) -> c_uint;
    pub fn xcb_change_save_set(
        c: *mut xcb_connection_t,
        mode: u8,
        window: xcb_window_t,
    ) -> c_uint;
    pub fn xcb_set_selection_owner(
        c: *mut xcb_connection_t,
        owner: xcb_window_t,
        selection: xcb_atom_t,
        time: xcb_timestamp_t,
    ) -> c_uint;
    pub fn xcb_get_selection_owner(c: *mut xcb_connection_t, selection: xcb_atom_t) -> c_uint;
    pub fn xcb_get_selection_owner_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_selection_owner_reply_t;
    pub fn xcb_intern_atom(
        c: *mut xcb_connection_t,
        only_if_exists: u8,
        name_len: u16,
        name: *const c_char,
    ) -> c_uint;
    pub fn xcb_intern_atom_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_intern_atom_reply_t;
    pub fn xcb_query_extension(
        c: *mut xcb_connection_t,
        name_len: u16,
        name: *const c_char,
    ) -> c_uint;
    pub fn xcb_query_extension_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_query_extension_reply_t;
    pub fn xcb_get_atom_name(c: *mut xcb_connection_t, atom: xcb_atom_t) -> c_uint;
    pub fn xcb_get_atom_name_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_atom_name_reply_t;

    pub fn xcb_change_property(
        c: *mut xcb_connection_t,
        mode: u8,
        window: xcb_window_t,
        property: xcb_atom_t,
        type_: xcb_atom_t,
        format: u8,
        data_len: u32,
        data: *const c_void,
    ) -> c_uint;
    pub fn xcb_delete_property(
        c: *mut xcb_connection_t,
        window: xcb_window_t,
        property: xcb_atom_t,
    ) -> c_uint;
    pub fn xcb_get_property(
        c: *mut xcb_connection_t,
        delete: u8,
        window: xcb_window_t,
        property: xcb_atom_t,
        type_: xcb_atom_t,
        long_offset: u32,
        long_length: u32,
    ) -> c_uint;
    pub fn xcb_get_property_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_property_reply_t;

    pub fn xcb_create_gc(
        c: *mut xcb_connection_t,
        cid: xcb_gcontext_t,
        drawable: xcb_drawable_t,
        value_mask: u32,
        value_list: *const u32,
    ) -> c_uint;
    pub fn xcb_change_gc(
        c: *mut xcb_connection_t,
        gc: xcb_gcontext_t,
        value_mask: u32,
        value_list: *const u32,
    ) -> c_uint;
    pub fn xcb_free_gc(c: *mut xcb_connection_t, gc: xcb_gcontext_t) -> c_uint;
    pub fn xcb_poly_fill_rectangle(
        c: *mut xcb_connection_t,
        drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        rects_len: u32,
        rects: *const xcb_rectangle_t,
    ) -> c_uint;
    pub fn xcb_poly_rectangle(
        c: *mut xcb_connection_t,
        drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        rects_len: u32,
        rects: *const xcb_rectangle_t,
    ) -> c_uint;
    pub fn xcb_poly_line(
        c: *mut xcb_connection_t,
        coordinate_mode: u8,
        drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        points_len: u32,
        points: *const xcb_point_t,
    ) -> c_uint;
    pub fn xcb_poly_point(
        c: *mut xcb_connection_t,
        coordinate_mode: u8,
        drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        points_len: u32,
        points: *const xcb_point_t,
    ) -> c_uint;
    pub fn xcb_image_text_8(
        c: *mut xcb_connection_t,
        string_len: u8,
        drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        x: i16,
        y: i16,
        string: *const c_char,
    ) -> c_uint;
    pub fn xcb_fill_poly(
        c: *mut xcb_connection_t,
        drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        shape: u8,
        coordinate_mode: u8,
        points_len: u32,
        points: *const xcb_point_t,
    ) -> c_uint;
    pub fn xcb_copy_area(
        c: *mut xcb_connection_t,
        src_drawable: xcb_drawable_t,
        dst_drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        src_x: i16,
        src_y: i16,
        dst_x: i16,
        dst_y: i16,
        width: u16,
        height: u16,
    ) -> c_uint;
    pub fn xcb_put_image(
        c: *mut xcb_connection_t,
        format: u8,
        drawable: xcb_drawable_t,
        gc: xcb_gcontext_t,
        width: u16,
        height: u16,
        dst_x: i16,
        dst_y: i16,
        left_pad: u8,
        depth: u8,
        data_len: u32,
        data: *const u8,
    ) -> c_uint;
    pub fn xcb_get_image(
        c: *mut xcb_connection_t,
        format: u8,
        drawable: xcb_drawable_t,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        plane_mask: u32,
    ) -> c_uint;
    pub fn xcb_get_image_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_image_reply_t;
    pub fn xcb_open_font(
        c: *mut xcb_connection_t,
        fid: xcb_fontable_t,
        name_len: u16,
        name: *const c_char,
    ) -> c_uint;
    pub fn xcb_query_font(c: *mut xcb_connection_t, font: xcb_fontable_t) -> c_uint;
    pub fn xcb_query_font_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_query_font_reply_t;
    pub fn xcb_list_fonts(
        c: *mut xcb_connection_t,
        max_names: u16,
        pattern_len: u16,
        pattern: *const c_char,
    ) -> c_uint;
    pub fn xcb_list_fonts_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_list_fonts_reply_t;

    pub fn xcb_create_pixmap(
        c: *mut xcb_connection_t,
        depth: u8,
        pid: xcb_pixmap_t,
        drawable: xcb_drawable_t,
        width: u16,
        height: u16,
    ) -> c_uint;
    pub fn xcb_free_pixmap(c: *mut xcb_connection_t, pixmap: xcb_pixmap_t) -> c_uint;
    pub fn xcb_create_cursor(
        c: *mut xcb_connection_t,
        cid: xcb_cursor_t,
        source: xcb_pixmap_t,
        mask: xcb_pixmap_t,
        fore_red: u16,
        fore_green: u16,
        fore_blue: u16,
        back_red: u16,
        back_green: u16,
        back_blue: u16,
        x: u16,
        y: u16,
    ) -> c_uint;
    pub fn xcb_create_glyph_cursor(
        c: *mut xcb_connection_t,
        cid: xcb_cursor_t,
        source_font: xcb_fontable_t,
        mask_font: xcb_fontable_t,
        source_char: u16,
        mask_char: u16,
        fore_red: u16,
        fore_green: u16,
        fore_blue: u16,
        back_red: u16,
        back_green: u16,
        back_blue: u16,
    ) -> c_uint;
    pub fn xcb_free_cursor(c: *mut xcb_connection_t, cursor: xcb_cursor_t) -> c_uint;

    pub fn xcb_get_window_attributes(c: *mut xcb_connection_t, window: xcb_window_t) -> c_uint;
    pub fn xcb_get_window_attributes_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_window_attributes_reply_t;
    pub fn xcb_query_tree(c: *mut xcb_connection_t, window: xcb_window_t) -> c_uint;
    pub fn xcb_query_tree_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_query_tree_reply_t;
    pub fn xcb_get_geometry(c: *mut xcb_connection_t, drawable: xcb_drawable_t) -> c_uint;
    pub fn xcb_get_geometry_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_geometry_reply_t;
    pub fn xcb_get_keyboard_mapping(
        c: *mut xcb_connection_t,
        first_keycode: xcb_keycode_t,
        count: u8,
    ) -> c_uint;
    pub fn xcb_get_keyboard_mapping_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_keyboard_mapping_reply_t;
    pub fn xcb_get_modifier_mapping(c: *mut xcb_connection_t) -> c_uint;
    pub fn xcb_get_modifier_mapping_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_get_modifier_mapping_reply_t;
    pub fn xcb_query_pointer(c: *mut xcb_connection_t, window: xcb_window_t) -> c_uint;
    pub fn xcb_query_pointer_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_query_pointer_reply_t;
    pub fn xcb_translate_coordinates(
        c: *mut xcb_connection_t,
        src_window: xcb_window_t,
        dst_window: xcb_window_t,
        src_x: i16,
        src_y: i16,
    ) -> c_uint;
    pub fn xcb_translate_coordinates_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_translate_coordinates_reply_t;
    pub fn xcb_send_event(
        c: *mut xcb_connection_t,
        propagate: u8,
        destination: xcb_window_t,
        event_mask: u32,
        event: *const c_char,
    ) -> c_uint;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_selection_owner_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub owner: xcb_window_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_intern_atom_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub atom: xcb_atom_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_query_extension_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub present: u8,
    pub major_opcode: u8,
    pub first_event: u8,
    pub first_error: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_atom_name_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub name_len: u16,
    pub pad1: [u8; 22],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_get_image_reply_t {
    pub response_type: u8,
    pub depth: u8,
    pub sequence: u16,
    pub length: u32,
    pub visual: xcb_visualid_t,
    pub pad0: [u8; 20],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_list_fonts_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub names_len: u16,
    pub pad1: [u8; 22],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_translate_coordinates_reply_t {
    pub response_type: u8,
    pub same_screen: u8,
    pub sequence: u16,
    pub length: u32,
    pub child: xcb_window_t,
    pub dst_x: i16,
    pub dst_y: i16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_rectangle_t {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_point_t {
    pub x: i16,
    pub y: i16,
}

pub const XCB_XKB_ID_USE_CORE_KBD: u16 = 0x100;
pub const XCB_XKB_EVENT_TYPE_STATE_NOTIFY: u16 = 1 << 2;
pub const XCB_XKB_STATE_NOTIFY: u8 = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_xkb_use_extension_reply_t {
    pub response_type: u8,
    pub supported: u8,
    pub sequence: u16,
    pub length: u32,
    pub server_major: u16,
    pub server_minor: u16,
    pub pad0: [u8; 20],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct xcb_xkb_get_state_reply_t {
    pub response_type: u8,
    pub device_id: u8,
    pub sequence: u16,
    pub length: u32,
    pub mods: u8,
    pub base_mods: u8,
    pub latched_mods: u8,
    pub locked_mods: u8,
    pub group: u8,
    pub locked_group: u8,
    pub base_group: i16,
    pub latched_group: i16,
    pub compat_state: u8,
    pub grab_mods: u8,
    pub compat_grab_mods: u8,
    pub lookup_mods: u8,
    pub compat_lookup_mods: u8,
    pub pad0: u8,
    pub ptr_btn_state: u16,
    pub pad1: [u8; 6],
}

#[link(name = "xcb-xkb")]
extern "C" {
    pub fn xcb_xkb_use_extension(
        c: *mut xcb_connection_t,
        wanted_major: u16,
        wanted_minor: u16,
    ) -> c_uint;
    pub fn xcb_xkb_use_extension_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_xkb_use_extension_reply_t;
    pub fn xcb_xkb_select_events(
        c: *mut xcb_connection_t,
        device_spec: u16,
        affect_which: u16,
        clear: u16,
        select_all: u16,
        affect_map: u16,
        map: u16,
        details: *const c_void,
    ) -> c_uint;
    pub fn xcb_xkb_get_state(c: *mut xcb_connection_t, device_spec: u16) -> c_uint;
    pub fn xcb_xkb_get_state_reply(
        c: *mut xcb_connection_t,
        cookie: c_uint,
        e: *mut *mut xcb_generic_event_t,
    ) -> *mut xcb_xkb_get_state_reply_t;
    pub fn xcb_xkb_latch_lock_state(
        c: *mut xcb_connection_t,
        device_spec: u16,
        affect_mod_locks: u8,
        mod_locks: u8,
        lock_group: u8,
        group_lock: u8,
        affect_mod_latches: u8,
        latch_group: u8,
        group_latch: u16,
    ) -> c_uint;
}
