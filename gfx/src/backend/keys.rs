use crate::backend::types::KeyboardMapping;
use crate::keysyms::{
    KEY_Delete, KEY_Down, KEY_End, KEY_Home, KEY_Insert, KEY_KP_Delete, KEY_KP_Down, KEY_KP_End,
    KEY_KP_Enter, KEY_KP_Home, KEY_KP_Insert, KEY_KP_Left, KEY_KP_Next, KEY_KP_Prior, KEY_KP_Right,
    KEY_KP_Up, KEY_Left, KEY_Next, KEY_Prior, KEY_Return, KEY_Right, KEY_Up,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NormalizedKey {
    pub keysym: u32,
    pub original_keysym: u32,
    pub ch: Option<char>,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

pub fn keysym_col(keycode: u8, col: usize, mapping: &KeyboardMapping) -> u32 {
    let start = 8usize;
    let base = (keycode as usize).saturating_sub(start) * mapping.keysyms_per_keycode as usize;
    mapping.keysyms.get(base + col).copied().unwrap_or(0)
}

pub fn base_keysym(keycode: u8, mapping: &KeyboardMapping) -> u32 {
    keysym_col(keycode, 0, mapping)
}

pub fn keysym_to_char(ks: u32) -> Option<char> {
    match ks {
        0x20..=0x7E | 0xA0..=0xFF => std::char::from_u32(ks),
        0x0100_0000..=0x0110_FFFF => std::char::from_u32(ks - 0x0100_0000),
        _ => match crate::keysyms::keysym_to_ucs(ks) {
            0 => None,
            u => std::char::from_u32(u),
        },
    }
}

pub fn keycode_to_char(
    keycode: u8,
    shift: bool,
    group: usize,
    mapping: &KeyboardMapping,
) -> Option<char> {
    let level = usize::from(shift);
    let per = mapping.keysyms_per_keycode as usize;
    let mut cols = [group * 2 + level, group * 2, level, 0];
    if per > 0 {
        for c in &mut cols {
            if *c >= per {
                *c = 0;
            }
        }
    }
    for col in cols {
        let ks = keysym_col(keycode, col, mapping);
        if ks != 0 {
            if let Some(c) = keysym_to_char(ks) {
                return Some(c);
            }
        }
    }
    None
}

pub const fn group_of(state: u16) -> usize {
    ((state >> 13) & 0x03) as usize
}

#[allow(non_upper_case_globals)]
const fn map_keypad(ks: u32) -> u32 {
    match ks {
        KEY_KP_Home => KEY_Home,
        KEY_KP_Left => KEY_Left,
        KEY_KP_Up => KEY_Up,
        KEY_KP_Right => KEY_Right,
        KEY_KP_Down => KEY_Down,
        KEY_KP_Prior => KEY_Prior,
        KEY_KP_Next => KEY_Next,
        KEY_KP_End => KEY_End,
        KEY_KP_Insert => KEY_Insert,
        KEY_KP_Delete => KEY_Delete,
        KEY_KP_Enter => KEY_Return,
        _ => ks,
    }
}

pub fn normalize(keycode: u32, state: u16, mapping: &KeyboardMapping) -> NormalizedKey {
    let shift = state & 0x01 != 0;
    let ctrl = state & 0x04 != 0;
    let alt = state & 0x08 != 0;
    let logo = state & 0x40 != 0;
    let original = base_keysym(keycode as u8, mapping);
    NormalizedKey {
        keysym: map_keypad(original),
        original_keysym: original,
        ch: keycode_to_char(keycode as u8, shift, group_of(state), mapping),
        shift,
        ctrl,
        alt,
        logo,
    }
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod tests;
