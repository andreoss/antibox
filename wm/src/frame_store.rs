use crate::frame::FrameWindow;
use crate::id::ClientId;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub struct FrameStore {
    inner: HashMap<ClientId, FrameWindow>,
}

impl Default for FrameStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameStore {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, cid: &ClientId) -> Option<&FrameWindow> {
        self.inner.get(cid)
    }

    pub fn get_mut(&mut self, cid: &ClientId) -> Option<&mut FrameWindow> {
        self.inner.get_mut(cid)
    }

    pub fn insert(&mut self, cid: ClientId, fw: FrameWindow) {
        self.inner.insert(cid, fw);
    }

    pub fn remove(&mut self, cid: &ClientId) -> Option<FrameWindow> {
        self.inner.remove(cid)
    }

    pub fn contains_key(&self, cid: &ClientId) -> bool {
        self.inner.contains_key(cid)
    }

    pub fn keys(&self) -> impl Iterator<Item = ClientId> + '_ {
        self.inner.keys().copied()
    }

    pub fn values(&self) -> impl Iterator<Item = &FrameWindow> {
        self.inner.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut FrameWindow> {
        self.inner.values_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ClientId, &FrameWindow)> {
        self.inner.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Index<&ClientId> for FrameStore {
    type Output = FrameWindow;
    fn index(&self, cid: &ClientId) -> &FrameWindow {
        self.get(cid).expect("no frame for ClientId")
    }
}

impl IndexMut<&ClientId> for FrameStore {
    fn index_mut(&mut self, cid: &ClientId) -> &mut FrameWindow {
        self.get_mut(cid).expect("no frame for ClientId")
    }
}
