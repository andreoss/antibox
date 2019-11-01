use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::{DisplayBackend, EventMask, WindowHandle, WmWindowClass};
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

pub(crate) type Preview = (SnapZone, Rect, Vec<Box<dyn WindowHandle>>);

fn snap_margin() -> i32 {
    antibox_core::scale::scaled(8).max(2)
}

pub fn zone_at(p: Point, mon: Rect) -> Option<SnapZone> {
    let m = snap_margin();
    let corner = (mon.w.min(mon.h) / 4).max(m * 4);
    let near_l = p.x <= mon.x + m;
    let near_r = p.x >= mon.x + mon.w - 1 - m;
    let near_t = p.y <= mon.y + m;
    let near_b = p.y >= mon.y + mon.h - 1 - m;
    let mut z = SnapZone::default();
    if near_l {
        z.h = Some(Horz::Left);
    } else if near_r {
        z.h = Some(Horz::Right);
    }
    if near_t {
        z.v = Some(Vert::Top);
    } else if near_b {
        z.v = Some(Vert::Bottom);
    }
    if z.h.is_some() && z.v.is_none() {
        if p.y <= mon.y + corner {
            z.v = Some(Vert::Top);
        } else if p.y >= mon.y + mon.h - corner {
            z.v = Some(Vert::Bottom);
        }
    } else if z.v.is_some() && z.h.is_none() {
        if p.x <= mon.x + corner {
            z.h = Some(Horz::Left);
        } else if p.x >= mon.x + mon.w - corner {
            z.h = Some(Horz::Right);
        }
    }
    if z.is_none() {
        None
    } else {
        Some(z)
    }
}

pub fn zone_for_window(fr: Rect, mon: Rect) -> Option<SnapZone> {
    let m = snap_margin();
    let over_l = mon.x + m - fr.x;
    let over_r = (fr.x + fr.w) - (mon.x + mon.w - m);
    let over_t = mon.y + m - fr.y;
    let over_b = (fr.y + fr.h) - (mon.y + mon.h - m);
    let mut z = SnapZone::default();
    if over_l > 0 || over_r > 0 {
        if over_l >= over_r {
            z.h = Some(Horz::Left);
        } else {
            z.h = Some(Horz::Right);
        }
    }
    if over_t > 0 || over_b > 0 {
        if over_t >= over_b {
            z.v = Some(Vert::Top);
        } else {
            z.v = Some(Vert::Bottom);
        }
    }
    if z.is_none() {
        None
    } else {
        Some(z)
    }
}

pub fn show_preview<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, zone: SnapZone, rect: Rect) {
    if let Some((ref z, ref r, _)) = wm.snap_preview {
        if *z == zone && *r == rect {
            return;
        }
    }
    clear_preview(wm);
    let b = match wm.backend.clone() {
        Some(b) => b,
        None => return,
    };
    let t = antibox_core::scale::scaled(3).max(2) as u16;
    let w = rect.w.max(1) as u16;
    let h = rect.h.max(1) as u16;
    let mut wins: Vec<Box<dyn WindowHandle>> = Vec::with_capacity(4);
    for (sx, sy, sw, sh) in crate::drag_outline::ring_rectangles(w, h, t) {
        let strip = Rect::new(
            rect.x + sx as i32,
            rect.y + sy as i32,
            (sw as i32).max(1),
            (sh as i32).max(1),
        );
        let win = match b.create_window(b.root().as_parent(), strip, WmWindowClass::InputOutput, true, EventMask::EXPOSURE) {
            Ok(win) => win,
            Err(_) => continue,
        };
        let _ = win.map();
        let _ = win.raise();
        if let Ok(g) = b.create_graphics(win.id()) {
            let _ = g.set_foreground(antibox_ui::theme::sel_line());
            let _ = g.fill_rect(0, 0, sw.max(1), sh.max(1));
        }
        wins.push(win);
    }
    let _ = b.flush();
    wm.snap_preview = Some((zone, rect, wins));
}

pub fn clear_preview<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some((_, _, wins)) = wm.snap_preview.take() {
        for win in wins {
            let _ = win.unmap();
            let _ = win.destroy();
        }
        if let Some(b) = wm.backend() {
            let _ = b.flush();
        }
    }
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
    crate::wmaction::clear_max_state(wm, id);
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
