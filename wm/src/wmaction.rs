use crate::action::*;
use crate::id::{ClientId, FrameId};
use crate::manager::WindowManager;
use crate::placement;
use crate::wmstate::WinLayer;
use antibox_core::backend::hints::mwm_func;
use antibox_core::backend::{DisplayBackend, EventMask, GrabMode, PointerGrab};
use antibox_core::point::Point;
use antibox_core::rect::Rect;

fn fid<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) -> Option<ClientId> {
    wm.focused_window
}

pub(crate) fn mwm_allows<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    id: ClientId,
    func_bit: u32,
) -> bool {
    wm.frames
        .get(&id)
        .and_then(|f| f.client().mwm_hints())
        .map_or(true, |h| h.allows(func_bit))
}

use crate::geom::screen_dims as dims;

fn fullscreen_rect<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    id: ClientId,
) -> Rect {
    let (sw, sh) = dims(wm);
    let whole = Rect::new(0, 0, sw, sh);
    let fw = match wm.frame(id) {
        Some(fw) => fw,
        None => return whole,
    };
    let mons = placement::monitors_for_screen(&wm.monitors, sw, sh);
    if let Some([t, b, l, r]) = fw.client().fullscreen_monitors {
        let m = |i: u32| mons.get(i as usize);
        if let (Some(mt), Some(mb), Some(ml), Some(mr)) = (m(t), m(b), m(l), m(r)) {
            let top = mt.y as i32;
            let left = ml.x as i32;
            let right = mr.x as i32 + mr.width as i32;
            let bottom = mb.y as i32 + mb.height as i32;
            if right > left && bottom > top {
                return Rect::new(left, top, right - left, bottom - top);
            }
        }
    }
    let fr = fw.frame_rect();
    crate::snap::monitor_at(wm, crate::geom::center_of(fr))
}

fn cfg<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, w: u32, r: &[u32]) {
    if let Some(b) = wm.backend() {
        let _ = b.configure_window(w, r);
    }
}

fn workarea<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) -> (i32, i32, i32, i32) {
    let r = crate::geom::workarea(wm);
    (r.x, r.y, r.w, r.h)
}

pub fn handle_wm_action<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    action: &Action,
) {
    match action {
        Action::Window(WindowOp::Maximize) => maximize(wm),
        Action::Window(WindowOp::MaximizeVert) => maximize_axis(wm, true, false),
        Action::Window(WindowOp::MaximizeHoriz) => maximize_axis(wm, false, true),
        Action::Window(WindowOp::Minimize) => minimize(wm),
        Action::Misc(MiscOp::Show) => show(wm),
        Action::Window(WindowOp::Restore) => restore(wm),
        Action::Window(WindowOp::Fullscreen) => fullscreen(wm),
        Action::Window(WindowOp::Shade) | Action::Window(WindowOp::Rollup) => shade(wm),
        Action::Window(WindowOp::Hide) => hide(wm),
        Action::Window(WindowOp::Close) => close(wm),
        Action::Window(WindowOp::Kill) => kill(wm),
        Action::Window(WindowOp::Move) => move_win(wm),
        Action::Window(WindowOp::Resize) => resize_win(wm),
        Action::Window(WindowOp::Raise) => raise(wm),
        Action::Window(WindowOp::Lower) => lower(wm),
        Action::Window(WindowOp::Depth) => depth(wm),
        Action::Layer(LayerOp::Layer(n)) => set_layer(wm, *n),
        Action::Layer(LayerOp::AboveAll) => set_layer_named(wm, WinLayer::AboveAll),
        Action::Layer(LayerOp::Dock) => set_layer_named(wm, WinLayer::Dock),
        Action::Layer(LayerOp::Fullscreen) => set_layer_named(wm, WinLayer::Fullscreen),
        Action::Layer(LayerOp::Menu) => set_layer_named(wm, WinLayer::Menu),
        Action::Layer(LayerOp::Normal) => set_layer_named(wm, WinLayer::Normal),
        Action::Layer(LayerOp::OnTop) => set_layer_named(wm, WinLayer::OnTop),
        Action::Layer(LayerOp::Below) => set_layer_named(wm, WinLayer::Below),
        Action::Layer(LayerOp::Desktop) => set_layer_named(wm, WinLayer::Desktop),
        Action::Tile(TileOp::Cascade) => {
            wm.save_layout();
            cascade(wm);
        }
        Action::Tile(TileOp::Tile) => {
            wm.save_layout();
            tile_all(wm);
        }
        Action::Tile(TileOp::TileVertical) => {
            wm.save_layout();
            tile_vertical(wm);
        }
        Action::Tile(TileOp::TileHorizontal) => {
            wm.save_layout();
            tile_horizontal(wm);
        }
        Action::Tile(TileOp::Arrange) => {
            wm.save_layout();
            arrange(wm);
        }
        Action::Tile(TileOp::UndoArrange) => undo_arrange(wm),
        Action::Tile(TileOp::TileLeft) => tile_directional(wm, 0),
        Action::Tile(TileOp::TileRight) => tile_directional(wm, 1),
        Action::Tile(TileOp::TileTop) => tile_directional(wm, 2),
        Action::Tile(TileOp::TileBottom) => tile_directional(wm, 3),
        Action::Tile(TileOp::TileTopLeft) => tile_directional(wm, 4),
        Action::Tile(TileOp::TileTopRight) => tile_directional(wm, 5),
        Action::Tile(TileOp::TileBottomLeft) => tile_directional(wm, 6),
        Action::Tile(TileOp::TileBottomRight) => tile_directional(wm, 7),
        Action::Tile(TileOp::SnapLeft) => {
            crate::snap::keyboard_snap(wm, crate::snap::SnapDir::Left);
        }
        Action::Tile(TileOp::SnapRight) => {
            crate::snap::keyboard_snap(wm, crate::snap::SnapDir::Right);
        }
        Action::Tile(TileOp::SnapUp) => crate::snap::keyboard_snap(wm, crate::snap::SnapDir::Up),
        Action::Tile(TileOp::SnapDown) => {
            crate::snap::keyboard_snap(wm, crate::snap::SnapDir::Down);
        }
        Action::Tile(TileOp::TileCenter) => tile_center(wm),
        Action::Workspace(WorkspaceOp::MinimizeAll) => minimize_all(wm),
        Action::Workspace(WorkspaceOp::HideAll) => hide_all(wm),
        Action::Workspace(WorkspaceOp::ShowDesktop) => show_desktop(wm),
        Action::Workspace(WorkspaceOp::OccupyAllOrCurrent) => occupy_all(wm),
        Action::Workspace(WorkspaceOp::NextLayout) => next_layout(wm),
        Action::Workspace(WorkspaceOp::SetLayout(ws, idx)) => {
            if let Some(&layout) = crate::layout::Layout::ALL.get(*idx as usize) {
                wm.set_layout(*ws, layout);
            }
        }
        Action::Workspace(WorkspaceOp::WorkspaceMenu(_)) => {}
        Action::Focus(FocusOp::ClickToFocus) => set_focus_mode(wm, 1),
        Action::Focus(FocusOp::Explicit) => set_focus_mode(wm, 3),
        Action::Focus(FocusOp::MouseSloppy) => set_focus_mode(wm, 2),
        Action::Focus(FocusOp::MouseStrict) => set_focus_mode(wm, 4),
        Action::Focus(FocusOp::QuietSloppy) => set_focus_mode(wm, 5),
        Action::Focus(FocusOp::Custom) => set_focus_mode(wm, 0),
        Action::Misc(MiscOp::WinOptions) => reload_winoptions(wm),
        Action::Misc(MiscOp::ReloadKeys) => reload_keys(wm),
        _ => {}
    }
}

fn maximize<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(id) = fid(wm) {
        if mwm_allows(wm, id, mwm_func::MAXIMIZE) {
            set_maximized(wm, id);
        }
    }
}

fn maximize_axis<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    vert: bool,
    horz: bool,
) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if !mwm_allows(wm, id, mwm_func::MAXIMIZE) {
        return;
    }
    let (was_v, was_h) = wm
        .frames
        .get(&id)
        .map_or((false, false), |f| (f.state().max_vert, f.state().max_horz));
    let want_v = if vert { !was_v } else { was_v };
    let want_h = if horz { !was_h } else { was_h };
    set_max_state(wm, id, want_v, want_h);
}

pub(crate) fn set_maximized<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let full = wm
        .frames
        .get(&id)
        .map_or(false, |f| f.state().max_vert && f.state().max_horz);
    set_max_state(wm, id, !full, !full);
}

pub(crate) fn set_max_state<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
    want_vert: bool,
    want_horz: bool,
) {
    set_max_state_ext(wm, id, want_vert, want_horz, false);
}

pub(crate) fn clear_max_state<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let maxed = wm
        .frames
        .get(&id)
        .map_or(false, |f| f.state().max_vert || f.state().max_horz);
    if maxed {
        set_max_state(wm, id, false, false);
    }
}

pub(crate) fn set_max_state_ext<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
    want_vert: bool,
    want_horz: bool,
    force: bool,
) {
    if want_vert || want_horz {
        let ws = wm.frames.get(&id).map_or(0, |f| f.workspace());
        let ws = if ws == !0 { wm.active_workspace } else { ws };
        if wm.layout_for(ws).is_tiled() {
            return;
        }
    }
    let (fr, cl, was_v, was_h, cur, hints, decorated) = match wm.frame(id) {
        Some(f) => (
            f.frame().id(),
            wm.xid_index.xid_of(id),
            f.state().max_vert,
            f.state().max_horz,
            f.frame_rect(),
            f.client().size_hints().cloned(),
            f.decorated(),
        ),
        None => return,
    };
    if !force && want_vert == was_v && want_horz == was_h {
        return;
    }
    let (wx, wy, ww, wh) = workarea(wm);
    let was_any = was_v || was_h;
    let now_any = want_vert || want_horz;
    let normal = if was_any {
        wm.frame(id).and_then(|f| f.saved_rect).unwrap_or(cur)
    } else {
        cur
    };
    let tx = if want_horz { wx } else { normal.x };
    let ty = if want_vert { wy } else { normal.y };
    let mut tw = if want_horz { ww } else { normal.w };
    let mut th_ = if want_vert { wh } else { normal.h };
    let eb = if (want_vert && want_horz) || !decorated {
        0
    } else {
        crate::frame::border_width()
    };
    let title = if decorated {
        crate::frame::title_bar_height()
    } else {
        0
    };
    let eb_bottom = if (want_vert && want_horz) || !decorated {
        0
    } else {
        crate::frame::bottom_border_width()
    };
    let band = if decorated && antibox_ui::theme::title_offset_side() {
        crate::frame::title_bar_height()
    } else {
        0
    };
    let dec_w = eb * 2
        + if antibox_ui::theme::title_vertical() {
            band
        } else {
            0
        };
    let dec_h = crate::frame::top_for(eb, title)
        + eb_bottom
        + if antibox_ui::theme::title_on_bottom() {
            band
        } else {
            0
        };
    if let Some(h) = hints {
        let (cw, ch) = h.constrain((tw - dec_w).max(1), (th_ - dec_h).max(1));
        tw = cw + dec_w;
        th_ = ch + dec_h;
    }
    let target = Rect::new(tx.max(0), ty.max(0), tw.max(1), th_.max(1));
    let client_rel = if let Some(f) = wm.frame_mut(id) {
        f.state_mut().max_vert = want_vert;
        f.state_mut().max_horz = want_horz;
        f.state_mut().maximized = want_vert && want_horz;
        f.saved_rect = if now_any { Some(normal) } else { None };
        f.set_frame_rect(target);
        let cr = f.client_rect();
        (
            (cr.x - target.x).max(0) as u32,
            (cr.y - target.y).max(0) as u32,
            cr.w.max(1) as u32,
            cr.h.max(1) as u32,
        )
    } else {
        (0, 0, target.w.max(1) as u32, target.h.max(1) as u32)
    };
    cfg(
        wm,
        cl,
        &[client_rel.0, client_rel.1, client_rel.2, client_rel.3],
    );
    cfg(
        wm,
        fr,
        &[
            target.x.max(0) as u32,
            target.y.max(0) as u32,
            target.w.max(1) as u32,
            target.h.max(1) as u32,
        ],
    );
    wm.reposition_resize_handles(id);
    publish_net_wm_state(wm, id);
    crate::handler::redraw_frame_decor(wm, id);
    if let Some(b) = wm.backend() {
        let _ = b.flush();
    }
}

pub(crate) fn refit_maximized<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ids: Vec<(ClientId, bool, bool)> = wm
        .frames
        .iter()
        .filter(|(_, f)| f.state().max_vert || f.state().max_horz)
        .map(|(&id, f)| (id, f.state().max_vert, f.state().max_horz))
        .collect();
    for (id, v, h) in ids {
        set_max_state_ext(wm, id, v, h, true);
    }
}

fn place_fullscreen<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, id: ClientId) {
    let full = fullscreen_rect(wm, id);
    if let Some(fw) = wm.frame_mut(id) {
        fw.set_frame_rect(full);
    }
    cfg(
        wm,
        wm.xid_index.xid_of(id),
        &[0, 0, full.w as u32, full.h as u32],
    );
    if let Some(fw) = wm.frame(id) {
        cfg(
            wm,
            fw.frame().id(),
            &[full.x as u32, full.y as u32, full.w as u32, full.h as u32],
        );
    }
    wm.reposition_resize_handles(id);
}

pub(crate) fn refit_fullscreen<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ids: Vec<ClientId> = wm
        .frames
        .iter()
        .filter(|(_, f)| f.state().fullscreen)
        .map(|(&id, _)| id)
        .collect();
    for id in ids {
        place_fullscreen(wm, id);
    }
}

pub(crate) fn reapply_fullscreen<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let fs = wm.frames.get(&id).map_or(false, |f| f.state().fullscreen);
    if fs {
        place_fullscreen(wm, id);
    }
}

fn minimize<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if !mwm_allows(wm, id, mwm_func::MINIMIZE) {
        return;
    }
    let was = wm.frames.get(&id).map_or(false, |f| f.state().minimized);
    if let Some(fw) = wm.frame_mut(id) {
        fw.minimize();
    }
    let now = wm.frames.get(&id).map_or(false, |f| f.state().minimized);
    if now && !was {
        set_minimized_visible(wm, id, true);
        placement::set_transients_minimized(&mut wm.frames, id, true);
    } else if !now && was {
        set_minimized_visible(wm, id, false);
        placement::set_transients_minimized(&mut wm.frames, id, false);
    }
}

fn restore<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if wm.frames.get(&id).map_or(false, |f| f.state().fullscreen) {
        set_fullscreen(wm, id, false);
    }
    set_max_state(wm, id, false, false);
    if wm.frames.get(&id).map_or(false, |f| f.state().minimized) {
        if let Some(fw) = wm.frame_mut(id) {
            fw.state_mut().minimized = false;
        }
        set_minimized_visible(wm, id, false);
        placement::set_transients_minimized(&mut wm.frames, id, false);
    }
}

fn fullscreen<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(id) = fid(wm) {
        let was = wm.frames.get(&id).map_or(false, |f| f.state().fullscreen);
        set_fullscreen(wm, id, !was);
    }
}

pub(crate) fn set_fullscreen<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
    want: bool,
) {
    let (was, cur) = match wm.frame(id) {
        Some(f) => (f.state().fullscreen, f.frame_rect()),
        None => return,
    };
    if want == was {
        return;
    }
    let full = fullscreen_rect(wm, id);
    let restore = wm
        .frames
        .get(&id)
        .and_then(|f| f.saved_fullscreen_rect)
        .unwrap_or(cur);
    if let Some(fw) = wm.frame_mut(id) {
        fw.state_mut().fullscreen = want;
        if want {
            fw.saved_fullscreen_rect = Some(cur);
            fw.set_layer(WinLayer::Fullscreen);
            fw.set_frame_rect(full);
        } else {
            fw.saved_fullscreen_rect = None;
            fw.set_frame_rect(restore);
            fw.set_layer(WinLayer::from_ewmh_state(
                fw.state().above,
                fw.state().below,
                WinLayer::default_for_window_type(fw.client().window_type()),
            ));
        }
    }
    if want {
        cfg(
            wm,
            wm.xid_index.xid_of(id),
            &[0, 0, full.w as u32, full.h as u32],
        );
        if let Some(fw) = wm.frame(id) {
            cfg(
                wm,
                fw.frame().id(),
                &[full.x as u32, full.y as u32, full.w as u32, full.h as u32],
            );
        }
    } else {
        let decorated = wm
            .frame(id)
            .map_or(true, super::frame::FrameWindow::decorated);
        let bw = if decorated {
            crate::frame::border_width()
        } else {
            0
        } as u32;
        let th = if decorated {
            crate::frame::title_bar_height()
        } else {
            0
        } as u32;
        let w = restore.w.max(1) as u32;
        let h = restore.h.max(1) as u32;
        let top = crate::frame::top_for(bw as i32, th as i32) as u32;
        cfg(
            wm,
            wm.xid_index.xid_of(id),
            &[
                bw,
                top,
                w.saturating_sub(2 * bw).max(1),
                h.saturating_sub(bw + top).max(1),
            ],
        );
        if let Some(fw) = wm.frame(id) {
            cfg(
                wm,
                fw.frame().id(),
                &[restore.x.max(0) as u32, restore.y.max(0) as u32, w, h],
            );
        }
    }
    publish_net_wm_state(wm, id);
    placement::restack_windows(wm);
    crate::handler::redraw_frame_decor(wm, id);
}

fn shade<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(id) = fid(wm) {
        set_shaded(wm, id, None);
    }
}

pub(crate) fn set_shaded<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
    want: Option<bool>,
) {
    let (fr, cur, is_shaded) = match wm.frame(id) {
        Some(f) => (f.frame().id(), f.frame_rect(), f.state().shaded),
        None => return,
    };
    let target = want.unwrap_or(!is_shaded);
    if target == is_shaded {
        return;
    }
    let title_h = crate::frame::title_block_height();
    let new_h = {
        let f = match wm.frame_mut(id) { Some(v) => v, None => return };
        f.state_mut().shaded = target;
        if target {
            f.saved_shade_height = Some(cur.h);
            title_h
        } else {
            f.saved_shade_height.take().unwrap_or(cur.h)
        }
    };
    if let Some(f) = wm.frame_mut(id) {
        f.set_frame_rect(Rect::new(cur.x, cur.y, cur.w, new_h));
    }
    cfg(
        wm,
        fr,
        &[
            cur.x.max(0) as u32,
            cur.y.max(0) as u32,
            cur.w.max(1) as u32,
            new_h.max(1) as u32,
        ],
    );
    let client_id = wm.xid_index.xid_of(id);
    if target {
        wm.expect_client_unmap(client_id);
    }
    if let Some(f) = wm.frame(id) {
        if target {
            let _ = f.client().xwindow.unmap();
        } else {
            let _ = f.client().xwindow.map();
        }
    }
    publish_net_wm_state(wm, id);
    crate::handler::redraw_frame_decor(wm, id);
    if let Some(b) = wm.backend() {
        let _ = b.flush();
    }
}

fn hide<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if let Some(fw) = wm.frame_mut(id) {
        fw.state_mut().minimized = true;
    }
    set_minimized_visible(wm, id, true);
    placement::set_transients_minimized(&mut wm.frames, id, true);
}

pub(crate) fn set_minimized_visible<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
    minimized: bool,
) {
    let (fr, cl) = match wm
        .frames
        .get(&id)
        .map(|f| (f.frame().id(), wm.xid_index.xid_of(id)))
    {
        Some(v) => v,
        None => return,
    };
    if minimized {
        wm.expect_client_unmap(cl);
    }
    if let Some(b) = wm.backend() {
        if minimized {
            let _ = b.unmap_window(fr);
            let _ = b.unmap_window(cl);
        } else {
            let _ = b.map_window(cl);
            let _ = b.map_window(fr);
        }
        let _ = b.flush();
    }
    let state = if minimized {
        antibox_core::backend::hints::wm_state::ICONIC
    } else {
        antibox_core::backend::hints::wm_state::NORMAL
    };
    if let Some(b) = wm.backend() {
        crate::ewmh::set_wm_state(b, &wm.atoms, cl, state);
    }
    if minimized && wm.focused_window == wm.cid_for_xid(cl) {
        wm.focused_window = None;
        if wm.last_focused_window == wm.cid_for_xid(cl) {
            wm.last_focused_window = None;
        }
        crate::focus::recover_focus(wm);
    }
    publish_net_wm_state(wm, id);
}

fn close<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(id) = fid(wm) {
        if mwm_allows(wm, id, mwm_func::CLOSE) {
            close_client(wm, id);
        }
    }
}

pub(crate) fn publish_net_wm_state<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let st = match wm.frame(id) {
        Some(f) => *f.state(),
        None => return,
    };
    let a = |name: &str| wm.atoms.get(name);
    let managed: Vec<u32> = [
        "_NET_WM_STATE_MAXIMIZED_VERT",
        "_NET_WM_STATE_MAXIMIZED_HORZ",
        "_NET_WM_STATE_HIDDEN",
        "_NET_WM_STATE_SHADED",
        "_NET_WM_STATE_FULLSCREEN",
        "_NET_WM_STATE_DEMANDS_ATTENTION",
        "_NET_WM_STATE_ABOVE",
        "_NET_WM_STATE_BELOW",
        "_NET_WM_STATE_STICKY",
        "_NET_WM_STATE_SKIP_TASKBAR",
        "_NET_WM_STATE_SKIP_PAGER",
    ]
    .iter()
    .filter_map(|n| a(n))
    .collect();
    let mut v: Vec<u32> = wm
        .frames
        .get(&id)
        .map(|f| {
            f.client()
                .wm_state
                .iter().cloned()
                .filter(|atom| !managed.contains(atom))
                .collect()
        })
        .unwrap_or_default();
    let mut push = |atom: Option<u32>| {
        if let Some(x) = atom {
            v.push(x);
        }
    };
    if st.max_vert {
        push(a("_NET_WM_STATE_MAXIMIZED_VERT"));
    }
    if st.max_horz {
        push(a("_NET_WM_STATE_MAXIMIZED_HORZ"));
    }
    if st.minimized {
        push(a("_NET_WM_STATE_HIDDEN"));
    }
    if st.shaded {
        push(a("_NET_WM_STATE_SHADED"));
    }
    if st.fullscreen {
        push(a("_NET_WM_STATE_FULLSCREEN"));
    }
    if st.urgent {
        push(a("_NET_WM_STATE_DEMANDS_ATTENTION"));
    }
    if st.above {
        push(a("_NET_WM_STATE_ABOVE"));
    }
    if st.below {
        push(a("_NET_WM_STATE_BELOW"));
    }
    if st.sticky {
        push(a("_NET_WM_STATE_STICKY"));
    }
    if st.skip_taskbar {
        push(a("_NET_WM_STATE_SKIP_TASKBAR"));
    }
    if st.skip_pager {
        push(a("_NET_WM_STATE_SKIP_PAGER"));
    }
    let (cl, state_atom) = match (wm.frame(id), wm.atoms.get("_NET_WM_STATE")) {
        (_, Some(sa)) => (wm.xid_index.xid_of(id), sa),
        _ => return,
    };
    if let Some(f) = wm.frame_mut(id) {
        f.client_mut().wm_state = v.clone();
    }
    if let Some(b) = wm.backend() {
        const ATOM_ATOM: u32 = 4;
        let _ = b.change_property32(
            antibox_core::backend::PropMode::Replace,
            cl,
            state_atom,
            ATOM_ATOM,
            &v,
        );
        let _ = b.flush();
    }
}

pub(crate) fn close_client<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let cid = wm.xid_index.xid_of(id);
    let supports_delete = wm
        .atoms
        .get("WM_DELETE_WINDOW")
        .and_then(|a| wm.frame(id).map(|f| f.client().has_protocol(a)))
        .unwrap_or(false);
    let own = wm.self_windows.contains(&cid);
    if let Some(b) = wm.backend() {
        if supports_delete {
            crate::ewmh::close_window(b, &wm.atoms, cid);
        } else if own {
            let _ = b.destroy_window(cid);
        } else {
            let _ = b.kill_client(cid);
        }
        let _ = b.flush();
    }
}

fn kill<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    kill_client_id(wm, id);
}

pub(crate) fn kill_client_id<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let cid = wm.xid_index.xid_of(id);
    if let Some(b) = wm.backend() {
        let _ = b.kill_client(cid);
        let _ = b.flush();
    }
}

fn set_layer<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, n: i32) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if let Some(layer) = WinLayer::from_i32(n) {
        if let Some(fw) = wm.frame_mut(id) {
            fw.set_layer(layer);
        }
        placement::restack_windows(wm);
    }
}

fn raise<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    wm.raise_to_top(id);
    placement::restack_windows(wm);
}

fn lower<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    wm.lower_to_bottom(id);
    placement::restack_windows(wm);
}

fn depth<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if let Some(fw) = wm.frame_mut(id) {
        let new = match fw.layer() {
            WinLayer::Normal => WinLayer::Below,
            WinLayer::Below => WinLayer::Desktop,
            _ => WinLayer::Normal,
        };
        fw.set_layer(new);
    }
    placement::restack_windows(wm);
}

fn occupy_all<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    toggle_occupy_all(wm, id);
}

fn next_layout<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ws = wm.active_workspace;
    let next = wm.layout_for(ws).next();
    wm.set_layout(ws, next);
}

pub(crate) fn toggle_occupy_all<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let ws = wm.active_workspace();
    let target = if let Some(fw) = wm.frame_mut(id) {
        let pinned = fw.workspace() == !0;
        let t = if pinned { ws } else { !0 };
        fw.set_workspace(t);
        Some(t)
    } else {
        None
    };
    if let (Some(t), Some(b)) = (target, wm.backend()) {
        crate::ewmh::set_wm_desktop(b, &wm.atoms, wm.xid_index.xid_of(id), t);
    }
    publish_net_wm_state(wm, id);
    if let (Some(b), Some(fw)) = (wm.backend(), wm.frame(id)) {
        let focused = wm.focused_window == Some(id);
        crate::drag::paint_frame_decorations(
            fw,
            b,
            focused,
            &wm.theme_colours,
            wm.config.gradients,
        );
    }
}

fn move_win<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if !mwm_allows(wm, id, mwm_func::MOVE) {
        return;
    }
    if let Some(fw) = wm.frame(id) {
        let fr = fw.frame_rect();
        let cx = fr.x + fr.w / 2;
        let cy = fr.y + fr.h / 2;
        let fwid = fw.frame().id();
        if let Some(b) = wm.backend() {
            if b.grab_pointer(PointerGrab {
                cursor: wm.cursors[crate::cursors::idx::MOVE],
                ..PointerGrab::new(
                    fwid,
                    EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION
                        | EventMask::BUTTON_MOTION,
                )
            })
            .is_ok()
            {
                let _ = b.grab_keyboard(false, fwid, 0, GrabMode::Async, GrabMode::Async);
                wm.drag_state = Some((
                    FrameId(fwid),
                    Point::new(cx, cy),
                    crate::wmstate::ResizeEdge::None,
                    fr,
                ));
                wm.keymaps.push(crate::bindings::move_resize_keymap());
            }
        }
    }
}

fn resize_win<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if !mwm_allows(wm, id, mwm_func::RESIZE) {
        return;
    }
    if wm.frame(id).map_or(false, |fw| fw.state().shaded) {
        return;
    }
    if let Some(fw) = wm.frame(id) {
        let fr = fw.frame_rect();
        let fwid = fw.frame().id();
        if let Some(b) = wm.backend() {
            if b.grab_pointer(PointerGrab {
                cursor: wm.cursors[crate::cursors::idx::SIZE_BOTTOM_RIGHT],
                ..PointerGrab::new(
                    fwid,
                    EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION
                        | EventMask::BUTTON_MOTION,
                )
            })
            .is_ok()
            {
                let _ = b.grab_keyboard(false, fwid, 0, GrabMode::Async, GrabMode::Async);
                wm.drag_state = Some((
                    FrameId(fwid),
                    Point::new(fr.x + fr.w, fr.y + fr.h),
                    crate::wmstate::ResizeEdge::BottomRight,
                    fr,
                ));
                wm.keymaps.push(crate::bindings::move_resize_keymap());
            }
        }
    }
}

fn show<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if let Some(fw) = wm.frame_mut(id) {
        fw.state_mut().minimized = false;
    }
    if let Some(b) = wm.backend() {
        let _ = b.map_window(wm.xid_index.xid_of(id));
    }
    placement::set_transients_minimized(&mut wm.frames, id, false);
}

fn minimize_all<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    for id in wm.frames.keys().collect::<Vec<_>>() {
        let already = wm.frames.get(&id).map_or(true, |f| f.state().minimized);
        if already {
            continue;
        }
        if let Some(fw) = wm.frame_mut(id) {
            fw.state_mut().minimized = true;
        }
        set_minimized_visible(wm, id, true);
        placement::set_transients_minimized(&mut wm.frames, id, true);
    }
}

fn hide_all<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    minimize_all(wm);
}

fn show_desktop<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    set_showing_desktop(wm, !wm.showing_desktop);
}

pub(crate) fn set_showing_desktop<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    on: bool,
) {
    if on == wm.showing_desktop {
        return;
    }
    if on {
        let active = wm.active_workspace;
        let prev_focus = wm.focused_window;
        wm.focused_window = None;
        let mut hidden = Vec::new();
        let mut order: Vec<ClientId> = wm
            .insertion_order
            .iter().cloned()
            .filter(|id| wm.frames.contains_key(id))
            .collect();
        for id in wm.frames.keys().collect::<Vec<_>>() {
            if !order.contains(&id) {
                order.push(id);
            }
        }
        for id in order {
            let visible = wm.frames.get(&id).map_or(false, |f| {
                crate::manager::workspace_visible(
                    f.workspace(),
                    f.state().sticky,
                    f.state().minimized,
                    active,
                )
            });
            if visible {
                if let Some(fw) = wm.frame_mut(id) {
                    fw.state_mut().minimized = true;
                }
                set_minimized_visible(wm, id, true);
                placement::set_transients_minimized(&mut wm.frames, id, true);
                hidden.push(id);
            }
        }
        wm.desktop_focus = prev_focus.filter(|f| hidden.contains(f));
        wm.desktop_hidden = hidden;
        wm.showing_desktop = true;
    } else {
        for id in std::mem::replace(&mut wm.desktop_hidden, Vec::new()) {
            if let Some(fw) = wm.frame_mut(id) {
                fw.state_mut().minimized = false;
            }
            set_minimized_visible(wm, id, false);
            placement::set_transients_minimized(&mut wm.frames, id, false);
        }
        wm.showing_desktop = false;
        placement::restack_windows(wm);
        if let Some(f) = wm.desktop_focus.take() {
            if wm.frames.contains_key(&f) {
                crate::focus::focus_window(wm, f);
            }
        }
    }
    if let Some(b) = wm.backend() {
        crate::ewmh::update_showing_desktop(b, &wm.atoms, wm.showing_desktop);
    }
}

fn set_layer_named<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    layer: WinLayer,
) {
    let id = match fid(wm) { Some(v) => v, None => return };
    if let Some(fw) = wm.frame_mut(id) {
        fw.set_layer(layer);
    }
    placement::restack_windows(wm);
}

fn set_focus_mode<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    mode: u32,
) {
    wm.config.set_focus_mode(mode);
}

fn reload_winoptions<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    eprintln!("Reload winoptions");
    wm.win_options = crate::wmconfig::Config::load_winoptions();
}

fn reload_keys<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    eprintln!("Reload keys triggered");
    let backend = wm.backend.clone();
    if let Some(b) = backend.as_ref() {
        let prefs = crate::wmconfig::Config::load_prefs();
        let entries: Vec<crate::keys_parser::KeyEntry> = prefs
            .keys
            .iter()
            .filter_map(|(c, a)| crate::keys_parser::parse_key_binding(c, a))
            .collect();
        wm.key_bindings = crate::bindings::KeyBindings::new();
        let _ = wm.key_bindings.register_all(b, &entries);
        let _ = b.flush();
    }
}

fn cascade<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ws = wm.active_workspace;
    let order: Vec<ClientId> = wm
        .insertion_order
        .iter().cloned()
        .filter(|id| {
            wm.frames.get(id).map_or(false, |f| {
                let s = f.state();
                !s.minimized && !s.skip_taskbar && (f.workspace() == ws || f.workspace() == !0)
            })
        })
        .collect();
    if order.is_empty() {
        return;
    }

    let sizes: Vec<(i32, i32)> = order
        .iter()
        .filter_map(|id| {
            wm.frames
                .get(id)
                .map(|f| (f.frame_rect().w, f.frame_rect().h))
        })
        .collect();
    let (ax, ay, aw, ah) = workarea(wm);
    let step = crate::frame::title_block_height();
    let rects = placement::cascade_layout(&sizes, Rect::new(ax, ay, aw, ah), step);

    let bw = crate::frame::border_width();
    let th = crate::frame::title_bar_height();
    for id in &order {
        clear_max_state(wm, *id);
    }
    let mut ops = Vec::new();
    for (id, r) in order.iter().zip(rects.iter()) {
        if let Some(fw) = wm.frames.get_mut(id) {
            fw.set_frame_rect(*r);
            ops.push((*id, fw.frame().id(), *r));
        }
    }
    if let Some(b) = wm.backend() {
        let top = crate::frame::top_for(bw, th);
        for (cid, fid, r) in &ops {
            let iw = (r.w - bw * 2).max(1) as u32;
            let ih = (r.h - top - bw).max(1) as u32;
            let _ = b.configure_window(*fid, &[r.x as u32, r.y as u32, r.w as u32, r.h as u32]);
            let _ = b.configure_window(wm.xid_index.xid_of(*cid), &[bw as u32, top as u32, iw, ih]);
        }
        let _ = b.flush();
    }

    for id in &order {
        wm.raise_to_top(*id);
    }
    placement::restack_windows(wm);
    if let Some(b) = wm.backend() {
        let _ = b.flush();
    }
}

fn tileable_ids<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) -> Vec<ClientId> {
    let ws = wm.active_workspace;
    wm.insertion_order
        .iter().cloned()
        .filter(|id| {
            wm.frames.get(id).map_or(false, |f| {
                let s = f.state();
                !s.minimized && !s.skip_taskbar && (f.workspace() == ws || f.workspace() == !0)
            })
        })
        .collect()
}

fn apply_tile_rects<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    placed: impl IntoIterator<Item = (ClientId, Rect)>,
) {
    let placed: Vec<(ClientId, Rect)> = placed.into_iter().collect();
    for (id, _) in &placed {
        clear_max_state(wm, *id);
    }
    let mut ops = Vec::new();
    for (id, r) in placed {
        let client_xid = wm.xid_index.xid_of(id);
        if let Some(fw) = wm.frame_mut(id) {
            fw.set_frame_rect(r);
            let cr = fw.client_rect();
            let crel = (
                (cr.x - r.x).max(0),
                (cr.y - r.y).max(0),
                cr.w.max(1),
                cr.h.max(1),
            );
            ops.push((client_xid, fw.frame().id(), r, crel));
        }
    }
    if let Some(b) = wm.backend() {
        for &(cid, fid, r, crel) in &ops {
            let _ = b.configure_window(
                cid,
                &[crel.0 as u32, crel.1 as u32, crel.2 as u32, crel.3 as u32],
            );
            let _ = b.configure_window(fid, &[r.x as u32, r.y as u32, r.w as u32, r.h as u32]);
        }
        let _ = b.flush();
    }
}

fn tile_all<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ids = tileable_ids(wm);
    if ids.is_empty() {
        return;
    }
    let (ax, ay, aw, ah) = workarea(wm);
    let n = ids.len() as i32;
    let cols = (n as f64).sqrt().ceil() as i32;
    let rows = (n + cols - 1) / cols;
    let cw = aw / cols;
    let ch = ah / rows;
    let placed: Vec<(ClientId, Rect)> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            let x = ax + (i as i32 % cols) * cw;
            let y = ay + (i as i32 / cols) * ch;
            (id, Rect::new(x, y, cw, ch))
        })
        .collect();
    apply_tile_rects(wm, placed);
}

fn tile_vertical<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ids = tileable_ids(wm);
    if ids.is_empty() {
        return;
    }
    let (ax, ay, aw, ah) = workarea(wm);
    let cw = aw / ids.len() as i32;
    let placed: Vec<(ClientId, Rect)> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, Rect::new(ax + i as i32 * cw, ay, cw, ah)))
        .collect();
    apply_tile_rects(wm, placed);
}

fn tile_horizontal<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ids = tileable_ids(wm);
    if ids.is_empty() {
        return;
    }
    let (ax, ay, aw, ah) = workarea(wm);
    let ch = ah / ids.len() as i32;
    let placed: Vec<(ClientId, Rect)> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, Rect::new(ax, ay + i as i32 * ch, aw, ch)))
        .collect();
    apply_tile_rects(wm, placed);
}

pub fn compute_tile_directional_rect(
    dir: u32,
    wa_x: i32,
    wa_y: i32,
    wa_w: i32,
    wa_h: i32,
) -> Option<Rect> {
    match dir {
        0 => Some(Rect::new(wa_x, wa_y, wa_w / 2, wa_h)),
        1 => Some(Rect::new(wa_x + wa_w / 2, wa_y, wa_w / 2, wa_h)),
        2 => Some(Rect::new(wa_x, wa_y, wa_w, wa_h / 2)),
        3 => Some(Rect::new(wa_x, wa_y + wa_h / 2, wa_w, wa_h / 2)),
        4 => Some(Rect::new(wa_x, wa_y, wa_w / 2, wa_h / 2)),
        5 => Some(Rect::new(wa_x + wa_w / 2, wa_y, wa_w / 2, wa_h / 2)),
        6 => Some(Rect::new(wa_x, wa_y + wa_h / 2, wa_w / 2, wa_h / 2)),
        7 => Some(Rect::new(
            wa_x + wa_w / 2,
            wa_y + wa_h / 2,
            wa_w / 2,
            wa_h / 2,
        )),
        _ => None,
    }
}

pub fn compute_tile_center_rect(wa_w: i32, wa_h: i32) -> Rect {
    let cw = wa_w * 2 / 3;
    let ch = wa_h * 2 / 3;
    let cx = (wa_w - cw) / 2;
    let cy = (wa_h - ch) / 2;
    Rect::new(cx, cy, cw, ch)
}

fn tile_directional<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, dir: u32) {
    let id = match fid(wm) { Some(v) => v, None => return };
    let (wa_x, wa_y, wa_w, wa_h) = workarea(wm);
    let r = match compute_tile_directional_rect(dir, wa_x, wa_y, wa_w, wa_h) {
        Some(r) => r,
        None => return,
    };
    let frame_id = match wm.frame(id).map(super::frame::FrameWindow::frame_id) {
        Some(fid) => fid,
        None => return,
    };
    clear_max_state(wm, id);
    crate::drag::apply_frame_rect(wm, frame_id, r);
    wm.reposition_resize_handles(id);
    if let Some(b) = wm.backend() {
        let _ = b.flush();
    }
}

fn tile_center<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let id = match fid(wm) { Some(v) => v, None => return };
    let (_x, _y, wa_w, wa_h) = workarea(wm);
    let r = compute_tile_center_rect(wa_w, wa_h);
    let frame_id = match wm.frame(id).map(super::frame::FrameWindow::frame_id) {
        Some(fid) => fid,
        None => return,
    };
    clear_max_state(wm, id);
    crate::drag::apply_frame_rect(wm, frame_id, r);
    wm.reposition_resize_handles(id);
    if let Some(b) = wm.backend() {
        let _ = b.flush();
    }
}

fn arrange<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    tile_all(wm);
}

fn undo_arrange<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    eprintln!("Undo arrange — restoring previous layout");
    wm.restore_layout();
}

#[cfg(test)]
#[path = "wmaction_tests.rs"]
mod tests;
