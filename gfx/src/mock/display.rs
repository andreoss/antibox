use super::window::{Lifecycle, LifecycleLog, MockWindow};
use crate::backend::EventQueue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct MockDisplay {
    pub next_id: Arc<Mutex<u32>>,
    pub windows: Arc<Mutex<HashMap<u32, MockWindow>>>,
    pub atoms: Arc<Mutex<HashMap<String, u32>>>,
    pub events: Arc<Mutex<EventQueue>>,
    pub root_window: u32,
    pub width: crate::sync::atomic::AtomicU16,
    pub height: crate::sync::atomic::AtomicU16,
    pub depth: u8,
    pub visual: u32,
    pub screen_num: usize,
    pub warps: Arc<Mutex<Vec<(u32, i16, i16)>>>,
    pub lifecycle: LifecycleLog,
    pub broken: Arc<Mutex<std::collections::HashSet<u32>>>,
}

impl MockDisplay {
    pub fn new(width: u16, height: u16, depth: u8) -> Self {
        Self::with_screen(width, height, depth, 0, 1)
    }

    pub fn with_screen(
        width: u16,
        height: u16,
        depth: u8,
        screen_num: usize,
        root_window: u32,
    ) -> Self {
        let lifecycle = Arc::new(Mutex::new(Vec::new()));
        let mut windows = HashMap::new();
        windows.insert(
            root_window,
            MockWindow::with_lifecycle(root_window, lifecycle.clone()),
        );
        Self {
            next_id: Arc::new(Mutex::new(root_window + 1)),
            windows: Arc::new(Mutex::new(windows)),
            atoms: Arc::new(Mutex::new(HashMap::new())),
            events: Arc::new(Mutex::new(EventQueue::new())),
            root_window,
            width: crate::sync::atomic::AtomicU16::new(width),
            height: crate::sync::atomic::AtomicU16::new(height),
            depth,
            visual: 42,
            screen_num,
            warps: Arc::new(Mutex::new(Vec::new())),
            lifecycle,
            broken: Arc::new(Mutex::new(std::collections::HashSet::new())),
        }
    }

    pub fn get_window(&self, id: u32) -> Option<MockWindow> {
        self.windows.lock().unwrap().get(&id).cloned()
    }

    pub fn lifecycle_events(&self) -> Vec<(u32, Lifecycle)> {
        self.lifecycle.lock().unwrap().clone()
    }
}

#[cfg(test)]
#[path = "display_tests.rs"]
mod tests;
