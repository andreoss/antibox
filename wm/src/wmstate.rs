#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum WinLayer {
    Desktop = 0,
    Below = 2,
    Normal = 4,
    OnTop = 6,
    Dock = 8,
    AboveDock = 10,
    Menu = 12,
    Fullscreen = 14,
    AboveAll = 15,
}

impl Default for WinLayer {
    fn default() -> WinLayer {
        WinLayer::Normal
    }
}

impl WinLayer {
    pub fn from_i32(n: i32) -> Option<Self> {
        match n {
            0 => Some(WinLayer::Desktop),
            2 => Some(WinLayer::Below),
            4 => Some(WinLayer::Normal),
            6 => Some(WinLayer::OnTop),
            8 => Some(WinLayer::Dock),
            10 => Some(WinLayer::AboveDock),
            12 => Some(WinLayer::Menu),
            14 => Some(WinLayer::Fullscreen),
            15 => Some(WinLayer::AboveAll),
            _ => None,
        }
    }

    pub const fn to_i32(self) -> i32 {
        self as i32
    }

    pub fn default_for_window_type(window_type: crate::client::WindowType) -> WinLayer {
        match window_type {
            crate::client::WindowType::Desktop => WinLayer::Desktop,
            crate::client::WindowType::Dock => WinLayer::Dock,
            crate::client::WindowType::Menu => WinLayer::Menu,
            _ => WinLayer::Normal,
        }
    }

    pub fn from_ewmh_state(above: bool, below: bool, base: Self) -> WinLayer {
        if above {
            WinLayer::OnTop
        } else if below {
            WinLayer::Below
        } else {
            base
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WindowState {
    pub maximized: bool,
    pub max_vert: bool,
    pub max_horz: bool,
    pub minimized: bool,
    pub shaded: bool,
    pub fullscreen: bool,
    pub urgent: bool,
    pub above: bool,
    pub below: bool,
    pub sticky: bool,
    pub skip_taskbar: bool,
    pub skip_pager: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    None,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[cfg(test)]
#[path = "wmstate_tests.rs"]
mod tests;
