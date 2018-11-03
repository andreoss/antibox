use crate::point::Point;
use crate::rect::Rect;

#[derive(Debug, Clone, PartialEq)]
pub enum BackendEvent {
    MapRequest {
        window: u32,
    },
    ConfigureRequest {
        window: u32,
        parent: u32,
        rect: Rect,
        border_width: u32,
        value_mask: u16,
    },
    DestroyNotify {
        window: u32,
    },
    UnmapNotify {
        window: u32,
    },
    Expose {
        window: u32,
        rect: Rect,
    },
    ClientMessage {
        window: u32,
        message_type: u32,
        format: u8,
        data: [u32; 5],
    },
    ButtonPress {
        window: u32,
        event: u32,
        point: Point,
        root: Point,
        button: u8,
        state: u16,
    },
    ButtonRelease {
        window: u32,
        point: Point,
        button: u8,
    },
    MotionNotify {
        window: u32,
        point: Point,
        root: Point,
        state: u16,
    },
    KeyPress {
        window: u32,
        event: u32,
        keycode: u32,
        state: u16,
    },
    KeyRelease {
        window: u32,
        keycode: u32,
        state: u16,
    },
    EnterNotify {
        window: u32,
        mode: u8,
    },
    LeaveNotify {
        window: u32,
        mode: u8,
    },
    FocusIn {
        window: u32,
    },
    FocusOut {
        window: u32,
    },
    PropertyNotify {
        window: u32,
        atom: u32,
        state: u8,
    },
    CreateNotify {
        window: u32,
        parent: u32,
        rect: Rect,
        border_width: u32,
    },
    ReparentNotify {
        window: u32,
        parent: u32,
        x: i32,
        y: i32,
    },
    MapNotify {
        window: u32,
    },
    ConfigureNotify {
        window: u32,
        rect: Rect,
    },
    MappingNotify {
        request: u8,
        first_keycode: u8,
        count: u8,
    },
    ShapeNotify {
        window: u32,
        shaped: bool,
    },
    SelectionNotify {
        requestor: u32,
        selection: u32,
        target: u32,
        property: u32,
        time: u32,
    },
    SelectionRequest {
        owner: u32,
        requestor: u32,
        selection: u32,
        target: u32,
        property: u32,
        time: u32,
    },
    SelectionClear {
        owner: u32,
        selection: u32,
        time: u32,
    },
    ScreenSizeChanged {
        width: u16,
        height: u16,
    },
    KeyboardChanged,
}

impl BackendEvent {
    pub fn name(&self) -> &'static str {
        match self {
            BackendEvent::MapRequest { .. } => "MapRequest",
            BackendEvent::ConfigureRequest { .. } => "ConfigureRequest",
            BackendEvent::DestroyNotify { .. } => "DestroyNotify",
            BackendEvent::UnmapNotify { .. } => "UnmapNotify",
            BackendEvent::Expose { .. } => "Expose",
            BackendEvent::ClientMessage { .. } => "ClientMessage",
            BackendEvent::ButtonPress { .. } => "ButtonPress",
            BackendEvent::ButtonRelease { .. } => "ButtonRelease",
            BackendEvent::MotionNotify { .. } => "MotionNotify",
            BackendEvent::KeyPress { .. } => "KeyPress",
            BackendEvent::KeyRelease { .. } => "KeyRelease",
            BackendEvent::EnterNotify { .. } => "EnterNotify",
            BackendEvent::LeaveNotify { .. } => "LeaveNotify",
            BackendEvent::FocusIn { .. } => "FocusIn",
            BackendEvent::FocusOut { .. } => "FocusOut",
            BackendEvent::PropertyNotify { .. } => "PropertyNotify",
            BackendEvent::CreateNotify { .. } => "CreateNotify",
            BackendEvent::ReparentNotify { .. } => "ReparentNotify",
            BackendEvent::MapNotify { .. } => "MapNotify",
            BackendEvent::ConfigureNotify { .. } => "ConfigureNotify",
            BackendEvent::MappingNotify { .. } => "MappingNotify",
            BackendEvent::ShapeNotify { .. } => "ShapeNotify",
            BackendEvent::SelectionNotify { .. } => "SelectionNotify",
            BackendEvent::SelectionRequest { .. } => "SelectionRequest",
            BackendEvent::SelectionClear { .. } => "SelectionClear",
            BackendEvent::ScreenSizeChanged { .. } => "ScreenSizeChanged",
            BackendEvent::KeyboardChanged => "KeyboardChanged",
        }
    }

    pub fn variant_index(&self) -> usize {
        match self {
            BackendEvent::MapRequest { .. } => 0,
            BackendEvent::ConfigureRequest { .. } => 1,
            BackendEvent::DestroyNotify { .. } => 2,
            BackendEvent::UnmapNotify { .. } => 3,
            BackendEvent::Expose { .. } => 4,
            BackendEvent::ClientMessage { .. } => 5,
            BackendEvent::ButtonPress { .. } => 6,
            BackendEvent::ButtonRelease { .. } => 7,
            BackendEvent::MotionNotify { .. } => 8,
            BackendEvent::KeyPress { .. } => 9,
            BackendEvent::KeyRelease { .. } => 10,
            BackendEvent::EnterNotify { .. } => 11,
            BackendEvent::LeaveNotify { .. } => 12,
            BackendEvent::FocusIn { .. } => 13,
            BackendEvent::FocusOut { .. } => 14,
            BackendEvent::PropertyNotify { .. } => 15,
            BackendEvent::CreateNotify { .. } => 16,
            BackendEvent::ReparentNotify { .. } => 17,
            BackendEvent::MapNotify { .. } => 18,
            BackendEvent::MappingNotify { .. } => 19,
            BackendEvent::ShapeNotify { .. } => 20,
            BackendEvent::SelectionNotify { .. } => 21,
            BackendEvent::SelectionRequest { .. } => 22,
            BackendEvent::SelectionClear { .. } => 23,
            BackendEvent::ConfigureNotify { .. } => 24,
            BackendEvent::ScreenSizeChanged { .. } => 25,
            BackendEvent::KeyboardChanged => 26,
        }
    }

    pub fn window(&self) -> Option<u32> {
        match self {
            BackendEvent::MappingNotify { .. } | BackendEvent::ScreenSizeChanged { .. } | BackendEvent::KeyboardChanged => {
                None
            }
            BackendEvent::MapRequest { window }
            | BackendEvent::ConfigureRequest { window, .. }
            | BackendEvent::DestroyNotify { window }
            | BackendEvent::UnmapNotify { window }
            | BackendEvent::Expose { window, .. }
            | BackendEvent::ClientMessage { window, .. }
            | BackendEvent::ButtonPress { window, .. }
            | BackendEvent::ButtonRelease { window, .. }
            | BackendEvent::MotionNotify { window, .. }
            | BackendEvent::KeyPress { window, .. }
            | BackendEvent::KeyRelease { window, .. }
            | BackendEvent::EnterNotify { window, .. }
            | BackendEvent::LeaveNotify { window, .. }
            | BackendEvent::FocusIn { window, .. }
            | BackendEvent::FocusOut { window, .. }
            | BackendEvent::PropertyNotify { window, .. }
            | BackendEvent::CreateNotify { window, .. }
            | BackendEvent::ReparentNotify { window, .. }
            | BackendEvent::MapNotify { window }
            | BackendEvent::ConfigureNotify { window, .. }
            | BackendEvent::ShapeNotify { window, .. } => Some(*window),
            BackendEvent::SelectionNotify { requestor: w, .. }
            | BackendEvent::SelectionRequest { owner: w, .. }
            | BackendEvent::SelectionClear { owner: w, .. } => Some(*w),
        }
    }
}

#[derive(Debug)]
pub struct QueuedEvent {
    pub event: BackendEvent,
    pub serial: u64,
}

pub trait EventHandler {
    fn handle_event(&mut self, event: &BackendEvent);
}

#[derive(Debug, Default)]
pub struct EventQueue {
    events: Vec<QueuedEvent>,
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue { events: Vec::new() }
    }

    pub fn push(&mut self, event: BackendEvent) {
        self.events.push(QueuedEvent { event, serial: 0 });
    }

    pub fn pop(&mut self) -> Option<BackendEvent> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0).event)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

}
