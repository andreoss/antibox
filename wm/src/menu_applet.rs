use crate::applet::Applet;
use crate::render::ThemeColors;
use antibox_core::backend::*;
use antibox_core::error::Result;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::any::Any;
use std::sync::Arc;


pub struct MenuApplet {
    window: Box<dyn WindowHandle>,
    w: u16,
    h: u16,
    pressed: bool,
    pending: bool,
    colours: ThemeColors,
}

impl MenuApplet {
    pub fn new(conn: &Arc<dyn DisplayBackend>, parent: u32) -> Result<Self> {
        let label = antibox_ui::theme::menu_label();
        let w = antibox_ui::metrics::text_w(label.chars().count().max(4)) + antibox_ui::metrics::pad() * 4;
        let h = antibox_ui::metrics::panel_height();
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w, h),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE | EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
        )?;
        Ok(Self {
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
        use antibox_ui::theme;
        theme::panel_surface(g, self.w, self.h, self.colours.task_bar_colour);
        let flush = theme::taskbar_flush();
        let full_bleed = flush && theme::start_cap_px() > 0;
        if full_bleed {
            theme::panel_edge(g, self.w);
        }
        let inset = if full_bleed {
            0
        } else if flush {
            antibox_core::scale::scaled(1) as i16
        } else {
            antibox_ui::metrics::button_inset() as i16
        };
        let start_pt = theme::start_font_pt();
        let pt = if start_pt > 0 {
            start_pt
        } else {
            antibox_ui::metrics::font_pt()
        };
        let mut font = FontSpec::ui(pt);
        if !theme::taskbar_plain() {
            font = font.bold();
        }
        if theme::start_italic() {
            font = font.italic();
        }
        let bh = if flush {
            (self.h as i16 - 2 * inset).max(1) as u16
        } else {
            antibox_ui::metrics::button_height() as u16
        };
        if theme::themed_start_button(g, 0, inset, self.w, bh, self.pressed) {
            if let Some(slivers) = theme::start_cap_slivers(self.w, bh) {
                for (sx, sy, sw, sh) in slivers {
                    let clip =
                        Rect::new(i32::from(sx), i32::from(inset + sy), i32::from(sw), i32::from(sh));
                    let _ = g.push_clip(&clip);
                    theme::panel_surface(g, self.w, self.h, self.colours.task_bar_colour);
                    if full_bleed {
                        theme::panel_edge(g, self.w);
                    }
                    let _ = g.pop_clip();
                }
            }
            let label = theme::menu_label_raw();
            let gs = if theme::has_start_glyph() {
                if label.is_empty() {
                    (bh as i16 * 4 / 5).max(10)
                } else {
                    (bh as i16 * 5 / 8).max(8)
                }
            } else {
                0
            };
            if label.is_empty() && gs == 0 {
                return;
            }
            let _ = g.set_font(&font);
            let tw = if label.is_empty() {
                0
            } else {
                g.text_width(&label).unwrap_or(0) as i16
            };
            let ggap = if gs > 0 && tw > 0 {
                (antibox_ui::metrics::gap() as i16).max(2)
            } else {
                0
            };
            let cap = (theme::start_cap_px() as i16).min(bh as i16 / 2);
            let block = gs + ggap + tw;
            let off = if self.pressed {
                antibox_core::scale::scaled(1) as i16
            } else {
                0
            };
            let bx = if label.is_empty() {
                (self.w as i16 - cap - block) / 2 + off
            } else {
                ((self.w as i16 - cap - block) / 2).max(antibox_ui::metrics::pad() as i16) + off
            };
            if gs > 0 {
                let gy = inset + (bh as i16 - gs) / 2 + off;
                let _ = theme::themed_start_glyph(g, bx, gy, gs as u16);
            }
            if label.is_empty() {
                return;
            }
            let lx = bx + gs + ggap;
            let bl = antibox_ui::metrics::baseline(i32::from(inset), i32::from(bh)) as i16 + off;
            let shadow = theme::start_shadow();
            if shadow != 0 || theme::title_text_shadow() {
                let _ = g.set_foreground(if shadow != 0 { shadow } else { theme::text() });
                let _ = g.draw_text_transparent(lx + 1, bl + 1, &label);
            }
            let _ = g.set_foreground(theme::taskbar_text_colour());
            let _ = g.draw_text_transparent(lx, bl, &label);
            return;
        }
        let label = theme::menu_label();
        let mut b = antibox_ui::widget::PanelButton::new(
            Rect::new(0, i32::from(inset), i32::from(self.w), i32::from(bh)),
            self.colours.button_bg,
            self.colours.button_fg,
            &font,
            &label,
        );
        b.sunken = self.pressed;
        b.on_bar = true;
        antibox_ui::widget::panel_button(g, &b, self.window.id());
    }

    fn preferred_width(&self) -> u32 {
        use antibox_ui::theme;
        if theme::has_start_button_element() && theme::menu_label_raw().is_empty() {
            let flush = theme::taskbar_flush();
            let bh = if flush {
                antibox_ui::metrics::panel_height() - 2 * antibox_core::scale::scaled(1)
            } else {
                antibox_ui::metrics::button_height()
            };
            return bh.max(1) as u32;
        }
        let chars = theme::menu_label().chars().count().max(4);
        let mut text_w = antibox_ui::metrics::text_w(chars);
        let start_pt = i32::from(theme::start_font_pt());
        if start_pt > 0 {
            text_w = text_w * start_pt / (antibox_ui::metrics::font_pt() as i32).max(1);
        }
        let mut w = antibox_ui::metrics::pad() * 4 + text_w;
        if theme::has_start_glyph() {
            w += antibox_ui::metrics::panel_height() * 5 / 8 + antibox_ui::metrics::gap();
        }
        w += i32::from(theme::start_cap_px());
        w as u32
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
