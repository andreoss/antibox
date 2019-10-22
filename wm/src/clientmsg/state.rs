use antibox_core::backend::DisplayBackend;

use crate::manager::WindowManager;
use crate::placement::restack_windows;

pub fn net_wm_state_request<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    action: u32,
    a1: u32,
    a2: u32,
) {
    let backend = wm.backend.clone();
    let cid = match wm.cid_for_xid(w) { Some(v) => v, None => return  };
    let a_maxv = wm.atoms.get("_NET_WM_STATE_MAXIMIZED_VERT");
    let a_maxh = wm.atoms.get("_NET_WM_STATE_MAXIMIZED_HORZ");
    let a_shade = wm.atoms.get("_NET_WM_STATE_SHADED");
    let a_full = wm.atoms.get("_NET_WM_STATE_FULLSCREEN");
    let (a1, a2) = (a1, a2);

    let (cur_v, cur_h, cur_shade, cur_full) = match wm.frame(cid) {
        Some(f) => (
            f.state().max_vert,
            f.state().max_horz,
            f.state().shaded,
            f.state().fullscreen,
        ),
        None => return,
    };
    let ws = wm.frame(cid).map_or(0, |f| f.workspace());
    let ws = if ws == !0 { wm.active_workspace } else { ws };
    let tiled = wm.layout_for(ws).is_tiled();
    let want = |cur: bool| match action {
        0 => false,
        1 => true,
        _ => !cur,
    };
    let (mut want_v, mut want_h) = (cur_v, cur_h);
    let (mut max_changed, mut do_shade, mut do_full) = (false, None, None);
    for atom in [a1, a2].iter().cloned() {
        if atom == 0 {
            continue;
        }
        if Some(atom) == a_maxv {
            want_v = want(cur_v);
            max_changed = true;
        } else if Some(atom) == a_maxh {
            want_h = want(cur_h);
            max_changed = true;
        } else if Some(atom) == a_shade {
            do_shade = Some(want(cur_shade));
        } else if Some(atom) == a_full {
            do_full = Some(want(cur_full));
        }
    }

    if let Some(ref b) = backend {
        if let Some(fw) = wm.frames.get_mut(&cid) {
            fw.client_mut()
                .net_state_request(b.as_ref(), &wm.atoms, action, a1, a2);
            if tiled {
                fw.client_mut().net_state_request(
                    b.as_ref(),
                    &wm.atoms,
                    0,
                    a_maxv.unwrap_or(0),
                    a_maxh.unwrap_or(0),
                );
            }
        }
    }
    if let Some(want_full) = do_full {
        crate::wmaction::set_fullscreen(wm, cid, want_full);
    }
    if max_changed && (want_v != cur_v || want_h != cur_h) {
        crate::wmaction::set_max_state(wm, cid, want_v, want_h);
    }
    if let Some(want_shade) = do_shade {
        crate::wmaction::set_shaded(wm, cid, Some(want_shade));
    }
    let was_sticky = wm.frame(cid).map_or(false, |f| f.state().sticky);
    if let Some(fw) = wm.frames.get_mut(&cid) {
        fw.sync_state_from_ewmh(&wm.atoms);
    }
    let now_sticky = wm.frame(cid).map_or(false, |f| f.state().sticky);
    if now_sticky != was_sticky {
        wm.apply_workspace_visibility();
        crate::focus::recover_focus(wm);
    }
    if let Some(ref b) = backend {
        if let Some(fw) = wm.frame(cid) {
            crate::ewmh::update_allowed_actions(
                b.as_ref(),
                &wm.atoms,
                fw.client_xid(),
                fw.client().size_hints(),
                fw.client().mwm_hints(),
            );
        }
    }
    restack_windows(wm);
}
