pub mod state;
pub mod xdnd;

use crate::manager::WindowManager;
use crate::wmaction::set_minimized_visible;
use antibox_core::backend::{DisplayBackend, StackMode};
use antibox_core::rect::Rect;

pub fn parse_restack_mode(mode: u32) -> StackMode {
    match mode {
        1 => StackMode::Below,
        2 => StackMode::TopIf,
        3 => StackMode::BottomIf,
        4 => StackMode::Opposite,
        _ => StackMode::Above,
    }
}

pub fn apply_moveresize_flags(
    mut fr: Rect,
    flags: u32,
    x: i32,
    y: i32,
    nw: i32,
    nh: i32,
) -> Rect {
    if flags & 0x100 != 0 {
        fr.x = x;
    }
    if flags & 0x200 != 0 {
        fr.y = y;
    }
    if flags & 0x400 != 0 && nw > 0 {
        fr.w = nw;
    }
    if flags & 0x800 != 0 && nh > 0 {
        fr.h = nh;
    }
    fr
}

pub fn client_message<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    mt: u32,
    d: [u32; 5],
) {
    if let Some(ncw) = wm.atoms.get("_NET_CLOSE_WINDOW") {
        if mt == ncw {
            let xid = if w == 0 { d[0] } else { w };
            let cid = match wm.cid_for_xid(xid) {
                Some(cid) => cid,
                None => return,
            };
            crate::wmaction::close_client(wm, cid);
            return;
        }
    }
    if let Some(wcs) = wm.atoms.get("WM_CHANGE_STATE") {
        if mt == wcs {
            use antibox_core::backend::hints::wm_state;
            let want_min = d[0] == wm_state::ICONIC;
            let cid = match wm.cid_for_xid(w) { Some(v) => v, None => return };
            let was_min = wm.frames.get(&cid).map_or(false, |fw| fw.state().minimized);
            if want_min != was_min && wm.frames.contains_key(&cid) {
                if let Some(fw) = wm.frame_mut(cid) {
                    fw.state_mut().minimized = want_min;
                }
                set_minimized_visible(wm, cid, want_min);
                crate::placement::set_transients_minimized(&mut wm.frames, cid, want_min);
            }
            return;
        }
    }
    if let Some(nsw) = wm.atoms.get("_NET_WM_STATE") {
        if mt == nsw {
            state::net_wm_state_request(wm, w, d[0], d[1], d[2]);
            return;
        }
    }
    if let Some(ncd) = wm.atoms.get("_NET_CURRENT_DESKTOP") {
        if mt == ncd {
            wm.activate_workspace(d[0]);
            return;
        }
    }
    if let Some(nsd) = wm.atoms.get("_NET_SHOWING_DESKTOP") {
        if mt == nsd {
            crate::wmaction::set_showing_desktop(wm, d[0] != 0);
            return;
        }
    }
    if let Some(na) = wm.atoms.get("_NET_ACTIVE_WINDOW") {
        if mt == na {
            let source = d[0];
            let cid = match wm.cid_for_xid(w) { Some(v) => v, None => return };
            let other_workspace = wm
                .frame(cid)
                .map_or(false, |f| f.workspace() != !0 && f.workspace() != wm.active_workspace());
            if source == 2 || !other_workspace {
                crate::focus::activate_window(wm, cid);
            } else {
                crate::focus::demand_attention(wm, cid);
            }
            return;
        }
    }
    if let Some(nwmp) = wm.atoms.get("_NET_WM_PING") {
        if mt == nwmp {
            if let Some(backend) = wm.backend() {
                crate::ewmh::handle_ping(backend, &wm.atoms, d[2], d[1]);
            }
            return;
        }
    }
    if let Some(nrfe) = wm.atoms.get("_NET_REQUEST_FRAME_EXTENTS") {
        if mt == nrfe {
            let decorated = wm
                .cid_for_xid(w)
                .and_then(|cid| wm.frame(cid))
                .map_or(true, super::frame::FrameWindow::decorated);
            if let Some(backend) = wm.backend() {
                crate::ewmh::set_frame_extents(backend, &wm.atoms, w, decorated);
            }
            return;
        }
    }
    if let Some(nwd) = wm.atoms.get("_NET_WM_DESKTOP") {
        if mt == nwd {
            let desktop = d[0];
            if let Some(cid) = wm.cid_for_xid(w) {
                if let Some(fw) = wm.frame_mut(cid) {
                    fw.set_workspace(desktop);
                }
            }
            if let Some(backend) = wm.backend() {
                crate::ewmh::set_wm_desktop(backend, &wm.atoms, w, desktop);
            }
            wm.apply_workspace_visibility();
            crate::focus::recover_focus(wm);
            wm.layout_for(wm.active_workspace)
                .arrange(wm, wm.active_workspace);
            return;
        }
    }
    if let Some(nrw) = wm.atoms.get("_NET_RESTACK_WINDOW") {
        if mt == nrw {
            let sibling = if d[1] == 0 { None } else { Some(d[1]) };
            let mode = parse_restack_mode(d[2]);
            if let Some(backend) = wm.backend() {
                if let Ok(handle) = backend.wrap_window(w) {
                    let _ = handle.restack(sibling, mode);
                    let _ = backend.flush();
                }
            }
            return;
        }
    }
    if let Some(nwmr) = wm.atoms.get("_NET_WM_MOVERESIZE") {
        if mt == nwmr {
            let cid = match wm.cid_for_xid(w) { Some(v) => v, None => return };
            crate::drag::start_moveresize(wm, cid, d[0] as i32, d[1] as i32, d[2]);
            return;
        }
    }
    if let Some(nmrw) = wm.atoms.get("_NET_MOVERESIZE_WINDOW") {
        if mt == nmrw {
            let flags = d[0];
            let x = d[1] as i32;
            let y = d[2] as i32;
            let nw = d[3] as i32;
            let nh = d[4] as i32;
            let new_client = match wm.cid_for_xid(w).and_then(|cid| wm.frame(cid)) {
                Some(fw) => apply_moveresize_flags(fw.client_rect(), flags, x, y, nw, nh),
                None => return,
            };
            const CFG_ALL: u16 = 0xF;
            crate::handler::configure_request(wm, w, new_client, CFG_ALL);
            return;
        }
    }
    if let Some(nwfm) = wm.atoms.get("_NET_WM_FULLSCREEN_MONITORS") {
        if mt == nwfm {
            if let Some(cid) = wm.cid_for_xid(w) {
                if let Some(fw) = wm.frame_mut(cid) {
                    fw.client_mut().fullscreen_monitors = Some([d[0], d[1], d[2], d[3]]);
                }
                crate::wmaction::reapply_fullscreen(wm, cid);
            }
        }
    }
    if let Some(xe) = wm.atoms.get("XdndEnter") {
        if mt == xe {
            xdnd::handle_xdnd_enter(wm, w, d);
            return;
        }
    }
    if let Some(xl) = wm.atoms.get("XdndLeave") {
        if mt == xl {
            xdnd::handle_xdnd_leave(wm, d);
            return;
        }
    }
    if let Some(xp) = wm.atoms.get("XdndPosition") {
        if mt == xp {
            xdnd::handle_xdnd_position(wm, w, d);
            return;
        }
    }
    if let Some(xd) = wm.atoms.get("XdndDrop") {
        if mt == xd {
            xdnd::handle_xdnd_drop(wm, w, d);
        }
    }
}

#[cfg(test)]
mod tests;
