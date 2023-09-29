use crate::applet::Applet;
use crate::render::ThemeColors;
use antibox_core::backend::*;
use antibox_core::error::Result;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::any::Any;
use std::sync::Arc;

pub const LABEL: &str = "Start";

pub struct MenuApplet {
    window: Box<dyn WindowHandle>,
    w: u16,
    h: u16,
    pressed: bool,
    pending: bool,
    colours: ThemeColors,
}

impl MenuApplet {
    pub fn new(conn: &Arc<dyn DisplayBackend>, parent: u32) -> Result<MenuApplet> {
        let w = antibox_ui::metrics::text_w(LABEL.chars().count()) + antibox_ui::metrics::pad() * 4;
        let h = antibox_ui::metrics::panel_height();
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w, h),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE | EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
        )?;
        Ok(MenuApplet {
            window,
            w: w as u16,
            h: h as u16,
            pressed: false,
            pending: false,
            colours: ThemeColors::default(),
        })
    }

    pub fn set_colours(&mut self, tc: &ThemeColors) {
        self.colours = *tc;
    }

    pub fn set_pressed(&mut self, v: bool) {
        self.pressed = v;
    }
}

impl Applet for MenuApplet {
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }

    fn paint(&self, g: &dyn GraphicsContext) {
        antibox_ui::theme::panel_surface(
            g,
            self.w,
            self.h,
            self.colours.task_bar_colour,
        );
        let font = FontSpec::role_styled(
            FontRole::NormalTaskBar,
            antibox_ui::metrics::font_pt(),
            true,
            false,
        );
        let _ = g.set_font(&font);
        let vin = antibox_ui::metrics::button_inset();
        let bh = (i32::from(self.h) - 2 * vin).max(2);
        let mut b = antibox_ui::widget::PanelButton::new(
            Rect::new(0, vin, i32::from(self.w), bh),
            antibox_ui::theme::face(),
            antibox_ui::theme::text(),
            &font,
            LABEL,
        );
        b.sunken = self.pressed;
        b.on_bar = true;
        antibox_ui::widget::panel_button(g, &b, self.window.id());
    }

    fn preferred_width(&self) -> u32 {
        u32::from(self.w)
    }

    fn preferred_height(&self) -> u32 {
        antibox_ui::metrics::button_height() as u32
    }

    fn handle_click(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
        self.pressed = true;
        self.pending = true;
        Some(self.window.id())
    }

    fn handle_release(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
        self.pressed = false;
        Some(self.window.id())
    }

    fn set_theme_colours(&mut self, tc: &ThemeColors) {
        self.colours = *tc;
    }

    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.w = w;
        self.h = h;
        let _ = self.window.move_window(Point::new(i32::from(x), i32::from(y)));
        let _ = self.window.resize(w, h);
    }

    fn handle_leave(&mut self) {
        self.pressed = false;
    }

    fn take_action(&mut self) -> Option<crate::action::Action> {
        if self.pending {
            self.pending = false;
            self.pressed = false;
            Some(crate::action::Action::Menu(crate::action::MenuOp::RootMenu))
        } else {
            None
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
#[path = "menu_applet_tests.rs"]
mod tests;
