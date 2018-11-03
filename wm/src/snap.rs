use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
use antibox_core::point::Point;
use antibox_core::rect::Rect;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Horz {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vert {
    Top,
    Bottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SnapZone {
    pub h: Option<Horz>,
    pub v: Option<Vert>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SnapDir {
    Left,
    Right,
    Up,
    Down,
}

impl SnapZone {
    pub fn is_none(&self) -> bool {
        self.h.is_none() && self.v.is_none()
    }

    pub fn dir_index(&self) -> Option<u32> {
        match (self.h, self.v) {
            (Some(Horz::Left), None) => Some(0),
            (Some(Horz::Right), None) => Some(1),
            (None, Some(Vert::Top)) => Some(2),
            (None, Some(Vert::Bottom)) => Some(3),
            (Some(Horz::Left), Some(Vert::Top)) => Some(4),
            (Some(Horz::Right), Some(Vert::Top)) => Some(5),
            (Some(Horz::Left), Some(Vert::Bottom)) => Some(6),
            (Some(Horz::Right), Some(Vert::Bottom)) => Some(7),
            (None, None) => None,
        }
    }
}

pub fn zone_rect(zone: SnapZone, area: Rect) -> Option<Rect> {
    crate::wmaction::compute_tile_directional_rect(
        zone.dir_index()?,
        area.x,
        area.y,
        area.w,
        area.h,
    )
}
pub fn compose(cur: Option<SnapZone>, dir: SnapDir) -> Option<SnapZone> {
    let mut z = cur.unwrap_or_default();
    match dir {
        SnapDir::Left => {
            z.h = if z.h == Some(Horz::Left) {
                None
            } else {
                Some(Horz::Left)
            }
        }
        SnapDir::Right => {
            z.h = if z.h == Some(Horz::Right) {
                None
            } else {
                Some(Horz::Right)
            }
        }
        SnapDir::Up => {
            z.v = if z.v == Some(Vert::Top) {
                None
            } else {
                Some(Vert::Top)
            }
        }
        SnapDir::Down => {
            z.v = if z.v == Some(Vert::Bottom) {
                None
            } else {
                Some(Vert::Bottom)
            }
        }
    }
    if !z.is_none() { Some(z) } else { None }
}

pub fn current_zone(fr: Rect, area: Rect) -> Option<SnapZone> {
    for h in [None, Some(Horz::Left), Some(Horz::Right)].iter().cloned() {
        for v in [None, Some(Vert::Top), Some(Vert::Bottom)].iter().cloned() {
            let z = SnapZone { h, v };
            if zone_rect(z, area) == Some(fr) {
                return Some(z);
            }
        }
    }
    None
}

pub fn monitor_at<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>, at: Point) -> Rect {
    let (sw, sh) = wm.backend().map_or((1, 1), |b| {
        (b.screen_width() as i32, b.screen_height() as i32)
    });
    let mons = crate::placement::monitors_for_screen(&wm.monitors, sw, sh);
    mons.iter()
        .find(|m| {
            at.x >= m.x as i32
                && at.x < m.x as i32 + m.width as i32
                && at.y >= m.y as i32
                && at.y < m.y as i32 + m.height as i32
        })
        .or_else(|| mons.first())
        .map_or(Rect::new(0, 0, sw, sh), |m| {
            Rect::new(m.x as i32, m.y as i32, m.width as i32, m.height as i32)
        })
}

pub fn snap_area<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>, at: Point) -> Rect {
    let mon = monitor_at(wm, at);
    let wa = match wm.workareas.first().cloned() {
        Some(wa) => wa,
        None => return mon,
    };
    let x = mon.x.max(wa.x);
    let y = mon.y.max(wa.y);
    let x2 = (mon.x + mon.w).min(wa.x + wa.w);
    let y2 = (mon.y + mon.h).min(wa.y + wa.h);
    if x2 > x && y2 > y {
        Rect::new(x, y, x2 - x, y2 - y)
    } else {
        mon
    }
}

pub fn keyboard_snap<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    dir: SnapDir,
) {
    let id = match wm.focused_window { Some(v) => v, None => return };
    let fr = match wm.frame(id).map(super::frame::FrameWindow::frame_rect) {
        Some(fr) => fr,
        None => return,
    };
    let center = crate::geom::center_of(fr);
    let area = snap_area(wm, center);
    let cur = current_zone(fr, area);
    if let Some(z) = compose(cur, dir) {
        let target = match zone_rect(z, area) {
            Some(t) => t,
            None => return,
        };
        if cur.is_none() {
            if let Some(fw) = wm.frame_mut(id) {
                fw.snap_saved = Some(fr);
            }
        }
        apply_snap_rect(wm, id, target);
        set_snap_zone(wm, id, Some(z));
    } else {
        let saved = wm.frame_mut(id).and_then(|fw| fw.snap_saved.take());
        set_snap_zone(wm, id, None);
        if let Some(r) = saved {
            apply_snap_rect(wm, id, r);
        }
    }
}

pub fn set_snap_zone<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
    zone: Option<SnapZone>,
) {
    let atom = wm.atoms.get("_WM_SNAP_ZONE").unwrap_or(0);
    let backend = wm.backend.clone();
    let fw = match wm.frame_mut(id) {
        Some(fw) => fw,
        None => return,
    };
    fw.snap_zone = zone;
    let cid = fw.client_xid();
    if atom == 0 {
        return;
    }
    if let Some(b) = backend.as_ref().map(|v| v.as_ref()) {
        match zone.and_then(|z| z.dir_index()) {
            Some(idx) => {
                let _ = b.change_property32(
                    antibox_core::backend::PropMode::Replace,
                    cid,
                    atom,
                    6,
                    &[idx],
                );
            }
            None => {
                let _ = b.delete_property(cid, atom);
            }
        }
    }
}

pub fn apply_snap_rect<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
    r: Rect,
) {
    let fwid = match wm.frame(id).map(super::frame::FrameWindow::frame_id) {
        Some(fid) => fid,
        None => return,
    };
    crate::drag::apply_frame_rect(wm, fwid, r);
    wm.reposition_resize_handles(id);
    if let Some(b) = wm.backend() {
        let _ = b.flush();
    }
}

#[cfg(test)]
#[path = "snap_tests.rs"]
mod tests;
