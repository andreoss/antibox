use crate::sync::atomic::LazyLock;

use crate::backend::event::BackendEvent;

const EVENT_VARIANT_COUNT: usize = 27;

type AtomResolver = Box<dyn Fn(u32) -> Option<String> + Send + Sync>;

static ATOM_RESOLVER: LazyLock<Option<AtomResolver>> = LazyLock::new();

static EVENT_FILTER: LazyLock<Option<Box<EventFilter>>> = LazyLock::new();

pub struct EventFilter {
    enabled: [bool; EVENT_VARIANT_COUNT],
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventFilter {
    pub const fn new() -> Self {
        Self {
            enabled: [false; EVENT_VARIANT_COUNT],
        }
    }

    pub fn new_default() -> Self {
        let mut f = Self::new();
        f.set(
            &BackendEvent::KeyPress {
                window: 0,
                event: 0,
                keycode: 0,
                state: 0,
            },
            true,
        );
        f.set(
            &BackendEvent::KeyRelease {
                window: 0,
                keycode: 0,
                state: 0,
            },
            true,
        );
        f.set(
            &BackendEvent::ButtonPress {
                window: 0,
                event: 0,
                point: crate::point::Point::ZERO,
                root: crate::point::Point::ZERO,
                button: 0,
                state: 0,
            },
            true,
        );
        f.set(
            &BackendEvent::ButtonRelease {
                window: 0,
                point: crate::point::Point::ZERO,
                button: 0,
            },
            true,
        );
        f.set(&BackendEvent::EnterNotify { window: 0, mode: 0 }, true);
        f.set(&BackendEvent::LeaveNotify { window: 0, mode: 0 }, true);
        f.set(&BackendEvent::FocusIn { window: 0 }, true);
        f.set(&BackendEvent::FocusOut { window: 0 }, true);
        f.set(
            &BackendEvent::CreateNotify {
                window: 0,
                parent: 0,
                rect: crate::rect::Rect::ZERO,
                border_width: 0,
            },
            true,
        );
        f.set(&BackendEvent::DestroyNotify { window: 0 }, true);
        f.set(&BackendEvent::UnmapNotify { window: 0 }, true);
        f.set(&BackendEvent::MapNotify { window: 0 }, true);
        f.set(&BackendEvent::MapRequest { window: 0 }, true);
        f.set(
            &BackendEvent::ReparentNotify {
                window: 0,
                parent: 0,
                x: 0,
                y: 0,
            },
            true,
        );
        f.set(
            &BackendEvent::ConfigureRequest {
                window: 0,
                parent: 0,
                rect: crate::rect::Rect::ZERO,
                border_width: 0,
                value_mask: 0,
            },
            true,
        );
        f.set(
            &BackendEvent::ClientMessage {
                window: 0,
                message_type: 0,
                format: 0,
                data: [0; 5],
            },
            true,
        );
        f.set(
            &BackendEvent::MappingNotify {
                request: 0,
                first_keycode: 0,
                count: 0,
            },
            true,
        );
        f
    }

    pub fn set(&mut self, ev: &BackendEvent, enabled: bool) {
        if let Some(idx) = variant_index(ev) {
            self.enabled[idx] = enabled;
        }
    }

    pub fn is_enabled(&self, ev: &BackendEvent) -> bool {
        variant_index(ev).is_some_and(|idx| self.enabled[idx])
    }
}

const fn variant_index(ev: &BackendEvent) -> Option<usize> {
    Some(ev.variant_index())
}

pub fn set_atom_resolver(f: impl Fn(u32) -> Option<String> + Send + Sync + 'static) {
    if let Ok(mut guard) = ATOM_RESOLVER.lock() {
        *guard = Some(Box::new(f));
    }
}

pub fn resolve_atom(atom: u32) -> Option<String> {
    ATOM_RESOLVER
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|r| r(atom)))
}

fn fmt_atom(atom: u32) -> String {
    match resolve_atom(atom) {
        Some(name) => format!("{name} (0x{atom:X})"),
        None => format!("0x{atom:X}"),
    }
}

pub fn should_log(ev: &BackendEvent) -> bool {
    EVENT_FILTER
        .lock()
        .ok()
        .map_or(true, |guard| guard.as_ref().map_or(true, |f| f.is_enabled(ev)))
}

pub fn init_log_events() {
    if let Ok(mut guard) = EVENT_FILTER.lock() {
        if guard.is_none() {
            *guard = Some(Box::new(EventFilter::new_default()));
        }
    }
}

pub const fn event_name(ev: &BackendEvent) -> &'static str {
    ev.name()
}

pub fn format_event(ev: &BackendEvent) -> String {
    match ev {
        BackendEvent::MapRequest { window } => {
            format!("window=0x{window:X}: MapRequest")
        }
        BackendEvent::ConfigureRequest {
            window,
            parent,
            rect,
            border_width,
            value_mask,
        } => {
            let mut s = format!(
                "window=0x{:X}: ConfigureRequest parent=0x{:X} \
                 ({},{}) {}x{} border={}",
                window,
                parent,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                border_width,
            );
            if *value_mask != 0 {
                use std::fmt::Write;
                let _ = write!(s, " mask=0x{value_mask:X}");
            }
            s
        }
        BackendEvent::DestroyNotify { window } => {
            format!("window=0x{window:X}: DestroyNotify")
        }
        BackendEvent::UnmapNotify { window } => {
            format!("window=0x{window:X}: UnmapNotify")
        }
        BackendEvent::Expose { window, rect } => {
            format!(
                "window=0x{:X}: Expose ({},{}) {}x{} count=0",
                window, rect.x, rect.y, rect.w, rect.h,
            )
        }
        BackendEvent::ClientMessage {
            window,
            message_type,
            format,
            data,
        } => {
            format!(
                "window=0x{:X}: ClientMessage atom={} \
                 fmt={} data=0x{:X},0x{:X},0x{:X},0x{:X},0x{:X}",
                window,
                fmt_atom(*message_type),
                format,
                data[0],
                data[1],
                data[2],
                data[3],
                data[4],
            )
        }
        BackendEvent::ButtonPress {
            window,
            event,
            point,
            root: _,
            button,
            state,
        } => {
            format!(
                "window=0x{:X}: ButtonPress event=0x{:X} \
                 ({},{}) button={} state=0x{:X}",
                window,
                event,
                point.x,
                point.y,
                button,
                state,
            )
        }
        BackendEvent::ButtonRelease {
            window,
            point,
            button,
        } => {
            format!(
                "window=0x{:X}: ButtonRelease ({},{}) button={}",
                window, point.x, point.y, button,
            )
        }
        BackendEvent::MotionNotify {
            window,
            point,
            root,
            state,
        } => {
            format!(
                "window=0x{:X}: MotionNotify ({},{}) root=({},{}) state=0x{:X}",
                window, point.x, point.y, root.x, root.y, state,
            )
        }
        BackendEvent::KeyPress {
            window,
            event,
            keycode,
            state,
        } => {
            format!(
                "window=0x{window:X}: KeyPress event=0x{event:X} \
                 keycode={keycode} state=0x{state:X}",
            )
        }
        BackendEvent::KeyRelease {
            window,
            keycode,
            state,
        } => {
            format!(
                "window=0x{window:X}: KeyRelease keycode={keycode} state=0x{state:X}"
            )
        }
        BackendEvent::EnterNotify { window, mode } => {
            format!(
                "window=0x{:X}: EnterNotify mode={}",
                window,
                focus_mode_str(*mode),
            )
        }
        BackendEvent::LeaveNotify { window, mode } => {
            format!(
                "window=0x{:X}: LeaveNotify mode={}",
                window,
                focus_mode_str(*mode),
            )
        }
        BackendEvent::FocusIn { window } => {
            format!("window=0x{window:X}: FocusIn")
        }
        BackendEvent::FocusOut { window } => {
            format!("window=0x{window:X}: FocusOut")
        }
        BackendEvent::PropertyNotify {
            window,
            atom,
            state,
        } => {
            let state_str = match *state {
                0 => "NewValue",
                1 => "Delete",
                s => {
                    return format!(
                        "window=0x{:X}: PropertyNotify atom={} state={}?",
                        window,
                        fmt_atom(*atom),
                        s,
                    )
                }
            };
            format!(
                "window=0x{:X}: PropertyNotify atom={} state={}",
                window,
                fmt_atom(*atom),
                state_str,
            )
        }
        BackendEvent::CreateNotify {
            window,
            parent,
            rect,
            border_width,
        } => {
            format!(
                "window=0x{:X}: CreateNotify parent=0x{:X} \
                 ({},{}) {}x{} border={}",
                window,
                parent,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                border_width,
            )
        }
        BackendEvent::ReparentNotify {
            window,
            parent,
            x,
            y,
        } => {
            format!(
                "window=0x{window:X}: ReparentNotify parent=0x{parent:X} ({x},{y})"
            )
        }
        BackendEvent::MapNotify { window } => {
            format!("window=0x{window:X}: MapNotify")
        }
        BackendEvent::ConfigureNotify { window, rect } => {
            format!("window=0x{:X}: ConfigureNotify {}x{}", window, rect.w, rect.h)
        }
        BackendEvent::MappingNotify {
            request,
            first_keycode,
            count,
        } => {
            let req_str = match *request {
                0 => "Modifier",
                1 => "Keyboard",
                2 => "Pointer",
                r => {
                    return format!(
                        "MappingNotify request={r}? first={first_keycode} count={count}"
                    )
                }
            };
            format!(
                "MappingNotify request={req_str} first={first_keycode} count={count}"
            )
        }
        BackendEvent::ShapeNotify { window, shaped } => {
            format!("window=0x{window:X}: ShapeNotify shaped={shaped}")
        }
        BackendEvent::SelectionNotify {
            requestor,
            selection,
            target,
            property,
            time,
        } => {
            format!(
                "requestor=0x{requestor:X}: SelectionNotify sel=0x{selection:X} \
                 target=0x{target:X} prop=0x{property:X} time={time}",
            )
        }
        BackendEvent::SelectionRequest {
            owner,
            requestor,
            selection,
            target,
            property,
            time,
        } => {
            format!(
                "owner=0x{owner:X}: SelectionRequest requestor=0x{requestor:X} \
                 sel=0x{selection:X} target=0x{target:X} prop=0x{property:X} time={time}",
            )
        }
        BackendEvent::SelectionClear {
            owner,
            selection,
            time,
        } => {
            format!(
                "owner=0x{owner:X}: SelectionClear sel=0x{selection:X} time={time}"
            )
        }
        BackendEvent::ScreenSizeChanged { width, height } => {
            format!("ScreenSizeChanged {width}x{height}")
        }
        BackendEvent::KeyboardChanged => "KeyboardChanged".to_string(),
    }
}

const fn focus_mode_str(mode: u8) -> &'static str {
    match mode {
        0 => "Normal",
        1 => "Grab",
        2 => "Ungrab",
        3 => "WhileGrabbed",
        _ => "?",
    }
}

#[cfg(test)]
#[path = "logevent_tests.rs"]
mod tests;
