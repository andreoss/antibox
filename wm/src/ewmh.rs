use antibox_core::backend::{DisplayBackend, PropMode, Strut};

const ATOM_ATOM: u32 = 4;
const ATOM_CARDINAL: u32 = 6;
const ATOM_STRING: u32 = 31;
const ATOM_WINDOW: u32 = 33;

fn utf8_type(atoms: &antibox_core::backend::AtomManager) -> u32 {
    atoms.get("UTF8_STRING").unwrap_or(ATOM_STRING)
}

pub fn set_prop32<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    name: &str,
    win: u32,
    type_atom: u32,
    data: &[u32],
) {
    if let Some(atom) = atoms.get(name) {
        let _ = backend.change_property32(PropMode::Replace, win, atom, type_atom, data);
    }
}

pub fn update_client_list<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    frame_ids: &[u32],
) {
    set_prop32(
        backend,
        atoms,
        "_NET_CLIENT_LIST",
        backend.root().read_id(),
        ATOM_WINDOW,
        frame_ids,
    );
}

pub fn update_client_list_stacking<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    frame_ids: &[u32],
) {
    set_prop32(
        backend,
        atoms,
        "_NET_CLIENT_LIST_STACKING",
        backend.root().read_id(),
        ATOM_WINDOW,
        frame_ids,
    );
}

pub fn set_active_window<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
) {
    set_active_window_prop(backend, atoms, window);
    let _ = backend.set_input_focus(1, window, 0);
}

pub fn set_active_window_prop<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
) {
    let data = [window, 0, 0, 0, 0];
    set_prop32(
        backend,
        atoms,
        "_NET_ACTIVE_WINDOW",
        backend.root().read_id(),
        ATOM_WINDOW,
        &data,
    );
}

pub fn send_take_focus<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
    timestamp: u32,
) {
    if let (Some(take), Some(proto)) = (atoms.get("WM_TAKE_FOCUS"), atoms.get("WM_PROTOCOLS")) {
        let data = [take, timestamp, 0, 0, 0];
        let _ = backend.send_event(false, window, 0, proto, &data);
    }
}

pub fn set_wm_state<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
    state: u32,
) {
    if let Some(atom) = atoms.get("WM_STATE") {
        let data = [state, 0];
        let _ = backend.change_property32(PropMode::Replace, window, atom, atom, &data);
    }
}

pub fn close_window<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
) -> bool {
    let wm_delete = match atoms.get("WM_DELETE_WINDOW") {
        Some(a) => a,
        None => return false,
    };
    let wm_protos = match atoms.get("WM_PROTOCOLS") {
        Some(a) => a,
        None => return false,
    };
    let data = [wm_delete, backend.last_event_time(), 0, 0, 0];
    let _ = backend.send_event(false, window, 0, wm_protos, &data);
    let _ = backend.flush();
    true
}

pub fn init_ewmh<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    workspace_count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = backend.root().read_id();
    let wm_win = backend
        .create_window(
            root,
            antibox_core::rect::Rect::new(0, 0, 1, 1),
            antibox_core::backend::WmWindowClass::InputOutput,
            true,
            antibox_core::backend::EventMask::NO_EVENT,
        )?
        .id();

    let wm_check = atoms.get("_NET_SUPPORTING_WM_CHECK");
    if let Some(atom) = wm_check {
        backend.change_property32(PropMode::Replace, root, atom, ATOM_WINDOW, &[wm_win])?;
        backend.change_property32(PropMode::Replace, wm_win, atom, ATOM_WINDOW, &[wm_win])?;
    }
    if let Some(atom) = atoms.get("_NET_WM_NAME") {
        let name = b"antibox\0";
        let utf8 = utf8_type(atoms);
        backend.change_property8(PropMode::Replace, wm_win, atom, utf8, name)?;
        backend.change_property8(PropMode::Replace, root, atom, utf8, name)?;
    }
    if let Some(atom) = atoms.get("_NET_SUPPORTED") {
        let supported = atoms.supported_list();
        backend.change_property32(PropMode::Replace, root, atom, ATOM_ATOM, &supported)?;
    }
    if let Some(atom) = atoms.get("_NET_NUMBER_OF_DESKTOPS") {
        backend.change_property32(
            PropMode::Replace,
            root,
            atom,
            ATOM_CARDINAL,
            &[workspace_count],
        )?;
    }
    if let Some(atom) = atoms.get("_NET_CURRENT_DESKTOP") {
        backend.change_property32(PropMode::Replace, root, atom, ATOM_CARDINAL, &[0])?;
    }
    if let Some(atom) = atoms.get("_NET_DESKTOP_GEOMETRY") {
        backend.change_property32(
            PropMode::Replace,
            root,
            atom,
            ATOM_CARDINAL,
            &[
                backend.screen_width() as u32,
                backend.screen_height() as u32,
            ],
        )?;
    }
    if let Some(atom) = atoms.get("_NET_DESKTOP_VIEWPORT") {
        backend.change_property32(PropMode::Replace, root, atom, ATOM_CARDINAL, &[0, 0])?;
    }
    if let Some(atom) = atoms.get("_NET_DESKTOP_LAYOUT") {
        backend.change_property32(
            PropMode::Replace,
            root,
            atom,
            ATOM_CARDINAL,
            &[0, workspace_count, 1, 0],
        )?;
    }
    if let Some(atom) = atoms.get("_NET_SHOWING_DESKTOP") {
        backend.change_property32(PropMode::Replace, root, atom, ATOM_CARDINAL, &[0])?;
    }
    backend.map_window(wm_win)?;
    backend.flush()?;
    Ok(())
}

pub fn update_current_desktop<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    desktop: u32,
) {
    set_prop32(
        backend,
        atoms,
        "_NET_CURRENT_DESKTOP",
        backend.root().read_id(),
        ATOM_CARDINAL,
        &[desktop],
    );
}

pub fn update_showing_desktop<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    showing: bool,
) {
    set_prop32(
        backend,
        atoms,
        "_NET_SHOWING_DESKTOP",
        backend.root().read_id(),
        ATOM_CARDINAL,
        &[showing as u32],
    );
}

pub fn parse_desktop_names(data: &[u8]) -> Vec<String> {
    let mut parts: Vec<String> = data
        .split(|&b| b == 0)
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    if parts.last().map_or(false, String::is_empty) {
        parts.pop();
    }
    parts
}

pub fn update_desktop_names<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    names: &[String],
) {
    if let Some(atom) = atoms.get("_NET_DESKTOP_NAMES") {
        let mut data = Vec::new();
        for name in names {
            data.extend_from_slice(name.as_bytes());
            data.push(0);
        }
        let _ = backend.change_property8(
            PropMode::Replace,
            backend.root().read_id(),
            atom,
            utf8_type(atoms),
            &data,
        );
    }
}

pub fn handle_ping<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
    timestamp: u32,
) {
    if let Some(ping) = atoms.get("_NET_WM_PING") {
        if let Some(proto) = atoms.get("WM_PROTOCOLS") {
            let data = [ping, timestamp, window, 0, 0];
            let _ = backend.send_event(false, window, 0, proto, &data);
        }
    }
}

pub fn set_frame_extents<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
    decorated: bool,
) {
    let data = if decorated {
        let bw = crate::frame::border_width() as u32;
        let bb = crate::frame::bottom_border_width() as u32;
        [bw, bw, crate::frame::title_block_height() as u32, bb]
    } else {
        [0, 0, 0, 0]
    };
    set_prop32(
        backend,
        atoms,
        "_NET_FRAME_EXTENTS",
        window,
        ATOM_CARDINAL,
        &data,
    );
}

pub fn allowed_action_names(
    resizable: bool,
    mwm: Option<&antibox_core::backend::MwmHints>,
) -> Vec<&'static str> {
    use antibox_core::backend::hints::mwm_func;
    let func = |bit: u32| -> bool { mwm.map_or(true, |h| h.allows(bit)) };
    let can_move = func(mwm_func::MOVE);
    let can_min = func(mwm_func::MINIMIZE);
    let can_close = func(mwm_func::CLOSE);
    let can_resize = resizable && func(mwm_func::RESIZE);
    let can_max = resizable && func(mwm_func::MAXIMIZE);
    let mut names: Vec<&'static str> = Vec::new();
    if can_move {
        names.push("_NET_WM_ACTION_MOVE");
    }
    if can_min {
        names.push("_NET_WM_ACTION_MINIMIZE");
    }
    names.push("_NET_WM_ACTION_SHADE");
    names.push("_NET_WM_ACTION_STICK");
    names.push("_NET_WM_ACTION_CHANGE_DESKTOP");
    if can_close {
        names.push("_NET_WM_ACTION_CLOSE");
    }
    names.push("_NET_WM_ACTION_ABOVE");
    names.push("_NET_WM_ACTION_BELOW");
    if can_resize {
        names.push("_NET_WM_ACTION_RESIZE");
    }
    if can_max {
        names.push("_NET_WM_ACTION_MAXIMIZE_HORZ");
        names.push("_NET_WM_ACTION_MAXIMIZE_VERT");
        names.push("_NET_WM_ACTION_FULLSCREEN");
    }
    names
}

pub fn update_allowed_actions<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
    size_hints: Option<&antibox_core::backend::SizeHints>,
    mwm: Option<&antibox_core::backend::MwmHints>,
) {
    let resizable = !size_hints.map_or(false, antibox_core::SizeHints::is_fixed);
    let names = allowed_action_names(resizable, mwm);
    let action_atoms: Vec<u32> = names.iter().filter_map(|name| atoms.get(name)).collect();
    set_prop32(
        backend,
        atoms,
        "_NET_WM_ALLOWED_ACTIONS",
        window,
        ATOM_ATOM,
        &action_atoms,
    );
}

pub fn set_wm_desktop<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
    desktop: u32,
) {
    set_prop32(
        backend,
        atoms,
        "_NET_WM_DESKTOP",
        window,
        ATOM_CARDINAL,
        &[desktop],
    );
}

pub fn opacity_percent_to_card32(percent: i32) -> Option<u32> {
    if !(1..=100).contains(&percent) {
        return None;
    }
    let pct = percent as u32;
    let omax: u32 = 0xFFFF_FFFF;
    let oper = omax / 100;
    let orem = omax - oper * 100;
    Some(pct * oper + pct * orem / 100)
}

pub fn update_window_opacity<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    client: u32,
    frame: u32,
    fallback: Option<u32>,
) {
    let atom = match atoms.get("_NET_WM_WINDOW_OPACITY") {
        Some(a) => a,
        None => return,
    };
    let opacity = backend
        .get_property(client, atom, 0, 0, 1)
        .ok()
        .and_then(|v| v)
        .filter(|d| d.len() >= 4)
        .map(|d| u32::from_ne_bytes([d[0], d[1], d[2], d[3]]))
        .or(fallback);
    match opacity {
        Some(v) => {
            let _ = backend.change_property32(PropMode::Replace, frame, atom, ATOM_CARDINAL, &[v]);
        }
        None => {
            let _ = backend.delete_property(frame, atom);
        }
    }
}

pub fn get_wm_desktop<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    window: u32,
) -> Option<u32> {
    let atom = atoms.get("_NET_WM_DESKTOP")?;
    let bytes = backend.get_property(window, atom, 0, 0, 1).ok()??;
    if bytes.len() >= 4 {
        Some(u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    } else {
        None
    }
}

pub fn update_desktop_geometry<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
) {
    set_prop32(
        backend,
        atoms,
        "_NET_DESKTOP_GEOMETRY",
        backend.root().read_id(),
        ATOM_CARDINAL,
        &[
            backend.screen_width() as u32,
            backend.screen_height() as u32,
        ],
    );
}

pub fn update_workarea<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    struts: &[Strut],
    workspace_count: u32,
) {
    let sw = backend.screen_width() as u32;
    let sh = backend.screen_height() as u32;
    let mut left = 0u32;
    let mut right = 0u32;
    let mut top = 0u32;
    let mut bottom = 0u32;
    for s in struts {
        left = left.max(s.left);
        right = right.max(s.right);
        top = top.max(s.top);
        bottom = bottom.max(s.bottom);
    }
    let wx = left;
    let wy = top;
    let ww = sw.saturating_sub(left + right);
    let wh = sh.saturating_sub(top + bottom);
    if let Some(atom) = atoms.get("_NET_WORKAREA") {
        let mut data = Vec::with_capacity(workspace_count as usize * 4);
        for _ in 0..workspace_count {
            data.push(wx);
            data.push(wy);
            data.push(ww);
            data.push(wh);
        }
        let _ = backend.change_property32(
            PropMode::Replace,
            backend.root().read_id(),
            atom,
            ATOM_CARDINAL,
            &data,
        );
    }
}

pub fn init_xdnd<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
) {
    set_prop32(
        backend,
        atoms,
        "XdndAware",
        backend.root().read_id(),
        ATOM_ATOM,
        &[5],
    );
}

pub fn send_xdnd_status<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    target: u32,
    rect: antibox_core::rect::Rect,
    source: u32,
    accept: bool,
) {
    if let Some(status_atom) = atoms.get("XdndStatus") {
        let action_atom = atoms.get("XdndActionCopy").unwrap_or(0);
        let flags = u32::from(accept) | 2u32;
        let rect_pos = ((rect.x as u32) << 16) | (rect.y as u32 & 0xFFFF);
        let rect_size = ((rect.w as u32) << 16) | (rect.h as u32 & 0xFFFF);
        let data = [
            target,
            flags,
            rect_pos,
            rect_size,
            if accept { action_atom } else { 0 },
        ];
        let _ = backend.send_event(false, source, 0, status_atom, &data);
    }
}

pub fn send_xdnd_finished<H: DisplayBackend + 'static + ?Sized>(
    backend: &H,
    atoms: &antibox_core::backend::AtomManager,
    target: u32,
    source: u32,
    accepted: bool,
) {
    if let Some(finished_atom) = atoms.get("XdndFinished") {
        let action_atom = atoms.get("XdndActionCopy").unwrap_or(0);
        let data = [
            target,
            u32::from(accepted),
            0,
            0,
            if accepted { action_atom } else { 0 },
        ];
        let _ = backend.send_event(false, source, 0, finished_atom, &data);
    }
}

#[cfg(test)]
#[path = "ewmh_tests.rs"]
mod tests;
