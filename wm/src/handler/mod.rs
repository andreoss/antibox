use crate::compat::ClampExt;
pub mod configure;
pub use self::configure::configure_request;

use crate::client::ClientWindow;
use crate::frame::{border_width, title_bar_height, FrameWindow};
use crate::id::ClientId;
use crate::manager::WindowManager;
use crate::option::{GeoFlags, WindowFlags};
use crate::placement::{
    add_to_transient_chain, remove_from_transient_chain, restack_windows,
    update_workarea_from_struts,
};
use crate::render;
use crate::wmstate::WinLayer;
use antibox_core::backend::{BackendEvent, DisplayBackend, EventMask, MapState};
use antibox_core::rect::Rect;

pub(crate) fn window_type_decorated(wt: crate::client::WindowType) -> bool {
    use crate::client::WindowType;
    match wt {
        WindowType::Dock | WindowType::Desktop | WindowType::Splash => false,
        _ => true,
    }
}

pub(crate) fn hide_from_taskbar_on_map(client: &ClientWindow) -> bool {
    !window_type_decorated(client.window_type())
}

pub(crate) fn want_decorated(client: &ClientWindow) -> bool {
    window_type_decorated(client.window_type())
        && !client
            .mwm_hints()
            .map_or(false, antibox_core::MwmHints::undecorated)
        && !client.is_csd()
}

pub fn map_request<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, window: u32) {
    map_request_ex(wm, window, false);
}

pub(crate) fn client_list_ids<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
) -> Vec<u32> {
    let mut ids: Vec<u32> = wm
        .map_order
        .iter().cloned()
        .filter(|id| wm.frames.contains_key(id))
        .map(|id| wm.xid_index.xid_of(id))
        .collect();
    for id in wm.frames.keys() {
        let raw = wm.xid_index.xid_of(id);
        if !ids.contains(&raw) {
            ids.push(raw);
        }
    }
    ids
}

pub fn map_request_ex<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    window: u32,
    adopt: bool,
) {
    if wm.xid_index.contains_xid(window) {
        return;
    }
    let map_t0 = std::time::Instant::now();
    let xw = match wm.backend().unwrap().wrap_window(window) {
        Ok(xw) => xw,
        Err(_) => return,
    };
    let own_mask = if wm.self_windows.contains(&window) {
        EventMask::PROPERTY_CHANGE
            | EventMask::KEY_PRESS
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::POINTER_MOTION
            | EventMask::EXPOSURE
            | EventMask::STRUCTURE_NOTIFY
    } else {
        EventMask::PROPERTY_CHANGE
    };
    let _ = xw.select_input(own_mask);
    let _ = xw.select_shape_input();
    let full_geom = xw.get_geometry_rect().unwrap_or(Rect::new(0, 0, 400, 300));
    let (cw, ch) = (full_geom.w, full_geom.h);
    let mut client = ClientWindow::new(xw);
    client.read_initial_properties(wm.backend().unwrap(), &wm.atoms);
    let t_props = map_t0.elapsed();
    let decorated = want_decorated(&client);
    let pos = if !decorated || adopt {
        antibox_core::point::Point::new(full_geom.x, full_geom.y)
    } else if let Some(p) = crate::placement::requested_position(client.size_hints()) {
        crate::placement::clamp_below_struts(p, ch, wm)
    } else if client.window_type() == crate::client::WindowType::Dialog
        || client.transient_for.is_some()
    {
        crate::placement::place_dialog(wm, cw, ch)
    } else {
        let p = wm.layout_for(wm.active_workspace).place_new(cw, ch, wm);
        crate::placement::clamp_below_struts(p, ch, wm)
    };
    let cr = Rect::new(pos.x, pos.y, cw, ch);
    let inset = if client.is_csd() {
        client.csd_extents()
    } else {
        [0; 4]
    };
    if adopt {
        wm.expect_client_unmap(window);
    }
    if let Ok((fxw, fr)) = FrameWindow::create_frame_inset(
        wm.backend().unwrap(),
        window,
        cr,
        decorated,
        wm.theme_colours.border_active,
        inset,
    ) {
        let mut fw = FrameWindow::new(client, fxw);
        fw.decorated = decorated;
        if hide_from_taskbar_on_map(fw.client()) {
            fw.state_mut().skip_taskbar = true;
        }
        fw.state_mut().urgent = fw.client().wm_hints().map_or(false, |h| h.urgency);
        fw.shapes_protect = wm.config.shapes_protect_client;
        fw.frame_rect = fr;
        fw.client_rect = Rect::new(cr.x - inset[0], cr.y - inset[2], cw, ch);
        match crate::ewmh::get_wm_desktop(wm.backend().unwrap(), &wm.atoms, window) {
            Some(0xFFFF_FFFF) => fw.set_workspace(!0),
            Some(d) if d < wm.workspace_count() => fw.set_workspace(d),
            _ => fw.set_workspace(wm.active_workspace()),
        }
        let dock_ci = fw.client().class_instance().map(ToString::to_string);
        let is_dock_app = dock_ci
            .as_ref().map(|v| v.as_ref())
            .map_or(false, |ci| wm.dock_manager.try_dock(Some(ci)));
        if is_dock_app {
            wm.dock_manager.dock(window);
            if let Some(b) = wm.backend() {
                wm.dock_manager.adapt_with(b);
            }
            fw.state_mut().skip_taskbar = true;
        }
        fw.layout_shape();
        fw.create_pointer_windows(wm.backend().unwrap(), &wm.cursors);
        let mut skip_focus = false;
        if let Some(ci) = fw.client().class_instance() {
            let (_found, idx) = wm.win_options.find(ci);
            if let Some(wo) = wm.win_options.get(idx).filter(|o| o.class_instance == ci) {
                if wo.has_option(WindowFlags::DO_NOT_MANAGE) {
                    return;
                }
                if let Some(ws) = wo.placement.workspace {
                    fw.set_workspace(ws as u32);
                }
                if wo.has_option(WindowFlags::ALL_WORKSPACES) {
                    fw.set_workspace(!0);
                }
                fw.set_layer(wo.placement.layer.unwrap_or(fw.layer()));
                if wo.has_option(WindowFlags::IGNORE_TASKBAR)
                    || wo.has_option(WindowFlags::IGNORE_OVERRIDE_REDIRECT)
                {
                    fw.state_mut().skip_taskbar = true;
                }
                if wo.has_option(WindowFlags::DO_NOT_FOCUS)
                    || wo.has_option(WindowFlags::NO_FOCUS_ON_MAP)
                {
                    skip_focus = true;
                }
                if wo.has_option(WindowFlags::MINIMIZED) {
                    fw.state_mut().minimized = true;
                }
                if wo.has_option(WindowFlags::MAXIMIZED_BOTH) {
                    fw.state_mut().max_vert = true;
                    fw.state_mut().max_horz = true;
                }
                fw.state_mut().maximized = fw.state().max_vert && fw.state().max_horz;
                if wo.has_option(WindowFlags::FULLSCREEN) {
                    fw.state_mut().fullscreen = true;
                    fw.set_layer(WinLayer::Fullscreen);
                }
                if !wo.geom.gflags.is_empty() {
                    let new_fr = {
                        let gx = if wo.geom.gflags.contains(GeoFlags::X) {
                            wo.geom.gx
                        } else {
                            fr.x
                        };
                        let gy = if wo.geom.gflags.contains(GeoFlags::Y) {
                            wo.geom.gy
                        } else {
                            fr.y
                        };
                        let gw = if wo.geom.gflags.contains(GeoFlags::W) {
                            wo.geom.gw as i32
                        } else {
                            fr.w
                        };
                        let gh = if wo.geom.gflags.contains(GeoFlags::H) {
                            wo.geom.gh as i32
                        } else {
                            fr.h
                        };
                        Rect::new(gx, gy, gw, gh)
                    };
                    fw.set_frame_rect(new_fr);
                    let _ = fw.frame().configure(
                        Some(new_fr.x),
                        Some(new_fr.y),
                        Some(new_fr.w as u16),
                        Some(new_fr.h as u16),
                    );
                    let _ = fw.client().xwindow.configure(
                        Some(new_fr.x + border_width()),
                        Some(new_fr.y + crate::frame::title_block_height()),
                        Some((new_fr.w - border_width() * 2) as u16),
                        Some(
                            (new_fr.h
                                - crate::frame::title_block_height()
                                - crate::frame::bottom_border_width())
                                as u16,
                        ),
                    );
                }
            }
        }

        let trans_owner = fw.transient_for();
        let frame_xid = fw.frame().id();
        let cid = fw.client_id();
        wm.xid_index.insert(cid, window, frame_xid);
        wm.frames.insert(cid, fw);
        wm.insertion_order.push(cid);
        if !wm.map_order.contains(&cid) {
            wm.map_order.push(cid);
        }
        let t_frame = map_t0.elapsed();
        if let Some(owner_id) = trans_owner {
            if let Some(owner_cid) = wm.cid_for_xid(owner_id) {
                add_to_transient_chain(&mut wm.frames, cid, owner_cid);
            }
        }
        let ids = client_list_ids(wm);
        crate::ewmh::update_client_list(wm.backend().unwrap(), &wm.atoms, &ids);
        let ws = wm.frame(cid).map_or(0, FrameWindow::workspace);
        crate::ewmh::set_wm_desktop(wm.backend().unwrap(), &wm.atoms, window, ws);
        crate::ewmh::set_wm_state(
            wm.backend().unwrap(),
            &wm.atoms,
            window,
            antibox_core::backend::hints::wm_state::NORMAL,
        );
        let decorated = wm.frames.get(&cid).map_or(true, FrameWindow::decorated);
        crate::ewmh::set_frame_extents(wm.backend().unwrap(), &wm.atoms, window, decorated);
        if let Some(frame) = wm.frame(cid).map(|f| f.frame().id()) {
            let fb = winoption_opacity(wm, cid);
            crate::ewmh::update_window_opacity(wm.backend().unwrap(), &wm.atoms, window, frame, fb);
        }
        crate::ewmh::update_allowed_actions(
            wm.backend().unwrap(),
            &wm.atoms,
            window,
            wm.frame(cid).and_then(|f| f.client().size_hints()),
            wm.frame(cid).and_then(|f| f.client().mwm_hints()),
        );
        update_workarea_from_struts(wm);
        apply_initial_net_wm_state(wm, cid);
        restack_windows(wm);
        let ws = wm.active_workspace;
        wm.layout_for(ws).arrange(wm, ws);
        if wm
            .frames
            .get(&cid)
            .map_or(false, |f| f.client().suppresses_map_focus())
        {
            skip_focus = true;
        }
        if !skip_focus {
            let prev = wm.focused_window;
            wm.focused_window = Some(cid);
            if let Some(fw) = wm.frame(cid) {
                crate::focus::give_input_focus(wm.backend().unwrap(), &wm.atoms, fw, window);
            }
            if let (Some(b), Some(old)) = (
                wm.backend(),
                prev.filter(|&o| o != cid).and_then(|o| wm.frame(o)),
            ) {
                crate::drag::paint_frame_decorations(
                    old,
                    b,
                    false,
                    &wm.theme_colours,
                    wm.config.gradients,
                );
            }
        }
        if let (Some(b), Some(fw)) = (wm.backend(), wm.frame(cid)) {
            let focused = wm.focused_window == Some(cid);
            crate::drag::paint_frame_decorations(
                fw,
                b,
                focused,
                &wm.theme_colours,
                wm.config.gradients,
            );
        }
        let _ = wm.backend().unwrap().flush();
        if crate::wmapp::event_timing_enabled() {
            let total = map_t0.elapsed();
            if total >= std::time::Duration::from_millis(12) {
                eprintln!(
                    "map phases: props={:.1} frame={:.1} tail={:.1} total={:.1}ms",
                    t_props.as_secs_f64() * 1000.0,
                    (t_frame - t_props).as_secs_f64() * 1000.0,
                    (total - t_frame).as_secs_f64() * 1000.0,
                    total.as_secs_f64() * 1000.0,
                );
            }
        }
    }
}

pub(crate) fn apply_initial_net_wm_state<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    id: ClientId,
) {
    let (wants_max, wants_full, wants_shaded) = match wm.frame(id) {
        Some(f) if !f.client().wm_state().is_empty() => {
            let c = f.client();
            (
                c.has_net_state(&wm.atoms, "_NET_WM_STATE_MAXIMIZED_VERT")
                    || c.has_net_state(&wm.atoms, "_NET_WM_STATE_MAXIMIZED_HORZ"),
                c.has_net_state(&wm.atoms, "_NET_WM_STATE_FULLSCREEN"),
                c.has_net_state(&wm.atoms, "_NET_WM_STATE_SHADED"),
            )
        }
        _ => return,
    };
    if wants_full {
        crate::wmaction::set_fullscreen(wm, id, true);
    } else if wants_max {
        crate::wmaction::set_maximized(wm, id);
    }
    if wants_shaded {
        crate::wmaction::set_shaded(wm, id, Some(true));
    }
    if let Some(f) = wm.frames.get_mut(&id) {
        f.sync_state_from_ewmh(&wm.atoms);
    }
}

pub fn set_window_decorated<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: ClientId,
    decorated: bool,
) {
    let backend = wm.backend.clone();
    let b = match backend.as_ref().map(|v| v.as_ref()) {
        Some(b) => b,
        None => return,
    };
    let frame_id;
    {
        let fw = match wm.frame_mut(w) {
            Some(fw) => fw,
            None => return,
        };
        if fw.decorated() == decorated {
            return;
        }
        let old = fw.frame_rect();
        let old_bw = fw.effective_border();
        let old_th = if fw.decorated() {
            title_bar_height()
        } else {
            0
        };
        let cr = fw.client_rect();
        let (cw, ch) = (cr.w, cr.h);
        let cx = old.x + old_bw;
        let cy = old.y + crate::frame::top_for(old_bw, old_th);

        fw.decorated = decorated;
        let new_bw = fw.effective_border();
        let new_bb = fw.effective_bottom_border();
        let new_th = if decorated { title_bar_height() } else { 0 };
        let new_top = crate::frame::top_for(new_bw, new_th);
        let new_frame = Rect::new(
            cx - new_bw,
            cy - new_top,
            cw + new_bw * 2,
            ch + new_top + new_bb,
        );
        fw.set_frame_rect(new_frame);
        fw.client_rect = Rect::new(cx, cy, cw, ch);
        frame_id = fw.frame().id();

        let _ = b.configure_window(
            frame_id,
            &[
                new_frame.x as u32,
                new_frame.y as u32,
                new_frame.w.clamped(1, std::u16::MAX as i32) as u32,
                new_frame.h.clamped(1, std::u16::MAX as i32) as u32,
            ],
        );
        let _ = fw.client().xwindow.configure(
            Some(new_bw),
            Some(new_th + new_bw),
            Some(cw.clamped(1, std::u16::MAX as i32) as u16),
            Some(ch.clamped(1, std::u16::MAX as i32) as u16),
        );
        fw.layout_shape();
        fw.layout_pointer_windows(b);
    }
    crate::ewmh::set_frame_extents(b, &wm.atoms, wm.xid_index.xid_of(w), decorated);
    let _ = b.clear_area(true, frame_id, Rect::ZERO);
    let _ = b.flush();
}

pub(crate) fn apply_csd_extents<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: ClientId,
) {
    let backend = wm.backend.clone();
    let b = match backend.as_ref().map(|v| v.as_ref()) {
        Some(b) => b,
        None => return,
    };
    let fw = match wm.frame_mut(w) {
        Some(fw) => fw,
        None => return,
    };
    let [il, ir, it, ib] = if fw.client().is_csd() {
        fw.client().csd_extents()
    } else {
        [0; 4]
    };
    let bw = fw.effective_border();
    let th = if fw.decorated() {
        title_bar_height()
    } else {
        0
    };
    let fr = fw.frame_rect();
    let cr = fw.client_rect();
    let top = crate::frame::top_for(bw, th);
    let vx = fr.x + bw;
    let vy = fr.y + top;
    let new_frame = Rect::new(
        fr.x,
        fr.y,
        (cr.w + bw * 2 - il - ir).max(1),
        (cr.h + top + fw.effective_bottom_border() - it - ib).max(1),
    );
    let new_client = Rect::new(vx - il, vy - it, cr.w, cr.h);
    if new_frame == fr && new_client == cr {
        return;
    }
    let frame_id = fw.frame().id();
    fw.set_frame_rect(new_frame);
    fw.client_rect = new_client;
    let _ = b.configure_window(
        frame_id,
        &[
            new_frame.x as u32,
            new_frame.y as u32,
            new_frame.w.clamped(1, std::u16::MAX as i32) as u32,
            new_frame.h.clamped(1, std::u16::MAX as i32) as u32,
        ],
    );
    let _ = fw.client().xwindow.configure(
        Some(bw - il),
        Some(crate::frame::top_for(bw, th) - it),
        Some(cr.w.clamped(1, std::u16::MAX as i32) as u16),
        Some(cr.h.clamped(1, std::u16::MAX as i32) as u16),
    );
    fw.layout_pointer_windows(b);
    let _ = b.clear_area(true, frame_id, Rect::ZERO);
    let _ = b.flush();
}

pub fn destroy<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, w: u32) {
    wm.self_windows.remove(&w);
    let dragged_frame = wm.frame_by_xid(w).map(FrameWindow::frame_id);
    if dragged_frame.is_some() && wm.drag_state.map(|d| d.0) == dragged_frame {
        wm.drag_state = None;
        wm.drag_pending = None;
        crate::drag_outline::hide(wm);
        crate::resize_popup::hide(wm);
        if let Some(b) = wm.backend() {
            let _ = b.ungrab_pointer(0);
        }
    }
    if let Some(cid) = wm.cid_for_xid(w) {
        remove_from_transient_chain(&mut wm.frames, cid);
        if let Some(mut fw) = wm.frames.remove(&cid) {
            wm.xid_index.remove(cid);
            if let Some(conn) = wm.backend() {
                fw.destroy_pointer_windows(conn);
                let _ = conn.destroy_window(fw.frame().id());
                let _ = conn.flush();
            }
        } else {
            wm.xid_index.remove(cid);
        }
        wm.insertion_order.retain(|&id| id != cid);
        wm.map_order.retain(|&id| id != cid);
        if wm.focused_window == Some(cid) {
            wm.focused_window = None;
            crate::focus::recover_focus(wm);
        }
        if wm.last_focused_window == Some(cid) {
            wm.last_focused_window = None;
        }
    }
    let ids = client_list_ids(wm);
    if let Some(conn) = wm.backend() {
        crate::ewmh::update_client_list(conn, &wm.atoms, &ids);
        update_workarea_from_struts(wm);
        restack_windows(wm);
    }
    let ws = wm.active_workspace;
    wm.layout_for(ws).arrange(wm, ws);
}

pub fn unmap<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, w: u32) {
    if wm.consume_expected_unmap(w) {
        return;
    }
    if !wm.xid_index.contains_client_xid(w) {
        return;
    }
    let geom = wm.frame_by_xid(w).map(FrameWindow::client_rect);
    if let (Some(b), Some(r)) = (wm.backend.clone(), geom) {
        let root = b.root().read_id();
        let _ = b.reparent_window(w, root, antibox_core::point::Point::new(r.x, r.y));
        if let Some(ws_atom) = wm.atoms.get("WM_STATE") {
            let _ = b.change_property32(
                antibox_core::backend::PropMode::Replace,
                w,
                ws_atom,
                ws_atom,
                &[0, 0],
            );
        }
    }
    destroy(wm, w);
}

pub fn redraw_all_frames<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) {
    let b = match wm.backend() {
        Some(b) => b,
        None => return,
    };
    for (id, fw) in wm.frames.iter() {
        let mut cache = fw.gfx.borrow_mut();
        if cache.is_none() {
            if let Ok(g) = b.create_graphics(fw.frame().id()) {
                *cache = Some(g);
            }
        }
        if let Some(g) = cache.as_ref().map(|v| v.as_ref()) {
            let focused = wm.focused_window == Some(*id);
            let _ = render::draw_frame(fw, g, focused, &wm.theme_colours, wm.config.gradients);
        }
    }
}

pub(crate) fn winoption_opacity<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    w: ClientId,
) -> Option<u32> {
    let ci = wm.frame(w)?.client().class_instance()?;
    let (found, idx) = wm.win_options.find(ci);
    if !found {
        return None;
    }
    crate::ewmh::opacity_percent_to_card32(wm.win_options.get(idx)?.opacity)
}

pub(crate) fn redraw_frame_decor<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    id: ClientId,
) {
    let focused = wm.focused_window == Some(id);
    if let (Some(b), Some(fw)) = (wm.backend(), wm.frame(id)) {
        crate::drag::paint_frame_decorations(
            fw,
            b,
            focused,
            &wm.theme_colours,
            wm.config.gradients,
        );
        let _ = b.flush();
    }
}

pub fn expose<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>, w: u32, _r: Rect) {
    if let (Some(b), Some(fw)) = (wm.backend(), wm.frame_by_xid(w)) {
        let focused = wm
            .cid_for_xid(w)
            .map_or(false, |cid| wm.focused_window == Some(cid));
        crate::drag::paint_frame_decorations(
            fw,
            b,
            focused,
            &wm.theme_colours,
            wm.config.gradients,
        );
    }
}

pub fn property_notify<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    a: u32,
) {
    if let Some(cid) = wm.xid_index.client_id_for(w) {
        if let Some(fw) = wm.frames.get_mut(&cid) {
            fw.client_mut().handle_property_notify(
                &wm.atoms,
                &BackendEvent::PropertyNotify {
                    window: w,
                    atom: a,
                    state: 0,
                },
            );
        }
    }
    let is_decor_hint =
        Some(a) == wm.atoms.get("_GTK_FRAME_EXTENTS") || Some(a) == wm.atoms.get("_MOTIF_WM_HINTS");
    if is_decor_hint {
        let change = wm
            .frame_by_xid(w)
            .map(|fw| (want_decorated(fw.client()), fw.decorated()))
            .filter(|(want, cur)| want != cur)
            .map(|(want, _)| want);
        if let Some(want) = change {
            if let Some(cid) = wm.cid_for_xid(w) {
                set_window_decorated(wm, cid, want);
            }
        }
        if Some(a) == wm.atoms.get("_GTK_FRAME_EXTENTS") {
            if let Some(cid) = wm.cid_for_xid(w) {
                apply_csd_extents(wm, cid);
            }
        }
    }
    let is_strut = Some(a) == wm.atoms.get("_NET_WM_STRUT")
        || Some(a) == wm.atoms.get("_NET_WM_STRUT_PARTIAL");
    if is_strut {
        update_workarea_from_struts(wm);
    }
    if Some(a) == wm.atoms.get("_NET_WM_WINDOW_OPACITY") {
        if let Some(cid) = wm.cid_for_xid(w) {
            if let Some(frame) = wm.frame(cid).map(|f| f.frame().id()) {
                let fb = winoption_opacity(wm, cid);
                crate::ewmh::update_window_opacity(wm.backend().unwrap(), &wm.atoms, w, frame, fb);
            }
        }
    }
    if Some(a) == wm.atoms.get("WM_NORMAL_HINTS") || Some(a) == wm.atoms.get("_MOTIF_WM_HINTS") {
        if Some(a) == wm.atoms.get("WM_NORMAL_HINTS") {
            if let Some(atom) = wm.atoms.get("WM_NORMAL_HINTS") {
                if let Some(fw) = wm.frame_by_xid_mut(w) {
                    fw.client_mut().refresh_size_hints(atom);
                }
            }
        }
        if let Some(cid) = wm.cid_for_xid(w) {
            let backend = wm.backend.clone();
            if let Some(b) = backend.as_ref().map(|v| v.as_ref()) {
                let sh = wm.frame(cid).and_then(|f| f.client().size_hints());
                let mwm = wm.frame(cid).and_then(|f| f.client().mwm_hints());
                crate::ewmh::update_allowed_actions(b, &wm.atoms, w, sh, mwm);
            }
        }
    }
    let is_title = Some(a) == wm.atoms.get("_NET_WM_NAME") || Some(a) == wm.atoms.get("WM_NAME");
    if is_title {
        if let Some(cid) = wm.cid_for_xid(w) {
            redraw_frame_decor(wm, cid);
        }
    }
    if Some(a) == wm.atoms.get("WM_HINTS") {
        if let Some(cid) = wm.xid_index.client_id_for(w) {
            if let Some(fw) = wm.frames.get_mut(&cid) {
                fw.sync_state_from_ewmh(&wm.atoms);
            }
        }
        if let Some(cid) = wm.cid_for_xid(w) {
            redraw_frame_decor(wm, cid);
        }
    }
    let root = wm.backend().map(|b| b.root().read_id());
    if Some(w) == root && Some(a) == wm.atoms.get("_NET_DESKTOP_NAMES") {
        if let (Some(b), Some(atom)) = (wm.backend(), wm.atoms.get("_NET_DESKTOP_NAMES")) {
            if let Ok(Some(bytes)) = b.get_property(w, atom, 0, 0, 1024) {
                let names = crate::ewmh::parse_desktop_names(&bytes);
                if !names.is_empty() && names != wm.workspace_names {
                    wm.workspace_names = names;
                    wm.workspace_names_dirty = true;
                }
            }
        }
    }
}

pub fn client_message<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    mt: u32,
    d: [u32; 5],
) {
    if !crate::wmapp::event_timing_enabled() {
        crate::clientmsg::client_message(wm, w, mt, d);
        return;
    }
    let t0 = std::time::Instant::now();
    crate::clientmsg::client_message(wm, w, mt, d);
    let el = t0.elapsed();
    if el.as_secs_f64() * 1000.0 >= 12.0 {
        let name = wm
            .backend()
            .and_then(|b| b.get_atom_name(mt).ok())
            .unwrap_or_default();
        eprintln!(
            "slow client_message {:.1}ms type={} ({}) window={}",
            el.as_secs_f64() * 1000.0,
            name,
            mt,
            w
        );
    }
}

pub fn enter_notify<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, window: u32) {
    crate::focus::enter_notify(wm, window);
}

pub fn shape_notify<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, w: u32) {
    if let Some(fw) = wm.frame_by_xid_mut(w) {
        fw.client_mut().f_shaped = true;
        fw.set_shape();
    }
}

pub fn mapping_notify<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(ref b) = wm.backend {
        let _ = wm.key_bindings.regrab_all(b);
    }
}

fn wm_state_value<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
    window: u32,
) -> Option<u32> {
    let atom = wm.atoms.get("WM_STATE")?;
    let b = wm.backend()?;
    if let Ok(Some(d)) = b.get_property(window, atom, 0, 0, 2) {
        if d.len() >= 4 {
            return Some(crate::compat::u32_ne([d[0], d[1], d[2], d[3]]));
        }
    }
    None
}

pub(crate) fn saved_stacking_order<H: DisplayBackend + 'static + ?Sized>(
    wm: &WindowManager<H>,
) -> Vec<u32> {
    let atom = match wm.atoms.get("_NET_CLIENT_LIST_STACKING") {
        Some(a) => a,
        None => return Vec::new(),
    };
    let b = match wm.backend() {
        Some(b) => b,
        None => return Vec::new(),
    };
    match b.get_property(b.root().read_id(), atom, 0, 0, 4096) {
        Ok(Some(data)) => data
            .chunks_exact(4)
            .map(|c| crate::compat::u32_ne([c[0], c[1], c[2], c[3]]))
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn apply_saved_stacking(children: &[u32], saved: &[u32]) -> Vec<u32> {
    let mut listed: std::collections::VecDeque<u32> = saved
        .iter().cloned()
        .filter(|w| children.contains(w))
        .collect();
    children
        .iter()
        .map(|&c| {
            if saved.contains(&c) {
                listed.pop_front().unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

pub fn manage_existing_windows<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    let root = wm.backend().unwrap().root().read_id();
    let tree = match wm.backend().unwrap().query_tree(root) {
        Ok(tree) => tree,
        Err(_) => return,
    };
    let children = apply_saved_stacking(&tree.children, &saved_stacking_order(wm));
    for &child in &children {
        if child == root || wm.xid_index.contains_xid(child) {
            continue;
        }
        let attrs = match wm.backend().unwrap().get_window_attributes(child) {
            Ok(attrs) => attrs,
            Err(_) => continue,
        };
        if attrs.override_redirect {
            continue;
        }
        if attrs.map_state != MapState::Viewable {
            use antibox_core::backend::hints::wm_state;
            match wm_state_value(wm, child) {
                Some(s) if s == wm_state::ICONIC => {
                    map_request_ex(wm, child, true);
                    if let Some(fw) = wm.frame_by_xid_mut(child) {
                        fw.minimize();
                    }
                    if let Some(cid) = wm.cid_for_xid(child) {
                        crate::wmaction::set_minimized_visible(wm, cid, true);
                    }
                }
                Some(s) if s == wm_state::NORMAL => {
                    map_request_ex(wm, child, true);
                }
                _ => {}
            }
            continue;
        }
        map_request_ex(wm, child, true);
    }
    wm.apply_workspace_visibility();
    crate::focus::recover_focus(wm);
}

#[cfg(test)]
mod client_list_tests;

#[cfg(test)]
mod stacking_restore_tests;

#[cfg(test)]
mod decoration_tests;
