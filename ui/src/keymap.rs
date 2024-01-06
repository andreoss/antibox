use antibox_core::backend::{DisplayBackend, KeyboardMapping};
use antibox_core::sync::atomic::LazyRwLock;
use std::sync::Arc;

static KEYMAP: LazyRwLock<Option<Arc<KeyboardMapping>>> = LazyRwLock::new();

pub fn keymap<H: DisplayBackend + ?Sized>(conn: &H) -> Option<Arc<KeyboardMapping>> {
    if let Ok(g) = KEYMAP.read() {
        if let Some(m) = g.as_ref() {
            return Some(m.clone());
        }
    }
    let min = conn.setup_min_keycode();
    let max = conn.setup_max_keycode();
    let count = (max as usize - min as usize + 1) as u8;
    let m = match conn.get_keyboard_mapping(min, count) {
        Ok(m) => Arc::new(m),
        Err(_) => return None,
    };
    if let Ok(mut g) = KEYMAP.write() {
        *g = Some(m.clone());
    }
    Some(m)
}

pub fn invalidate_keymap() {
    if let Ok(mut g) = KEYMAP.write() {
        *g = None;
    }
}

pub fn keysym_for_keycode<H: DisplayBackend + ?Sized>(conn: &H, keycode: u32) -> u32 {
    let min = conn.setup_min_keycode();
    if let Some(m) = keymap(conn) {
        let off =
            (keycode as usize).saturating_sub(min as usize) * (m.keysyms_per_keycode as usize);
        if off < m.keysyms.len() {
            return m.keysyms[off];
        }
    }
    0
}
