#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(i32)]
pub enum WinLayer {
    Desktop = 0,
    Below = 2,
    #[default]
    Normal = 4,
    OnTop = 6,
    Dock = 8,
    AboveDock = 10,
    Menu = 12,
    Fullscreen = 14,
    AboveAll = 15,
}

impl WinLayer {
    pub const fn from_i32(n: i32) -> Option<Self> {
        match n {
            0 => Some(Self::Desktop),
            2 => Some(Self::Below),
            4 => Some(Self::Normal),
            6 => Some(Self::OnTop),
            8 => Some(Self::Dock),
            10 => Some(Self::AboveDock),
            12 => Some(Self::Menu),
            14 => Some(Self::Fullscreen),
            15 => Some(Self::AboveAll),
            _ => None,
        }
    }

    pub const fn to_i32(self) -> i32 {
        self as i32
    }

    pub const fn default_for_window_type(window_type: crate::client::WindowType) -> Self {
        match window_type {
            crate::client::WindowType::Desktop => Self::Desktop,
            crate::client::WindowType::Dock => Self::Dock,
            crate::client::WindowType::Menu => Self::Menu,
            _ => Self::Normal,
        }
    }

    pub const fn from_ewmh_state(above: bool, below: bool, base: Self) -> Self {
        if above {
            Self::OnTop
        } else if below {
            Self::Below
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
