use antibox_core::backend::{BackendEvent, DisplayBackend, GraphicsContext, WindowHandle};
use std::any::Any;
use std::sync::Arc;

pub fn fit_label(g: &dyn GraphicsContext, text: &str, max_w: u16) -> String {
    let max_w = max_w as u32;
    if max_w == 0 {
        return String::new();
    }
    if g.text_width(text).unwrap_or(0) <= max_w {
        return text.to_string();
    }
    let ell_w = g.text_width("\u{2026}").unwrap_or(0);
    if ell_w >= max_w {
        return String::new();
    }
    let avail = max_w - ell_w;
    let mut out = String::new();
    for c in text.chars() {
        out.push(c);
        if g.text_width(&out).unwrap_or(0) > avail {
            out.pop();
            break;
        }
    }
    out.push('\u{2026}');
    out
}

pub trait Applet {
    fn window(&self) -> &dyn WindowHandle;
    fn paint(&self, g: &dyn GraphicsContext);
    fn preferred_width(&self) -> u32;
    fn preferred_height(&self) -> u32;
    fn handle_click(&mut self, x: i32, y: i32, button: u8) -> Option<u32>;
    fn handle_release(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
        None
    }
    fn take_relayout(&mut self) -> bool {
        false
    }
    fn set_graph_width(&mut self, _w: u16) {}
    fn set_theme_colours(&mut self, _tc: &crate::render::ThemeColors) {}
    fn wants_resize_cursor(&self) -> bool {
        false
    }
    fn handle_motion(&mut self, _x: i32, _y: i32) {}
    fn handle_enter(&mut self) {}
    fn handle_leave(&mut self) {}
    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16);

    fn owns_window(&self, id: u32) -> bool {
        self.window().id() == id
    }
    fn handle_other_event(&mut self, event: &BackendEvent, conn: &Arc<dyn DisplayBackend>) {
        let _ = (event, conn);
    }

    fn take_action(&mut self) -> Option<crate::action::Action> {
        None
    }

    fn tick_tooltip(&mut self) {}
    fn tooltip_pending(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait AppletContainer {
    fn relayout(&mut self);
    fn add_applet(&mut self, applet: Box<dyn Applet>);
    fn remove_applet(&mut self, idx: usize);
}
