use std::os::raw::{c_int, c_void};

pub type xkb_keycode_t = u32;
pub type xkb_keysym_t = u32;
pub type xkb_layout_index_t = u32;
pub type xkb_level_index_t = u32;

#[link(name = "xkbcommon")]
extern "C" {
    pub fn xkb_keymap_num_levels_for_key(
        keymap: *mut c_void,
        key: xkb_keycode_t,
        layout: xkb_layout_index_t,
    ) -> xkb_level_index_t;
    pub fn xkb_keymap_key_get_syms_by_level(
        keymap: *mut c_void,
        key: xkb_keycode_t,
        layout: xkb_layout_index_t,
        level: xkb_level_index_t,
        syms_out: *mut *const xkb_keysym_t,
    ) -> c_int;
    pub fn xkb_state_serialize_layout(state: *mut c_void, components: u32) -> xkb_layout_index_t;
}

pub const XKB_STATE_LAYOUT_EFFECTIVE: u32 = 1 << 7;
