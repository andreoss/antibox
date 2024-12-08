use super::*;
use crate::glyphs::{self, GlyphSource};
use std::sync::Arc;

#[test]
fn paints_into_the_bound_buffer() {
    let store = BufferStore::new();
    let id = store.create(8, 8);
    let g = WaylandGraphics::new(Arc::clone(&store), id);
    g.set_foreground(0x00FF_0000).unwrap();
    g.fill_rect(0, 0, 8, 8).unwrap();
    g.set_foreground(0x0000_00FF).unwrap();
    g.draw_line(0, 0, 7, 0).unwrap();
    let pm = store.snapshot(id).unwrap();
    assert_eq!(&pm.data[0..4], &[0x00, 0x00, 0xFF, 0xFF]);
    let mid = (4 * 8 + 4) * 4;
    assert_eq!(&pm.data[mid..mid + 4], &[0xFF, 0x00, 0x00, 0xFF]);
}

#[test]
fn drawable_id_is_reported() {
    let store = BufferStore::new();
    let id = store.create(2, 2);
    let g = WaylandGraphics::new(store, id);
    assert_eq!(g.drawable(), id);
}

struct StripeGlyphs;

impl GlyphSource for StripeGlyphs {
    fn text_width(&self, _font: &FontSpec, text: &str) -> u32 {
        text.chars().count() as u32 * 4
    }

    fn metrics(&self, _font: &FontSpec) -> (u16, u16, u16) {
        (8, 2, 10)
    }

    fn draw(
        &self,
        canvas: &mut SoftCanvas,
        _font: &FontSpec,
        x: i16,
        y: i16,
        text: &str,
        fg: u32,
        _bg: Option<u32>,
    ) {
        for (i, _) in text.chars().enumerate() {
            canvas.fill_rect(x + i as i16 * 4, y, 2, 2, fg);
        }
    }
}

#[test]
fn text_goes_through_the_installed_glyph_source() {
    let store = BufferStore::new();
    let id = store.create(32, 16);
    let g = WaylandGraphics::new(store.clone(), id);
    let _ = g.set_foreground(0x00FF_0000);
    let _ = g.draw_text_transparent(2, 4, "ab");
    let before = store.snapshot(id).unwrap();
    assert!(before.data.iter().all(|b| *b == 0));

    glyphs::install(Arc::new(StripeGlyphs));
    let _ = g.draw_text_transparent(2, 4, "ab");
    let after = store.snapshot(id).unwrap();
    let at = |x: usize, y: usize| {
        let o = (y * 32 + x) * 4;
        [after.data[o], after.data[o + 1], after.data[o + 2]]
    };
    assert_eq!(at(2, 4), [0xFF, 0, 0]);
    assert_eq!(at(6, 4), [0xFF, 0, 0]);
    assert_eq!(at(10, 4), [0, 0, 0]);
}

#[test]
fn metrics_fall_back_without_a_source() {
    let font = FontSpec::ui(20);
    let (a, d, h) = glyphs::fallback_metrics(&font);
    assert_eq!(h, 20);
    assert_eq!(a + d, h);
    assert!(glyphs::fallback_width(&font, "abcd") > 0);
}
