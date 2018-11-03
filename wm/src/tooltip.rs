use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_core::scale::scaled;

fn margin_x() -> i32 {
    antibox_ui::metrics::pad() + antibox_ui::metrics::gap()
}

fn margin_y() -> i32 {
    antibox_ui::metrics::gap() + scaled(1)
}

fn tooltip_font() -> FontSpec {
    FontSpec::role(FontRole::ToolTip, antibox_ui::metrics::font_pt())
}

fn wrap_width(conn: &dyn DisplayBackend) -> u16 {
    (conn.screen_width() as i32 - margin_x() * 2 - scaled(2) * 2).max(1) as u16
}

static TT_SHOW_DELAY_MS: antibox_core::sync::atomic::AtomicU64 = antibox_core::sync::atomic::AtomicU64::new(500);

pub fn set_show_delay_ms(ms: u64) {
    TT_SHOW_DELAY_MS.store(ms, std::sync::atomic::Ordering::Relaxed);
}

fn show_delay() -> std::time::Duration {
    std::time::Duration::from_millis(TT_SHOW_DELAY_MS.load(std::sync::atomic::Ordering::Relaxed))
}

static TT_LIFETIME_MS: antibox_core::sync::atomic::AtomicU64 = antibox_core::sync::atomic::AtomicU64::new(5000);

pub fn set_lifetime_ms(ms: u64) {
    TT_LIFETIME_MS.store(ms, std::sync::atomic::Ordering::Relaxed);
}

fn tt_lifetime() -> std::time::Duration {
    std::time::Duration::from_millis(TT_LIFETIME_MS.load(std::sync::atomic::Ordering::Relaxed))
}

pub struct ToolTip {
    pub window: Option<Box<dyn WindowHandle>>,
    pub text: String,
    pub visible: bool,
    pending: Option<(String, Rect, std::time::Instant)>,
    shown_at: Option<std::time::Instant>,
}

impl ToolTip {
    pub fn new() -> ToolTip {
        ToolTip {
            window: None,
            text: String::new(),
            visible: false,
            pending: None,
            shown_at: None,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn request(&mut self, conn: &dyn DisplayBackend, text: &str, near: Rect) {
        if text.is_empty() {
            self.hide(conn);
            return;
        }
        if self.visible {
            if self.text != text {
                self.text = text.to_string();
                self.show(conn, near);
            }
            return;
        }
        match &self.pending {
            Some((t, _, _)) if t == text => {}
            _ => self.pending = Some((text.to_string(), near, std::time::Instant::now())),
        }
    }

    pub fn pump(&mut self, conn: &dyn DisplayBackend) {
        if let Some((_, _, start)) = &self.pending {
            if start.elapsed() >= show_delay() {
                let (text, near, _) = self.pending.take().expect("pending checked above");
                self.text = text;
                self.show(conn, near);
                return;
            }
        }
        if self.visible {
            if self.shown_at.map_or(false, |t| t.elapsed() >= tt_lifetime()) {
                self.hide(conn);
            } else {
                self.paint(conn);
            }
        }
    }

    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn show(&mut self, conn: &dyn DisplayBackend, near: Rect) {
        if self.text.is_empty() {
            return;
        }
        self.hide(conn);
        let (text_w, text_h) = match antibox_ui::textmeasure::with_measure_context(conn, |g| {
            let _ = g.set_font(&tooltip_font());
            let (tw, th, _) =
                antibox_ui::textmeasure::measure(g, &self.text, Some(wrap_width(conn)));
            (tw as i32, th as i32)
        }) {
            Some(v) => v,
            None => return,
        };
        let w = (text_w + margin_x() * 2).max(antibox_ui::metrics::text_w(6));
        let h = text_h + margin_y() * 2;

        let mut x = near.x + near.w / 2 - w / 2;
        let mut y = near.y + near.h + 2;
        let sw = conn.screen_width() as i32;
        if x + w > sw {
            x = sw - w - 2;
        }
        if x < 2 {
            x = 2;
        }
        if y + h > conn.screen_height() as i32 {
            y = near.y - h - 2;
        }

        let mask = EventMask::EXPOSURE;
        if let Ok(win) = conn.create_window(
            conn.root().as_parent(),
            Rect::new(x, y, w, h),
            WmWindowClass::InputOutput,
            true,
            mask,
        ) {
            let _ = win.map();
            self.window = Some(win);
            self.visible = true;
            self.shown_at = Some(std::time::Instant::now());
            self.paint(conn);
        }
    }

    pub fn hide(&mut self, _conn: &dyn DisplayBackend) {
        if let Some(ref win) = self.window {
            let _ = win.unmap();
            let _ = win.destroy();
        }
        self.visible = false;
        self.window = None;
        self.pending = None;
        self.shown_at = None;
    }

    pub fn paint(&self, conn: &dyn DisplayBackend) {
        let win = match &self.window { Some(v) => v, None => return };
        let g = match conn.create_graphics(win.id()) {
            Ok(g) => g,
            Err(_) => return,
        };
        let _ = g.set_font(&tooltip_font());
        let (gw, gh) = win.get_geometry().unwrap_or((100, 24));
        let bg = antibox_ui::theme::tooltip_bg();
        let fg = antibox_ui::theme::tooltip_fg();
        let _ = g.set_foreground(bg);
        let _ = g.fill_rect(0, 0, gw, gh);
        let _ = g.set_foreground(fg);
        let _ = g.draw_rect(0, 0, gw.saturating_sub(1), gh.saturating_sub(1));
        let _ = g.set_foreground(fg);
        let _ = g.set_background(bg);
        antibox_ui::textmeasure::draw(
            &*g,
            margin_x() as i16,
            margin_y() as i16,
            &self.text,
            Some(wrap_width(conn)),
        );
    }
}

impl Default for ToolTip {
    fn default() -> ToolTip {
        Self::new()
    }
}

pub fn show_window_tip(
    slot: &mut Option<ToolTip>,
    conn: &dyn DisplayBackend,
    window: &dyn WindowHandle,
    text: &str,
) {
    if text.is_empty() {
        hide_tip(slot, conn);
        return;
    }
    let (rx, ry) = window
        .translate_coords(antibox_core::point::Point::new(0, 0))
        .map_or((0, 0), |p| (p.x, p.y));
    let (w, h) = window.get_geometry().unwrap_or((1, 1));
    slot.get_or_insert_with(ToolTip::new).request(
        conn,
        text,
        Rect::new(rx, ry, w as i32, h as i32),
    );
}

pub fn show_rect_tip(
    slot: &mut Option<ToolTip>,
    conn: &dyn DisplayBackend,
    window: &dyn WindowHandle,
    local: (i16, i16, u16, u16),
    text: &str,
) {
    if text.is_empty() {
        hide_tip(slot, conn);
        return;
    }
    let (lx, ly, w, h) = local;
    let (rx, ry) = window
        .translate_coords(antibox_core::point::Point::new(lx as i32, ly as i32))
        .map_or((lx as i32, ly as i32), |p| (p.x, p.y));
    slot.get_or_insert_with(ToolTip::new).request(
        conn,
        text,
        Rect::new(rx, ry, w as i32, h as i32),
    );
}

pub fn hide_tip(slot: &mut Option<ToolTip>, conn: &dyn DisplayBackend) {
    if let Some(tt) = slot.as_mut() {
        tt.hide(conn);
    }
}

pub fn pump_tip(slot: &mut Option<ToolTip>, conn: &dyn DisplayBackend) {
    if let Some(tt) = slot.as_mut() {
        tt.pump(conn);
    }
}

pub fn tip_pending(slot: &Option<ToolTip>) -> bool {
    slot.as_ref().map_or(false, ToolTip::is_pending)
}

macro_rules! impl_applet_tooltip {
    (@tail) => {
        fn handle_leave(&mut self) {
            crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
        }
        fn tick_tooltip(&mut self) {
            crate::tooltip::pump_tip(&mut self.tooltip, self.conn.as_ref());
        }
        fn tooltip_pending(&self) -> bool {
            crate::tooltip::tip_pending(&self.tooltip)
        }
    };
    (@motion) => {
        fn handle_motion(&mut self, _x: i32, _y: i32) {
            self.handle_enter();
        }
    };
    (hover $($m:ident).+) => {
        fn handle_enter(&mut self) {
            let text = self.$($m).+();
            crate::tooltip::show_window_tip(
                &mut self.tooltip,
                self.conn.as_ref(),
                &*self.window,
                &text,
            );
        }
        impl_applet_tooltip!(@tail);
    };
    ($($m:ident).+) => {
        impl_applet_tooltip!(@motion);
        impl_applet_tooltip!(hover $($m).+);
    };
}


#[cfg(test)]
#[path = "tooltip_tests.rs"]
mod tests;
