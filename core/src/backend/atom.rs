 use antibox_gfx::error::Result;
use crate::backend::DisplayBackend;

#[derive(Debug, Default)]
pub struct AtomManager {
    pub(crate) atoms: std::collections::HashMap<String, u32>,
    rev: std::collections::HashMap<u32, String>,
}

impl AtomManager {
    pub fn new() -> Self {
        Self {
            atoms: std::collections::HashMap::new(),
            rev: std::collections::HashMap::new(),
        }
    }

    pub fn intern<C: DisplayBackend + ?Sized>(
        &mut self,
        conn: &C,
        name: &str,
    ) -> Result<u32> {
        if let Some(&atom) = self.atoms.get(name) {
            return Ok(atom);
        }
        let atom = conn.intern_atom(name)?;
        self.atoms.insert(name.to_string(), atom);
        self.rev.insert(atom, name.to_string());
        Ok(atom)
    }

    pub fn get(&self, name: &str) -> Option<u32> {
        self.atoms.get(name).cloned()
    }

    pub fn supported_list(&self) -> Vec<u32> {
        SUPPORTED_ATOM_NAMES
            .iter()
            .filter_map(|name| self.get(name))
            .collect()
    }

    pub fn intern_all<C: DisplayBackend + ?Sized>(
        &mut self,
        conn: &C,
    ) -> Result<()> {
        for name in ALL_ATOM_NAMES {
            self.intern(conn, name)?;
        }
        Ok(())
    }

    pub fn reverse_map(&self) -> std::collections::HashMap<u32, String> {
        self.rev.clone()
    }
}

include!("atom_supported.rs");
include!("atom_all.rs");

#[cfg(test)]
#[path = "atom_tests.rs"]
mod tests;
