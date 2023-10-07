use crate::id::{ClientId, FrameId};
use crate::manager::WindowManager;
use crate::placement::{restack_windows, set_transients_minimized};
use crate::wmaction::mwm_allows;
use crate::wmstate::ResizeEdge;
use antibox_core::backend::hints::mwm_func;
use antibox_core::backend::{DisplayBackend, EventMask, GrabMode, PointerGrab};
use antibox_core::point::Point;
use antibox_core::rect::Rect;

fn root_pointer<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) -> Option<Point> {
    let b = wm.backend()?;
    let root = b.root().read_id();
    b.query_pointer(root)
        .ok()
        .map(|ps| Point::new(ps.root_x as i32, ps.root_y as i32))
}

pub fn button_press<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    button: u8,
    state: u16,
    p: Point,
) {
    let prev_fid = wm
        .focused_window
        .and_then(|o| wm.frame(o).map(super::frame::FrameWindow::frame_id));
    let found = wm
        .frames
        .iter()
        .find(|(_, fw)| fw.frame().id() == w)
        .map(|(&id, _)| id);
    let client_id = match found {
        Some(cid) => cid,
        None => {
            if let Some(cid) = wm.cid_for_xid(w) {
                focus_frame(wm, cid, prev_fid);
                if let Some(b) = wm.backend() {
                    let _ = b.allow_events(2, 0);
                    let _ = b.flush();
                }
            } else {
                crate::container::handle_click(wm, w);
            }
            return;
        }
    };
    if wm.dock_manager.is_dock_app(wm.xid_index.xid_of(client_id)) {
        if wm.dock_manager.is_collapsed() {
            wm.dock_manager.expand();
            if let Some(b) = wm.backend.clone() {
                wm.dock_manager.adapt_with(&*b);
            }
            return;
        }
        match wm
            .dock_manager
            .handle_button(wm.xid_index.xid_of(client_id), button, state, p)
        {
            crate::dock::DockButtonResult::GrabPointer => {
                if let Some(backend) = wm.backend.as_ref().map(|v| v.as_ref()) {
                    let (gx, gy) = wm.dock_manager.dragged_grid_pos(backend);
                    wm.dock_manager.set_drag_origin(gx, gy);
                    let _ = backend.grab_pointer(PointerGrab {
                        cursor: wm.cursors[crate::cursors::idx::MOVE],
                        ..PointerGrab::new(
                            wm.xid_index.xid_of(client_id),
                            EventMask::BUTTON_RELEASE
                                | EventMask::POINTER_MOTION
                                | EventMask::BUTTON_MOTION,
                        )
                    });
                }
            }
            crate::dock::DockButtonResult::CloseWindow(w) => {
                if let Some(cid) = wm.cid_for_xid(w) {
                    if let Some(fw) = wm.frame(cid) {
                        if fw
                            .client()
                            .has_protocol(wm.atoms.get("WM_DELETE_WINDOW").unwrap_or(0))
                        {
                            crate::ewmh::close_window(wm.backend().unwrap(), &wm.atoms, w);
                        } else if let Some(b) = wm.backend() {
                            let _ = b.destroy_window(w);
                        }
                    }
                }
                wm.dock_manager.adapt_with(wm.backend().unwrap());
            }
            crate::dock::DockButtonResult::RotateForward
            | crate::dock::DockButtonResult::RotateBackward => {
                wm.dock_manager.adapt_with(wm.backend().unwrap());
            }
            crate::dock::DockButtonResult::ShowMenu => {
                let mut items: Vec<crate::dockmenu::DockMenuAction> = Vec::new();
                let dockids: Vec<u32> = wm.dock_manager.iter().collect();
                for &id in &dockids {
                    let label = wm
                        .cid_for_xid(id)
                        .and_then(|cid| wm.frames.get(&cid))
                        .map_or_else(
                            || format!("<{}>", id),
                            |fw| {
                                let ci = fw.client().class_instance();
                                let title = fw.client().title();
                                ci.unwrap_or(title).to_string()
                            },
                        );
                    items.push(crate::dockmenu::DockMenuAction { label, window: id });
                }
                if !items.is_empty() {
                    let mut menu = crate::dockmenu::dock_menu(items);
                    if let Some(b) = wm.backend() {
                        menu.show(b, p);
                        if let Some(rb) = wm.render_backend.as_ref() {
                            menu.enable_filter(rb);
                        }
                        wm.dock_menu = Some(menu);
                    }
                }
            }
            _ => {}
        }
        return;
    }
    let cid = client_id;

    if button == 4 || button == 5 {
        let on_title = wm
            .frames
            .get(&cid)
            .map_or(false, |fw| fw.title_bar_rect().contains(p));
        if on_title {
            let shaded = wm.frames.get(&cid).map_or(false, |fw| fw.state().shaded);
            if button == 4 && !shaded {
                crate::wmaction::set_shaded(wm, cid, Some(true));
            } else if button == 5 && shaded {
                crate::wmaction::set_shaded(wm, cid, Some(false));
            }
        }
        return;
    }
    let tab_hit = wm.frames.get(&cid).and_then(|fw| {
        if fw.tab_strip_h() == 0 {
            None
        } else if let Some(t) = fw.tab_close_at_point(p.x, p.y) {
            Some((t, true))
        } else {
            fw.tab_at_point(p.x, p.y).map(|t| (t, false))
        }
    });
    if let Some((xid, close)) = tab_hit {
        if close {
            if xid == wm.xid_index.xid_of(cid) {
                crate::wmaction::close_client(wm, cid);
            } else if let Some(b) = wm.backend() {
                crate::ewmh::close_window(b, &wm.atoms, xid);
                let _ = b.flush();
            }
        } else if xid != wm.xid_index.xid_of(cid) {
            crate::wmaction::tab_select(wm, cid, xid);
        }
        return;
    }
    if let Some(btn) = wm
        .frames
        .get(&cid)
        .and_then(|fw| fw.hit_test_button(p.x, p.y))
    {
        focus_frame(wm, cid, prev_fid);
        if let Some(fw) = wm.frame_mut(cid) {
            fw.set_pressed_button(Some(btn));
        }
        if let Some(b) = wm.backend() {
            let _ = b.grab_pointer(PointerGrab::new(
                w,
                EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION | EventMask::BUTTON_MOTION,
            ));
        }
        repaint_frame(wm, FrameId(w));
        return;
    }
    let edge = wm
        .frames
        .get(&cid)
        .map_or(ResizeEdge::None, |fw| fw.hit_test_edge(p));
    if edge != ResizeEdge::None && !mwm_allows(wm, cid, mwm_func::RESIZE) {
        focus_frame(wm, cid, prev_fid);
        return;
    }
    if edge != ResizeEdge::None {
        if wm.frame(cid).map_or(false, |fw| fw.snap_zone.is_some()) {
            crate::snap::set_snap_zone(wm, cid, None);
            if let Some(fw) = wm.frame_mut(cid) {
                fw.snap_saved = None;
            }
        }
        let init_rect = wm
            .frames
            .get(&cid)
            .map(super::frame::FrameWindow::frame_rect)
            .unwrap_or_default();
        let start = root_pointer(wm).unwrap_or(p);
        if let Some(backend) = wm.backend() {
            if backend
                .grab_pointer(PointerGrab {
                    cursor: wm.cursors[crate::cursors::resize_edge_cursor(&edge)],
                    ..PointerGrab::new(
                        w,
                        EventMask::BUTTON_RELEASE
                            | EventMask::POINTER_MOTION
                            | EventMask::BUTTON_MOTION,
                    )
                })
                .is_ok()
            {
                wm.drag_state = Some((FrameId(w), start, edge, init_rect));
            }
        }
        return;
    }
    if button == 3 {
        let key = cid;
        let ws_count = wm.config.workspace_count;
        let pos = wm.frame(key).map(|fw| {
            let fr = fw.frame_rect();
            Point::new(fr.x + p.x, fr.y + p.y)
        });
        let tc = wm.theme_colours;
        let join = wm.join_candidates();
        if let Some(b) = wm.backend() {
            if let Some(pos) = pos {
                let mut menu =
                    crate::winmenu::WindowActionMenu::for_focused_client_opts(
                        ws_count,
                        &tc,
                        &join,
                        wm.focused_shaded(),
                    );
                menu.show(b, pos);
                if let Some(rb) = wm.render_backend.as_ref() {
                    menu.enable_filter(rb);
                }
                wm.win_menu = Some(menu);
            }
        }
        return;
    }
    focus_frame(wm, cid, prev_fid);
    if button == 1 {
        let sysmenu_pos = wm.frame(cid).and_then(|fw| {
            fw.sys_menu_rect().contains(p).then(|| {
                let fr = fw.frame_rect();
                let r = fw.sys_menu_rect();
                Point::new(fr.x + r.x, fr.y + r.y + r.h)
            })
        });
        if let Some(pos) = sysmenu_pos {
            let ws_count = wm.config.workspace_count;
            let tc = wm.theme_colours;
            let join = wm.join_candidates();
            if let Some(b) = wm.backend() {
                let mut menu =
                    crate::winmenu::WindowActionMenu::for_focused_client_opts(
                        ws_count,
                        &tc,
                        &join,
                        wm.focused_shaded(),
                    );
                menu.show(b, pos);
                if let Some(rb) = wm.render_backend.as_ref() {
                    menu.enable_filter(rb);
                }
                wm.win_menu = Some(menu);
            }
            return;
        }
    }
    let on_title = wm
        .frames
        .get(&cid)
        .map_or(false, |fw| fw.title_bar_rect().contains(p));
    if on_title && button == 1 {
        let now = wm.backend().map_or(0, DisplayBackend::last_event_time);
        let dbl = is_double_click(wm.last_title_click, now, cid);
        if dbl {
            wm.last_title_click = None;
            if wm.frames.get(&cid).map_or(false, |fw| fw.state().shaded) {
                crate::wmaction::set_shaded(wm, cid, Some(false));
            } else if mwm_allows(wm, cid, mwm_func::MAXIMIZE) {
                crate::wmaction::set_maximized(wm, cid);
            }
            return;
        }
        wm.last_title_click = Some((now, cid));
    }
    if on_title && mwm_allows(wm, cid, mwm_func::MOVE) {
        let start = root_pointer(wm).unwrap_or(p);
        let was_max = wm
            .frames
            .get(&cid)
            .map_or(false, |fw| fw.state().max_vert || fw.state().max_horz);
        if was_max {
            let max_rect = wm
                .frames
                .get(&cid)
                .map(super::frame::FrameWindow::frame_rect)
                .unwrap_or_default();
            crate::wmaction::set_max_state_ext(wm, cid, false, false, false);
            let (new_w, new_h) = wm.frames.get(&cid).map_or((max_rect.w, max_rect.h), |fw| {
                (fw.frame_rect().w, fw.frame_rect().h)
            });
            let frac = if max_rect.w > 0 {
                ((start.x - max_rect.x) as f32 / max_rect.w as f32).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let title_h = crate::frame::title_bar_height();
            let off_y = (start.y - max_rect.y).clamp(0, (title_h - 1).max(0));
            let nx = (start.x as f32 - frac * new_w as f32) as i32;
            let ny = start.y - off_y;
            apply_frame_rect(
                wm,
                FrameId(w),
                Rect::new(nx.max(0), ny.max(0), new_w, new_h),
            );
        }
        let init_rect = wm
            .frames
            .get(&cid)
            .map(super::frame::FrameWindow::frame_rect)
            .unwrap_or_default();
        if let Some(backend) = wm.backend() {
            if backend
                .grab_pointer(PointerGrab {
                    cursor: wm.cursors[crate::cursors::idx::MOVE],
                    ..PointerGrab::new(
                        w,
                        EventMask::BUTTON_RELEASE
                            | EventMask::POINTER_MOTION
                            | EventMask::BUTTON_MOTION,
                    )
                })
                .is_ok()
            {
                wm.drag_state = Some((FrameId(w), start, ResizeEdge::None, init_rect));
            }
        }
    }
}

fn focus_frame<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    cid: ClientId,
    prev_fid: Option<FrameId>,
) {
    let cid = crate::focus::modal_redirect(wm, cid);
    let focus_changed = wm.focused_window != Some(cid);
    wm.raise_to_top(cid);
    wm.focused_window = Some(cid);
    let backend = wm.backend.clone();
    if let Some(ref backend_arc) = backend {
        let b = backend_arc.as_ref();
        let client_xid = wm.xid_index.xid_of(cid);
        if let Some(fw) = wm.frame(cid) {
            crate::focus::give_input_focus(b, &wm.atoms, fw, client_xid);
        } else {
            crate::ewmh::set_active_window_prop(b, &wm.atoms, client_xid);
        }
        if focus_changed {
            if let Some(old) =
                prev_fid.and_then(|fid| wm.frames.values().find(|f| f.frame_id() == fid))
            {
                paint_frame_decorations(old, b, false, &wm.theme_colours, wm.config.gradients);
            }
        }
        if let Some(fw) = wm.frame(cid) {
            paint_frame_decorations(fw, b, true, &wm.theme_colours, wm.config.gradients);
        }
    }
    restack_windows(wm);
}

use antibox_core::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
static MULTI_CLICK_MS: AtomicU32 = AtomicU32::new(400);

pub fn set_multi_click_ms(ms: u32) {
    MULTI_CLICK_MS.store(ms, Ordering::Relaxed);
}

pub fn is_double_click(last: Option<(u32, ClientId)>, now: u32, cid: ClientId) -> bool {
    let threshold = MULTI_CLICK_MS.load(Ordering::Relaxed);
    last.map_or(false, |(t, win)| win == cid && now != 0 && now.wrapping_sub(t) <= threshold)
}

pub fn compute_drag_rect(init_rect: Rect, dx: i32, dy: i32, edge: ResizeEdge) -> Rect {
    if edge == ResizeEdge::None {
        Rect::new(init_rect.x + dx, init_rect.y + dy, init_rect.w, init_rect.h)
    } else {
        let mut new_rect = init_rect;
        match edge {
            ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft => {
                let nw = (init_rect.w - dx).max(100);
                new_rect.x = init_rect.x + init_rect.w - nw;
                new_rect.w = nw;
            }
            ResizeEdge::Right | ResizeEdge::TopRight | ResizeEdge::BottomRight => {
                new_rect.w = (init_rect.w + dx).max(100);
            }
            _ => {}
        }
        match edge {
            ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight => {
                let nh = (init_rect.h - dy).max(50);
                new_rect.y = init_rect.y + init_rect.h - nh;
                new_rect.h = nh;
            }
            ResizeEdge::Bottom | ResizeEdge::BottomLeft | ResizeEdge::BottomRight => {
                new_rect.h = (init_rect.h + dy).max(50);
            }
            _ => {}
        }
        new_rect
    }
}

pub fn moveresize_edge(direction: u32) -> Option<ResizeEdge> {
    Some(match direction {
        0 => ResizeEdge::TopLeft,
        1 => ResizeEdge::Top,
        2 => ResizeEdge::TopRight,
        3 => ResizeEdge::Right,
        4 | 9 => ResizeEdge::BottomRight,
        5 => ResizeEdge::Bottom,
        6 => ResizeEdge::BottomLeft,
        7 => ResizeEdge::Left,
        8 | 10 => ResizeEdge::None,
        _ => return None,
    })
}

pub fn start_moveresize<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    client_id: ClientId,
    x_root: i32,
    y_root: i32,
    direction: u32,
) {
    if direction == 11 {
        if let Some(b) = wm.backend() {
            let _ = b.ungrab_pointer(0);
            let _ = b.ungrab_keyboard(0);
        }
        wm.drag_state = None;
        return;
    }
    let edge = match moveresize_edge(direction) {
        Some(e) => e,
        None => return,
    };
    if edge != ResizeEdge::None && wm.frame(client_id).map_or(false, |fw| fw.state().shaded) {
        return;
    }
    let keyboard = direction == 9 || direction == 10;
    let (frame_id, init_rect) = match wm.frame(client_id) {
        Some(fw) => (fw.frame_id(), fw.frame_rect()),
        None => return,
    };
    let start = Point::new(x_root, y_root);
    let cursor = wm.cursors[crate::cursors::resize_edge_cursor(&edge)];
    if let Some(backend) = wm.backend() {
        if backend
            .grab_pointer(PointerGrab {
                cursor,
                ..PointerGrab::new(
                    frame_id.raw(),
                    EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION
                        | EventMask::BUTTON_MOTION,
                )
            })
            .is_ok()
        {
            if keyboard {
                let _ = backend.grab_keyboard(
                    false,
                    frame_id.raw(),
                    0,
                    GrabMode::Async,
                    GrabMode::Async,
                );
            }
            wm.drag_state = Some((frame_id, start, edge, init_rect));
        }
    }
}

pub fn constrain_resize(
    mut r: Rect,
    edge: ResizeEdge,
    hints: Option<&antibox_core::backend::SizeHints>,
) -> Rect {
    if edge == ResizeEdge::None {
        return r;
    }
    let h = match hints { Some(v) => v, None => return r  };
    let dec_w = crate::frame::border_width() * 2;
    let dec_h = crate::frame::title_block_height() + crate::frame::bottom_border_width();
    let (cw, ch) = h.constrain(r.w - dec_w, r.h - dec_h);
    let nw = cw + dec_w;
    let nh = ch + dec_h;
    if matches!(
        edge,
        ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft
    ) {
        r.x += r.w - nw;
    }
    if matches!(
        edge,
        ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight
    ) {
        r.y += r.h - nh;
    }
    r.w = nw;
    r.h = nh;
    r
}

fn try_unsnap<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, frame_id: FrameId, init_rect: Rect, start: Point, cur: Point) -> Option<(Rect, Point)> {
    let mut found = None;
    for (id, fw) in wm.frames.iter() {
        if fw.frame_id() == frame_id && fw.snap_zone.is_some() {
            found = Some((*id, fw.snap_saved));
            break;
        }
    }
    let (id, saved) = found?;
    let saved = saved?;
    let moved = (cur.x - start.x).abs() + (cur.y - start.y).abs();
    if moved < antibox_core::scale::scaled(8) {
        return None;
    }
    let gx = (start.x - init_rect.x).clamp(0, init_rect.w.max(1));
    let gy = (start.y - init_rect.y).clamp(0, init_rect.h.max(1));
    let new_gx = (gx as i64 * saved.w.max(1) as i64 / init_rect.w.max(1) as i64) as i32;
    let new_gy = gy.min((saved.h - 1).max(0));
    let restored = Rect::new(cur.x - new_gx, cur.y - new_gy, saved.w, saved.h);
    crate::snap::set_snap_zone(wm, id, None);
    if let Some(fw) = wm.frame_mut(id) {
        fw.snap_saved = None;
    }
    Some((restored, cur))
}

pub fn motion_notify<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    p: Point,
    root: Point,
) {
    if wm.dock_manager.is_dragging() {
        if let Some((app, nx, ny)) = wm.dock_manager.handle_motion(p) {
            let _ = wm
                .backend()
                .unwrap()
                .configure_window(app, &[nx as u32, ny as u32]);
            let _ = wm.backend().unwrap().flush();
        }
        return;
    }
    let (dw, start, edge, init_rect) = match wm.drag_state {
        Some(s) if s.0 == FrameId(w) => s,
        _ => return,
    };
    let cur = root;
    let (start, init_rect) = if edge == ResizeEdge::None {
        match try_unsnap(wm, dw, init_rect, start, cur) {
            Some((r, o)) => {
                wm.drag_state = Some((dw, o, edge, r));
                (o, r)
            }
            None => (start, init_rect),
        }
    } else {
        (start, init_rect)
    };

    let dx = cur.x - start.x;
    let dy = cur.y - start.y;
    let new_rect = {
        let fw = match wm.frames.values().find(|fw| fw.frame_id() == dw) {
            Some(fw) => fw,
            None => return,
        };
        let hints = fw.client().size_hints();
        constrain_resize(compute_drag_rect(init_rect, dx, dy, edge), edge, hints)
    };
    if edge == ResizeEdge::None {
        let backend = wm.backend.clone();
        if let Some(b) = backend {
            let sw = b.screen_width() as i32;
            let threshold = antibox_core::scale::scaled(10);
            if cur.x < -threshold {
                let ws = wm.active_workspace();
                if ws > 0 {
                    wm.activate_workspace(ws - 1);
                    wm.drag_state = Some((dw, Point::new(cur.x + sw, cur.y), edge, init_rect));
                    if wm.config.warp_pointer_on_edge_switch {
                        let root = b.root().read_id();
                        let _ = b.warp_pointer(0, root, Rect::ZERO, Point::new(cur.x + sw, cur.y));
                    }
                }
                return;
            }
            if cur.x > sw + threshold {
                let ws = wm.active_workspace();
                if ws + 1 < wm.config.workspace_count {
                    wm.activate_workspace(ws + 1);
                    wm.drag_state = Some((dw, Point::new(cur.x - sw, cur.y), edge, init_rect));
                    if wm.config.warp_pointer_on_edge_switch {
                        let root = b.root().read_id();
                        let _ = b.warp_pointer(0, root, Rect::ZERO, Point::new(cur.x - sw, cur.y));
                    }
                }
                return;
            }
        }
    }
    if wm.config.opaque_move {
        apply_frame_rect(wm, dw, new_rect);
    } else {
        wm.drag_pending = Some(new_rect);
        crate::drag_outline::show(wm, new_rect);
    }
    let readout = wm
        .frames
        .values()
        .find(|fw| fw.frame_id() == dw)
        .map(|fw| crate::resize_popup::readout(new_rect, edge, fw.client().size_hints()));
    if let Some(text) = readout {
        let center = crate::geom::center_of(new_rect);
        crate::resize_popup::show(wm, &text, center);
    }
    if edge == ResizeEdge::None {
        let mon = crate::snap::monitor_at(wm, cur);
        let zone = crate::snap::zone_at(cur, mon).or_else(|| crate::snap::zone_for_window(new_rect, mon));
        let target = zone.and_then(|z| {
            crate::snap::zone_rect(z, crate::snap::snap_area(wm, cur)).map(|r| (z, r))
        });
        match target {
            Some((z, r)) => crate::snap::show_preview(wm, z, r),
            None => crate::snap::clear_preview(wm),
        }
    }
}

pub(crate) fn apply_frame_rect<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    frame_id: FrameId,
    new_rect: Rect,
) {
    let backend = wm.backend.clone();
    let focused_id = wm.focused_window;
    let theme = wm.theme_colours;
    let gradients = wm.config.gradients;
    if let Some(fw) = wm.frames.values_mut().find(|fw| fw.frame_id() == frame_id) {
        let old = fw.frame_rect();
        if old == new_rect {
            return;
        }
        let resized = old.w != new_rect.w || old.h != new_rect.h;
        fw.set_frame_rect(new_rect);
        if let Some(b) = backend.as_ref().map(|v| v.as_ref()) {
            let _ = b.configure_window(
                frame_id.raw(),
                &[
                    new_rect.x as u32,
                    new_rect.y as u32,
                    new_rect.w.clamp(1, u16::MAX as i32) as u32,
                    new_rect.h.clamp(1, u16::MAX as i32) as u32,
                ],
            );
            if resized {
                let cr = fw.client_rect();
                let cx = (cr.x - new_rect.x).max(0);
                let cy = (cr.y - new_rect.y).max(0);
                let iw = cr.w.clamp(1, u16::MAX as i32) as u16;
                let ih = cr.h.clamp(1, u16::MAX as i32) as u16;
                let _ = fw
                    .client()
                    .xwindow
                    .configure(Some(cx), Some(cy), Some(iw), Some(ih));
                let focused = focused_id == Some(fw.client_id());
                paint_frame_decorations(fw, b, focused, &theme, gradients);
                fw.layout_pointer_windows(b);
            }
            send_client_configure(fw, b);
            let _ = b.flush();
        }
    }
}

pub(crate) fn send_client_configure<H: DisplayBackend + 'static + ?Sized>(
    fw: &crate::frame::FrameWindow,
    b: &H,
) {
    let cr = fw.client_rect();
    let _ = b.send_configure_notify(fw.client_xid(), cr, 0);
}

pub(crate) fn paint_frame_decorations<H: DisplayBackend + 'static + ?Sized>(
    fw: &crate::frame::FrameWindow,
    b: &H,
    focused: bool,
    theme: &crate::render::ThemeColors,
    gradients: bool,
) {
    let fr = fw.frame_rect();
    let (w, h) = (fr.w as u16, fr.h as u16);
    if w == 0 || h == 0 {
        return;
    }
    let bw = fw.effective_border().max(0) as u16;
    let top = (bw + crate::frame::title_bar_height() as u16).min(h);
    if let Ok(pm) = b.create_pixmap(w, h, b.screen_depth()) {
        if let Ok(pg) = b.create_graphics(pm) {
            let _ = crate::render::draw_frame(fw, &*pg, focused, theme, gradients);
            let mut cache = fw.gfx.borrow_mut();
            if cache.is_none() {
                if let Ok(g) = b.create_graphics(fw.frame().id()) {
                    *cache = Some(g);
                }
            }
            if let Some(wg) = cache.as_ref().map(|v| v.as_ref()) {
                if fw.title_offset() {
                    let _ = wg.copy_from(pm, Rect::px(0, 0, w, h), Point::ZERO);
                } else {
                    let _ = wg.copy_from(pm, Rect::px(0, 0, w, top), Point::ZERO);
                    if fw.tab_strip_h() > 0 {
                        let sr = fw.tab_strip_rect();
                        let _ = wg.copy_from(
                            pm,
                            Rect::px(0, sr.y as i16, w, sr.h.max(0) as u16),
                            Point::new(0, sr.y),
                        );
                    }
                    if bw > 0 {
                        let bb = fw.effective_bottom_border().max(0) as u16;
                        if h > bb {
                            let _ = wg.copy_from(
                                pm,
                                Rect::px(0, (h - bb) as i16, w, bb),
                                Point::new(0, (h - bb) as i32),
                            );
                        }
                        let _ = wg.copy_from(pm, Rect::px(0, 0, bw, h), Point::ZERO);
                        if w > bw {
                            let _ = wg.copy_from(
                                pm,
                                Rect::px((w - bw) as i16, 0, bw, h),
                                Point::new((w - bw) as i32, 0),
                            );
                        }
                    }
                }
            }
        }
        let _ = b.free_pixmap(pm);
        return;
    }
    let mut cache = fw.gfx.borrow_mut();
    if cache.is_none() {
        if let Ok(g) = b.create_graphics(fw.frame().id()) {
            *cache = Some(g);
        }
    }
    if let Some(g) = cache.as_ref().map(|v| v.as_ref()) {
        let _ = crate::render::draw_frame(fw, g, focused, theme, gradients);
    }
}

fn repaint_frame<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    frame_id: FrameId,
) {
    let backend = wm.backend.clone();
    let focused_id = wm.focused_window;
    let theme = wm.theme_colours;
    let gradients = wm.config.gradients;
    if let Some(fw) = wm.frames.values_mut().find(|fw| fw.frame_id() == frame_id) {
        if let Some(b) = backend.as_ref().map(|v| v.as_ref()) {
            let focused = focused_id == Some(fw.client_id());
            paint_frame_decorations(fw, b, focused, &theme, gradients);
            let _ = b.flush();
        }
    }
}

fn run_title_button<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    cid: ClientId,
    btn: u8,
) {
    match btn {
        2 if mwm_allows(wm, cid, mwm_func::CLOSE) => crate::wmaction::close_client(wm, cid),
        5 | 0 => {
            let was_minimized = wm.frames.get(&cid).map_or(false, |fw| fw.state().minimized);
            if !was_minimized && mwm_allows(wm, cid, mwm_func::MINIMIZE) {
                if let Some(fw) = wm.frame_mut(cid) {
                    fw.state_mut().minimized = true;
                }
                crate::wmaction::set_minimized_visible(wm, cid, true);
                set_transients_minimized(&mut wm.frames, cid, true);
            }
        }
        4 if mwm_allows(wm, cid, mwm_func::MAXIMIZE) => crate::wmaction::set_maximized(wm, cid),
        1 => {
            let maxed = wm.frames.get(&cid).map_or(false, |fw| fw.state().maximized);
            if maxed {
                crate::wmaction::set_maximized(wm, cid);
            }
            crate::wmaction::set_shaded(wm, cid, Some(false));
        }
        6 => crate::wmaction::set_shaded(wm, cid, Some(true)),
        3 => crate::wmaction::set_shaded(wm, cid, Some(false)),
        7 => open_window_menu(wm, cid),
        8 => crate::wmaction::toggle_occupy_all(wm, cid),
        _ => {}
    }
}

fn open_window_menu<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    cid: ClientId,
) {
    let pos = match wm.frame(cid).map(|fw| {
        let r = fw.frame_rect();
        let btn_x = fw
            .title_button_layout()
            .iter()
            .find(|(id, ..)| *id == 7)
            .map_or(0, |(_, _, _, b)| b.x);
        let bar_bottom = fw.effective_border() + crate::frame::title_bar_height();
        Point::new(r.x + btn_x, r.y + bar_bottom)
    }) {
        Some(pos) => pos,
        None => return,
    };
    let join = wm.join_candidates();
    let mut menu = crate::winmenu::WindowActionMenu::for_focused_client_opts(
        wm.config.workspace_count,
        &wm.theme_colours,
        &join,
        wm.focused_shaded(),
    );
    if let Some(b) = wm.backend() {
        menu.show(b, pos);
        if let Some(rb) = wm.render_backend.as_ref() {
            menu.enable_filter(rb);
        }
    }
    wm.win_menu = Some(menu);
}

pub const KBD_DRAG_STEP: i32 = 16;

pub fn arrow_delta(keysym: u32, step: i32) -> Option<(i32, i32)> {
    Some(match keysym {
        0xFF51 => (-step, 0),
        0xFF52 => (0, -step),
        0xFF53 => (step, 0),
        0xFF54 => (0, step),
        _ => return None,
    })
}

pub fn keyboard_drag<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    keysym: u32,
) -> bool {
    let (frame_id, _start, edge, init_rect) = match wm.drag_state {
        Some(s) => s,
        None => return false,
    };
    match keysym {
        0xFF1B => {
            if wm.config.opaque_move {
                apply_frame_rect(wm, frame_id, init_rect);
            }
            wm.drag_pending = None;
            end_keyboard_drag(wm);
            true
        }
        0xFF0D => {
            if !wm.config.opaque_move {
                if let Some(pending) = wm.drag_pending.take() {
                    apply_frame_rect(wm, frame_id, pending);
                }
            }
            end_keyboard_drag(wm);
            true
        }
        _ => {
            let (dx, dy) = match arrow_delta(keysym, KBD_DRAG_STEP) {
                Some(d) => d,
                None => return false,
            };
            let pending = wm.drag_pending;
            let computed = {
                let fw = match wm.frames.values().find(|fw| fw.frame_id() == frame_id) {
                    Some(fw) => fw,
                    None => return false,
                };
                let cur = pending.unwrap_or_else(|| fw.frame_rect());
                let hints = fw.client().size_hints();
                constrain_resize(compute_drag_rect(cur, dx, dy, edge), edge, hints)
            };
            if wm.config.opaque_move {
                apply_frame_rect(wm, frame_id, computed);
            } else {
                wm.drag_pending = Some(computed);
                crate::drag_outline::show(wm, computed);
            }
            let readout = wm
                .frames
                .values()
                .find(|fw| fw.frame_id() == frame_id)
                .map(|fw| crate::resize_popup::readout(computed, edge, fw.client().size_hints()));
            if let Some(text) = readout {
                let center = crate::geom::center_of(computed);
                crate::resize_popup::show(wm, &text, center);
            }
            true
        }
    }
}

fn end_keyboard_drag<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(b) = wm.backend() {
        let _ = b.ungrab_pointer(0);
        let _ = b.ungrab_keyboard(0);
    }
    wm.drag_state = None;
    wm.drag_pending = None;
    crate::snap::clear_preview(wm);
    crate::resize_popup::hide(wm);
    crate::drag_outline::hide(wm);
}

pub fn button_release<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    _w: u32,
    p: Point,
) {

    let pressed = wm
        .frames
        .iter()
        .find_map(|(&cid, fw)| fw.pressed_button().map(|b| (cid, fw.frame_id(), b)));
    if let Some((cid, fid, btn)) = pressed {
        if let Some(b) = wm.backend() {
            let _ = b.ungrab_pointer(0);
        }
        if let Some(fw) = wm.frame_mut(cid) {
            fw.set_pressed_button(None);
        }
        repaint_frame(wm, fid);
        let still_over = wm
            .frames
            .get(&cid)
            .and_then(|fw| fw.hit_test_button(p.x, p.y))
            == Some(btn);
        if still_over {
            run_title_button(wm, cid, btn);
        }
        return;
    }

    if wm.dock_manager.is_dragging() {
        if let Some(b) = wm.backend() {
            let _ = b.ungrab_pointer(0);
        }
        let backend = wm.backend.clone();
        let drop_target = backend.as_ref().map(|v| v.as_ref()).and_then(|b| {
            if let Ok(ptr) = b.query_pointer(b.root().read_id()) {
                let sw = b.screen_width() as i32;
                let sh = b.screen_height() as i32;
                wm.dock_manager
                    .compute_drop_target(ptr.root_x, ptr.root_y, sw, sh)
            } else {
                None
            }
        });
        if wm.dock_manager.handle_release(drop_target) {
            wm.dock_manager.adapt_with(wm.backend().unwrap());
        }
        return;
    }

    let prev_drag = wm.drag_state.take();
    crate::resize_popup::hide(wm);
    crate::drag_outline::hide(wm);
    let pending = wm.drag_pending.take();
    let (drag_frame_id, _start, _edge, _init_rect) = match prev_drag {
        Some(d) => d,
        None => {
            if let Some(backend) = wm.backend() {
                let _ = backend.ungrab_pointer(0);
                let _ = backend.ungrab_keyboard(0);
            }
            return;
        }
    };

    if let Some(backend) = wm.backend() {
        let _ = backend.ungrab_pointer(0);
        let _ = backend.ungrab_keyboard(0);
    }
    let snap_target = wm.snap_preview.as_ref().map(|(z, r, _)| (*z, *r));
    crate::snap::clear_preview(wm);
    let snapped = _edge == ResizeEdge::None && snap_target.is_some();
    if _edge == ResizeEdge::None {
        if let Some((zone, target)) = snap_target {
            let mut cid_opt = None;
            for (id, fw) in wm.frames.iter() {
                if fw.frame_id() == drag_frame_id {
                    cid_opt = Some(*id);
                    break;
                }
            }
            if let Some(cid) = cid_opt {
                if wm.frame(cid).map_or(false, |f| f.state().shaded) {
                    crate::wmaction::set_shaded(wm, cid, Some(false));
                }
                let needs_save = match wm.frame(cid) {
                    Some(fw) => fw.snap_zone.is_none(),
                    None => false,
                };
                if needs_save {
                    let h = wm
                        .frame(cid)
                        .map_or(_init_rect.h, |f| f.frame_rect().h);
                    if let Some(fw) = wm.frame_mut(cid) {
                        fw.snap_saved =
                            Some(Rect::new(_init_rect.x, _init_rect.y, _init_rect.w, h));
                    }
                }
                crate::snap::apply_snap_rect(wm, cid, target);
                crate::snap::set_snap_zone(wm, cid, Some(zone));
            }
        }
    }
    if !wm.config.opaque_move && !snapped {
        if let Some(pending) = pending {
            apply_frame_rect(wm, drag_frame_id, pending);
        }
    }
}

#[cfg(test)]
#[path = "drag_tests.rs"]
mod tests;
