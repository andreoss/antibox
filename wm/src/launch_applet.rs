 use antibox_core::error::Result;
use crate::applet::Applet;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

pub struct LaunchApplet {
    pub(crate) window: Box<dyn WindowHandle>,
    conn: Arc<dyn DisplayBackend>,
    face: antibox_core::colour::Colour,
    items: Vec<String>,
    w: u16,
    h: u16,
    pressed: Option<usize>,
    pending: Option<crate::action::Action>,
    tooltip: Option<crate::tooltip::ToolTip>,
}

fn cell() -> i16 {
    (antibox_ui::metrics::panel_height() * 3 / 4).max(10) as i16
}

impl LaunchApplet {
    pub fn new(conn: &Arc<dyn DisplayBackend>, parent: u32, items: Vec<String>) -> Result<Self> {
        let h = antibox_ui::metrics::panel_height().max(8) as u16;
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, 1, i32::from(h)),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW,
        )?;
        Ok(Self {
            window,
            conn: Arc::clone(conn),
            face: antibox_ui::theme::face(),
            items,
            w: 1,
            h,
            pressed: None,
            pending: None,
            tooltip: None,
        })
    }

    fn item_at(&self, x: i32, y: i32) -> Option<usize> {
        let c = i32::from(cell() as u16);
        if y < 0 || y >= i32::from(self.h) {
            return None;
        }
        let i = x / c.max(1);
        (x >= 0 && (i as usize) < self.items.len()).then_some(i as usize)
    }
}

impl Applet for LaunchApplet {
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }

    fn set_theme_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.face = tc.task_bar_colour;
    }

    fn paint(&self, g: &dyn GraphicsContext) {
        let _ = g.set_foreground(self.face);
        let _ = g.fill_rect(0, 0, self.w, self.h);
        let c = cell() as u16;
        let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
        for (i, item) in self.items.iter().enumerate() {
            let x = i as i16 * c as i16;
            let sunken = self.pressed == Some(i);
            let inset = antibox_core::scale::scaled(1).max(1) as i16;
            let side = c.saturating_sub(inset as u16 * 2);
            antibox_ui::theme::bevel(g, x + inset, inset, side, self.h.saturating_sub(inset as u16 * 2), sunken);
            let letter: String = item
                .chars()
                .next()
                .map(|ch| ch.to_uppercase().to_string())
                .unwrap_or_default();
            let tw = g.text_width(&letter).unwrap_or(6) as i16;
            let lx = x + ((c as i16 - tw) / 2).max(0);
            let baseline = antibox_ui::metrics::baseline(0, i32::from(self.h)) as i16;
            let _ = g.set_foreground(antibox_ui::theme::text());
            let _ = g.set_background(self.face);
            let _ = g.draw_text_transparent(lx, baseline, &letter);
        }
    }

    fn preferred_width(&self) -> u32 {
        if self.items.is_empty() {
            return 0;
        }
        self.items.len() as u32 * cell() as u32
    }

    fn preferred_height(&self) -> u32 {
        antibox_ui::metrics::panel_height() as u32
    }

    fn handle_click(&mut self, x: i32, y: i32, button: u8) -> Option<u32> {
        if button == 1 {
            self.pressed = self.item_at(x, y);
        }
        None
    }

    fn handle_release(&mut self, x: i32, y: i32, button: u8) -> Option<u32> {
        if button == 1 {
            if let (Some(down), Some(up)) = (self.pressed.take(), self.item_at(x, y)) {
                if down == up {
                    self.pending = Some(crate::action::Action::Misc(
                        crate::action::MiscOp::Command(self.items[up].clone()),
                    ));
                }
            }
        }
        None
    }

    fn take_action(&mut self) -> Option<crate::action::Action> {
        self.pending.take()
    }

    fn handle_motion(&mut self, x: i32, y: i32) {
        let tip = self.item_at(x, y).map(|i| self.items[i].clone());
        match tip {
            Some(text) => crate::tooltip::show_rect_tip(
                &mut self.tooltip,
                self.conn.as_ref(),
                &*self.window,
                (0, 0, self.w, self.h),
                (self.w, self.h),
                &text,
            ),
            None => crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref()),
        }
    }

    fn handle_leave(&mut self) {
        self.pressed = None;
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn tick_tooltip(&mut self) {
        crate::tooltip::pump_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn tooltip_pending(&self) -> bool {
        crate::tooltip::tip_pending(&self.tooltip)
    }

    fn shutdown(&mut self) {
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.w = w;
        self.h = h;
        let _ = self
            .window
            .configure(Some(i32::from(x)), Some(i32::from(y)), Some(w), Some(h));
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
