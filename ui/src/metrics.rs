use antibox_gfx::scale::scaled;
use antibox_gfx::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;

static FONT_PT: AtomicU16 = AtomicU16::new(9);

pub fn set_font_pt(pt: u16) {
    FONT_PT.store(pt.clamp(6, 72), Ordering::Relaxed);
}

pub fn font_px() -> i32 {
    scaled(font_pt() as i32 * 4 / 3)
}

pub fn panel_height() -> i32 {
    font_px() + pad() * 2 + gap()
}

pub fn pad() -> i32 {
    scaled(crate::theme::pad_base() as i32)
}

pub fn gap() -> i32 {
    scaled(crate::theme::gap_base() as i32)
}

pub fn icon() -> i32 {
    scaled(16)
}

pub fn item_gap() -> i32 {
    scaled(crate::theme::item_gap_base() as i32)
}

pub fn button_inset() -> i32 {
    gap()
}

pub fn button_height() -> i32 {
    (panel_height() - button_inset() * 2).max(1)
}

pub fn menu_item_height() -> i32 {
    (font_px() + gap() * 2).max(scaled(16))
}

pub fn field_height() -> i32 {
    font_px() + pad() * 2
}

pub fn font_pt() -> u16 {
    FONT_PT.load(Ordering::Relaxed)
}

pub fn baseline(y: i32, h: i32) -> i32 {
    y + h / 2 + font_px() * 3 / 8
}

pub fn text_w(chars: usize) -> i32 {
    chars as i32 * (font_px() * 3 / 5)
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
