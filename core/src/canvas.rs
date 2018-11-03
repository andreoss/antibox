#[inline]
pub const fn rgb(pixel: u32) -> [u8; 3] {
    [
        ((pixel >> 16) & 0xFF) as u8,
        ((pixel >> 8) & 0xFF) as u8,
        (pixel & 0xFF) as u8,
    ]
}
