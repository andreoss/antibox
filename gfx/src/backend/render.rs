use super::types::{
    EventMask, FontSpec, PointerGrab, PropMode, RootWindow, ShapeOp, StackMode, WmWindowClass,
};
use crate::point::{Dimension, Point};
use crate::rect::Rect;
use std::cell::RefCell;
use std::error::Error;
use std::sync::Arc;

type TextWidthFn = Arc<dyn Fn(&FontSpec, &str) -> u32 + Send + Sync>;

thread_local! {
    static GLOBAL_TEXT_WIDTH: RefCell<Option<TextWidthFn>> = RefCell::new(None);
}

pub fn set_global_font_providers(text_width: TextWidthFn) -> bool {
    GLOBAL_TEXT_WIDTH.with(|g| *g.borrow_mut() = Some(text_width));
    true
}

pub fn global_text_width(spec: &FontSpec, text: &str) -> Option<u32> {
    GLOBAL_TEXT_WIDTH.with(|g| g.borrow().as_ref().map(|f| f(spec, text)))
}

pub trait WindowHandle: std::any::Any + Send + Sync {
    fn id(&self) -> u32;

    fn map(&self) -> Result<(), Box<dyn Error>>;

    fn unmap(&self) -> Result<(), Box<dyn Error>>;

    fn destroy(&self) -> Result<(), Box<dyn Error>>;

    fn configure(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        w: Option<u16>,
        h: Option<u16>,
    ) -> Result<(), Box<dyn Error>>;

    fn raise(&self) -> Result<(), Box<dyn Error>>;

    fn lower(&self) -> Result<(), Box<dyn Error>>;

    fn reparent(&self, parent: u32, point: Point) -> Result<(), Box<dyn Error>>;

    fn set_title(&self, title: &str) -> Result<(), Box<dyn Error>>;

    fn set_class(&self, instance: &str, class: &str) -> Result<(), Box<dyn Error>>;

    fn select_input(&self, event_mask: EventMask) -> Result<(), Box<dyn Error>>;

    fn select_input_checked(&self, event_mask: EventMask) -> Result<(), Box<dyn Error>> {
        self.select_input(event_mask)
    }

    fn get_property(
        &self,
        atom: u32,
        offset: u32,
        length: u32,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error>>;

    fn get_geometry(&self) -> Result<(u16, u16), Box<dyn Error>>;

    fn get_geometry_rect(&self) -> Result<Rect, Box<dyn Error>> {
        let (w, h) = self.get_geometry()?;
        Ok(Rect::new(0, 0, w as i32, h as i32))
    }

    fn move_window(&self, point: Point) -> Result<(), Box<dyn Error>>;

    fn resize(&self, w: u16, h: u16) -> Result<(), Box<dyn Error>>;

    fn restack(&self, sibling: Option<u32>, mode: StackMode) -> Result<(), Box<dyn Error>>;

    fn translate_coords(&self, point: Point) -> Result<Point, Box<dyn Error>>;

    fn select_shape_input(&self) -> Result<(), Box<dyn Error>> {
        let _ = self;
        Ok(())
    }

    fn set_shape_rectangles(
        &self,
        _rects: &[(i16, i16, u16, u16)],
        _op: ShapeOp,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn combine_shape(&self, _src: u32, _op: ShapeOp) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn query_shaped(&self) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }
}

pub trait GraphicsContext: std::any::Any + Send + Sync {
    fn set_foreground(&self, pixel: crate::colour::Colour) -> Result<(), Box<dyn Error>>;

    fn set_background(&self, pixel: crate::colour::Colour) -> Result<(), Box<dyn Error>>;

    fn fill_rect(&self, x: i16, y: i16, w: u16, h: u16) -> Result<(), Box<dyn Error>>;

    fn draw_rect(&self, x: i16, y: i16, w: u16, h: u16) -> Result<(), Box<dyn Error>>;

    fn draw_text(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn Error>>;

    fn draw_text_transparent(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn Error>> {
        self.draw_text(x, y, text)
    }

    fn draw_text_rotated_ccw(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn Error>> {
        self.draw_text(x, y, text)
    }

    fn draw_line(&self, x1: i16, y1: i16, x2: i16, y2: i16) -> Result<(), Box<dyn Error>>;

    fn clear_rect(&self, rect: &Rect) -> Result<(), Box<dyn Error>>;

    fn set_font(&self, font: &FontSpec) -> Result<(), Box<dyn Error>>;

    fn drawable(&self) -> u32;

    fn draw_string(&self, x: i16, y: i16, text: &str) -> Result<(), Box<dyn Error>>;

    fn text_width(&self, _text: &str) -> Result<u32, Box<dyn Error>> {
        Ok(0)
    }

    fn font_metrics(&self) -> (u16, u16, u16) {
        (0, 0, 0)
    }

    fn fill_polygon(&self, _points: &[(i16, i16)]) -> Result<(), Box<dyn Error>> {
        let _ = self;
        Ok(())
    }

    fn draw_image(
        &self,
        _x: i16,
        _y: i16,
        _w: u16,
        _h: u16,
        _data: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let _ = self;
        Ok(())
    }

    fn copy_area(
        &self,
        _x: i16,
        _y: i16,
        _w: u16,
        _h: u16,
        _dx: i16,
        _dy: i16,
    ) -> Result<(), Box<dyn Error>> {
        let _ = self;
        Ok(())
    }

    fn copy_from(&self, _src: u32, _src_area: Rect, _dst: Point) -> Result<(), Box<dyn Error>> {
        let _ = self;
        Ok(())
    }

    fn draw_3d_rect(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        sunken: bool,
    ) -> Result<(), Box<dyn Error>>;

    fn draw_pixmap(&self, x: i16, y: i16, data: &PixmapData) -> Result<(), Box<dyn Error>>;

    fn draw_point(&self, x: i16, y: i16) -> Result<(), Box<dyn Error>> {
        self.fill_rect(x, y, 1, 1)
    }

    fn draw_arc(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        angle1: i16,
        angle2: i16,
    ) -> Result<(), Box<dyn Error>> {
        let _ = (x, y, w, h, angle1, angle2);
        Ok(())
    }

    fn fill_arc(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        angle1: i16,
        angle2: i16,
    ) -> Result<(), Box<dyn Error>> {
        let _ = (x, y, w, h, angle1, angle2);
        Ok(())
    }

    fn push_clip(&self, rect: &Rect) -> Result<(), Box<dyn Error>> {
        let _ = rect;
        Ok(())
    }

    fn pop_clip(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn fill_gradient_v(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        top: u32,
        bottom: u32,
    ) -> Result<(), Box<dyn Error>> {
        for (off, bh, colour) in gradient_v_bands(h, top, bottom) {
            self.set_foreground(colour)?;
            self.fill_rect(x, y + off as i16, w, bh)?;
        }
        Ok(())
    }

    fn fill_gradient_h(
        &self,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        left: u32,
        right: u32,
    ) -> Result<(), Box<dyn Error>> {
        for (off, bw, colour) in gradient_h_bands(w, left, right) {
            self.set_foreground(colour)?;
            self.fill_rect(x + off as i16, y, bw, h)?;
        }
        Ok(())
    }

    fn composite_pixmap(
        &self,
        _src_pixmap: u32,
        _src_size: Dimension,
        _dest: Point,
        _src: Point,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub fn gradient_v_bands(h: u16, top: u32, bottom: u32) -> Vec<(u16, u16, u32)> {
    let steps = h.clamp(4, 32);
    let strip_h = (h as f32 / steps as f32).ceil() as u16;
    (0..steps)
        .map(|i| {
            let t = i as f32 / (steps - 1).max(1) as f32;
            (i * strip_h, strip_h, blend_colour(top, bottom, t))
        })
        .collect()
}

pub fn gradient_h_bands(w: u16, left: u32, right: u32) -> Vec<(u16, u16, u32)> {
    let steps = w.clamp(4, 64);
    (0..steps)
        .map(|i| {
            let t = i as f32 / (steps - 1).max(1) as f32;
            let x0 = (i as u32 * w as u32 / steps as u32) as u16;
            let x1 = ((i as u32 + 1) * w as u32 / steps as u32) as u16;
            (x0, x1 - x0, blend_colour(left, right, t))
        })
        .collect()
}

pub fn blend_colour(a: u32, b: u32, t: f32) -> u32 {
    crate::colour::lerp(a, b, t)
}

#[derive(Debug, Clone)]
pub struct PixmapData {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

impl PixmapData {
    pub const fn new(width: u16, height: u16, data: Vec<u8>) -> PixmapData {
        PixmapData {
            width,
            height,
            data,
        }
    }

    pub fn subimage(&self, x: u16, y: u16, w: u16, h: u16) -> Option<Self> {
        if x.checked_add(w)? > self.width || y.checked_add(h)? > self.height || w == 0 || h == 0 {
            return None;
        }
        let mut out = Vec::with_capacity(w as usize * h as usize * 4);
        let row_bytes = self.width as usize * 4;
        let start = y as usize * row_bytes + x as usize * 4;
        for row in 0..h as usize {
            let off = start + row * row_bytes;
            out.extend_from_slice(&self.data[off..off + w as usize * 4]);
        }
        Some(Self::new(w, h, out))
    }

    pub fn vertical_offset(&self) -> u16 {
        for y in 0..self.height {
            for x in 0..self.width {
                let ai = (y as usize * self.width as usize + x as usize) * 4 + 3;
                if self.data[ai] != 0 {
                    return y;
                }
            }
        }
        self.height
    }

    #[must_use]
    pub fn scaled(&self, w: u16, h: u16) -> PixmapData {
        if w == 0 || h == 0 || self.width == 0 || self.height == 0 {
            return Self::new(0, 0, Vec::new());
        }
        if w == self.width && h == self.height {
            return self.clone();
        }
        let mut out = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h as usize {
            let sy = y * self.height as usize / h as usize;
            for x in 0..w as usize {
                let sx = x * self.width as usize / w as usize;
                let off = (sy * self.width as usize + sx) * 4;
                out.extend_from_slice(&self.data[off..off + 4]);
            }
        }
        Self::new(w, h, out)
    }

}

pub trait RenderBackend: std::any::Any + Send + Sync {
    fn root(&self) -> RootWindow;

    fn select_root_input_checked(&self, _mask: EventMask) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn screen_width(&self) -> u16;

    fn screen_height(&self) -> u16;

    fn screen_depth(&self) -> u8;

    fn create_window(
        &self,
        parent: u32,
        rect: Rect,
        class: WmWindowClass,
        override_redirect: bool,
        event_mask: EventMask,
    ) -> Result<Box<dyn WindowHandle>, Box<dyn Error>>;

    fn wrap_window(&self, xid: u32) -> Result<Box<dyn WindowHandle>, Box<dyn Error>>;

    fn create_graphics(&self, drawable: u32) -> Result<Box<dyn GraphicsContext>, Box<dyn Error>>;

    fn intern_atom(&self, name: &str) -> Result<u32, Box<dyn Error>>;

    fn change_property8(
        &self,
        mode: PropMode,
        window: u32,
        atom: u32,
        type_atom: u32,
        data: &[u8],
    ) -> Result<(), Box<dyn Error>>;

    fn change_property32(
        &self,
        mode: PropMode,
        window: u32,
        atom: u32,
        type_atom: u32,
        data: &[u32],
    ) -> Result<(), Box<dyn Error>>;

    fn get_property(
        &self,
        window: u32,
        atom: u32,
        type_atom: u32,
        offset: u32,
        length: u32,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error>>;

    fn delete_property(&self, window: u32, atom: u32) -> Result<(), Box<dyn Error>>;

    fn grab_pointer(&self, grab: PointerGrab) -> Result<(), Box<dyn Error>>;

    fn ungrab_pointer(&self, time: u32) -> Result<(), Box<dyn Error>>;

    fn send_event(
        &self,
        propagate: bool,
        destination: u32,
        event_mask: u32,
        message_type: u32,
        data: &[u32; 5],
    ) -> Result<(), Box<dyn Error>>;

    fn create_pixmap(&self, _w: u16, _h: u16, _depth: u8) -> Result<u32, Box<dyn Error>> {
        Err("create_pixmap not implemented".into())
    }

    fn free_pixmap(&self, _pixmap: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn convert_selection(
        &self,
        _requestor: u32,
        _selection: u32,
        _target: u32,
        _property: u32,
        _time: u32,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
