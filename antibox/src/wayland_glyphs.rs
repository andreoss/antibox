use antibox_core::backend::FontSpec;
use antibox_core::canvas::SoftCanvas;
use antibox_wayland::glyphs::GlyphSource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct FreeTypeGlyphs {
    faces: Mutex<HashMap<String, Option<Arc<antibox_x11::xcb::ft::FtFont>>>>,
}

impl FreeTypeGlyphs {
    fn face(&self, font: &FontSpec) -> Option<Arc<antibox_x11::xcb::ft::FtFont>> {
        let pattern = format!("{}:size={}", font.family, font.size.max(1));
        let mut cache = self
            .faces
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache
            .entry(pattern.clone())
            .or_insert_with(|| antibox_x11::xcb::ft::open(&pattern))
            .clone()
    }
}

impl GlyphSource for FreeTypeGlyphs {
    fn text_width(&self, font: &FontSpec, text: &str) -> u32 {
        self.face(font).map_or_else(
            || antibox_wayland::glyphs::fallback_width(font, text),
            |f| f.text_width(text),
        )
    }

    fn metrics(&self, font: &FontSpec) -> (u16, u16, u16) {
        self.face(font).map_or_else(
            || antibox_wayland::glyphs::fallback_metrics(font),
            |f| f.metrics(),
        )
    }

    fn draw(
        &self,
        canvas: &mut SoftCanvas,
        font: &FontSpec,
        x: i16,
        y: i16,
        text: &str,
        fg: u32,
        bg: Option<u32>,
    ) {
        let Some(face) = self.face(font) else {
            return;
        };
        if let Some(bg) = bg {
            let (_, _, h) = face.metrics();
            let w = face.text_width(text) as u16;
            let (_, descent, _) = face.metrics();
            canvas.fill_rect(x, y - (h - descent) as i16, w, h, bg);
        }
        let colour = antibox_core::canvas::rgb(fg);
        let mut pen = i32::from(x);
        for ch in text.chars() {
            let Some(g) = face.rasterize(ch as u32) else {
                continue;
            };
            let gx = pen + i32::from(g.left);
            let gy = i32::from(y) - i32::from(g.top);
            for row in 0..g.height as usize {
                for col in 0..g.width as usize {
                    let a = u32::from(g.coverage[row * g.width as usize + col]);
                    if a > 0 {
                        canvas.blend(gx + col as i32, gy + row as i32, colour, a);
                    }
                }
            }
            pen += i32::from(g.advance);
        }
    }
}

pub fn install() {
    antibox_wayland::glyphs::install(Arc::new(FreeTypeGlyphs::default()));
}
