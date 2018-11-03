use crate::inputline::{base_keysym, InputLine};
use antibox_gfx::backend::*;
use antibox_gfx::keysyms::{KEY_Escape, KEY_KP_Enter, KEY_Return};
use antibox_gfx::rect::Rect;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SearchEvent {
    None,
    Changed,
    Submitted,
    Cancelled,
}

pub struct SearchBar {
    pub conn: Arc<dyn RenderBackend>,
    pub window: Box<dyn WindowHandle>,
    pub input: InputLine,
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

impl SearchBar {
    pub fn new(
        conn: &Arc<dyn RenderBackend>,
        parent: u32,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mask = EventMask::EXPOSURE | EventMask::BUTTON_PRESS;
        let window = conn.create_window(
            parent,
            Rect::new(x as i32, y as i32, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            mask,
        )?;
        let (ix, iy, iw, ih) = Self::input_layout(w, h);
        let mut input = InputLine::new(conn, window.id(), ix, iy, iw, ih)?;
        input.frameless = true;
        input.placeholder = "Search".to_string();
        Ok(Self {
            conn: Arc::clone(conn),
            window,
            input,
            x,
            y,
            w,
            h,
        })
    }

    fn zone_w() -> i16 {
        (crate::metrics::font_px() + crate::metrics::gap() * 2) as i16
    }

    fn input_layout(w: u16, h: u16) -> (i16, i16, u16, u16) {
        let gz = Self::zone_w();
        let vpad = crate::metrics::gap() as i16;
        let iw = (w as i16 - gz * 2).max(1) as u16;
        let ih = (h as i16 - vpad * 2).max(1) as u16;
        (gz, vpad, iw, ih)
    }

    pub fn set_rect(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.x = x;
        self.y = y;
        self.w = w;
        self.h = h;
        let _ = self
            .window
            .configure(Some(x as i32), Some(y as i32), Some(w), Some(h));
        let (ix, iy, iw, ih) = Self::input_layout(w, h);
        self.input.x = ix;
        self.input.y = iy;
        self.input.w = iw;
        self.input.h = ih;
        let _ = self
            .input
            .window
            .configure(Some(ix as i32), Some(iy as i32), Some(iw), Some(ih));
    }

    pub fn set_placeholder(&mut self, text: &str) {
        self.input.placeholder = text.to_string();
        self.input.repaint();
    }

    pub fn text(&self) -> &str {
        self.input.get_text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.input.set_text(text);
        self.repaint();
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.input.set_focus(focused);
    }

    pub fn show(&self) {
        let _ = self.window.map();
        self.input.show();
        self.repaint();
        self.input.repaint();
    }

    pub fn hide(&self) {
        self.input.hide();
        let _ = self.window.unmap();
    }

    pub fn owns_window(&self, id: u32) -> bool {
        self.window.id() == id || self.input.window_id() == id
    }

    fn clear_zone(&self) -> Rect {
        let zw = Self::zone_w() as i32;
        Rect::new(self.w as i32 - zw, 0, zw, self.h as i32)
    }

    #[allow(non_upper_case_globals)]
    pub fn handle_key(
        &mut self,
        keycode: u32,
        state: u16,
        mapping: &KeyboardMapping,
    ) -> SearchEvent {
        let before = self.input.text().to_string();
        if self.input.handle_key(keycode, state, mapping) {
            return if self.input.text() == before {
                SearchEvent::None
            } else {
                SearchEvent::Changed
            };
        }
        match base_keysym(keycode as u8, mapping) {
            KEY_Return | KEY_KP_Enter => SearchEvent::Submitted,
            KEY_Escape => SearchEvent::Cancelled,
            _ => SearchEvent::None,
        }
    }

    pub fn handle_button(&mut self, window: u32, x: i32, y: i32, button: u8) -> SearchEvent {
        if window == self.input.window_id() {
            self.input.handle_button(x, y, button);
            return SearchEvent::None;
        }
        if window != self.window.id() {
            return SearchEvent::None;
        }
        if !self.input.text().is_empty() && self.clear_zone().contains_xy(x, y) {
            self.input.set_text("");
            self.repaint();
            return SearchEvent::Changed;
        }
        SearchEvent::None
    }

    pub fn handle_motion(&mut self, window: u32, x: i32) {
        if window == self.input.window_id() {
            self.input.handle_motion(x);
        }
    }

    pub fn handle_release(&mut self, window: u32, button: u8) {
        if window == self.input.window_id() {
            self.input.handle_release(button);
        }
    }

    pub fn repaint(&self) {
        let g = match self.conn.create_graphics(self.window.id()) {
            Ok(g) => g,
            Err(_) => return,
        };
        use crate::theme;
        theme::sunken_field(&*g, 0, 0, self.w, self.h);
        let zw = Self::zone_w();
        let cy = self.h as i16 / 2;
        let r = (self.h as i16 / 4).max(3);
        let cx = zw / 2 - r / 4;
        let gcx = cx - r / 2;
        let gcy = cy - r / 2 - r / 4;
        let _ = g.set_foreground(theme::disabled());
        let d = r as u16;
        let _ = g.draw_arc(gcx, gcy, d, d, 0, 360 * 64);
        let _ = g.draw_line(
            gcx + r - r / 4,
            gcy + r - r / 4,
            gcx + r + r / 2,
            gcy + r + r / 2,
        );
        if !self.input.text().is_empty() {
            let cz = self.clear_zone();
            let m = (self.h as i16 / 3).max(3);
            let l = cz.x as i16 + m;
            let rr = (cz.x + cz.w) as i16 - m;
            let t = cy - (rr - l) / 2;
            let b = cy + (rr - l) / 2;
            let _ = g.set_foreground(theme::text());
            crate::widget::draw_cross(&*g, l, t, rr, b, 1);
        }
        self.input.repaint();
    }
}

#[cfg(test)]
#[path = "searchbar_tests.rs"]
mod tests;
