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
    pub const fn name(&self) -> &'static str {
        match self {
            Self::MapRequest { .. } => "MapRequest",
            Self::ConfigureRequest { .. } => "ConfigureRequest",
            Self::DestroyNotify { .. } => "DestroyNotify",
            Self::UnmapNotify { .. } => "UnmapNotify",
            Self::Expose { .. } => "Expose",
            Self::ClientMessage { .. } => "ClientMessage",
            Self::ButtonPress { .. } => "ButtonPress",
            Self::ButtonRelease { .. } => "ButtonRelease",
            Self::MotionNotify { .. } => "MotionNotify",
            Self::KeyPress { .. } => "KeyPress",
            Self::KeyRelease { .. } => "KeyRelease",
            Self::EnterNotify { .. } => "EnterNotify",
            Self::LeaveNotify { .. } => "LeaveNotify",
            Self::FocusIn { .. } => "FocusIn",
            Self::FocusOut { .. } => "FocusOut",
            Self::PropertyNotify { .. } => "PropertyNotify",
            Self::CreateNotify { .. } => "CreateNotify",
            Self::ReparentNotify { .. } => "ReparentNotify",
            Self::MapNotify { .. } => "MapNotify",
            Self::ConfigureNotify { .. } => "ConfigureNotify",
            Self::MappingNotify { .. } => "MappingNotify",
            Self::ShapeNotify { .. } => "ShapeNotify",
            Self::SelectionNotify { .. } => "SelectionNotify",
            Self::SelectionRequest { .. } => "SelectionRequest",
            Self::SelectionClear { .. } => "SelectionClear",
            Self::ScreenSizeChanged { .. } => "ScreenSizeChanged",
            Self::KeyboardChanged => "KeyboardChanged",
        }
    }

    pub const fn variant_index(&self) -> usize {
        match self {
            Self::MapRequest { .. } => 0,
            Self::ConfigureRequest { .. } => 1,
            Self::DestroyNotify { .. } => 2,
            Self::UnmapNotify { .. } => 3,
            Self::Expose { .. } => 4,
            Self::ClientMessage { .. } => 5,
            Self::ButtonPress { .. } => 6,
            Self::ButtonRelease { .. } => 7,
            Self::MotionNotify { .. } => 8,
            Self::KeyPress { .. } => 9,
            Self::KeyRelease { .. } => 10,
            Self::EnterNotify { .. } => 11,
            Self::LeaveNotify { .. } => 12,
            Self::FocusIn { .. } => 13,
            Self::FocusOut { .. } => 14,
            Self::PropertyNotify { .. } => 15,
            Self::CreateNotify { .. } => 16,
            Self::ReparentNotify { .. } => 17,
            Self::MapNotify { .. } => 18,
            Self::MappingNotify { .. } => 19,
            Self::ShapeNotify { .. } => 20,
            Self::SelectionNotify { .. } => 21,
            Self::SelectionRequest { .. } => 22,
            Self::SelectionClear { .. } => 23,
            Self::ConfigureNotify { .. } => 24,
            Self::ScreenSizeChanged { .. } => 25,
            Self::KeyboardChanged => 26,
        }
    }

    pub const fn window(&self) -> Option<u32> {
        match self {
            Self::MappingNotify { .. } | Self::ScreenSizeChanged { .. } | Self::KeyboardChanged => {
                None
            }
            Self::MapRequest { window }
            | Self::ConfigureRequest { window, .. }
            | Self::DestroyNotify { window }
            | Self::UnmapNotify { window }
            | Self::Expose { window, .. }
            | Self::ClientMessage { window, .. }
            | Self::ButtonPress { window, .. }
            | Self::ButtonRelease { window, .. }
            | Self::MotionNotify { window, .. }
            | Self::KeyPress { window, .. }
            | Self::KeyRelease { window, .. }
            | Self::EnterNotify { window, .. }
            | Self::LeaveNotify { window, .. }
            | Self::FocusIn { window, .. }
            | Self::FocusOut { window, .. }
            | Self::PropertyNotify { window, .. }
            | Self::CreateNotify { window, .. }
            | Self::ReparentNotify { window, .. }
            | Self::MapNotify { window }
            | Self::ConfigureNotify { window, .. }
            | Self::ShapeNotify { window, .. } => Some(*window),
            Self::SelectionNotify { requestor: w, .. }
            | Self::SelectionRequest { owner: w, .. }
            | Self::SelectionClear { owner: w, .. } => Some(*w),
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
    pub const fn new() -> Self {
        Self { events: Vec::new() }
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
