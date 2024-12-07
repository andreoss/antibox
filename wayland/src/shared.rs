use antibox_core::backend::EventQueue;
use antibox_core::rect::Rect;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::{Arc, Mutex, MutexGuard};

pub const ROOT_WINDOW: u32 = 1;

pub const FIRST_WINDOW_ID: u32 = 0x10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WinKind {
    Root,
    Client,
    Server,
    InputOnly,
}

#[derive(Clone, Debug)]
pub struct WinRec {
    pub kind: WinKind,
    pub rect: Rect,
    pub mapped: bool,
    pub override_redirect: bool,
    pub depth: u8,
    pub parent: u32,
    pub event_mask: u64,
}

impl WinRec {
    pub const fn map_state(&self) -> antibox_core::backend::MapState {
        use antibox_core::backend::MapState;
        if self.mapped {
            MapState::Viewable
        } else {
            MapState::Unmapped
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    Map(u32),
    Unmap(u32),
    Destroy(u32),
    Kill(u32),
    Configure {
        win: u32,
        x: Option<i32>,
        y: Option<i32>,
        w: Option<u16>,
        h: Option<u16>,
    },
    Raise(u32),
    Lower(u32),
    Restack(Vec<u32>),
    Focus(u32),
    Warp {
        x: i16,
        y: i16,
    },
}

#[derive(Default)]
pub struct AtomTable {
    names: Vec<String>,
    by_name: HashMap<String, u32>,
}

impl AtomTable {
    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.by_name.get(name) {
            return id;
        }
        self.names.push(name.to_string());
        let id = self.names.len() as u32;
        self.by_name.insert(name.to_string(), id);
        id
    }

    pub fn name(&self, id: u32) -> Option<String> {
        if id == 0 {
            return None;
        }
        self.names.get((id - 1) as usize).cloned()
    }
}

#[derive(Clone, Debug)]
pub struct KeyGrab {
    pub keycode: u8,
    pub modifiers: u16,
    pub window: u32,
}

#[derive(Clone, Debug)]
pub struct ButtonGrab {
    pub button: u8,
    pub modifiers: u16,
    pub window: u32,
}

pub struct SharedState {
    pub events: EventQueue,
    pub intents: Vec<Intent>,
    pub atoms: AtomTable,
    pub props: HashMap<(u32, u32), Vec<u8>>,
    pub key_grabs: Vec<KeyGrab>,
    pub button_grabs: Vec<ButtonGrab>,
    pub pointer_grab: Option<u32>,
    pub implicit_grab: Option<u32>,
    pub keyboard_grab: Option<u32>,
    pub windows: HashMap<u32, WinRec>,
    pub selections: HashMap<u32, u32>,
    pub stack: Vec<u32>,
    pub focus: u32,
    pub last_time: u32,
    pub screen_w: u16,
    pub screen_h: u16,

    pub screen_scale: f64,
    pub pointer_x: i16,
    pub pointer_y: i16,
    pub pointer_mask: u16,
    pub key_mods: u16,
    pub display_fd: RawFd,
    pub keymap: Vec<u32>,
    pub keysyms_per_keycode: u8,
    background: Option<u32>,
    pointer_window: Option<u32>,
    next_id: u32,
    needs_redraw: bool,
}

impl SharedState {
    pub fn new(screen_w: u16, screen_h: u16) -> Self {
        let mut windows = HashMap::new();
        windows.insert(
            ROOT_WINDOW,
            WinRec {
                kind: WinKind::Root,
                rect: Rect::new(0, 0, screen_w as i32, screen_h as i32),
                mapped: true,
                override_redirect: false,
                depth: 32,
                parent: 0,
                event_mask: 0,
            },
        );
        Self {
            events: EventQueue::new(),
            intents: Vec::new(),
            atoms: AtomTable::default(),
            props: HashMap::new(),
            key_grabs: Vec::new(),
            button_grabs: Vec::new(),
            pointer_grab: None,
            implicit_grab: None,
            keyboard_grab: None,
            windows,
            selections: HashMap::new(),
            stack: Vec::new(),
            focus: ROOT_WINDOW,
            last_time: 0,
            screen_w,
            screen_h,
            screen_scale: 1.0,
            pointer_x: 0,
            pointer_y: 0,
            pointer_mask: 0,
            key_mods: 0,
            display_fd: -1,
            keymap: Vec::new(),
            keysyms_per_keycode: 0,
            background: None,
            pointer_window: None,
            next_id: FIRST_WINDOW_ID,
            needs_redraw: true,
        }
    }

    fn selects(&self, win: u32, mask: u64) -> bool {
        self.windows
            .get(&win)
            .is_some_and(|r| r.event_mask & mask != 0)
    }

    pub fn pointer_crossing(&mut self, cur: Option<u32>) -> (Option<u32>, Option<u32>) {
        if self.pointer_window == cur {
            return (None, None);
        }
        let prev = self.pointer_window;
        self.pointer_window = cur;
        use antibox_core::backend::EventMask;
        let leave = prev.filter(|w| self.selects(*w, EventMask::LEAVE_WINDOW.bits()));
        let enter = cur.filter(|w| self.selects(*w, EventMask::ENTER_WINDOW.bits()));
        (leave, enter)
    }

    pub fn set_root_background(&mut self, colour: u32) {
        self.background = Some(colour);
    }

    pub fn clear_colour(&self) -> [f32; 4] {
        match self.background {
            Some(c) => [
                ((c >> 16) & 0xFF) as f32 / 255.0,
                ((c >> 8) & 0xFF) as f32 / 255.0,
                (c & 0xFF) as f32 / 255.0,
                1.0,
            ],
            None => [0.1, 0.1, 0.1, 1.0],
        }
    }

    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("window id space exhausted");
        id
    }

    pub fn absolute_origin(&self, id: u32) -> (i32, i32) {
        let (mut x, mut y) = (0, 0);
        let mut cur = id;
        let mut guard = 0;
        while let Some(rec) = self.windows.get(&cur) {
            x += rec.rect.x;
            y += rec.rect.y;
            if rec.parent == ROOT_WINDOW || rec.parent == cur || guard > 32 {
                break;
            }
            cur = rec.parent;
            guard += 1;
        }
        (x, y)
    }

    pub fn window_at(&self, x: i32, y: i32) -> Option<u32> {
        let mut best: Option<(u32, i64)> = None;
        for (id, rec) in &self.windows {
            if !rec.mapped || matches!(rec.kind, WinKind::Client | WinKind::Root) {
                continue;
            }
            let (ax, ay) = self.absolute_origin(*id);
            let (w, h) = (rec.rect.w, rec.rect.h);
            if x >= ax && y >= ay && x < ax + w && y < ay + h {
                let area = w as i64 * h as i64;
                if best.map_or(true, |(_, a)| area < a) {
                    best = Some((*id, area));
                }
            }
        }
        best.map(|(id, _)| id)
    }

    pub fn propagate_event_target(&self, id: u32, mask: u64) -> Option<u32> {
        let mut cur = id;
        for _ in 0..32 {
            let rec = self.windows.get(&cur)?;
            if rec.event_mask & mask != 0 {
                return Some(cur);
            }
            if rec.parent == cur || rec.parent == ROOT_WINDOW {
                return None;
            }
            cur = rec.parent;
        }
        None
    }
}

#[derive(Clone)]
pub struct Shared(Arc<Mutex<SharedState>>);

impl Shared {
    pub fn new(screen_w: u16, screen_h: u16) -> Self {
        Self(Arc::new(Mutex::new(SharedState::new(screen_w, screen_h))))
    }

    pub fn lock(&self) -> MutexGuard<'_, SharedState> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn request_redraw(&self) {
        self.lock().needs_redraw = true;
    }

    pub fn take_needs_redraw(&self) -> bool {
        let mut s = self.lock();
        std::mem::replace(&mut s.needs_redraw, false)
    }
}

#[cfg(test)]
#[path = "shared_tests.rs"]
mod tests;
