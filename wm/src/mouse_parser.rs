use crate::action::*;

#[derive(Debug, Clone)]
pub struct MouseEntry {
    pub button_keysym: u32,
    pub modifiers: u16,
    pub action: Action,
}

pub fn mouse_button_from_keysym(keysym: u32) -> Option<u32> {
    match keysym {
        0x010014 => Some(1),
        0x010015 => Some(2),
        0x010016 => Some(3),
        0x010017 => Some(4),
        0x010018 => Some(5),
        _ => None,
    }
}

pub fn mouse_button_from_state(button: u8, keysym: u32) -> bool {
    match (button, keysym) {
        (1, 0x010014) | (2, 0x010015) | (3, 0x010016) | (4, 0x010017) | (5, 0x010018) => true,
        _ => false,
    }
}

pub const fn match_mouse_modifiers(state: u16, mods: u16) -> bool {
    (state & 0xFF) & !(0x02 | 0x10 | 0x20) == mods
}

pub fn modifiers_to_button_mask(mods: u16) -> u16 {
    let mut mask = 0u16;
    if mods & 0x01 != 0 {
        mask |= 0x01;
    }
    if mods & 0x04 != 0 {
        mask |= 0x04;
    }
    if mods & 0x08 != 0 {
        mask |= 0x08;
    }
    if mods & 0x40 != 0 {
        mask |= 0x40;
    }
    mask
}

#[cfg(test)]
#[path = "mouse_parser_tests.rs"]
mod tests;
