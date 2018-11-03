use std::collections::HashMap;
use std::fmt;
use antibox_core::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ClientId(pub u64);

impl ClientId {
    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn allocate() -> ClientId {
        ClientId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FrameId(pub u32);

impl FrameId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for FrameId {
    fn from(v: u32) -> FrameId {
        FrameId(v)
    }
}

impl fmt::Display for FrameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

pub struct XidIndex {
    client_by_xid: HashMap<u32, ClientId>,
    frame_by_xid: HashMap<u32, ClientId>,
    xid_of: HashMap<ClientId, u32>,
}

impl Default for XidIndex {
    fn default() -> XidIndex {
        Self::new()
    }
}

impl XidIndex {
    pub fn new() -> XidIndex {
        XidIndex {
            client_by_xid: HashMap::new(),
            frame_by_xid: HashMap::new(),
            xid_of: HashMap::new(),
        }
    }

    pub fn insert(&mut self, cid: ClientId, client_xid: u32, frame_xid: u32) -> ClientId {
        self.client_by_xid.insert(client_xid, cid);
        self.frame_by_xid.insert(frame_xid, cid);
        self.xid_of.insert(cid, client_xid);
        cid
    }

    pub fn remove(&mut self, cid: ClientId) {
        if let Some(xid) = self.xid_of.remove(&cid) {
            self.client_by_xid.remove(&xid);
        }
        self.frame_by_xid.retain(|_, &mut v| v != cid);
    }

    pub fn client_id_for(&self, xid: u32) -> Option<ClientId> {
        self.client_by_xid
            .get(&xid).cloned()
            .or_else(|| self.frame_by_xid.get(&xid).cloned())
    }

    pub fn xid_of(&self, cid: ClientId) -> u32 {
        self.xid_of.get(&cid).cloned().unwrap_or(0)
    }

    pub fn has(&self, cid: ClientId) -> bool {
        self.xid_of.contains_key(&cid)
    }

    pub fn contains_xid(&self, xid: u32) -> bool {
        self.client_by_xid.contains_key(&xid) || self.frame_by_xid.contains_key(&xid)
    }

    pub fn contains_client_xid(&self, xid: u32) -> bool {
        self.client_by_xid.contains_key(&xid)
    }

    pub fn len(&self) -> usize {
        self.xid_of.len()
    }

    pub fn is_empty(&self) -> bool {
        self.xid_of.is_empty()
    }
}
