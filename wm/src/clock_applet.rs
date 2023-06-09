#![allow(unsafe_code)]
 use antibox_core::error::Result;
use antibox_core::libc;
use crate::applet::Applet;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

const DEFAULT_FORMAT: &str = "%H:%M";

pub struct ClockApplet {
    conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    format: String,
    last_time: String,
    face: antibox_core::colour::Colour,
    text: u32,
    w: u16,
    h: u16,
    tooltip: Option<crate::tooltip::ToolTip>,
}

impl ClockApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        format: Option<String>,
    ) -> Result<Self> {
        let w = antibox_ui::metrics::text_w(DEFAULT_FORMAT.chars().count())
            + antibox_ui::metrics::pad() * 4;
        let h = antibox_ui::metrics::panel_height();
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w, h),
            WmWindowClass::InputOutput,
            true,
            EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::POINTER_MOTION
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE,
        )?;
        let fmt = format.unwrap_or_else(|| DEFAULT_FORMAT.to_string());
        Ok(ClockApplet {
            conn: Arc::clone(conn),
            window,
            format: fmt,
            last_time: String::new(),
            face: antibox_ui::theme::face(),
            text: antibox_ui::theme::text(),
            w: w as u16,
            h: h as u16,
            tooltip: None,
        })
    }

    pub fn set_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.face = tc.task_bar_colour;
        self.text = tc.button_fg;
    }

    pub fn set_base_format(&mut self, fmt: &str) -> bool {
        if self.format == fmt {
            return false;
        }
        self.format = fmt.to_string();
        self.last_time.clear();
        self.update();
        true
    }

    pub fn update(&mut self) -> bool {
        let time_str = current_time_str(&self.format);
        if time_str == self.last_time {
            false
        } else {
            self.last_time = time_str;
            true
        }
    }

    fn tooltip(&self) -> String {
        current_time_str("%A, %d %B %Y  %H:%M:%S")
    }
}

fn current_time_str(format: &str) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        libc::localtime_r(
            &secs as *const _,
            &mut tm as *mut _,
        );
    }
    let mut buf = vec![0u8; format.len() * 4 + 64];
    let c_format = format.to_owned() + "\0";
    let n = unsafe {
        libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            c_format.as_ptr() as *const libc::c_char,
            &tm as *const _,
        )
    };
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

impl Applet for ClockApplet {
    fn set_theme_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.set_colours(tc);
    }
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }
    fn paint(&self, g: &dyn GraphicsContext) {
        let (w, h) = (self.w, self.h);
        let hi = h as i16;
        let _ = g.set_foreground(self.face);
        let _ = g.fill_rect(0, 0, w, h);
        antibox_ui::theme::well(g, 0, 0, w, h);
        let _ = g.set_background(self.face);

        let time_str = if self.last_time.is_empty() {
            current_time_str(&self.format)
        } else {
            self.last_time.clone()
        };

        let x = antibox_ui::metrics::pad() as i16 * 2;
        let _ = g.set_font(&FontSpec::role(
            FontRole::Clock,
            antibox_ui::metrics::font_pt(),
        ));
        let _ = g.set_foreground(self.text);
        let avail = w.saturating_sub((x as u16) * 2);
        let time_str = antibox_ui::widget::fit_label(g, &time_str, avail);
        let baseline = antibox_ui::metrics::baseline(0, hi as i32) as i16;
        let _ = g.draw_text_transparent(x, baseline, &time_str);
    }
    fn preferred_width(&self) -> u32 {
        let text = if self.last_time.is_empty() {
            current_time_str(&self.format)
        } else {
            self.last_time.clone()
        };
        let spec = FontSpec::role(FontRole::Clock, antibox_ui::metrics::font_pt());
        let tw = global_text_width(&spec, &text).map_or_else(
            || antibox_ui::metrics::text_w(text.chars().count().max(1)),
            |w| w as i32,
        );
        (tw + antibox_ui::metrics::pad() * 4) as u32
    }
    fn preferred_height(&self) -> u32 {
        antibox_ui::metrics::panel_height() as u32
    }
    fn handle_click(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
        None
    }
    impl_applet_tooltip!(tooltip);
    fn owns_window(&self, id: u32) -> bool {
        self.window.id() == id
    }
    fn handle_other_event(&mut self, event: &BackendEvent, conn: &Arc<dyn DisplayBackend>) {
        if let BackendEvent::Expose { .. } = event {
            if let Ok(g) = conn.create_graphics(self.window.id()) {
                self.paint(&*g);
            }
        }
    }
    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.w = w;
        self.h = h;
        let _ = self
            .window
            .configure(Some(x as i32), Some(y as i32), Some(w), Some(h));
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "clock_applet_tests.rs"]
mod tests;
