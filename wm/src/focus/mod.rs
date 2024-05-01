use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;

pub fn next_workspace_with_windows<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    current: u32,
    forward: bool,
) -> Option<u32> {
    let n = wm.config.workspace_count;
    if n < 2 {
        return None;
    }
    for i in 1..n {
        let ws = if forward {
            (current + i) % n
        } else {
            (current + n - i) % n
        };
        if wm
            .frames
            .values()
            .any(|fw| fw.workspace() == ws || fw.workspace() == !0)
        {
            return Some(ws);
        }
    }
    None
}

const fn should_focus_on_enter<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
) -> bool {
    if wm.config.focus_follows_mouse && !wm.config.click_to_focus {
        return true;
    }
    match wm.config.focus_mode {
        1 | 3 => false,
        2 | 4 | 5 => true,
        _ => wm.config.focus_follows_mouse,
    }
}

const fn should_raise_on_focus<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
) -> bool {
    !matches!(wm.config.focus_mode, 4 | 5)
}

pub fn focus_window<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    client_id: ClientId,
) {
    if !wm.frames.contains_key(&client_id) {
        return;
    }
    let client_id = modal_redirect(wm, client_id);
    let backend = wm.backend.clone();
    let prev = wm.focused_window.replace(client_id);
    let changed = prev != Some(client_id);
    if changed {
        wm.last_focused_window = prev;
    }
    let raise = should_raise_on_focus(wm);
    if let Some(b) = backend.as_ref().map(AsRef::as_ref) {
        if let Some(fw) = wm.frame(client_id) {
            give_input_focus(b, &wm.atoms, fw, wm.xid_index.xid_of(client_id));
            if changed {
                if let Some(old) = prev.and_then(|o| wm.frame(o)) {
                    crate::drag::paint_frame_decorations(
                        old,
                        b,
                        false,
                        &wm.theme_colours,
                        wm.config.gradients,
                    );
                }
            }
            crate::drag::paint_frame_decorations(
                fw,
                b,
                true,
                &wm.theme_colours,
                wm.config.gradients,
            );
            let _ = b.flush();
        }
    }
    if wm.config.mouse_follows_focus {
        if let Some(b) = backend.as_ref().map(AsRef::as_ref) {
            if let Some(fw) = wm.frame(client_id) {
                let r = fw.frame_rect();
                let _ = b.warp_pointer(
                    0,
                    fw.frame().id(),
                    antibox_core::rect::Rect::ZERO,
                    antibox_core::point::Point::new(r.w / 2, r.h / 2),
                );
            }
        }
    }
    if raise && wm.frames.contains_key(&client_id) {
        wm.raise_to_top(client_id);
        crate::placement::restack_windows(wm);
    }
}

pub(crate) fn give_input_focus<H: DisplayBackend + 'static + ?Sized>(
    b: &H,
    atoms: &antibox_core::backend::AtomManager,
    fw: &crate::frame::FrameWindow,
    client_id: u32,
) {
    let has_take_focus = atoms
        .get("WM_TAKE_FOCUS")
        .is_some_and(|a| fw.client().has_protocol(a));
    let (set_input, send_take) = icccm_focus_actions(fw.client().wm_hints(), has_take_focus);
    if set_input {
        let _ = b.set_input_focus(1, client_id, 0);
    }
    if send_take {
        crate::ewmh::send_take_focus(b, atoms, client_id, b.last_event_time());
    }
    crate::ewmh::set_active_window_prop(b, atoms, client_id);
}

pub fn activate_window<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: ClientId,
) {
    let w = modal_redirect(wm, w);
    if !wm.frames.contains_key(&w) {
        return;
    }
    let ws = wm.frame(w).map(super::frame::FrameWindow::workspace);
    if let Some(ws) = ws {
        if ws != !0 && ws != wm.active_workspace() {
            wm.activate_workspace(ws);
        }
    }
    let was_min = wm.frames.get(&w).is_some_and(|f| f.state().minimized);
    if was_min {
        if let Some(f) = wm.frame_mut(w) {
            f.state_mut().minimized = false;
        }
        crate::wmaction::set_minimized_visible(wm, w, false);
        crate::placement::set_transients_minimized(&mut wm.frames, w, false);
    }
    let prev = wm.focused_window.replace(w);
    wm.raise_to_top(w);
    crate::placement::restack_windows(wm);
    if let Some(backend) = wm.backend() {
        if let Some(fw) = wm.frame(w) {
            give_input_focus(backend, &wm.atoms, fw, wm.xid_index.xid_of(w));
        }
    }
    if let Some(p) = prev {
        crate::handler::redraw_frame_decor(wm, p);
    }
    crate::handler::redraw_frame_decor(wm, w);
}

pub fn demand_attention<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: ClientId,
) {
    let Some(attn) = wm.atoms.get("_NET_WM_STATE_DEMANDS_ATTENTION") else { return };
    let backend = wm.backend.clone();
    if let Some(fw) = wm.frames.get_mut(&w) {
        if let Some(b) = backend.as_ref().map(AsRef::as_ref) {
            fw.client_mut().net_state_request(b, &wm.atoms, 1, attn, 0);
        }
        fw.state_mut().urgent = true;
    }
}

pub(crate) fn modal_redirect<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    w: ClientId,
) -> ClientId {
    let Some(modal) = wm.atoms.get("_NET_WM_STATE_MODAL") else { return w };
    let mut target = w;
    for _ in 0..64 {
        let child = wm.frames.iter().find_map(|(&id, f)| {
            if id != target
                && f.transient_for() == Some(wm.xid_index.xid_of(target))
                && f.client().wm_state().contains(&modal)
            {
                Some(id)
            } else {
                None
            }
        });
        match child {
            Some(c) => target = c,
            None => break,
        }
    }
    target
}

pub(crate) fn icccm_focus_actions(
    wm_hints: Option<&antibox_core::backend::WmHints>,
    has_take_focus: bool,
) -> (bool, bool) {
    use antibox_core::backend::hints::wm_hints_flags::INPUT_HINT;
    let wants_input = wm_hints.map_or(true, |h| {
        if h.flags & INPUT_HINT != 0 {
            h.input
        } else {
            true
        }
    });
    (wants_input, has_take_focus)
}

pub fn enter_notify<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, window: u32) {
    if !should_focus_on_enter(wm) {
        return;
    }
    let target = wm.cid_for_xid(window).or_else(|| {
        wm.frames
            .iter()
            .find(|(_, fw)| fw.frame().id() == window)
            .map(|(&id, _)| id)
    });
    if let Some(id) = target {
        focus_window(wm, id);
    }
}

pub fn cycle_focus<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, forward: bool) {
    if wm.frames.is_empty() {
        return;
    }
    let ids: Vec<ClientId> = wm
        .insertion_order
        .iter()
        .filter(|id| wm.frames.contains_key(id)).copied()
        .collect();
    if ids.is_empty() {
        return;
    }
    let next = match wm
        .focused_window
        .and_then(|w| ids.iter().position(|&id| id == w))
    {
        Some(i) if forward => (i + 1) % ids.len(),
        Some(i) => (i + ids.len() - 1) % ids.len(),
        None => 0,
    };
    let new_focus = ids[next];
    let backend = wm.backend.clone();
    let prev = wm.focused_window.replace(new_focus);
    wm.last_focused_window = prev;
    if let Some(b) = backend.as_ref().map(AsRef::as_ref) {
        if let Some(fw) = wm.frame(new_focus) {
            give_input_focus(b, &wm.atoms, fw, wm.xid_index.xid_of(new_focus));
        }
    }
    if let Some(old) = prev {
        crate::handler::redraw_frame_decor(wm, old);
    }
    crate::handler::redraw_frame_decor(wm, new_focus);
    wm.raise_to_top(new_focus);
    crate::placement::restack_windows(wm);
}

pub fn recover_focus<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let ws = wm.active_workspace;
    if let Some(focused) = wm.focused_window {
        if wm
            .frames
            .get(&focused)
            .is_some_and(|f| f.workspace() == ws || f.workspace() == !0 || f.state().sticky)
        {
            return;
        }
    }
    let usable = |id: ClientId| {
        wm.frames.get(&id).is_some_and(|f| {
            !f.state().minimized && (f.workspace() == ws || f.workspace() == !0 || f.state().sticky)
        })
    };
    let target = wm
        .last_focused_window
        .filter(|&id| usable(id))
        .or_else(|| {
            wm.insertion_order
                .iter()
                .rev().copied()
                .find(|&id| usable(id))
        })
        .or_else(|| wm.frames.keys().find(|&id| usable(id)));
    if let Some(id) = target {
        focus_window(wm, id);
    } else if let Some(b) = wm.backend.clone() {
        let prev = wm.focused_window.take();
        wm.last_focused_window = prev.or(wm.last_focused_window);
        let _ = b.set_input_focus(1, b.root().read_id(), 0);
        if let Some(old) = prev.and_then(|o| wm.frame(o)) {
            crate::drag::paint_frame_decorations(
                old,
                &*b,
                false,
                &wm.theme_colours,
                wm.config.gradients,
            );
        }
        crate::ewmh::set_active_window_prop(&*b, &wm.atoms, 0);
    }
}

#[cfg(test)]
mod tests;
