fn read_gtk_extents(wh: &dyn WindowHandle, atom: u32) -> (bool, [i32; 4]) {
    if let Ok(Some(d)) = wh.get_property(atom, 0, 4) {
        if d.len() >= 16 {
            let e = |i: usize| {
                i32::from_ne_bytes([d[i * 4], d[i * 4 + 1], d[i * 4 + 2], d[i * 4 + 3]])
            };
            return (true, [e(0), e(1), e(2), e(3)]);
        }
    }
    (false, [0; 4])
}

fn read_atom_list(wh: &dyn WindowHandle, atom: u32) -> Vec<u32> {
    if let Ok(Some(data)) = wh.get_property(atom, 0, 1024) {
        data.chunks_exact(4)
            .map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    } else {
        Vec::new()
    }
}

impl ClientWindow {
    fn fetch_bytes(&self, atoms: &AtomManager, name: &str) -> Option<Vec<u8>> {
        let atom = atoms.get(name)?;
        self.xwindow.get_property(atom, 0, 1024).ok().and_then(|v| v)
    }

    fn fetch_string(&self, atoms: &AtomManager, name: &str) -> Option<String> {
        self.fetch_bytes(atoms, name).and_then(|data| {
            String::from_utf8(data)
                .ok()
                .map(|s| s.trim_end_matches('\0').to_string())
        })
    }

    fn read_u32_prop(&self, atom: u32) -> Option<u32> {
        self.xwindow.get_property(atom, 0, 1).ok().and_then(|data| {
            data.filter(|d| d.len() >= 4)
                .map(|d| u32::from_ne_bytes([d[0], d[1], d[2], d[3]]))
        })
    }

    fn read_string_prop(&self, atom: u32, max_len: u32) -> Option<String> {
        self.xwindow
            .get_property(atom, 0, max_len)
            .ok()
            .and_then(|data| {
                data.map(|d| {
                    String::from_utf8_lossy(&d)
                        .trim_end_matches('\0')
                        .to_string()
                })
            })
    }

    pub fn refresh_size_hints(&mut self, normal_hints_atom: u32) {
        self.size_hints = self
            .xwindow
            .get_property(normal_hints_atom, 0, 18)
            .ok()
            .and_then(|v| v)
            .and_then(|data| SizeHints::from_bytes(&data));
    }

    pub fn read_initial_properties<H: DisplayBackend + 'static + ?Sized>(
        &mut self,
        backend: &H,
        atoms: &AtomManager,
    ) {
        if let Some(title) = self.fetch_string(atoms, "_NET_WM_NAME") {
            self.title = title;
        }
        if self.title.is_empty() {
            if let Some(title) = self.fetch_string(atoms, "WM_NAME") {
                self.title = title;
            }
        }
        if let Some(atom) = atoms.get("_NET_WM_WINDOW_TYPE") {
            if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 1) {
                if data.len() >= 4 {
                    let type_atom = u32::from_ne_bytes([data[0], data[1], data[2], data[3]]);
                    let is_type = |name: &str| -> bool { atoms.get(name) == Some(type_atom) };
                    self.window_type = if is_type("_NET_WM_WINDOW_TYPE_DESKTOP") {
                        WindowType::Desktop
                    } else if is_type("_NET_WM_WINDOW_TYPE_DOCK") {
                        WindowType::Dock
                    } else if is_type("_NET_WM_WINDOW_TYPE_TOOLBAR") {
                        WindowType::Toolbar
                    } else if is_type("_NET_WM_WINDOW_TYPE_MENU") {
                        WindowType::Menu
                    } else if is_type("_NET_WM_WINDOW_TYPE_UTILITY") {
                        WindowType::Utility
                    } else if is_type("_NET_WM_WINDOW_TYPE_SPLASH") {
                        WindowType::Splash
                    } else if is_type("_NET_WM_WINDOW_TYPE_DIALOG") {
                        WindowType::Dialog
                    } else {
                        WindowType::Normal
                    };
                }
            }
        }
        if let Some(atom) = atoms.get("_NET_WM_PID") {
            self.pid = self.read_u32_prop(atom).unwrap_or(0);
            if !self.xpra_resolved && self.pid != 0 {
                self.is_xpra = crate::proc_reader::pid_command(self.pid)
                    .map_or(false, |cmd| cmd.to_ascii_lowercase().contains("xpra"));
                self.xpra_resolved = true;
            }
        }
        if let Some(val) = self.fetch_string(atoms, "_NET_STARTUP_ID") {
            if !val.is_empty() {
                self.startup_id = Some(val);
            }
        }
        if let Some(atom) = atoms.get("WM_PROTOCOLS") {
            self.protocols = read_atom_list(&*self.xwindow, atom);
        }
        if let Some(atom) = atoms.get("WM_HINTS") {
            if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 9) {
                self.wm_hints = WmHints::from_bytes(&data);
            }
        }
        if let Some(atom) = atoms.get("WM_NORMAL_HINTS") {
            if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 18) {
                self.size_hints = SizeHints::from_bytes(&data);
            }
        }
        if let Some(atom) = atoms.get("_MOTIF_WM_HINTS") {
            if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 4) {
                self.mwm_hints = MwmHints::from_bytes(&data);
            }
        }
        if let Some(atom) = atoms.get("_GTK_FRAME_EXTENTS") {
            let (csd, ext) = read_gtk_extents(&*self.xwindow, atom);
            self.csd = csd;
            self.csd_extents = ext;
        }
        if let Some(atom) = atoms.get("_NET_WM_XAPP_PROGRESS") {
            self.progress = self.read_u32_prop(atom).map(|v| v.min(100) as u8);
        }
        if let Some(atom) = atoms.get("_NET_WM_USER_TIME") {
            self.user_time = self.read_u32_prop(atom).unwrap_or(0);
        }
        if let Some(atom) = atoms.get("WM_TRANSIENT_FOR") {
            self.transient_for = self.read_u32_prop(atom);
        }
        if let Some(atom) = atoms.get("_NET_WM_ICON") {
            if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 1 << 20) {
                self.icon = IconData::from_property(&data);
            }
        }
        if let Some(atom) = atoms.get("_NET_WM_STRUT") {
            if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 4) {
                self.strut = Strut::from_bytes(&data);
            }
        }
        if self.strut.is_none() {
            if let Some(atom) = atoms.get("_NET_WM_STRUT_PARTIAL") {
                if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 12) {
                    self.strut = Strut::from_bytes(&data);
                }
            }
        }

        if let Some(raw) = self.fetch_string(atoms, "WM_CLASS") {
            let parts: Vec<&str> = raw.splitn(2, '\0').collect();
            if parts.len() == 2 {
                let instance = parts[0].trim();
                let class = parts[1].trim();
                if !instance.is_empty() && !class.is_empty() {
                    self.class_instance = Some(format!("{}.{}", instance, class));
                }
            } else if !raw.is_empty() {
                self.class_instance = Some(raw);
            }
        }

        if let Some(atom) = atoms.get("WM_CLIENT_LEADER") {
            if let Ok(Some(data)) = self.xwindow.get_property(atom, 0, 1) {
                if data.len() >= 4 {
                    self.leader_window = u32::from_ne_bytes([data[0], data[1], data[2], data[3]]);
                }
            }
        }
        if self.leader_window == 0 {
            if let Some(hints) = &self.wm_hints {
                if hints.window_group != 0 {
                    self.leader_window = hints.window_group;
                }
            }
        }
        if self.leader_window != 0 {
            if let Some(atom) = atoms.get("SM_CLIENT_ID") {
                if let Ok(Some(data)) = backend.get_property(self.leader_window, atom, 0, 0, 256) {
                    let s = String::from_utf8_lossy(&data)
                        .trim_end_matches('\0')
                        .to_string();
                    if !s.is_empty() {
                        self.client_id = Some(s);
                    }
                }
            }
        }
        if let Some(atom) = atoms.get("WINDOW_ROLE") {
            self.window_role = self.read_string_prop(atom, 256);
        }
        if let Some(atom) = atoms.get("_NET_WM_STATE") {
            self.wm_state = read_atom_list(&*self.xwindow, atom);
        }
        self.read_user_time(backend, atoms);
    }

    fn read_user_time<H: DisplayBackend + 'static + ?Sized>(
        &mut self,
        backend: &H,
        atoms: &AtomManager,
    ) {
        let ut = match atoms.get("_NET_WM_USER_TIME") {
            Some(ut) => ut,
            None => return,
        };
        let time_window = atoms
            .get("_NET_WM_USER_TIME_WINDOW")
            .and_then(|a| self.read_u32_prop(a))
            .filter(|&w| w != 0);
        let value = match time_window {
            Some(w) => backend
                .get_property(w, ut, 0, 0, 1)
                .ok()
                .and_then(|v| v)
                .filter(|d| d.len() >= 4)
                .map(|d| u32::from_ne_bytes([d[0], d[1], d[2], d[3]])),
            None => self.read_u32_prop(ut),
        };
        if let Some(v) = value {
            self.user_time = v;
            self.user_time_set = true;
        }
    }

    pub fn net_state_request<H: DisplayBackend + 'static + ?Sized>(
        &mut self,
        backend: &H,
        atoms: &AtomManager,
        action: u32,
        atom1: u32,
        atom2: u32,
    ) {
        let apply = |state: &mut Vec<u32>, atom: u32, act: u32| {
            if atom == 0 {
                return;
            }
            let has = state.contains(&atom);
            match act {
                0 => {
                    state.retain(|&a| a != atom);
                }
                1 => {
                    if !has {
                        state.push(atom);
                    }
                }
                _ => {
                    if has {
                        state.retain(|&a| a != atom);
                    } else {
                        state.push(atom);
                    }
                }
            }
        };
        apply(&mut self.wm_state, atom1, action);
        if atom2 != 0 && atom2 != atom1 {
            apply(&mut self.wm_state, atom2, action);
        }
        if let Some(state_atom) = atoms.get("_NET_WM_STATE") {
            const ATOM_ATOM: u32 = 4;
            let _ = backend.change_property32(
                PropMode::Replace,
                self.xwindow.id(),
                state_atom,
                ATOM_ATOM,
                &self.wm_state,
            );
        }
    }

    pub fn handle_property_notify(
        &mut self,
        atoms: &AtomManager,
        event: &BackendEvent,
    ) {
        if let BackendEvent::PropertyNotify { atom, .. } = event {
            if Some(*atom) == atoms.get("_NET_WM_NAME") {
                if let Ok(Some(data)) = self.xwindow.get_property(*atom, 0, 1024) {
                    if let Ok(s) = String::from_utf8(data) {
                        self.title = s.trim_end_matches('\0').to_string();
                    }
                }
            } else if self.title.is_empty() && Some(*atom) == atoms.get("WM_NAME") {
                if let Ok(Some(data)) = self.xwindow.get_property(*atom, 0, 1024) {
                    let s = String::from_utf8_lossy(&data);
                    self.title = s.trim_end_matches('\0').to_string();
                }
            } else if Some(*atom) == atoms.get("WINDOW_ROLE") {
                self.window_role = self.read_string_prop(*atom, 256);
            } else if Some(*atom) == atoms.get("WM_PROTOCOLS") {
                self.protocols = read_atom_list(&*self.xwindow, *atom);
            } else if Some(*atom) == atoms.get("WM_HINTS") {
                if let Ok(Some(data)) = self.xwindow.get_property(*atom, 0, 9) {
                    self.wm_hints = WmHints::from_bytes(&data);
                }
            } else if Some(*atom) == atoms.get("WM_NORMAL_HINTS") {
                if let Ok(Some(data)) = self.xwindow.get_property(*atom, 0, 18) {
                    self.size_hints = SizeHints::from_bytes(&data);
                }
            } else if Some(*atom) == atoms.get("_MOTIF_WM_HINTS") {
                if let Ok(Some(data)) = self.xwindow.get_property(*atom, 0, 4) {
                    self.mwm_hints = MwmHints::from_bytes(&data);
                }
            } else if Some(*atom) == atoms.get("_GTK_FRAME_EXTENTS") {
                let (csd, ext) = read_gtk_extents(&*self.xwindow, *atom);
                self.csd = csd;
                self.csd_extents = ext;
            } else if Some(*atom) == atoms.get("_NET_WM_XAPP_PROGRESS") {
                self.progress = self.read_u32_prop(*atom).map(|v| v.min(100) as u8);
            } else if Some(*atom) == atoms.get("_NET_WM_USER_TIME") {
                self.user_time = self.read_u32_prop(*atom).unwrap_or(0);
            } else if Some(*atom) == atoms.get("_NET_WM_ICON") {
                if let Ok(Some(data)) = self.xwindow.get_property(*atom, 0, 1 << 20) {
                    self.icon = IconData::from_property(&data);
                }
            } else if Some(*atom) == atoms.get("_NET_WM_STRUT")
                || Some(*atom) == atoms.get("_NET_WM_STRUT_PARTIAL")
            {
                if let Ok(Some(data)) = self.xwindow.get_property(*atom, 0, 12) {
                    self.strut = Strut::from_bytes(&data);
                }
            }
        }
    }
}
