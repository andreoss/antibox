use crate::id::ClientId;

const CAPACITY: usize = 64;

#[derive(Default)]
pub struct FocusHistory {
    ring: Vec<ClientId>,
}

impl FocusHistory {
    pub const fn new() -> Self {
        Self { ring: Vec::new() }
    }

    pub fn note(&mut self, id: ClientId) {
        if self.ring.first() == Some(&id) {
            return;
        }
        self.ring.retain(|&x| x != id);
        self.ring.insert(0, id);
        self.ring.truncate(CAPACITY);
    }

    pub fn forget(&mut self, id: ClientId) {
        self.ring.retain(|&x| x != id);
    }

    pub fn entries(&self) -> &[ClientId] {
        &self.ring
    }

    pub fn order<F: Fn(ClientId) -> bool>(&self, current: Option<ClientId>, live: F) -> Vec<ClientId> {
        let mut out: Vec<ClientId> = Vec::with_capacity(self.ring.len());
        if let Some(c) = current.filter(|&c| live(c)) {
            out.push(c);
        }
        for &id in &self.ring {
            if live(id) && !out.contains(&id) {
                out.push(id);
            }
        }
        out
    }

    pub fn step_back<F: Fn(ClientId) -> bool>(&self, from: Option<ClientId>, live: F) -> Option<ClientId> {
        let start = from
            .and_then(|f| self.ring.iter().position(|&x| x == f))
            .map_or(0, |i| i + 1);
        self.ring[start.min(self.ring.len())..]
            .iter()
            .copied()
            .find(|&id| live(id))
    }

    pub fn step_forward<F: Fn(ClientId) -> bool>(&self, from: Option<ClientId>, live: F) -> Option<ClientId> {
        let at = from.and_then(|f| self.ring.iter().position(|&x| x == f))?;
        self.ring[..at].iter().rev().copied().find(|&id| live(id))
    }
}

#[cfg(test)]
#[path = "focus_history_tests.rs"]
mod tests;
