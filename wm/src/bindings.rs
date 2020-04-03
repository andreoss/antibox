use crate::action::*;
use antibox_core::backend::{DisplayBackend, GrabMode};
use std::sync::Arc;

const KEY_ESCAPE: u32 = 0xFF1B;
const KEY_LEFT: u32 = 0xFF51;
const KEY_RIGHT: u32 = 0xFF53;
const KEY_UP: u32 = 0xFF52;
const KEY_DOWN: u32 = 0xFF54;
const KEY_RETURN: u32 = 0xFF0D;
const KEY_SUPER_L: u32 = 0xFFEB;
const KEY_SUPER_R: u32 = 0xFFEC;

pub struct KeyBindings {
    bindings: Vec<KeyBinding>,
}

struct KeyBinding {
    keycode: u8,
    modifiers: u16,
    lock_combs: [u16; 4],
    action: Action,
}

impl Default for KeyBindings {
    fn default() -> KeyBindings {
        Self::new()
    }
}

const LOCK_MASK: u16 = 0x02 | 0x10 | 0x20;

impl KeyBindings {
    pub fn new() -> KeyBindings {
        KeyBindings {
            bindings: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn register_all<H: DisplayBackend + 'static + ?Sized>(
        &mut self,
        backend: &Arc<H>,
        entries: &[crate::keys_parser::KeyEntry],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let min_kc = backend.setup_min_keycode();
        let max_kc = backend.setup_max_keycode();
        let count = max_kc - min_kc + 1;
        let mapping = backend.get_keyboard_mapping(min_kc, count)?;
        let keysyms = &mapping.keysyms;
        let kpc = mapping.keysyms_per_keycode as usize;
        let root = backend.root().read_id();

        let lock_combs = [0u16, 0x02, 0x10, 0x02 | 0x10];

        for e in entries {
            if let Some(kc) = find_keycode(keysyms, kpc, min_kc, e.keysym) {
                for &locks in &lock_combs {
                    let _ = backend.grab_key(
                        false,
                        root,
                        e.modifiers | locks,
                        kc,
                        GrabMode::Async,
                        GrabMode::Async,
                    );
                }
                self.bindings.push(KeyBinding {
                    keycode: kc,
                    modifiers: e.modifiers,
                    lock_combs,
                    action: e.action.clone(),
                });
            }
        }
        self.grab_bare_super(backend);
        Ok(())
    }

    fn grab_bare_super<H: DisplayBackend + 'static + ?Sized>(&self, backend: &Arc<H>) {
        let min_kc = backend.setup_min_keycode();
        let max_kc = backend.setup_max_keycode();
        let count = max_kc - min_kc + 1;
        let mapping = match backend.get_keyboard_mapping(min_kc, count) {
            Ok(m) => m,
            Err(_) => return,
        };
        let kpc = mapping.keysyms_per_keycode as usize;
        let root = backend.root().read_id();
        let lock_combs = [0u16, 0x02, 0x10, 0x02 | 0x10];
        for sym in [KEY_SUPER_L, KEY_SUPER_R].iter().cloned() {
            if let Some(kc) = find_keycode(&mapping.keysyms, kpc, min_kc, sym) {
                for &locks in &lock_combs {
                    let _ =
                        backend.grab_key(false, root, locks, kc, GrabMode::Async, GrabMode::Async);
                }
            }
        }
    }

    pub fn regrab_all<H: DisplayBackend + 'static + ?Sized>(
        &self,
        backend: &Arc<H>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = backend.root().read_id();
        for b in &self.bindings {
            for &locks in &b.lock_combs {
                let _ = backend.grab_key(
                    false,
                    root,
                    b.modifiers | locks,
                    b.keycode,
                    GrabMode::Async,
                    GrabMode::Async,
                );
            }
        }
        self.grab_bare_super(backend);
        Ok(())
    }

    pub fn lookup(&self, keycode: u8, state: u16) -> Option<&Action> {
        let filtered = (state & 0xFF) & !LOCK_MASK;
        for b in &self.bindings {
            if b.keycode == keycode && b.modifiers == filtered {
                return Some(&b.action);
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeymapAction {
    Dispatch(Action),
    Drag,
    Pop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeymapLookup {
    Pass,
    Swallow,
    Drag,
    Pop,
    Dispatch(Action),
}

pub struct TransientKeymap {
    entries: Vec<(u32, KeymapAction)>,
}

impl TransientKeymap {
    pub const fn new(entries: Vec<(u32, KeymapAction)>) -> TransientKeymap {
        TransientKeymap { entries }
    }

    pub fn lookup(&self, keysym: u32) -> Option<&KeymapAction> {
        self.entries
            .iter()
            .find(|(ks, _)| *ks == keysym)
            .map(|(_, a)| a)
    }
}

#[derive(Default)]
pub struct KeymapStack {
    maps: Vec<TransientKeymap>,
}

impl KeymapStack {
    pub fn new() -> KeymapStack {
        KeymapStack { maps: Vec::new() }
    }

    pub fn is_active(&self) -> bool {
        !self.maps.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.maps.len()
    }

    pub fn push(&mut self, map: TransientKeymap) {
        self.maps.push(map);
    }

    pub fn pop(&mut self) -> bool {
        self.maps.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.maps.clear();
    }

    pub fn lookup(&self, keysym: u32) -> KeymapLookup {
        let top = match self.maps.last() {
            Some(top) => top,
            None => return KeymapLookup::Pass,
        };
        match top.lookup(keysym) {
            Some(KeymapAction::Dispatch(a)) => KeymapLookup::Dispatch(a.clone()),
            Some(KeymapAction::Drag) => KeymapLookup::Drag,
            Some(KeymapAction::Pop) => KeymapLookup::Pop,
            None => KeymapLookup::Swallow,
        }
    }
}

pub fn move_resize_keymap() -> TransientKeymap {
    TransientKeymap::new(vec![
        (KEY_LEFT, KeymapAction::Drag),
        (KEY_RIGHT, KeymapAction::Drag),
        (KEY_UP, KeymapAction::Drag),
        (KEY_DOWN, KeymapAction::Drag),
        (KEY_RETURN, KeymapAction::Drag),
        (KEY_ESCAPE, KeymapAction::Drag),
    ])
}

fn find_keycode(keysyms: &[u32], kpc: usize, min_kc: u8, target: u32) -> Option<u8> {
    for (idx, chunk) in keysyms.chunks(kpc).enumerate() {
        let keycode = (min_kc as usize + idx) as u8;
        if chunk.contains(&target) {
            return Some(keycode);
        }
    }
    None
}

pub fn keysym_for_keycode(conn: &dyn DisplayBackend, keycode: u32) -> u32 {
    let min = conn.setup_min_keycode();
    let max = conn.setup_max_keycode();
    if let Ok(m) = conn.get_keyboard_mapping(min, (max as usize - min as usize + 1) as u8) {
        let off =
            (keycode as usize).saturating_sub(min as usize) * (m.keysyms_per_keycode as usize);
        if off < m.keysyms.len() {
            return m.keysyms[off];
        }
    }
    0
}

#[cfg(test)]
#[path = "bindings_tests.rs"]
mod tests;
