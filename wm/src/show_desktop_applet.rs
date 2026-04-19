 use antibox_core::error::Result;
use crate::applet::Applet;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

pub struct ShowDesktopApplet {
    pub(crate) window: Box<dyn WindowHandle>,
    conn: Arc<dyn DisplayBackend>,
    face: antibox_core::colour::Colour,
    w: u16,
    h: u16,
    pressed: bool,
    pending: Option<crate::action::Action>,
    tooltip: Option<crate::tooltip::ToolTip>,
}

impl ShowDesktopApplet {
    pub fn new(conn: &Arc<dyn DisplayBackend>, parent: u32) -> Result<Self> {
        let side = antibox_ui::metrics::panel_height().max(8);
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, side * 2 / 3, side),
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
            w: (side * 2 / 3) as u16,
            h: side as u16,
            pressed: false,
            pending: None,
            tooltip: None,
        })
    }
}

impl Applet for ShowDesktopApplet {
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }

    fn set_theme_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.face = tc.task_bar_colour;
    }

    fn paint(&self, g: &dyn GraphicsContext) {
        let _ = g.set_foreground(self.face);
        let _ = g.fill_rect(0, 0, self.w, self.h);
        antibox_ui::theme::bevel(g, 0, 0, self.w, self.h, self.pressed);
        let inset = (i32::from(self.w) / 4).max(2) as i16;
        let mw = self.w.saturating_sub(inset as u16 * 2).max(4);
        let mh = self.h.saturating_sub(inset as u16 * 2).max(4);
        let _ = g.set_foreground(antibox_ui::theme::field());
        let _ = g.fill_rect(inset, inset, mw, mh);
        let _ = g.set_foreground(antibox_ui::theme::sel_bg());
        let _ = g.fill_rect(inset, inset, mw, (mh / 4).max(1));
        let _ = g.set_foreground(antibox_ui::theme::shadow());
        let _ = g.draw_rect(inset, inset, mw, mh);
    }

    fn preferred_width(&self) -> u32 {
        u32::from(self.w)
    }

    fn preferred_height(&self) -> u32 {
        u32::from(self.h)
    }

    fn handle_click(&mut self, _x: i32, _y: i32, button: u8) -> Option<u32> {
        if button == 1 {
            self.pressed = true;
            self.pending = Some(crate::action::Action::Workspace(
                crate::action::WorkspaceOp::ShowDesktop,
            ));
        }
        None
    }

    fn handle_release(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
        self.pressed = false;
        None
    }

    fn take_action(&mut self) -> Option<crate::action::Action> {
        self.pending.take()
    }

    fn handle_enter(&mut self) {
        crate::tooltip::show_rect_tip(
            &mut self.tooltip,
            self.conn.as_ref(),
            &*self.window,
            (0, 0, self.w, self.h),
            (self.w, self.h),
            "Show desktop",
        );
    }

    fn handle_leave(&mut self) {
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
