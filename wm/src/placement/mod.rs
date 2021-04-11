use crate::frame::{border_width, bottom_border_width, title_bar_height};
use crate::frame_store::FrameStore;
use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
use antibox_core::backend::MonitorInfo;
use antibox_core::backend::Strut;
use antibox_core::point::Point;
use antibox_core::rect::Rect;

pub fn add_to_transient_chain(frames: &mut FrameStore, child_id: ClientId, owner_id: ClientId) {
    let old_first = frames.get(&owner_id).and_then(|o| o.first_transient);
    if let Some(owner) = frames.get_mut(&owner_id) {
        owner.first_transient = Some(child_id);
    }
    if let Some(old) = old_first {
        if let Some(old_fw) = frames.get_mut(&old) {
            old_fw.prev_transient = Some(child_id);
        }
    }
    if let Some(child) = frames.get_mut(&child_id) {
        child.next_transient = old_first;
        child.prev_transient = None;
    }
}

pub fn remove_from_transient_chain(frames: &mut FrameStore, child_id: ClientId) {
    let (prev, next) = frames
        .get(&child_id)
        .map(|c| (c.prev_transient, c.next_transient))
        .unwrap_or_default();
    if let Some(prev_id) = prev {
        if let Some(pf) = frames.get_mut(&prev_id) {
            pf.next_transient = next;
        }
    }
    if let Some(next_id) = next {
        if let Some(nf) = frames.get_mut(&next_id) {
            nf.prev_transient = prev;
        }
    }
    let owner_xid = frames.get(&child_id).and_then(|c| c.client.transient_for);
    if let Some(owner_xid) = owner_xid {
        let owner_id = frames.iter().find_map(|(id, fw)| {
            if fw.client().xwindow.id() == owner_xid {
                Some(*id)
            } else {
                None
            }
        });
        if let Some(owner_id) = owner_id {
            if let Some(owner) = frames.get_mut(&owner_id) {
                if owner.first_transient == Some(child_id) {
                    owner.first_transient = next;
                }
            }
        }
    }
    if let Some(child) = frames.get_mut(&child_id) {
        child.next_transient = None;
        child.prev_transient = None;
    }
}

pub fn set_transients_minimized(frames: &mut FrameStore, id: ClientId, minimized: bool) {
    let mut cur = frames.get(&id).and_then(|f| f.first_transient);
    while let Some(tid) = cur {
        if let Some(tf) = frames.get_mut(&tid) {
            tf.state.minimized = minimized;
        }
        set_transients_minimized(frames, tid, minimized);
        cur = frames.get(&tid).and_then(|f| f.next_transient);
    }
}

pub(crate) fn stack_transients_above(
    bottom_to_top: &[ClientId],
    parent_of: impl Fn(ClientId) -> Option<ClientId>,
) -> Vec<ClientId> {
    use std::collections::HashSet;
    let present: HashSet<ClientId> = bottom_to_top.iter().cloned().collect();
    let mut emitted: HashSet<ClientId> = HashSet::new();
    let mut out: Vec<ClientId> = Vec::with_capacity(bottom_to_top.len());

    fn emit(
        w: ClientId,
        base: &[ClientId],
        parent_of: &dyn Fn(ClientId) -> Option<ClientId>,
        emitted: &mut HashSet<ClientId>,
        out: &mut Vec<ClientId>,
    ) {
        if !emitted.insert(w) {
            return;
        }
        out.push(w);
        for &c in base {
            if !emitted.contains(&c) && parent_of(c) == Some(w) {
                emit(c, base, parent_of, emitted, out);
            }
        }
    }

    for &w in bottom_to_top {
        let is_child = parent_of(w).map_or(false, |p| present.contains(&p));
        if !is_child {
            emit(w, bottom_to_top, &parent_of, &mut emitted, &mut out);
        }
    }
    for &w in bottom_to_top {
        if emitted.insert(w) {
            out.push(w);
        }
    }
    out
}

pub fn restack_windows<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    use crate::wmstate::WinLayer;
    let mut sorted: Vec<(ClientId, WinLayer, usize)> = wm
        .frames
        .iter()
        .map(|(id, fw)| {
            let order = wm
                .insertion_order
                .iter()
                .position(|&x| x == *id)
                .unwrap_or(0);
            (*id, fw.layer(), order)
        })
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
    let ids: Vec<ClientId> = sorted.into_iter().map(|(id, _, _)| id).collect();
    let bottom_to_top: Vec<ClientId> = ids.iter().rev().cloned().collect();
    let root = wm.backend().map_or(0, |b| b.root().read_id());
    let bottom_to_top = stack_transients_above(&bottom_to_top, |id| {
        wm.frame(id).and_then(|f| match f.transient_for() {
            Some(t) if t == root && f.client().leader_window() != 0 => {
                wm.cid_for_xid(f.client().leader_window())
            }
            other => other.and_then(|xid| wm.cid_for_xid(xid)),
        })
    });
    let mut frame_ids: Vec<u32> = Vec::with_capacity(bottom_to_top.len() + 1);
    let mut over_bar: Vec<u32> = Vec::new();
    for id in &bottom_to_top {
        if let Some(f) = wm.frames.get(id) {
            if f.layer() >= WinLayer::Fullscreen {
                over_bar.push(f.frame().id());
            } else {
                frame_ids.push(f.frame().id());
            }
        }
    }
    frame_ids.extend(wm.above_windows.iter().cloned());
    frame_ids.extend(over_bar);
    if let Some(backend) = wm.backend() {
        let _ = backend.restack_windows(&frame_ids);
        let stacking: Vec<u32> = bottom_to_top
            .iter()
            .map(|id| wm.xid_index.xid_of(*id))
            .collect();
        crate::ewmh::update_client_list_stacking(backend, &wm.atoms, &stacking);
        let _ = backend.flush();
    }
}

pub(crate) fn requested_position(
    hints: Option<&antibox_core::backend::SizeHints>,
) -> Option<Point> {
    use antibox_core::backend::hints::size_hints_flags::{P_POSITION, US_POSITION};
    hints.and_then(|h| {
        let user_set = h.flags & US_POSITION != 0;
        let program_set = h.flags & P_POSITION != 0 && (h.x != 0 || h.y != 0);
        if user_set || program_set {
            Some(Point::new(h.x, h.y))
        } else {
            None
        }
    })
}

pub(crate) fn workarea(wm: &WindowManager<impl DisplayBackend + ?Sized>) -> Rect {
    wm.workareas.first().cloned().unwrap_or_else(|| {
        if let Some(m) = wm.monitors.first() {
            Rect::new(m.x as i32, m.y as i32, m.width as i32, m.height as i32)
        } else {
            let sw = wm.backend().map_or(1920, |b| b.screen_width() as i32);
            let sh = wm.backend().map_or(1080, |b| b.screen_height() as i32);
            Rect::new(0, 0, sw, sh)
        }
    })
}

pub(crate) fn place_dialog(
    wm: &WindowManager<impl DisplayBackend + ?Sized>,
    w: i32,
    h: i32,
) -> Point {
    let wa = workarea(wm);
    Point::new(wa.x + (wa.w - w) / 2, wa.y + (wa.h - h) / 2)
}

fn overlap_area(a: &Rect, b: &Rect) -> i64 {
    let ix = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let iy = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if ix > 0 && iy > 0 {
        ix as i64 * iy as i64
    } else {
        0
    }
}

fn coverage(x: i32, y: i32, w: i32, h: i32, others: &[Rect]) -> i64 {
    let r = Rect::new(x, y, w, h);
    others.iter().map(|o| overlap_area(&r, o)).sum()
}

pub fn cascade_layout(sizes: &[(i32, i32)], area: Rect, step: i32) -> Vec<Rect> {
    let step = step.max(1);
    let mut out = Vec::with_capacity(sizes.len());
    let (mut cx, mut cy) = (area.x, area.y);
    for &(w, h) in sizes {
        let w = w.clamp(1, area.w.max(1));
        let h = h.clamp(1, area.h.max(1));
        if cx + w > area.x + area.w || cy + h > area.y + area.h {
            cx = area.x;
            cy = area.y;
        }
        out.push(Rect::new(cx, cy, w, h));
        cx += step;
        cy += step;
    }
    out
}

pub fn smart_placement(
    window_w: i32,
    window_h: i32,
    wm: &WindowManager<impl DisplayBackend + ?Sized>,
) -> Point {
    let wa = workarea(wm);
    let (mx, my) = (wa.x, wa.y);
    let (mx2, my2) = (wa.x + wa.w, wa.y + wa.h);

    let fw = window_w + border_width() * 2;
    let fh = window_h + crate::frame::title_block_height() + bottom_border_width();

    let others: Vec<Rect> = wm
        .frames
        .values()
        .filter(|f| {
            let s = f.state();
            !s.minimized && !s.maximized && f.workspace() == wm.active_workspace
        })
        .map(super::frame::FrameWindow::frame_rect)
        .collect();

    let margin = antibox_core::scale::scaled(EDGE_MARGIN);
    let (lox, hix) = place_range(mx, mx2, fw, margin);
    let (loy, hiy) = place_range(my, my2, fh, margin);

    let xs = axis_candidates(
        lox,
        hix,
        others
            .iter()
            .flat_map(|r| vec![r.x, r.x + r.w - fw, r.x + r.w + margin, r.x - fw - margin]),
    );
    let ys = axis_candidates(
        loy,
        hiy,
        others
            .iter()
            .flat_map(|r| vec![r.y, r.y + r.h - fh, r.y + r.h + margin, r.y - fh - margin]),
    );

    let (cx0, cy0) = (mx + (mx2 - mx) / 2, my + (my2 - my) / 2);
    let mut best = Point::new(mx, my);
    let mut best_cover = i64::MAX;
    let mut best_clear = i64::MIN;
    let mut best_cdist = i64::MAX;
    for &ty in &ys {
        for &tx in &xs {
            let cover = coverage(tx, ty, fw, fh, &others);
            if cover > best_cover {
                continue;
            }

            let clear = clearance(Rect::new(tx, ty, fw, fh), wa, &others);

            let cdist = ((tx + fw / 2 - cx0).abs() + (ty + fh / 2 - cy0).abs()) as i64;
            let better = cover < best_cover
                || (cover == best_cover && clear > best_clear)
                || (cover == best_cover && clear == best_clear && cdist < best_cdist);
            if better {
                best_cover = cover;
                best_clear = clear;
                best_cdist = cdist;
                best = Point::new(tx, ty);
            }
        }
    }

    Point::new(
        best.x + border_width(),
        best.y + crate::frame::title_block_height(),
    )
}

const EDGE_MARGIN: i32 = 16;

fn place_range(lo: i32, hi: i32, size: i32, margin: i32) -> (i32, i32) {
    let max_tl = (hi - size).max(lo);
    let (a, b) = (lo + margin, max_tl - margin);
    if b >= a {
        (a, b)
    } else {
        (lo, max_tl)
    }
}

fn axis_candidates(lo: i32, hi: i32, extra: impl Iterator<Item = i32>) -> Vec<i32> {
    if hi <= lo {
        return vec![lo];
    }
    const GRID: i32 = 24;
    let step = ((hi - lo) / GRID).max(1);
    let mut v: Vec<i32> = (0..)
        .map(|i| lo + i * step)
        .take_while(|&x| x < hi)
        .collect();
    v.push(hi);
    v.push(lo + (hi - lo) / 2);
    v.extend(extra.map(|e| e.clamp(lo, hi)));
    v.sort_unstable();
    v.dedup();
    v
}

fn clearance(win: Rect, wa: Rect, others: &[Rect]) -> i64 {
    let (l, t, r, b) = (win.x, win.y, win.x + win.w, win.y + win.h);
    let (mx, my, mx2, my2) = (wa.x, wa.y, wa.x + wa.w, wa.y + wa.h);
    let mut m = (l - mx).min(mx2 - r).min(t - my).min(my2 - b) as i64;
    for o in others {
        let dx = (o.x - r).max(l - (o.x + o.w)).max(0) as i64;
        let dy = (o.y - b).max(t - (o.y + o.h)).max(0) as i64;
        let gap = if dx == 0 && dy == 0 {
            0
        } else {
            ((dx * dx + dy * dy) as f64).sqrt() as i64
        };
        m = m.min(gap);
    }
    m
}

pub(crate) fn clamp_below_struts(
    pos: Point,
    client_h: i32,
    wm: &WindowManager<impl DisplayBackend + ?Sized>,
) -> Point {
    match wm.workareas.first() {
        Some(wa) => Point::new(pos.x, clamp_window_y(pos.y, client_h, *wa)),
        None => pos,
    }
}

fn clamp_window_y(pos_y: i32, client_h: i32, wa: Rect) -> i32 {
    let bw = border_width();
    let min_y = wa.y + title_bar_height() + bw;
    let max_y = (wa.y + wa.h - client_h - bottom_border_width()).max(min_y);
    pos_y.clamp(min_y, max_y)
}

pub(crate) fn monitors_for_screen(mons: &[MonitorInfo], sw: i32, sh: i32) -> Vec<MonitorInfo> {
    let full = MonitorInfo {
        x: 0,
        y: 0,
        width: sw.max(1) as u16,
        height: sh.max(1) as u16,
    };
    if mons.len() <= 1 {
        return vec![full];
    }
    let out: Vec<MonitorInfo> = mons
        .iter()
        .filter_map(|m| {
            let mx = (m.x as i32).clamp(0, sw);
            let my = (m.y as i32).clamp(0, sh);
            let mw = (m.x as i32 + m.width as i32).min(sw) - mx;
            let mh = (m.y as i32 + m.height as i32).min(sh) - my;
            if mw > 0 && mh > 0 {
                Some(MonitorInfo {
                    x: mx as i16,
                    y: my as i16,
                    width: mw as u16,
                    height: mh as u16,
                })
            } else {
                None
            }
        })
        .collect();
    if out.is_empty() {
        vec![full]
    } else {
        out
    }
}

pub fn update_workarea_from_struts<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
) {
    let sw = wm.backend().map_or(1920, |b| b.screen_width() as i32);
    let sh = wm.backend().map_or(1080, |b| b.screen_height() as i32);
    let mons = monitors_for_screen(&wm.monitors, sw, sh);
    let mut struts: Vec<(Strut, Option<usize>)> = wm
        .frames
        .values()
        .filter_map(|fw| {
            let s = fw.client().strut().cloned()?;
            let mon = mons.iter().position(|m| {
                let mx = m.x as i32;
                let my = m.y as i32;
                let mw = m.width as i32;
                let mh = m.height as i32;
                (s.left > 0 && mx == 0)
                    || (s.right > 0 && (mx + mw) == sw)
                    || (s.top > 0 && my == 0)
                    || (s.bottom > 0 && (my + mh) == sh)
            });
            Some((s, mon))
        })
        .collect();
    let rs = wm.reserved_strut;
    if rs.left | rs.right | rs.top | rs.bottom != 0 {
        struts.push((rs, None));
    }

    let num_mons = mons.len();
    let mut workareas: Vec<Rect> = Vec::with_capacity(num_mons);
    for (mi, m) in mons.iter().enumerate() {
        let left = struts
            .iter()
            .filter(|(_, mon)| mon.map_or(true, |i| i == mi))
            .map(|(s, _)| s.left as i32)
            .max()
            .unwrap_or(0);
        let right = struts
            .iter()
            .filter(|(_, mon)| mon.map_or(true, |i| i == mi))
            .map(|(s, _)| s.right as i32)
            .max()
            .unwrap_or(0);
        let top = struts
            .iter()
            .filter(|(_, mon)| mon.map_or(true, |i| i == mi))
            .map(|(s, _)| s.top as i32)
            .max()
            .unwrap_or(0);
        let bottom = struts
            .iter()
            .filter(|(_, mon)| mon.map_or(true, |i| i == mi))
            .map(|(s, _)| s.bottom as i32)
            .max()
            .unwrap_or(0);
        let mx = m.x as i32;
        let my = m.y as i32;
        let mw = m.width as i32;
        let mh = m.height as i32;
        workareas.push(Rect::new(
            mx + left,
            my + top,
            mw.saturating_sub(left + right),
            mh.saturating_sub(top + bottom),
        ));
    }
    let workarea_changed = wm.workareas != workareas;
    wm.workareas = workareas;
    if workarea_changed {
        crate::wmaction::refit_maximized(wm);
    }

    let single_struts: Vec<Strut> = struts.iter().map(|(s, _)| *s).collect();
    crate::ewmh::update_workarea(
        wm.backend().expect("backend present"),
        &wm.atoms,
        &single_struts,
        wm.config.workspace_count,
    );
    if let Some(atom) = wm.atoms.get("_WIN_WORKAREA") {
        let mut data = Vec::with_capacity(wm.workareas.len() * 4);
        for wa in &wm.workareas {
            data.push(wa.x as u32);
            data.push(wa.y as u32);
            data.push(wa.w as u32);
            data.push(wa.h as u32);
        }
        let backend = wm.backend().expect("backend present");
        let _ = backend.change_property32(
            antibox_core::backend::PropMode::Replace,
            backend.root().read_id(),
            atom,
            6,
            &data,
        );
    }
}

#[cfg(test)]
mod tests;
