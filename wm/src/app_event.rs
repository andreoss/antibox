use crate::action::*;
use crate::id::ClientId;
use antibox_core::logevent;
use antibox_core::point::Point;

pub(crate) fn event_timing_enabled() -> bool {
    use std::cell::RefCell;
    thread_local! {
        static ON: RefCell<Option<bool>> = RefCell::new(None);
    }
    ON.with(|c| {
        *c.borrow_mut()
            .get_or_insert_with(|| {
                std::env::var("ANTIBOX_LOOP_TIMING")
                    .map_or(false, |v| v != "0" && !v.is_empty())
            })
    })
}

impl App {
    pub(crate) fn drain_pending_events_count(&mut self) -> usize {
        let events = match self.event_loop.process_pending() { Ok(v) => v, Err(_) => return 0  };
        let n = events.len();
        let mut last_expose: std::collections::HashMap<u32, usize> =
            std::collections::HashMap::new();
        let mut last_motion: std::collections::HashMap<u32, usize> =
            std::collections::HashMap::new();
        let mut last_configure: std::collections::HashMap<u32, usize> =
            std::collections::HashMap::new();
        for (i, e) in events.iter().enumerate() {
            match e {
                BackendEvent::Expose { window, .. } => {
                    last_expose.insert(*window, i);
                }
                BackendEvent::MotionNotify { window, .. } => {
                    last_motion.insert(*window, i);
                }
                BackendEvent::ConfigureNotify { window, .. } => {
                    last_configure.insert(*window, i);
                }
                _ => {}
            }
        }
        for (i, event) in events.into_iter().enumerate() {
            if let BackendEvent::Expose { window, .. } = &event {
                if last_expose.get(window) != Some(&i) {
                    continue;
                }
            }
            if let BackendEvent::MotionNotify { window, .. } = &event {
                if last_motion.get(window) != Some(&i) {
                    continue;
                }
            }
            if let BackendEvent::ConfigureNotify { window, .. } = &event {
                if last_configure.get(window) != Some(&i) {
                    continue;
                }
            }
            if event_timing_enabled() {
                let t = Instant::now();
                self.dispatch_guarded(&event);
                let d = t.elapsed();
                if d >= Duration::from_millis(12) {
                    eprintln!(
                        "slow event {:.1}ms: {}",
                        d.as_secs_f64() * 1000.0,
                        logevent::format_event(&event)
                    );
                }
            } else {
                self.dispatch_guarded(&event);
            }
            if let BackendEvent::DestroyNotify { window } = &event {
                self.winlist.on_destroyed(*window);
            }
            if self.winlist.visible
                && matches!(
                    event,
                    BackendEvent::DestroyNotify { .. }
                        | BackendEvent::MapNotify { .. }
                        | BackendEvent::UnmapNotify { .. }
                        | BackendEvent::PropertyNotify { .. }
                )
            {
                self.winlist.refresh(&self.backend, &self.wm);
            }
        }
        n
    }

    pub(crate) fn dispatch_guarded(&mut self, event: &BackendEvent) {
        const MAX_CONSECUTIVE_PANICS: u32 = 5;
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.handle_backend_event(event);
        }));
        if caught.is_ok() {
            self.consecutive_panics = 0;
            return;
        }
        self.consecutive_panics += 1;
        let desc = logevent::format_event(event);
        let detail = crate::panic_guard::take_last().unwrap_or_default();
        eprintln!(
            "panic while handling {} ({}/{}); event dropped, continuing",
            desc,
            self.consecutive_panics,
            MAX_CONSECUTIVE_PANICS
        );
        if let Some(path) = crate::panic_guard::breadcrumb_path() {
            let _ = crate::panic_guard::record(&desc, &detail, &path);
        }
        let _ = self.backend.flush();
        if self.consecutive_panics >= MAX_CONSECUTIVE_PANICS {
            eprintln!("giving up after {} consecutive panics", MAX_CONSECUTIVE_PANICS);
            self.running = false;
        }
    }

    fn reload_config(&mut self) {
        let prefs = wmconfig::Config::load_prefs();

        apply_font_prefs(&self.backend, &prefs);

        self.wm.key_bindings = crate::bindings::KeyBindings::new();
        let _ = self
            .wm
            .key_bindings
            .register_all(&self.backend, &crate::keys_parser::entries_from(&prefs.keys));

        let (count, names) = wmconfig::workspaces_from(&prefs);
        let count_changed = count != self.wm.config.workspace_count;
        if self.wm.active_workspace >= count {
            self.wm.activate_workspace(count - 1);
        }
        self.wm.config.workspace_count = count;
        self.wm.workspace_names = names.clone();
        self.wm
            .set_workspace_layouts(crate::layout::Layout::parse_list(
                &prefs.workspace.layouts,
                count as usize,
            ));
        if count_changed {
            let _ = crate::ewmh::init_ewmh(&*self.backend, &self.wm.atoms, count);
        }
        crate::ewmh::update_desktop_names(&*self.backend, &self.wm.atoms, &names);

        if prefs.keyboard.layouts != self.keyboard_layouts_pref {
            self.keyboard_layouts_pref = prefs.keyboard.layouts.clone();
            self.rebuild_keyboard_applet(wmconfig::split_layout_list(&prefs.keyboard.layouts));
        }

        let mut strut_changed = false;
        if let Some(tb) = self.taskbar.as_mut() {
            tb.set_workspace_names(&names);
            for a in &mut tb.applets {
                let w = if a.as_any().is::<crate::cpu_status_applet::CpuStatusApplet>() {
                    Some(prefs.cpu.width)
                } else if a.as_any().is::<crate::mem_status_applet::MemStatusApplet>() {
                    Some(prefs.mem.width)
                } else if a.as_any().is::<crate::net_status_applet::NetStatusApplet>() {
                    Some(prefs.net.width)
                } else {
                    None
                };
                if let Some(w) = w {
                    a.set_graph_width(w);
                }
            }
            if tb.update_height() {
                strut_changed = true;
            }
            tb.reflow();
            let _ = tb.paint();
            self.wm.reserved_strut = tb.strut();
        }
        if strut_changed {
            crate::placement::update_workarea_from_struts(&mut self.wm);
        }
        let _ = self.backend.flush();
    }

    fn rebuild_keyboard_applet(&mut self, layouts: Vec<String>) {
        let colours = self.wm.theme_colours;
        if let Some(tb) = self.taskbar.as_mut() {
            if let Some(i) = tb
                .applets
                .iter()
                .position(|a| a.as_any().is::<crate::keyboard_applet::KeyboardApplet>())
            {
                if let Some(kb) = tb.applets[i]
                    .as_any_mut()
                    .downcast_mut::<crate::keyboard_applet::KeyboardApplet>()
                {
                    kb.shutdown();
                }
                tb.remove_applet(i);
            }
            if crate::layout_preferences::taskbar_wants(crate::layout_preferences::Widget::Keyboard)
            {
                let parent = tb.window.id();
                if let Ok(Some(kb)) = crate::keyboard_applet::KeyboardApplet::new(
                    &self.backend,
                    parent,
                    layouts,
                    &colours,
                ) {
                    let a: Box<dyn crate::applet::Applet> = Box::new(kb);
                    let _ = a.window().map();
                    tb.add_applet(a);
                }
            }
            let _ = tb.paint();
        }
    }

    pub(crate) fn handle_backend_event(&mut self, event: &BackendEvent) {
        if let BackendEvent::SelectionClear { owner, selection, .. } = event {
            let ours = self.wm_sn_owner.as_ref().map(|w| w.id());
            if self.wm_sn_atom != 0 && *selection == self.wm_sn_atom && Some(*owner) == ours {
                eprintln!(
                    "lost the WM_Sn manager selection; yielding the display to the new window manager"
                );
                self.running = false;
                return;
            }
        }
        if self.handle_super_tap(event) {
            return;
        }
        if let BackendEvent::ScreenSizeChanged { width, height } = event {
            self.handle_screen_resize(*width, *height);
            return;
        }
        if let BackendEvent::KeyboardChanged = event {
            crate::bindings::invalidate_keymap();
            if let Some(ref mut tb) = self.taskbar {
                let dirty = tb.update_keyboard();
                for wid in &dirty {
                    let _ = tb.paint_window(*wid);
                }
            }
            let _ = self.backend.flush();
            return;
        }
        if self.group_menu.as_ref().map_or(false, |m| m.visible) && self.handle_group_menu_event(event)
        {
            return;
        }
        if self.winlist.visible {
            let owned = match event {
                BackendEvent::ButtonPress { window, .. }
                | BackendEvent::ButtonRelease { window, .. }
                | BackendEvent::MotionNotify { window, .. }
                | BackendEvent::KeyPress { window, .. }
                | BackendEvent::Expose { window, .. }
                | BackendEvent::ConfigureNotify { window, .. } => self.winlist.owns_window(*window),
                _ => false,
            };
            if owned {
                self.handle_winlist_event(event);
                return;
            }
        }
        if self.preview.visible {
            if let BackendEvent::ButtonPress { point, .. } = event {
                self.preview
                    .handle_click(&mut self.wm, *point);
                self.preview.hide();
                return;
            } else if let BackendEvent::Expose { .. } = event {
                self.preview.paint(&self.backend, &self.wm);
                return;
            }
        }
        if self.switcher.visible {
            self.handle_switcher_event(event);
        } else {
            let mut switcher_consumed = false;
            if let BackendEvent::KeyPress { keycode, state, .. } = event {
                if self.lookup_keysym(*keycode) == 0xFF09 && *state & 0x08 != 0 {
                    self.switcher
                        .show(&self.backend, &self.wm, (*state & 0x01) == 0);
                    switcher_consumed = true;
                }
            }
            if switcher_consumed {
                return;
            }
            if self.wm.drag_state.is_some() {
                if let BackendEvent::KeyPress { keycode, .. } = event {
                    let ks = self.lookup_keysym(*keycode);
                    if crate::drag::keyboard_drag(&mut self.wm, ks) {
                        return;
                    }
                }
            }
            if let BackendEvent::ConfigureRequest { window, .. } = event {
                if self.route_to_sub_applet(*window, event) {
                    return;
                }
            }
            self.wm.handle_event(event);
            if std::mem::replace(&mut self.wm.workspace_names_dirty, false) {
                let names = self.wm.workspace_names.clone();
                if let Some(tb) = self.taskbar.as_mut() {
                    if tb.set_workspace_names(&names) {
                        let _ = tb.paint();
                    }
                }
                crate::ewmh::update_desktop_names(&*self.backend, &self.wm.atoms, &names);
            }
            if let Some(a) = self.wm.pending_action.take() {
                match a {
                    Action::Menu(MenuOp::WindowPickerList) => self.show_window_list(),
                    Action::Menu(MenuOp::Pager) => self.preview.show(&self.backend, &self.wm),
                    _ => {
                        if let Some(window) = event.window() {
                            self.route_to_sub_applet(window, event);
                        }
                    }
                }
            }
        }
        if let BackendEvent::ClientMessage {
            message_type,
            data,
            window,
            ..
        } = event
        {
            #[cfg(feature = "tray")]
            {
                if *message_type == self.tray_opcode_atom {
                    let _ = self.taskbar.as_mut().map(|tb| {
                        for a in &mut tb.applets {
                            if let Some(tray) = a.as_any_mut().downcast_mut::<TrayApplet>() {
                                tray.handle_client_message(*message_type, data);
                            }
                        }

                        tb.reflow();
                        let _ = tb.paint();
                    });
                }
                if let Some(xembed) = self.wm.atoms.get("_XEMBED") {
                    if *message_type == xembed {
                        let _ = self.taskbar.as_mut().map(|tb| {
                            for a in &mut tb.applets {
                                if let Some(tray) = a.as_any_mut().downcast_mut::<TrayApplet>() {
                                    tray.handle_xembed_message(*window, data);
                                }
                            }
                        });
                    }
                }
            }
        }
        #[cfg(feature = "tray")]
        {
            if let BackendEvent::DestroyNotify { window } = event {
                if let Some(tb) = self.taskbar.as_mut() {
                    if tb.tray_forget(*window) {
                        tb.reflow();
                        let _ = tb.paint();
                    }
                }
            }
        }
        self.dispatch_to_taskbar(event);
        if let Some(action) = self
            .taskbar
            .as_mut()
            .and_then(TaskBar::take_pending_action)
        {
            match action {
                Action::Menu(MenuOp::WindowPickerList) => self.show_window_list(),
                Action::Workspace(WorkspaceOp::WorkspaceMenu(ws)) => {
                    let current = self.wm.layout_for(ws);
                    if let Some(ref mut tb) = self.taskbar {
                        tb.show_workspace_menu(ws, current);
                    }
                }
                _ => self.wm.handle_action(&action),
            }
        }
    }

    fn dispatch_to_taskbar(&mut self, event: &BackendEvent) {
        let menu_open = self.taskbar.as_ref().map_or(false, |tb| {
            tb.menu.as_ref().map_or(false, |m| m.visible)
        });
        if menu_open {
            let on_menu = self.taskbar.as_ref().map_or(false, |tb| {
                tb.menu.as_ref().map_or(false, |m| {
                    event
                        .window()
                        .map_or(false, |w| m.window.as_ref().map_or(false, |mw| mw.id() == w))
                })
            });
            if on_menu {
                if let Some(ref mut tb) = self.taskbar {
                    tb.handle_menu_event(event, &*self.backend);
                }
                return;
            }
            if matches!(event, BackendEvent::ButtonPress { .. }) {
                if let Some(ref mut tb) = self.taskbar {
                    if let Some(ref mut m) = tb.menu {
                        m.hide();
                    }
                }
            }
        }
        let tb = match self.taskbar.as_ref() {
            Some(tb) => tb,
            None => return,
        };
        let own =
            |w: u32| -> bool { w == tb.window.id() || tb.applets.iter().any(|a| a.owns_window(w)) };
        match *event {
            BackendEvent::Expose { window, .. } if own(window) => {
                if !self.route_to_sub_applet(window, event) {
                    if let Some(tb) = self.taskbar.as_ref() {
                        let _ = tb.paint_window(window);
                    }
                }
            }
            BackendEvent::ButtonPress {
                window,
                point,
                button,
                ..
            } if own(window) => {
                if self.route_to_sub_applet(window, event) {
                    return;
                }
                if button == 3 {
                    if let Some(members) = self
                        .taskbar
                        .as_mut()
                        .and_then(|tb| tb.group_members_at(window, point.x))
                    {
                        self.show_group_menu(&members);
                        return;
                    }
                }
                if let Some(clicked) = self
                    .taskbar
                    .as_mut()
                    .and_then(|tb| tb.handle_click(window, point.x, point.y, button))
                {
                    if button == 3 {
                        if let Some(cid) = self.wm.cid_for_xid(clicked) {
                            crate::focus::focus_window(&mut self.wm, cid);
                        }
                        let ws_count = self.wm.config.workspace_count;
                        let root = self.backend.root().read_id();
                        let pos = self
                            .backend
                            .query_pointer(root)
                            .ok().map_or_else(|| Point::new(point.x, point.y), |p| {
                                Point::new(p.root_x as i32, p.root_y as i32)
                            });
                        let mut menu = crate::winmenu::WindowActionMenu::for_focused_client_opts(
                            ws_count,
                            &self.wm.theme_colours,
                            );
                        menu.show(&*self.backend, pos);
                        let rb: Arc<dyn RenderBackend> = self.wm.render_backend.clone().expect("render backend");
                        menu.enable_filter(&rb);
                        self.wm.win_menu = Some(menu);
                        return;
                    }
                    self.activate_taskbar_window(clicked);
                }
                if let Some(tb) = self.taskbar.as_ref() {
                    let _ = tb.paint_window(window);
                    let _ = self.backend.flush();
                }
            }
            BackendEvent::ButtonRelease {
                window,
                point,
                button,
                ..
            } if own(window) => {
                let clicked = self
                    .taskbar
                    .as_mut()
                    .and_then(|tb| tb.handle_release(window, point.x, point.y, button));
                if let Some(clicked) = clicked {
                    self.activate_taskbar_window(clicked);
                }
                if let Some(tb) = self.taskbar.as_ref() {
                    let _ = tb.paint();
                    let _ = self.backend.flush();
                }
            }
            BackendEvent::KeyPress { window, .. } if own(window) => {
                self.route_to_sub_applet(window, event);
            }
            BackendEvent::MotionNotify { window, point, .. } if own(window) => {
                let backend = &self.backend;
                let _ = self.taskbar.as_mut().map(|tb| {
                    for a in &mut tb.applets {
                        if a.window().id() == window {
                            a.handle_motion(point.x, point.y);
                        } else if a.owns_window(window) {
                            a.handle_other_event(event, backend);
                        }
                    }
                });
            }
            BackendEvent::EnterNotify { window, .. } if own(window) => {
                let backend = &self.backend;
                let _ = self.taskbar.as_mut().map(|tb| {
                    for a in &mut tb.applets {
                        if a.window().id() == window {
                            a.handle_enter();
                        } else if a.owns_window(window) {
                            a.handle_other_event(event, backend);
                        }
                    }
                });
            }
            BackendEvent::LeaveNotify { window, .. } if own(window) => {
                let backend = &self.backend;
                let _ = self.taskbar.as_mut().map(|tb| {
                    for a in &mut tb.applets {
                        if a.window().id() == window {
                            a.handle_leave();
                        } else if a.owns_window(window) {
                            a.handle_other_event(event, backend);
                        }
                    }
                });
            }
            _ => {}
        }
    }

    fn handle_super_tap(&mut self, event: &BackendEvent) -> bool {
        const SUPER_L: u32 = 0xFFEB;
        const SUPER_R: u32 = 0xFFEC;
        match event {
            BackendEvent::KeyPress { keycode, .. } => {
                let ks = self.lookup_keysym(*keycode);
                if ks == SUPER_L || ks == SUPER_R {
                    self.super_tap_armed = true;
                    return true;
                }
                self.super_tap_armed = false;
                false
            }
            BackendEvent::KeyRelease { keycode, .. } => {
                let ks = self.lookup_keysym(*keycode);
                if ks == SUPER_L || ks == SUPER_R {
                    self.super_tap_armed = false;
                    return true;
                }
                false
            }
            BackendEvent::ButtonPress { .. } => {
                self.super_tap_armed = false;
                false
            }
            _ => false,
        }
    }

    fn handle_switcher_event(&mut self, event: &BackendEvent) {
        const MODIFIERS: [u32; 9] = [
            0xFFE9, 0xFFEA, 0xFFE7, 0xFFE8, 0xFFE1, 0xFFE2, 0xFFE3, 0xFFE4, 0xFF7E,
        ];
        const ALT_META: [u32; 4] = [0xFFE9, 0xFFEA, 0xFFE7, 0xFFE8];
        match event {
            BackendEvent::KeyPress { keycode, state, .. } => {
                let ks = self.lookup_keysym(*keycode);
                if ks == 0xFF09 {
                    self.switcher.cycle((*state & 0x01) == 0);
                    self.switcher.paint(&self.backend);
                } else if MODIFIERS.contains(&ks) {
                } else {
                    use crate::switcher::SwitcherKey;
                    let outcome = crate::bindings::keymap(self.backend.as_ref())
                        .map_or(SwitcherKey::Unhandled, |m| {
                            self.switcher
                                .handle_search_key(&self.backend, *keycode, *state, &m)
                        });
                    match outcome {
                        SwitcherKey::Filtered => {}
                        SwitcherKey::Submitted
                        | SwitcherKey::Cancelled
                        | SwitcherKey::Unhandled => {
                            self.switcher.hide(&self.backend, &mut self.wm);
                        }
                    }
                }
            }
            BackendEvent::KeyRelease { keycode, .. }
                if ALT_META.contains(&self.lookup_keysym(*keycode)) =>
            {
                self.switcher.hide(&self.backend, &mut self.wm);
            }
            BackendEvent::Expose { .. } => self.switcher.paint(&self.backend),
            _ => {}
        }
    }

    fn lookup_keysym(&self, kc: u32) -> u32 {
        crate::bindings::keysym_for_keycode(self.backend.as_ref(), kc)
    }

    fn route_to_sub_applet(&mut self, window: u32, event: &BackendEvent) -> bool {
        let is_sub = self.taskbar.as_ref().map_or(false, |tb| {
            tb.applets
                .iter()
                .any(|a| a.owns_window(window) && a.window().id() != window)
        });
        if is_sub {
            let backend = &self.backend;
            let _ = self.taskbar.as_mut().map(|tb| {
                for a in &mut tb.applets {
                    if a.owns_window(window) && a.window().id() != window {
                        a.handle_other_event(event, backend);
                    }
                }
            });
        }
        is_sub
    }

    pub(crate) fn activate_taskbar_window(&mut self, clicked: u32) {
        let clicked = match self.wm.cid_for_xid(clicked) {
            Some(c) => c,
            None => return,
        };
        let minimized = self
            .wm
            .frames
            .get(&clicked)
            .map_or(false, |fw| fw.state().minimized);
        if minimized {
            if let Some(fw) = self.wm.frame_mut(clicked) {
                fw.state_mut().minimized = false;
            }
            crate::wmaction::set_minimized_visible(&mut self.wm, clicked, false);
            crate::placement::set_transients_minimized(&mut self.wm.frames, clicked, false);
            crate::focus::focus_window(&mut self.wm, clicked);
        } else if self.wm.focused_window == Some(clicked) {
            if let Some(fw) = self.wm.frame_mut(clicked) {
                fw.state_mut().minimized = true;
            }
            crate::wmaction::set_minimized_visible(&mut self.wm, clicked, true);
            crate::placement::set_transients_minimized(&mut self.wm.frames, clicked, true);
        } else {
            crate::focus::focus_window(&mut self.wm, clicked);
        }
    }

    pub(crate) fn handle_screen_resize(&mut self, width: u16, height: u16) {
        self.backend.set_screen_size(width, height);
        let (mm_w, mm_h) = self.backend.screen_size_mm();
        let dpi = antibox_core::scale::detect(
            width as u32,
            height as u32,
            mm_w,
            mm_h,
            self.backend.preferred_scale(),
        );
        antibox_core::scale::set_dpi(dpi);
        apply_xft_dpi(&*self.backend, antibox_core::scale::dpi());
        eprintln!("screen resized to {}x{}; DPI recalculated to {}", width, height, antibox_core::scale::dpi());
        self.wm.monitors = self.backend.query_monitors().unwrap_or_default();
        if let Some(ref mut tb) = self.taskbar {
            tb.fit_to_screen();
            self.wm.reserved_strut = tb.strut();
        }
        crate::placement::update_workarea_from_struts(&mut self.wm);
        crate::wmaction::refit_fullscreen(&mut self.wm);
        crate::ewmh::update_desktop_geometry(&*self.backend, &self.wm.atoms);
        self.clamp_windows_onscreen();
        let _ = self.backend.flush();
    }

    fn clamp_windows_onscreen(&mut self) {
        const MARGIN: i32 = 24;
        let sw = self.backend.screen_width() as i32;
        let sh = self.backend.screen_height() as i32;
        let moves: Vec<(ClientId, i32, i32)> = self
            .wm
            .frames
            .iter()
            .filter_map(|(&id, fw)| {
                let st = fw.state();
                if st.fullscreen || st.maximized || st.max_vert || st.max_horz || st.minimized {
                    return None;
                }
                let r = fw.frame_rect();
                let nx = r.x.clamp((MARGIN - r.w).min(0), (sw - MARGIN).max(0));
                let ny = r.y.clamp(0, (sh - MARGIN).max(0));
                if nx != r.x || ny != r.y { Some((id, nx, ny)) } else { None }
            })
            .collect();
        for (id, nx, ny) in moves {
            if let Some(fw) = self.wm.frame_mut(id) {
                let mut r = fw.frame_rect();
                r.x = nx;
                r.y = ny;
                fw.set_frame_rect(r);
            }
            if let Some(fw) = self.wm.frame(id) {
                let _ = fw.frame().move_window(Point::new(nx, ny));
                crate::drag::send_client_configure(fw, &*self.backend);
            }
        }
    }


    fn show_window_list(&mut self) {
        if self.winlist.visible {
            self.winlist.hide(&self.backend);
            return;
        }
        self.winlist.show(&self.backend, &self.wm);
        let id = self.winlist.client_id();
        if id != 0 {
            self.wm.self_windows.insert(id);
            crate::handler::map_request_ex(&mut self.wm, id, true);
            if let Some(cid) = self.wm.cid_for_xid(id) {
                crate::focus::focus_window(&mut self.wm, cid);
            }
        }
    }

    fn show_group_menu(&mut self, members: &[u32]) {
        use crate::menu::{MenuRow, MenuView};
        let rows: Vec<MenuRow<u32>> = members
            .iter()
            .filter_map(|&xid| {
                let cid = self.wm.cid_for_xid(xid)?;
                let title = self.wm.frames.get(&cid)?.client().title().to_string();
                Some(MenuRow::item(title, xid))
            })
            .collect();
        if rows.is_empty() {
            return;
        }
        let root = self.backend.root().read_id();
        let pos = self
            .backend
            .query_pointer(root)
            .map_or(Point::new(0, 0), |p| {
                Point::new(p.root_x as i32, p.root_y as i32)
            });
        let mut menu = MenuView::with_items(rows);
        menu.show(&*self.backend, pos);
        let rb: Arc<dyn RenderBackend> = self.wm.render_backend.clone().expect("render backend");
        menu.enable_filter(&rb);
        self.group_menu = Some(menu);
    }

    fn handle_group_menu_event(&mut self, event: &BackendEvent) -> bool {
        use crate::menu::MenuNav;
        let mut menu = match self.group_menu.take() {
            Some(menu) => menu,
            None => return false,
        };
        if !menu.visible {
            return false;
        }
        match event {
            BackendEvent::ButtonPress {
                window,
                point,
                root,
                button,
                ..
            } => {
                if menu.handle_bar_button(&*self.backend, *window, *point, *button) {
                    self.group_menu = Some(menu);
                    return true;
                }
                let target = if menu.contains_window(*window) {
                    menu.handle_click(*root)
                } else {
                    None
                };
                menu.hide(&*self.backend);
                if let Some(xid) = target {
                    if let Some(cid) = self.wm.cid_for_xid(xid) {
                        crate::focus::activate_window(&mut self.wm, cid);
                    }
                    let _ = self.backend.flush();
                }
                true
            }
            BackendEvent::MotionNotify { window, root, .. } if menu.contains_window(*window) => {
                menu.handle_motion(&*self.backend, *root);
                self.group_menu = Some(menu);
                true
            }
            BackendEvent::Expose { window, .. } if menu.contains_window(*window) => {
                menu.paint(&*self.backend);
                self.group_menu = Some(menu);
                true
            }
            BackendEvent::KeyPress { keycode, state, .. } => {
                let ks = self.lookup_keysym(*keycode);
                let nav = match crate::bindings::keymap(self.backend.as_ref()) {
                    Some(m) => menu.handle_key_input(&*self.backend, *keycode, *state, &m, ks),
                    None => menu.handle_key(&*self.backend, ks),
                };
                match nav {
                    MenuNav::Ignored | MenuNav::Handled => self.group_menu = Some(menu),
                    MenuNav::Close => menu.hide(&*self.backend),
                    MenuNav::Activate(xid) => {
                        menu.hide(&*self.backend);
                        if let Some(cid) = self.wm.cid_for_xid(xid) {
                        crate::focus::activate_window(&mut self.wm, cid);
                    }
                        let _ = self.backend.flush();
                    }
                }
                true
            }
            _ => {
                self.group_menu = Some(menu);
                false
            }
        }
    }

    fn handle_winlist_event(&mut self, event: &BackendEvent) {
        match event {
            BackendEvent::ButtonPress {
                button,
                point,
                window,
                ..
            } => {
                let _ = self.backend.allow_events(0, 0);
                if self.winlist.handle_bar_button(
                    &self.backend,
                    &mut self.wm,
                    *window,
                    *point,
                    *button,
                ) {
                    return;
                }
                if *button == 3 {
                    if let Some(id) = self.winlist.window_at(*point) {
                        self.winlist.paint(&self.backend);
                        if let Some(cid) = self.wm.cid_for_xid(id) {
                            crate::focus::focus_window(&mut self.wm, cid);
                        }
                        let ws_count = self.wm.config.workspace_count;
                        let root = self.backend.root().read_id();
                        let pos = self
                            .backend
                            .query_pointer(root)
                            .ok().map_or_else(|| Point::new(point.x, point.y), |p| Point::new(p.root_x as i32, p.root_y as i32));
                        let mut menu = crate::winmenu::WindowActionMenu::for_focused_client_opts(
                            ws_count,
                            &self.wm.theme_colours,
                            );
                        menu.show(&*self.backend, pos);
                        let rb: Arc<dyn RenderBackend> = self.wm.render_backend.clone().expect("render backend");
                        menu.enable_filter(&rb);
                        self.wm.win_menu = Some(menu);
                    }
                    return;
                }
                let activated =
                    self.winlist
                        .handle_click(&self.backend, &mut self.wm, *button, *point);
                if !activated {
                    let cid = self.winlist.client_id();
                    if let Some(cid) = self.wm.cid_for_xid(cid) {
                        crate::focus::focus_window(&mut self.wm, cid);
                    }
                }
            }
            BackendEvent::ButtonRelease { .. } => self.winlist.end_drag(),
            BackendEvent::MotionNotify { point, .. } => {
                self.winlist.handle_motion(&self.backend, *point);
            }
            BackendEvent::KeyPress { keycode, state, .. } => {
                let ks = self.lookup_keysym(*keycode);
                let close = if let Some(m) = crate::bindings::keymap(self.backend.as_ref()) {
                    self.winlist.handle_key_input(
                        &self.backend,
                        &mut self.wm,
                        *keycode,
                        *state,
                        &m,
                        ks,
                    )
                } else {
                    self.winlist.handle_key(&self.backend, &mut self.wm, ks)
                };
                if close {
                    self.winlist.hide(&self.backend);
                }
            }
            BackendEvent::Expose { .. } => self.winlist.paint(&self.backend),
            BackendEvent::ConfigureNotify { rect, .. } => {
                self.winlist
                    .on_configure(&self.backend, rect.w as u16, rect.h as u16);
            }
            _ => {}
        }
    }

    pub fn release_clients(&self) {
        let root = self.backend.root().read_id();
        for (id, fw) in self.wm.frames.iter() {
            let cr = fw.client_rect();
            let _ = self
                .backend
                .reparent_window(self.wm.xid_index.xid_of(*id), root, Point::new(cr.x, cr.y));
            let _ = fw.client().xwindow.map();
        }
        let _ = self.backend.flush();
    }
}

