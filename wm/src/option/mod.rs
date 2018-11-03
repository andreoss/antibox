pub mod parsing;

use crate::wmstate::WinLayer;

macro_rules! flag_set {
    ($name:ident($ty:ty) { $($flag:ident = $val:expr),+ $(,)* }) => {
        #[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
        pub struct $name(pub $ty);
        impl $name {
            $(pub const $flag: Self = $name($val);)+
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }
        }
        impl std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                $name(self.0 | rhs.0)
            }
        }
        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
        impl std::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                $name(self.0 & rhs.0)
            }
        }
        impl std::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                $name(!self.0)
            }
        }
    };
}

flag_set!(WindowFlags(u32) {
    ALL_WORKSPACES = 1 << 0,
    MAXIMIZED_VERT = 1 << 2,
    MAXIMIZED_HORZ = 1 << 3,
    MAXIMIZED_BOTH = 3 << 2,
    MINIMIZED = 1 << 4,
    FULLSCREEN = 1 << 5,
    DO_NOT_MANAGE = 1 << 10,
    DO_NOT_FOCUS = 1 << 11,
    IGNORE_TASKBAR = 1 << 12,
    IGNORE_OVERRIDE_REDIRECT = 1 << 16,
    NO_FOCUS_ON_MAP = 1 << 23,
});

flag_set!(GeoFlags(u8) {
    X = 1 << 0,
    Y = 1 << 1,
    W = 1 << 2,
    H = 1 << 3,
});

#[derive(Clone, Default)]
pub struct WinOptionPlacement {
    pub workspace: Option<i32>,
    pub layer: Option<WinLayer>,
}

#[derive(Clone, Default)]
pub struct WinOptionGeometry {
    pub gflags: GeoFlags,
    pub gx: i32,
    pub gy: i32,
    pub gw: u32,
    pub gh: u32,
}

#[derive(Clone, Default)]
pub struct WindowOption {
    pub class_instance: String,
    pub options: WindowFlags,
    pub option_mask: WindowFlags,
    pub placement: WinOptionPlacement,
    pub opacity: i32,
    pub geom: WinOptionGeometry,
}

impl WindowOption {
    pub fn new(class_instance: &str) -> WindowOption {
        WindowOption {
            class_instance: class_instance.to_string(),
            ..Default::default()
        }
    }
    pub fn has_option(&self, flag: WindowFlags) -> bool {
        (self.options & self.option_mask).intersects(flag)
    }
}

include!("options.rs");

#[cfg(test)]
mod tests;
