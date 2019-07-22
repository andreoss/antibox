
use super::bindings::*;
use super::connection::XcbConnection;
use super::font::XcbFont;
use antibox_core::backend::{DisplayBackend, FontSpec, GraphicsContext, PixmapData};
use antibox_core::colour::Colour;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::sync::Arc;

pub struct XcbGraphics {
    conn: Arc<XcbConnection>,
    drawable: u32,
    gc: u32,
    depth: u8,
    fg: antibox_core::sync::atomic::AtomicU32,
    bg: antibox_core::sync::atomic::AtomicU32,
    font: std::sync::Mutex<Option<XcbFont>>,
}

impl XcbGraphics {
    pub fn new(conn: Arc<XcbConnection>, drawable: u32, gc: u32, depth: u8) -> Self {
        Self {
            conn,
            drawable,
            gc,
            depth,
            fg: antibox_core::sync::atomic::AtomicU32::new(0),
            bg: antibox_core::sync::atomic::AtomicU32::new(0xFFFFFF),
            font: std::sync::Mutex::new(None),
        }
    }

    fn change_gc(&self, mask: u32, vals: &[u32]) {
        unsafe { xcb_change_gc(self.conn.raw(), self.gc, mask, vals.as_ptr()) };
    }
}

impl Drop for XcbGraphics {
    fn drop(&mut self) {
        unsafe { xcb_free_gc(self.conn.raw(), self.gc) };
    }
}

impl GraphicsContext for XcbGraphics {
    fn set_foreground(&self, pixel: Colour) -> Result<(), Box<dyn std::error::Error>> {
        self.fg.store(pixel, std::sync::atomic::Ordering::Relaxed);
        let v = [pixel];
        self.change_gc(XCB_GC_FOREGROUND, &v);
        Ok(())
    }

    fn set_background(&self, pixel: Colour) -> Result<(), Box<dyn std::error::Error>> {
        self.bg.store(pixel, std::sync::atomic::Ordering::Relaxed);
        let v = [pixel];
        self.change_gc(XCB_GC_BACKGROUND, &v);
        Ok(())
    }

    fn fill_rect(&self, x: i16, y: i16, w: u16, h: u16) -> Result<(), Box<dyn std::error::Error>> {
        let r = xcb_rectangle_t { x, y, width: w, height: h };
        unsafe { xcb_poly_fill_rectangle(self.conn.raw(), self.drawable, self.gc, 1, &r) };
        self.conn.flush()
    }

    fn draw_rect(&self, x: i16, y: i16, w: u16, h: u16) -> Result<(), Box<dyn std::error::Error>> {
        let r = xcb_rectangle_t { x, y, width: w, height: h };
        unsafe { xcb_poly_rectangle(self.conn.raw(), self.drawable, self.gc, 1, &r) };
        self.conn.flush()
    }

    fn draw_text(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let chars: Vec<xcb_char2b_t> = text
            .chars()
            .take(255)
            .map(|c| {
                let code = if c as u32 <= 0xFFFF { c as u32 as u16 } else { b'?' as u16 };
                xcb_char2b_t { byte1: (code >> 8) as u8, byte2: (code & 0xFF) as u8 }
            })
            .collect();
        unsafe {
            xcb_image_text_16(
                self.conn.raw(),
                chars.len() as u8,
                self.drawable,
                self.gc,
                x,
                y,
                chars.as_ptr(),
            )
        };
        self.conn.flush()
    }

    fn draw_text_transparent(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_text(x, y, text)
    }

    fn draw_text_rotated_ccw(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_text(x, y, text)
    }

    fn draw_line(&self, x1: i16, y1: i16, x2: i16, y2: i16) -> Result<(), Box<dyn std::error::Error>> {
        let pts = [xcb_point_t { x: x1, y: y1 }, xcb_point_t { x: x2, y: y2 }];
        unsafe { xcb_poly_line(self.conn.raw(), 0, self.drawable, self.gc, 2, pts.as_ptr()) };
        self.conn.flush()
    }

    fn clear_rect(&self, rect: &Rect) -> Result<(), Box<dyn std::error::Error>> {
        let old = self.fg.load(std::sync::atomic::Ordering::Relaxed);
        let _ = self.set_foreground(self.bg.load(std::sync::atomic::Ordering::Relaxed));
        self.fill_rect(rect.x as i16, rect.y as i16, rect.w as u16, rect.h as u16)?;
        let _ = self.set_foreground(old);
        Ok(())
    }

    fn set_font(&self, font: &FontSpec) -> Result<(), Box<dyn std::error::Error>> {
        let px = (font.size as f32 * 96.0 / 72.0).round() as u16;
        let cf = super::font::resolve_font(&self.conn, &font.family, px);
        if let Some(f) = cf {
            let id = f.id;
            *self.font.lock().unwrap() = Some(f);
            let v = [id];
            self.change_gc(XCB_GC_FONT, &v);
        }
        Ok(())
    }

    fn text_width(&self, text: &str) -> Result<u32, Box<dyn std::error::Error>> {
        Ok(self.font.lock().unwrap().as_ref().map_or(0, |f| f.text_width(text)))
    }

    fn font_metrics(&self) -> (u16, u16, u16) {
        self.font.lock().unwrap().as_ref().map_or((0, 0, 0), |f| f.metrics())
    }

    fn drawable(&self) -> u32 {
        self.drawable
    }

    fn draw_string(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_text(x, y, text)
    }

    fn draw_3d_rect(&self, x: i16, y: i16, w: u16, h: u16, sunken: bool) -> Result<(), Box<dyn std::error::Error>> {
        let _ = (x, y, w, h, sunken);
        Ok(())
    }

    fn draw_pixmap(&self, x: i16, y: i16, data: &PixmapData) -> Result<(), Box<dyn std::error::Error>> {
        let w = data.width;
        let h = data.height;
        let depth = if data.data.len() >= w as usize * h as usize * 4 { 32 } else { self.depth };
        unsafe {
            xcb_put_image(
                self.conn.raw(),
                XCB_IMAGE_FORMAT_Z_PIXMAP,
                self.drawable,
                self.gc,
                w,
                h,
                x,
                y,
                0,
                depth,
                data.data.len() as u32,
                data.data.as_ptr(),
            )
        };
        self.conn.flush()
    }

    fn draw_point(&self, x: i16, y: i16) -> Result<(), Box<dyn std::error::Error>> {
        let p = xcb_point_t { x, y };
        unsafe { xcb_poly_point(self.conn.raw(), 0, self.drawable, self.gc, 1, &p) };
        self.conn.flush()
    }

    fn fill_polygon(&self, points: &[(i16, i16)]) -> Result<(), Box<dyn std::error::Error>> {
        let pts: Vec<xcb_point_t> = points.iter().map(|&(x, y)| xcb_point_t { x, y }).collect();
        unsafe { xcb_fill_poly(self.conn.raw(), self.drawable, self.gc, 0, 0, pts.len() as u32, pts.as_ptr()) };
        self.conn.flush()
    }

    fn draw_image(&self, x: i16, y: i16, w: u16, h: u16, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            xcb_put_image(
                self.conn.raw(), XCB_IMAGE_FORMAT_Z_PIXMAP, self.drawable, self.gc,
                w, h, x, y, 0, self.depth, data.len() as u32, data.as_ptr(),
            )
        };
        self.conn.flush()
    }

    fn copy_area(&self, x: i16, y: i16, w: u16, h: u16, dx: i16, dy: i16) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_copy_area(self.conn.raw(), self.drawable, self.drawable, self.gc, x, y, dx, dy, w, h) };
        self.conn.flush()
    }

    fn copy_from(&self, src: u32, src_area: Rect, dst: Point) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            xcb_copy_area(
                self.conn.raw(), src, self.drawable, self.gc,
                src_area.x as i16, src_area.y as i16,
                dst.x as i16, dst.y as i16,
                src_area.w as u16, src_area.h as u16,
            )
        };
        self.conn.flush()
    }

    fn composite_pixmap(&self, _src_pixmap: u32, _src_size: antibox_core::point::Dimension, _dest: antibox_core::point::Point, _src: antibox_core::point::Point) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn draw_arc(&self, x: i16, y: i16, w: u16, h: u16, angle1: i16, angle2: i16) -> Result<(), Box<dyn std::error::Error>> {
        let _ = (x, y, w, h, angle1, angle2);
        Ok(())
    }
    fn fill_arc(&self, x: i16, y: i16, w: u16, h: u16, angle1: i16, angle2: i16) -> Result<(), Box<dyn std::error::Error>> {
        let _ = (x, y, w, h, angle1, angle2);
        Ok(())
    }
    fn push_clip(&self, _rect: &Rect) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn pop_clip(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
