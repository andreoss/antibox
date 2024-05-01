use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Dimension {
    pub w: i32,
    pub h: i32,
}

impl Dimension {
    pub const ZERO: Self = Self { w: 0, h: 0 };

    pub const fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }

    pub const fn px(w: u16, h: u16) -> Self {
        Self {
            w: w as i32,
            h: h as i32,
        }
    }

    pub fn as_px(&self) -> (u16, u16) {
        (
            self.w.max(0).min(u16::MAX as i32) as u16,
            self.h.max(0).min(u16::MAX as i32) as u16,
        )
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
#[path = "point_tests.rs"]
mod tests;
