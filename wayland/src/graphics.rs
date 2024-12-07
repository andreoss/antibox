use crate::buffers::BufferStore;
use antibox_core::backend::{FontSpec, GraphicsContext, PixmapData};
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use crate::glyphs;
use antibox_core::canvas::SoftCanvas;
use std::sync::{Arc, Mutex};

struct GcState {
    fg: u32,
    bg: u32,
    font: FontSpec,
}

pub struct WaylandGraphics {
    store: Arc<BufferStore>,
    drawable: u32,
    state: Mutex<GcState>,
}

impl WaylandGraphics {
    pub fn new(store: Arc<BufferStore>, drawable: u32) -> Self {
        Self {
            store,
            drawable,
            state: Mutex::new(GcState {
                fg: 0x0000_0000,
                bg: 0x00FF_FFFF,
                font: FontSpec::ui(12),
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GcState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn font(&self) -> FontSpec {
        self.lock().font.clone()
    }
}

use antibox_core::error::Result;

type R = Result<()>;

impl GraphicsContext for WaylandGraphics {
    fn set_foreground(&self, pixel: u32) -> R {
        self.lock().fg = pixel;
        Ok(())
    }

    fn set_background(&self, pixel: u32) -> R {
        self.lock().bg = pixel;
        Ok(())
    }

    fn fill_rect(&self, x: i16, y: i16, w: u16, h: u16) -> R {
        let fg = self.lock().fg;
        self.store
            .with(self.drawable, |c| c.fill_rect(x, y, w, h, fg));
        Ok(())
    }

    fn draw_rect(&self, x: i16, y: i16, w: u16, h: u16) -> R {
        let fg = self.lock().fg;
        self.store
            .with(self.drawable, |c| c.draw_rect(x, y, w, h, fg));
        Ok(())
    }

    fn draw_text(&self, x: i16, y: i16, text: &str) -> R {
        let (fg, bg) = {
            let s = self.lock();
            (s.fg, s.bg)
        };
        if let Some(src) = glyphs::source() {
            let font = self.font();
            self.store.with(self.drawable, |c| {
                src.draw(c, &font, x, y, text, fg, Some(bg));
            });
        }
        Ok(())
    }

    fn draw_text_transparent(&self, x: i16, y: i16, text: &str) -> R {
        let fg = self.lock().fg;
        if let Some(src) = glyphs::source() {
            let font = self.font();
            self.store.with(self.drawable, |c| {
                src.draw(c, &font, x, y, text, fg, None);
            });
        }
        Ok(())
    }

    fn draw_line(&self, x1: i16, y1: i16, x2: i16, y2: i16) -> R {
        let fg = self.lock().fg;
        self.store
            .with(self.drawable, |c| c.draw_line(x1, y1, x2, y2, fg));
        Ok(())
    }

    fn clear_rect(&self, rect: &Rect) -> R {
        let fg = self.lock().fg;
        self.store.with(self.drawable, |c| {
            c.fill_rect(
                rect.x as i16,
                rect.y as i16,
                rect.w as u16,
                rect.h as u16,
                fg,
            );
        });
        Ok(())
    }

    fn set_font(&self, font: &FontSpec) -> R {
        self.lock().font = font.clone();
        Ok(())
    }

    fn drawable(&self) -> u32 {
        self.drawable
    }

    fn draw_string(&self, x: i16, y: i16, text: &str) -> R {
        self.draw_text(x, y, text)
    }

    fn text_width(&self, text: &str) -> Result<u32> {
        let font = self.font();
        Ok(glyphs::source().map_or_else(
            || glyphs::fallback_width(&font, text),
            |src| src.text_width(&font, text),
        ))
    }

    fn font_metrics(&self) -> (u16, u16, u16) {
        let font = self.font();
        glyphs::source().map_or_else(
            || glyphs::fallback_metrics(&font),
            |src| src.metrics(&font),
        )
    }

    fn copy_from(&self, src: u32, src_area: Rect, dst: Point) -> R {
        self.store.copy_region(src, src_area, self.drawable, dst);
        Ok(())
    }

    fn push_clip(&self, rect: &Rect) -> R {
        let r = *rect;
        self.store.with(self.drawable, |c| c.push_clip(r));
        Ok(())
    }

    fn pop_clip(&self) -> R {
        self.store
            .with(self.drawable, SoftCanvas::pop_clip);
        Ok(())
    }

    fn copy_area(&self, x: i16, y: i16, w: u16, h: u16, dx: i16, dy: i16) -> R {
        self.store.copy_region(
            self.drawable,
            Rect::px(x, y, w, h),
            self.drawable,
            Point::new(dx as i32, dy as i32),
        );
        Ok(())
    }

    fn draw_point(&self, x: i16, y: i16) -> R {
        let fg = self.lock().fg;
        self.store.with(self.drawable, |c| c.draw_point(x, y, fg));
        Ok(())
    }

    fn fill_polygon(&self, points: &[(i16, i16)]) -> R {
        let fg = self.lock().fg;
        self.store
            .with(self.drawable, |c| c.fill_polygon(points, fg));
        Ok(())
    }

    fn draw_arc(&self, x: i16, y: i16, w: u16, h: u16, angle1: i16, angle2: i16) -> R {
        let fg = self.lock().fg;
        self.store.with(self.drawable, |c| {
            c.draw_arc(Rect::px(x, y, w, h), angle1 as i32, angle2 as i32, fg);
        });
        Ok(())
    }

    fn fill_arc(&self, x: i16, y: i16, w: u16, h: u16, angle1: i16, angle2: i16) -> R {
        let fg = self.lock().fg;
        self.store.with(self.drawable, |c| {
            c.fill_arc(Rect::px(x, y, w, h), angle1 as i32, angle2 as i32, fg);
        });
        Ok(())
    }

    fn draw_text_rotated_ccw(&self, x: i16, y: i16, text: &str) -> R {
        let fg = self.lock().fg;
        let Some(src) = glyphs::source() else {
            return Ok(());
        };
        let font = self.font();
        let w = src.text_width(&font, text).max(1) as u16;
        let (ascent, _, h) = src.metrics(&font);
        let h = h.max(1);
        let mut scratch = SoftCanvas::new(w, h);
        src.draw(&mut scratch, &font, 0, ascent as i16, text, fg, None);
        let (sw, sh) = (w as usize, h as usize);
        let rgba = scratch.as_rgba().to_vec();
        let dh = sw as i32;
        self.store.with(self.drawable, |c| {
            for sy in 0..sh {
                for sx in 0..sw {
                    let o = (sy * sw + sx) * 4;
                    let a = u32::from(rgba[o + 3]);
                    if a > 0 {
                        c.blend(
                            x as i32 + sy as i32,
                            (y as i32 - dh) + (sw - 1 - sx) as i32,
                            [rgba[o], rgba[o + 1], rgba[o + 2]],
                            a,
                        );
                    }
                }
            }
        });
        Ok(())
    }
    fn draw_3d_rect(&self, x: i16, y: i16, w: u16, h: u16, sunken: bool) -> R {
        let (fg, bg) = {
            let s = self.lock();
            (s.fg, s.bg)
        };
        self.store.with(self.drawable, |c| {
            c.draw_3d_rect(Rect::px(x, y, w, h), sunken, fg, bg);
        });
        Ok(())
    }

    fn draw_pixmap(&self, x: i16, y: i16, data: &PixmapData) -> R {
        self.store
            .with(self.drawable, |c| c.draw_pixmap(x, y, data));
        Ok(())
    }

    fn fill_gradient_v(&self, x: i16, y: i16, w: u16, h: u16, top: u32, bottom: u32) -> R {
        self.store.with(self.drawable, |c| {
            c.fill_gradient_v(x, y, w, h, top, bottom);
        });
        Ok(())
    }

    fn fill_gradient_h(&self, x: i16, y: i16, w: u16, h: u16, left: u32, right: u32) -> R {
        self.store.with(self.drawable, |c| {
            c.fill_gradient_h(x, y, w, h, left, right);
        });
        Ok(())
    }
}

#[cfg(test)]
#[path = "graphics_tests.rs"]
mod tests;
