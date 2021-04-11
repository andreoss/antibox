use crate::editcore::{EditCore, EditOutcome};
use antibox_gfx::backend::*;
use antibox_gfx::rect::Rect;
use std::sync::Arc;

pub struct InputLine {
    pub conn: Arc<dyn RenderBackend>,
    pub window: Box<dyn WindowHandle>,
    pub edit: EditCore,
    pub focused: bool,
    pub frameless: bool,
    pub dragging: bool,
    pub placeholder: String,
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

impl InputLine {
    pub fn new(
        conn: &Arc<dyn RenderBackend>,
        parent: u32,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mask = EventMask::EXPOSURE
            | EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE
            | EventMask::BUTTON_PRESS
            | EventMask::FOCUS_CHANGE
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW;
        let window = conn.create_window(
            parent,
            Rect::new(x as i32, y as i32, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            mask,
        )?;
        Ok(Self {
            conn: Arc::clone(conn),
            window,
            edit: EditCore::new(),
            focused: false,
            frameless: false,
            dragging: false,
            placeholder: String::new(),
            x,
            y,
            w,
            h,
        })
    }

    pub fn set_text(&mut self, text: &str) {
        self.edit.set_text(text);
        self.repaint();
    }

    pub fn text(&self) -> &str {
        self.edit.text()
    }

    pub fn get_text(&self) -> &str {
        self.edit.text()
    }

    pub const fn cursor_pos(&self) -> usize {
        self.edit.cursor()
    }

    pub const fn sel_start(&self) -> Option<usize> {
        self.edit.sel_start()
    }

    fn text_inset() -> i16 {
        crate::theme::field_inset() as i16
    }

    fn caret_margin() -> i16 {
        (crate::metrics::font_px() / 2).max(2) as i16
    }

    pub fn repaint(&self) {
        let g = match self.conn.create_graphics(self.window.id()) {
            Ok(g) => g,
            Err(_) => return,
        };
        use crate::theme;
        let field_bg = theme::field();
        let _ = g.set_font(&FontSpec::role(FontRole::Input, crate::metrics::font_pt()));
        if !self.frameless {
            theme::sunken_field(&*g, 0, 0, self.w, self.h);
        } else {
            let _ = g.set_foreground(field_bg);
            let _ = g.fill_rect(0, 0, self.w, self.h);
        }
        let text = self.edit.display_text();
        let text_x = Self::text_inset() - self.scroll_offset(&*g);
        let baseline = crate::metrics::baseline(0, self.h as i32) as i16;
        let width_of = |s: &str| g.text_width(s).unwrap_or(0) as i16;

        let cursor = self.edit.cursor();
        let (lo, hi) = match self.edit.sel_start() {
            Some(s) if s != cursor => (
                self.edit.display_index(s.min(cursor)),
                self.edit.display_index(s.max(cursor)),
            ),
            _ => (0, 0),
        };

        if hi > lo {
            let pre = width_of(&text[..lo]);
            let selw = width_of(&text[lo..hi]);
            let sel_x = text_x + pre;
            let _ = g.set_foreground(theme::sel_bg());
            let _ = g.fill_rect(sel_x, 2, selw.max(1) as u16, self.h - 4);
            let _ = g.set_foreground(theme::text());
            let _ = g.set_background(field_bg);
            let _ = g.draw_text(text_x, baseline, &text[..lo]);
            let _ = g.set_foreground(theme::sel_fg());
            let _ = g.set_background(theme::sel_bg());
            let _ = g.draw_text(sel_x, baseline, &text[lo..hi]);
            let _ = g.set_foreground(theme::text());
            let _ = g.set_background(field_bg);
            let _ = g.draw_text(sel_x + selw, baseline, &text[hi..]);
        } else if text.is_empty() && !self.placeholder.is_empty() {
            let _ = g.set_foreground(theme::disabled());
            let _ = g.set_background(field_bg);
            let _ = g.draw_text(text_x, baseline, &self.placeholder);
        } else {
            let _ = g.set_foreground(theme::text());
            let _ = g.set_background(field_bg);
            let _ = g.draw_text(text_x, baseline, &text);
        }

        if self.focused {
            let cx = text_x + width_of(&text[..self.edit.display_index(cursor)]);
            let _ = g.set_foreground(theme::text());
            let _ = g.fill_rect(cx, 2, 1, self.h - 4);
        }
    }

    pub fn handle_key(&mut self, keycode: u32, state: u16, mapping: &KeyboardMapping) -> bool {
        let key = keys::normalize(keycode, state, mapping);
        match self.edit.handle_key(&key) {
            EditOutcome::Changed | EditOutcome::Consumed => {
                self.repaint();
                true
            }
            EditOutcome::Ignored | EditOutcome::Submit | EditOutcome::Cancel | EditOutcome::Tab => {
                false
            }
        }
    }

    fn scroll_offset(&self, g: &dyn GraphicsContext) -> i16 {
        let text = self.edit.display_text();
        let cx = g
            .text_width(&text[..self.edit.display_index(self.edit.cursor())])
            .unwrap_or(0) as i16;
        let avail = (self.w as i16 - Self::text_inset() - Self::caret_margin()).max(1);
        (cx - avail).max(0)
    }

    fn index_at(&self, x: i32) -> usize {
        let g = match self.conn.create_graphics(self.window.id()) {
            Ok(g) => g,
            Err(_) => return self.edit.text().len(),
        };
        let _ = g.set_font(&FontSpec::role(FontRole::Input, crate::metrics::font_pt()));
        let text = self.edit.display_text();
        let target = x - Self::text_inset() as i32 + self.scroll_offset(&*g) as i32;
        let mut best = 0usize;
        let mut best_d = i32::MAX;
        for i in 0..=text.len() {
            if !text.is_char_boundary(i) {
                continue;
            }
            let w = g.text_width(&text[..i]).unwrap_or(0) as i32;
            let d = (w - target).abs();
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        self.edit.logical_index(best)
    }

    pub fn handle_button(&mut self, x: i32, _y: i32, button: u8) {
        self.handle_button_state(x, button, 0);
    }

    pub fn handle_button_state(&mut self, x: i32, button: u8, state: u16) {
        if button != 1 {
            return;
        }
        let idx = self.index_at(x);
        if state & 0x01 != 0 {
            if self.edit.sel_start().is_none() {
                self.edit.set_sel_start(Some(self.edit.cursor()));
            }
        } else {
            self.edit.set_cursor(idx);
            self.edit.set_sel_start(Some(self.edit.cursor()));
        }
        self.dragging = true;
        self.edit.set_cursor(idx);
        self.repaint();
    }

    pub fn handle_motion(&mut self, x: i32) {
        if !self.dragging {
            return;
        }
        let idx = self.index_at(x);
        if idx != self.edit.cursor() {
            self.edit.set_cursor(idx);
            self.repaint();
        }
    }

    pub fn handle_release(&mut self, button: u8) {
        if button == 1 {
            self.dragging = false;
            if self.edit.sel_start() == Some(self.edit.cursor()) {
                self.edit.set_sel_start(None);
                self.repaint();
            }
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
        self.repaint();
    }

    pub fn show(&self) {
        let _ = self.window.map();
    }
    pub fn hide(&self) {
        let _ = self.window.unmap();
    }
    pub fn window_id(&self) -> u32 {
        self.window.id()
    }
}

pub(crate) use antibox_gfx::backend::keys::base_keysym;

#[cfg(test)]
#[path = "inputline_tests.rs"]
mod tests;
