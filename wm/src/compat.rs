pub fn u32_ne(b: [u8; 4]) -> u32 {
    u32::from(b[0]) | (u32::from(b[1]) << 8) | (u32::from(b[2]) << 16) | (u32::from(b[3]) << 24)
}

pub fn i32_ne(b: [u8; 4]) -> i32 {
    u32_ne(b) as i32
}

pub fn u32_to_ne(v: u32) -> [u8; 4] {
    [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8]
}

pub fn in_range<T: PartialOrd>(x: T, lo: T, hi: T) -> bool {
    x >= lo && x <= hi
}

pub trait ClampExt: PartialOrd + Sized {
    fn clamped(self, lo: Self, hi: Self) -> Self {
        if self < lo {
            lo
        } else if self > hi {
            hi
        } else {
            self
        }
    }
}
impl<T: PartialOrd> ClampExt for T {}
