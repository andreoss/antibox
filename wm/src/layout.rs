use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
use antibox_core::point::Point;
use antibox_core::rect::Rect;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    Floating,
    Tall,
    Wide,
}

impl Default for Layout {
    fn default() -> Layout {
        Layout::Floating
    }
}

const TALL_NMASTER: usize = 1;

const TALL_FRAC_PCT: i32 = 50;

impl Layout {
    pub const ALL: &'static [Self] = &[Layout::Floating, Layout::Tall, Layout::Wide];

    pub fn name(self) -> &'static str {
        match self {
            Layout::Floating => "floating",
            Layout::Tall => "tall",
            Layout::Wide => "wide",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Layout::Floating => "Floating",
            Layout::Tall => "Tall (tiled)",
            Layout::Wide => "Wide (tiled)",
        }
    }

    pub fn is_tiled(self) -> bool {
        match self {
            Layout::Tall | Layout::Wide => true,
            Layout::Floating => false,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "floating" | "float" => Some(Layout::Floating),
            "tall" | "tile" | "tiled" => Some(Layout::Tall),
            "wide" | "mirror" | "wide-tiled" => Some(Layout::Wide),
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

    pub fn next(self) -> Self {
        let all = Self::ALL;
        let i = all.iter().position(|&l| l == self).unwrap_or(0);
        all[(i + 1) % all.len()]
    }

    pub fn place_new(
        self,
        w: i32,
        h: i32,
        wm: &WindowManager<impl DisplayBackend + ?Sized>,
    ) -> Point {
        match self {
            Layout::Floating => crate::placement::smart_placement(w, h, wm),
            Layout::Tall | Layout::Wide => {
                let wa = work_area(wm);
                Point::new(wa.x, wa.y)
            }
        }
    }

    pub fn arrange<H: DisplayBackend + 'static + ?Sized>(self, wm: &mut WindowManager<H>, ws: u32) {
        match self {
            Layout::Floating => {
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
            Layout::Tall => {
                arrange_with(wm, ws, |area, n| tile(TALL_FRAC_PCT, area, TALL_NMASTER, n));
            }
            Layout::Wide => arrange_with(wm, ws, |area, n| {
                tile_wide(TALL_FRAC_PCT, area, TALL_NMASTER, n)
            }),
        }
    }
}

fn work_area(wm: &WindowManager<impl DisplayBackend + ?Sized>) -> Rect {
    if let Some(wa) = wm.workareas.first() {
        return *wa;
    }
    if let Some(m) = wm.monitors.first() {
        return Rect::new(m.x as i32, m.y as i32, m.width as i32, m.height as i32);
    }
    let (sw, sh) = wm.backend().map_or((1920, 1080), |b| {
        (b.screen_width() as i32, b.screen_height() as i32)
    });
    Rect::new(0, 0, sw, sh)
}

fn split_vertically(n: usize, r: Rect) -> Vec<Rect> {
    if n <= 1 {
        return vec![r];
    }
    let mut out = Vec::with_capacity(n);
    let mut y = r.y;
    let mut remaining = r.h;
    for row in 0..n {
        let rows_left = (n - row) as i32;
        let h = remaining / rows_left;
        out.push(Rect::new(r.x, y, r.w, h));
        y += h;
        remaining -= h;
    }
    out
}

fn split_h_by(frac_pct: i32, r: Rect) -> (Rect, Rect) {
    let left_w = (r.w * frac_pct / 100).max(1).min((r.w - 1).max(1));
    (
        Rect::new(r.x, r.y, left_w, r.h),
        Rect::new(r.x + left_w, r.y, r.w - left_w, r.h),
    )
}

pub fn tile(frac_pct: i32, r: Rect, nmaster: usize, n: usize) -> Vec<Rect> {
    if n == 0 {
        return Vec::new();
    }
    if n <= nmaster || nmaster == 0 {
        return split_vertically(n, r);
    }
    let (master, stack) = split_h_by(frac_pct, r);
    let mut out = split_vertically(nmaster, master);
    out.extend(split_vertically(n - nmaster, stack));
    out
}

fn mirror_rect(r: Rect) -> Rect {
    Rect::new(r.y, r.x, r.h, r.w)
}

pub fn tile_wide(frac_pct: i32, r: Rect, nmaster: usize, n: usize) -> Vec<Rect> {
    tile(frac_pct, mirror_rect(r), nmaster, n)
        .into_iter()
        .map(mirror_rect)
        .collect()
}

fn tileable(f: &crate::frame::FrameWindow, ws: u32) -> bool {
    let s = f.state();
    let w = f.workspace();
    (w == ws || w == !0) && f.decorated() && !s.minimized && !s.fullscreen && !s.skip_taskbar
}

fn arrange_with<H, F>(wm: &mut WindowManager<H>, ws: u32, compute: F)
where
    H: DisplayBackend + 'static + ?Sized,
    F: Fn(Rect, usize) -> Vec<Rect>,
{
    let mut ids: Vec<ClientId> = wm
        .map_order
        .iter()
        .cloned()
        .filter(|id| wm.frames.get(id).map_or(false, |f| tileable(f, ws)))
        .collect();
    for (id, f) in wm.frames.iter() {
        if !ids.contains(id) && tileable(f, ws) {
            ids.push(*id);
        }
    }
    if ids.is_empty() {
        return;
    }
    let rects = compute(work_area(wm), ids.len());
    for (slot, (id, rect)) in ids.into_iter().zip(rects).enumerate() {
        clear_maximized(wm, id);
        if let Some(frame_id) = wm.frame(id).map(crate::frame::FrameWindow::frame_id) {
            crate::drag::apply_frame_rect(wm, frame_id, rect);
        }
        set_tile_slot(wm, id, Some(slot as u32));
    }
}

fn clear_maximized<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, id: ClientId) {
    let maxed = wm
        .frames
        .get(&id)
        .map_or(false, |f| f.state().max_vert || f.state().max_horz);
    if maxed {
        crate::wmaction::set_max_state(wm, id, false, false);
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
