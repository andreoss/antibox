use antibox_core::backend::DisplayBackend;
use antibox_core::cursor::Cursor;

pub mod idx {
    pub const LEFT: usize = 0;
    pub const MOVE: usize = 1;
    pub const RIGHT: usize = 2;
    pub const SIZE_BOTTOM: usize = 3;
    pub const SIZE_BOTTOM_LEFT: usize = 4;
    pub const SIZE_BOTTOM_RIGHT: usize = 5;
    pub const SIZE_LEFT: usize = 6;
    pub const SIZE_RIGHT: usize = 7;
    pub const SIZE_TOP: usize = 8;
    pub const SIZE_TOP_LEFT: usize = 9;
    pub const SIZE_TOP_RIGHT: usize = 10;
    pub const SIZE_H: usize = 11;
    pub const COUNT: usize = 12;
}

pub const CURSOR_SPECS: &[(u32, &str); idx::COUNT] = &[
    (68, "left_ptr"),
    (52, "fleur"),
    (94, "right_ptr"),
    (16, "bottom_side"),
    (12, "bottom_left_corner"),
    (14, "bottom_right_corner"),
    (70, "left_side"),
    (96, "right_side"),
    (138, "top_side"),
    (134, "top_left_corner"),
    (136, "top_right_corner"),
    (108, "sb_h_double_arrow"),
];

pub const fn resize_edge_cursor(edge: &crate::wmstate::ResizeEdge) -> usize {
    use crate::wmstate::ResizeEdge;
    match edge {
        ResizeEdge::Left => idx::SIZE_LEFT,
        ResizeEdge::Right => idx::SIZE_RIGHT,
        ResizeEdge::Top => idx::SIZE_TOP,
        ResizeEdge::Bottom => idx::SIZE_BOTTOM,
        ResizeEdge::TopLeft => idx::SIZE_TOP_LEFT,
        ResizeEdge::TopRight => idx::SIZE_TOP_RIGHT,
        ResizeEdge::BottomLeft => idx::SIZE_BOTTOM_LEFT,
        ResizeEdge::BottomRight => idx::SIZE_BOTTOM_RIGHT,
        ResizeEdge::None => idx::MOVE,
    }
}

pub fn init_cursors(backend: &dyn DisplayBackend) -> Vec<u32> {
    let mut cursors = Vec::with_capacity(idx::COUNT);
    for &(glyph, xname) in CURSOR_SPECS {
        let spec = Cursor::new(None, Some(glyph), Some(xname));
        let xid = spec.load(backend).unwrap_or(0);
        cursors.push(xid);
    }
    cursors
}

pub fn free_cursors(backend: &dyn DisplayBackend, cursors: &[u32]) {
    for &xid in cursors {
        if xid != 0 {
            let _ = backend.free_cursor(xid);
        }
    }
}

#[cfg(test)]
#[path = "cursors_tests.rs"]
mod tests;
