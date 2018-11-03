pub mod wm_hints_flags {
    pub const INPUT_HINT: u32 = 1 << 0;
    pub const STATE_HINT: u32 = 1 << 1;
    pub const ICON_PIXMAP_HINT: u32 = 1 << 2;
    pub const ICON_WINDOW_HINT: u32 = 1 << 3;
    pub const ICON_POSITION_HINT: u32 = 1 << 4;
    pub const ICON_MASK_HINT: u32 = 1 << 5;
    pub const WINDOW_GROUP_HINT: u32 = 1 << 6;
}

pub mod wm_state {
    pub const NORMAL: u32 = 1;
    pub const ICONIC: u32 = 3;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WmHints {
    pub flags: u32,
    pub input: bool,
    pub initial_state: u32,
    pub icon_pixmap: u32,
    pub icon_window: u32,
    pub icon_x: i32,
    pub icon_y: i32,
    pub icon_mask: u32,
    pub window_group: u32,
    pub urgency: bool,
}

impl WmHints {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 32 {
            return None;
        }
        let words: Vec<u32> = data
            .chunks_exact(4)
            .map(|c| (c[0] as u32) | ((c[1] as u32) << 8) | ((c[2] as u32) << 16) | ((c[3] as u32) << 24))
            .collect();
        if words.len() < 8 {
            return None;
        }
        Some(Self {
            flags: words[0],
            input: words[1] != 0,
            initial_state: words[2],
            icon_pixmap: words[3],
            icon_window: words[4],
            icon_x: words[5] as i32,
            icon_y: words[6] as i32,
            icon_mask: words[7],
            window_group: if words.len() > 8 { words[8] } else { 0 },
            urgency: words[0] & (1 << 8) != 0,
        })
    }
}

pub mod size_hints_flags {
    pub const US_POSITION: u32 = 1 << 0;
    pub const US_SIZE: u32 = 1 << 1;
    pub const P_POSITION: u32 = 1 << 2;
    pub const P_SIZE: u32 = 1 << 3;
    pub const P_MIN_SIZE: u32 = 1 << 4;
    pub const P_MAX_SIZE: u32 = 1 << 5;
    pub const P_RESIZE_INC: u32 = 1 << 6;
    pub const P_ASPECT: u32 = 1 << 7;
    pub const P_BASE_SIZE: u32 = 1 << 8;
    pub const P_WIN_GRAVITY: u32 = 1 << 9;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SizeHints {
    pub flags: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub width_inc: u32,
    pub height_inc: u32,
    pub min_aspect_num: u32,
    pub min_aspect_den: u32,
    pub max_aspect_num: u32,
    pub max_aspect_den: u32,
    pub base_width: u32,
    pub base_height: u32,
    pub win_gravity: u32,
}

impl SizeHints {
    pub fn constrain(&self, mut w: i32, mut h: i32) -> (i32, i32) {
        use self::size_hints_flags::{P_ASPECT, P_BASE_SIZE, P_MAX_SIZE, P_MIN_SIZE, P_RESIZE_INC};
        if self.flags & P_MIN_SIZE != 0 {
            w = w.max(self.min_width as i32);
            h = h.max(self.min_height as i32);
        }
        if self.flags & P_MAX_SIZE != 0 {
            if self.max_width > 0 {
                w = w.min(self.max_width as i32);
            }
            if self.max_height > 0 {
                h = h.min(self.max_height as i32);
            }
        }
        let has_base = self.flags & P_BASE_SIZE != 0;
        let base = |b: u32, min: u32| -> i32 {
            if has_base {
                b as i32
            } else {
                min as i32
            }
        };
        let bw = base(self.base_width, self.min_width);
        let bh = base(self.base_height, self.min_height);
        if self.flags & P_ASPECT != 0 {
            let aw = (w - bw).max(0) as i64;
            let ah = (h - bh).max(1) as i64;
            let (mnn, mnd) = (self.min_aspect_num as i64, self.min_aspect_den as i64);
            let (mxn, mxd) = (self.max_aspect_num as i64, self.max_aspect_den as i64);
            if mnn > 0 && mnd > 0 && aw * mnd < ah * mnn {
                w = bw + (ah * mnn / mnd) as i32;
            } else if mxn > 0 && mxd > 0 && aw * mxd > ah * mxn {
                w = bw + (ah * mxn / mxd) as i32;
            }
        }
        if self.flags & P_RESIZE_INC != 0 {
            if self.width_inc > 0 {
                let inc = self.width_inc as i32;
                w = bw + ((w - bw).max(0) / inc) * inc;
            }
            if self.height_inc > 0 {
                let inc = self.height_inc as i32;
                h = bh + ((h - bh).max(0) / inc) * inc;
            }
        }
        (w.max(1), h.max(1))
    }

    pub fn is_fixed(&self) -> bool {
        use self::size_hints_flags::{P_MAX_SIZE, P_MIN_SIZE};
        self.flags & P_MIN_SIZE != 0
            && self.flags & P_MAX_SIZE != 0
            && self.max_width > 0
            && self.max_height > 0
            && self.min_width == self.max_width
            && self.min_height == self.max_height
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 72 {
            return None;
        }
        let words: Vec<u32> = data
            .chunks_exact(4)
            .map(|c| (c[0] as u32) | ((c[1] as u32) << 8) | ((c[2] as u32) << 16) | ((c[3] as u32) << 24))
            .collect();
        if words.len() < 18 {
            return None;
        }
        Some(Self {
            flags: words[0],
            x: words[1] as i32,
            y: words[2] as i32,
            width: words[3],
            height: words[4],
            min_width: words[5],
            min_height: words[6],
            max_width: words[7],
            max_height: words[8],
            width_inc: words[9],
            height_inc: words[10],
            min_aspect_num: words[11],
            min_aspect_den: words[12],
            max_aspect_num: words[13],
            max_aspect_den: words[14],
            base_width: words[15],
            base_height: words[16],
            win_gravity: words[17],
        })
    }
}

pub mod mwm_hints_flags {
    pub const FUNCTIONS: u32 = 1 << 0;
    pub const DECORATIONS: u32 = 1 << 1;
}

pub mod mwm_func {
    pub const ALL: u32 = 1 << 0;
    pub const RESIZE: u32 = 1 << 1;
    pub const MOVE: u32 = 1 << 2;
    pub const MINIMIZE: u32 = 1 << 3;
    pub const MAXIMIZE: u32 = 1 << 4;
    pub const CLOSE: u32 = 1 << 5;
}

pub mod mwm_decor {
    pub const ALL: u32 = 1 << 0;
    pub const BORDER: u32 = 1 << 1;
    pub const TITLE: u32 = 1 << 3;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MwmHints {
    pub flags: u32,
    pub functions: u32,
    pub decorations: u32,
    pub input_mode: i32,
}

impl MwmHints {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }
        let words: Vec<u32> = data
            .chunks_exact(4)
            .map(|c| (c[0] as u32) | ((c[1] as u32) << 8) | ((c[2] as u32) << 16) | ((c[3] as u32) << 24))
            .collect();
        if words.len() < 4 {
            return None;
        }
        Some(Self {
            flags: words[0],
            functions: words[1],
            decorations: words[2],
            input_mode: words[3] as i32,
        })
    }

    pub fn undecorated(&self) -> bool {
        use self::mwm_decor::{ALL, BORDER, TITLE};
        use self::mwm_hints_flags::DECORATIONS;
        self.flags & DECORATIONS != 0 && self.decorations & (ALL | BORDER | TITLE) == 0
    }

    pub fn allows(&self, func_bit: u32) -> bool {
        use self::mwm_func::ALL;
        use self::mwm_hints_flags::FUNCTIONS;
        if self.flags & FUNCTIONS == 0 {
            return true;
        }
        let has_all = self.functions & ALL != 0;
        has_all ^ (self.functions & func_bit != 0)
    }
}

#[cfg(test)]
#[path = "hints_size_hints_constrain_tests.rs"]
mod size_hints_constrain_tests;

#[cfg(test)]
#[path = "hints_mwm_hints_tests.rs"]
mod mwm_hints_tests;
