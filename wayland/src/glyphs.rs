use antibox_core::backend::FontSpec;
use antibox_core::canvas::SoftCanvas;
use std::sync::{Arc, RwLock};

pub trait GlyphSource: Send + Sync {
    fn text_width(&self, font: &FontSpec, text: &str) -> u32;

    fn metrics(&self, font: &FontSpec) -> (u16, u16, u16);

    fn draw(
        &self,
        canvas: &mut SoftCanvas,
        font: &FontSpec,
        x: i16,
        y: i16,
        text: &str,
        fg: u32,
        bg: Option<u32>,
    );
}

static SOURCE: RwLock<Option<Arc<dyn GlyphSource>>> = RwLock::new(None);

pub fn install(source: Arc<dyn GlyphSource>) {
    if let Ok(mut g) = SOURCE.write() {
        *g = Some(source);
    }
}

pub fn source() -> Option<Arc<dyn GlyphSource>> {
    SOURCE.read().ok().and_then(|g| g.clone())
}

pub fn fallback_metrics(font: &FontSpec) -> (u16, u16, u16) {
    let h = font.size.max(1);
    let ascent = h.saturating_mul(4) / 5;
    (ascent, h - ascent, h)
}

pub fn fallback_width(font: &FontSpec, text: &str) -> u32 {
    let advance = u32::from(font.size.max(1)) * 3 / 5;
    advance * text.chars().count() as u32
}
