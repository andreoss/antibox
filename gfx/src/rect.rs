use crate::point::Point;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const ZERO: Self = Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };

    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    pub const fn px(x: i16, y: i16, w: u16, h: u16) -> Rect {
        Rect {
            x: x as i32,
            y: y as i32,
            w: w as i32,
            h: h as i32,
        }
    }

    pub fn as_px(&self) -> (i16, i16, u16, u16) {
        (
            self.x.max(i16::MIN as i32).min(i16::MAX as i32) as i16,
            self.y.max(i16::MIN as i32).min(i16::MAX as i32) as i16,
            self.w.max(0).min(u16::MAX as i32) as u16,
            self.h.max(0).min(u16::MAX as i32) as u16,
        )
    }

    pub const fn right(&self) -> i32 {
        self.x + self.w
    }

    pub const fn bottom(&self) -> i32 {
        self.y + self.h
    }

    pub fn contains(&self, p: Point) -> bool {
        self.contains_xy(p.x, p.y)
    }

    pub fn contains_xy(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    #[must_use]
    pub fn intersection(&self, other: &Self) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = self.right().min(other.right());
        let b = self.bottom().min(other.bottom());
        if r > x && b > y {
            Self::new(x, y, r - x, b - y)
        } else {
            Self::ZERO
        }
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}x{})", self.x, self.y, self.w, self.h)
    }
}

#[cfg(test)]
#[path = "rect_tests.rs"]
mod tests;
