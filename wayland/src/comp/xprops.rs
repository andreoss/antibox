use antibox_core::backend::BackendEvent;
use smithay::reexports::x11rb::properties::{
    WmHints, WmHintsState, WmSizeHints, WmSizeHintsSpecification,
};
use smithay::xwayland::xwm::WmWindowProperty;
use smithay::xwayland::X11Surface;

use super::state::Compositor;
use antibox_core::backend::hints::{size_hints_flags, wm_hints_flags};

fn encode_size_hints(h: &WmSizeHints) -> Vec<u8> {
    let mut words = [0u32; 18];
    if let Some((spec, x, y)) = h.position {
        words[0] |= match spec {
            WmSizeHintsSpecification::UserSpecified => size_hints_flags::US_POSITION,
            WmSizeHintsSpecification::ProgramSpecified => size_hints_flags::P_POSITION,
        };
        words[1] = x as u32;
        words[2] = y as u32;
    }
    if let Some((spec, w, hh)) = h.size {
        words[0] |= match spec {
            WmSizeHintsSpecification::UserSpecified => size_hints_flags::US_SIZE,
            WmSizeHintsSpecification::ProgramSpecified => size_hints_flags::P_SIZE,
        };
        words[3] = w as u32;
        words[4] = hh as u32;
    }
    if let Some((w, hh)) = h.min_size {
        words[0] |= size_hints_flags::P_MIN_SIZE;
        words[5] = w as u32;
        words[6] = hh as u32;
    }
    if let Some((w, hh)) = h.max_size {
        words[0] |= size_hints_flags::P_MAX_SIZE;
        words[7] = w as u32;
        words[8] = hh as u32;
    }
    if let Some((w, hh)) = h.size_increment {
        words[0] |= size_hints_flags::P_RESIZE_INC;
        words[9] = w as u32;
        words[10] = hh as u32;
    }
    if let Some((min, max)) = h.aspect {
        words[0] |= size_hints_flags::P_ASPECT;
        words[11] = min.numerator as u32;
        words[12] = min.denominator as u32;
        words[13] = max.numerator as u32;
        words[14] = max.denominator as u32;
    }
    if let Some((w, hh)) = h.base_size {
        words[0] |= size_hints_flags::P_BASE_SIZE;
        words[15] = w as u32;
        words[16] = hh as u32;
    }
    if let Some(g) = h.win_gravity {
        words[0] |= size_hints_flags::P_WIN_GRAVITY;
        words[17] = u32::from(g);
    }
    words.iter().flat_map(|w| w.to_ne_bytes()).collect()
}

fn encode_wm_hints(h: &WmHints) -> Vec<u8> {
    let mut words = [0u32; 9];
    if let Some(input) = h.input {
        words[0] |= wm_hints_flags::INPUT_HINT;
        words[1] = input as u32;
    }
    if let Some(state) = h.initial_state {
        words[0] |= wm_hints_flags::STATE_HINT;
        words[2] = match state {
            WmHintsState::Normal => 1,
            WmHintsState::Iconic => 3,
        };
    }
    if let Some(p) = h.icon_pixmap {
        words[0] |= wm_hints_flags::ICON_PIXMAP_HINT;
        words[3] = p;
    }
    if let Some(w) = h.icon_window {
        words[0] |= wm_hints_flags::ICON_WINDOW_HINT;
        words[4] = w;
    }
    if let Some((x, y)) = h.icon_position {
        words[0] |= wm_hints_flags::ICON_POSITION_HINT;
        words[5] = x as u32;
        words[6] = y as u32;
    }
    if let Some(m) = h.icon_mask {
        words[0] |= wm_hints_flags::ICON_MASK_HINT;
        words[7] = m;
    }
    if let Some(g) = h.window_group {
        words[0] |= wm_hints_flags::WINDOW_GROUP_HINT;
        words[8] = g;
    }
    if h.urgent {
        words[0] |= 1 << 8;
    }
    words.iter().flat_map(|w| w.to_ne_bytes()).collect()
}

fn window_type_atom_name(surface: &X11Surface) -> &'static str {
    use smithay::xwayland::xwm::WmWindowType;
    match surface.window_type() {
        Some(WmWindowType::Dialog) => "_NET_WM_WINDOW_TYPE_DIALOG",
        Some(WmWindowType::DropdownMenu) => "_NET_WM_WINDOW_TYPE_DROPDOWN_MENU",
        Some(WmWindowType::Menu) => "_NET_WM_WINDOW_TYPE_MENU",
        Some(WmWindowType::Notification) => "_NET_WM_WINDOW_TYPE_NOTIFICATION",
        Some(WmWindowType::PopupMenu) => "_NET_WM_WINDOW_TYPE_POPUP_MENU",
        Some(WmWindowType::Splash) => "_NET_WM_WINDOW_TYPE_SPLASH",
        Some(WmWindowType::Toolbar) => "_NET_WM_WINDOW_TYPE_TOOLBAR",
        Some(WmWindowType::Tooltip) => "_NET_WM_WINDOW_TYPE_TOOLTIP",
        Some(WmWindowType::Utility) => "_NET_WM_WINDOW_TYPE_UTILITY",
        _ => "_NET_WM_WINDOW_TYPE_NORMAL",
    }
}

impl Compositor {
    pub(crate) fn put_prop(&self, id: u32, name: &str, data: Vec<u8>) {
        let mut s = self.shared.lock();
        let atom = s.atoms.intern(name);
        if s.props.get(&(id, atom)) == Some(&data) {
            return;
        }
        s.props.insert((id, atom), data);
        s.events.push(BackendEvent::PropertyNotify {
            window: id,
            atom,
            state: 0,
        });
    }

    fn drop_prop(&self, id: u32, name: &str) {
        let mut s = self.shared.lock();
        let atom = s.atoms.intern(name);
        if s.props.remove(&(id, atom)).is_some() {
            s.events.push(BackendEvent::PropertyNotify {
                window: id,
                atom,
                state: 1,
            });
        }
    }

    pub(crate) fn sync_x11_prop(&mut self, id: u32, surface: &X11Surface, prop: WmWindowProperty) {
        match prop {
            WmWindowProperty::Title => {
                let title = surface.title();
                self.put_prop(id, "_NET_WM_NAME", title.as_bytes().to_vec());
                self.put_prop(id, "WM_NAME", title.into_bytes());
            }
            WmWindowProperty::Class => {
                let mut data = surface.instance().into_bytes();
                data.push(0);
                data.extend_from_slice(surface.class().as_bytes());
                data.push(0);
                self.put_prop(id, "WM_CLASS", data);
            }
            WmWindowProperty::NormalHints => {
                if let Some(h) = surface.size_hints() {
                    self.put_prop(id, "WM_NORMAL_HINTS", encode_size_hints(&h));
                } else {
                    self.drop_prop(id, "WM_NORMAL_HINTS");
                }
            }
            WmWindowProperty::Hints => {
                if let Some(h) = surface.hints() {
                    self.put_prop(id, "WM_HINTS", encode_wm_hints(&h));
                } else {
                    self.drop_prop(id, "WM_HINTS");
                }
            }
            WmWindowProperty::TransientFor => {
                let parent = surface.is_transient_for().and_then(|xid| {
                    self.clients.iter().find_map(|(cid, w)| {
                        w.x11_surface()
                            .map(|x| x.window_id() == xid)
                            .unwrap_or(false)
                            .then_some(*cid)
                    })
                });
                match parent {
                    Some(p) => self.put_prop(id, "WM_TRANSIENT_FOR", p.to_ne_bytes().to_vec()),
                    None => self.drop_prop(id, "WM_TRANSIENT_FOR"),
                }
            }
            WmWindowProperty::WindowType => {
                let type_atom = {
                    let mut s = self.shared.lock();
                    s.atoms.intern(window_type_atom_name(surface))
                };
                self.put_prop(id, "_NET_WM_WINDOW_TYPE", type_atom.to_ne_bytes().to_vec());
            }
            WmWindowProperty::MotifHints => {
                if !surface.is_decorated() {
                    self.drop_prop(id, "_MOTIF_WM_HINTS");
                } else {
                    let words: [u32; 4] = [
                        antibox_core::backend::hints::mwm_hints_flags::DECORATIONS,
                        0,
                        0,
                        0,
                    ];
                    let data = words.iter().flat_map(|w| w.to_ne_bytes()).collect();
                    self.put_prop(id, "_MOTIF_WM_HINTS", data);
                }
            }
            WmWindowProperty::Pid => {
                if let Some(pid) = surface.pid() {
                    self.put_prop(id, "_NET_WM_PID", pid.to_ne_bytes().to_vec());
                }
            }
            WmWindowProperty::Protocols | WmWindowProperty::StartupId => {}
        }
    }

    pub(crate) fn sync_x11_props(&mut self, id: u32, surface: &X11Surface) {
        for prop in [
            WmWindowProperty::Title,
            WmWindowProperty::Class,
            WmWindowProperty::NormalHints,
            WmWindowProperty::Hints,
            WmWindowProperty::TransientFor,
            WmWindowProperty::WindowType,
            WmWindowProperty::MotifHints,
            WmWindowProperty::Pid,
        ] {
            self.sync_x11_prop(id, surface, prop);
        }
    }
}
