use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
use antibox_core::point::Point;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    Floating,
}

impl Default for Layout {
    fn default() -> Layout {
        Layout::Floating
    }
}

impl Layout {
    pub const ALL: &'static [Self] = &[Layout::Floating];

    pub fn name(self) -> &'static str {
        match self {
            Layout::Floating => "floating",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Layout::Floating => "Floating",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "floating" | "float" => Some(Layout::Floating),
            _ => None,
        }
    }

    pub fn parse_list(s: &str, count: usize) -> Vec<Self> {
        let mut out: Vec<Self> = s
            .split(&[',', ' '][..])
            .filter(|t| !t.trim().is_empty())
            .map(|t| Self::parse(t).unwrap_or_default())
            .collect();
        out.resize(count.max(1), Self::default());
        out.truncate(count.max(1));
        out
    }

    pub fn place_new(
        self,
        w: i32,
        h: i32,
        wm: &WindowManager<impl DisplayBackend + ?Sized>,
    ) -> Point {
        crate::placement::smart_placement(w, h, wm)
    }

    pub fn arrange<H: DisplayBackend + 'static + ?Sized>(self, wm: &mut WindowManager<H>, ws: u32) {
        let ids: Vec<ClientId> = wm
            .frames
            .iter()
            .filter(|(_, f)| {
                let w = f.workspace();
                w == ws || w == !0
            })
            .map(|(&id, _)| id)
            .collect();
        for id in ids {
            set_tile_slot(wm, id, None);
        }
    }
}

fn set_tile_slot<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    id: ClientId,
    slot: Option<u32>,
) {
    let atom = wm.atoms.get("_WM_TILE_SLOT").unwrap_or(0);
    if atom == 0 {
        return;
    }
    let b = match wm.backend() {
        Some(b) => b,
        None => return,
    };
    let cid_xid = wm.xid_index.xid_of(id);
    match slot {
        Some(s) => {
            let _ = b.change_property32(
                antibox_core::backend::PropMode::Replace,
                cid_xid,
                atom,
                6,
                &[s],
            );
        }
        None => {
            let _ = b.delete_property(cid_xid, atom);
        }
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
