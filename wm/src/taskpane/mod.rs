 use antibox_core::error::Result;
mod data;
mod drag;
mod render;
pub use self::render::tick_urgent_phase;
mod sync;
pub use self::data::TaskSync;

use self::data::{Drag, TaskButton};
use crate::applet::Applet;
use crate::render::ThemeColors;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

pub struct TaskPane {
    conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    pub(crate) buttons: Vec<TaskButton>,
    pub(crate) colours: ThemeColors,
    pane_w: u16,
    pane_h: u16,
    hovered: Option<usize>,
    pub(crate) vertical: bool,
    pub(crate) scroll: i16,
    tooltip: Option<crate::tooltip::ToolTip>,
    preview: crate::winpreview::WindowPreview,
    drag: Option<Drag>,
    focused_id: u32,
    pub(crate) visible_count: usize,
    pub(crate) overflow: Option<(i16, i16, u16, u16)>,
    pub(crate) overflow_ids: Vec<u32>,
    pub(crate) overflow_before: Option<(i16, i16, u16, u16)>,
    pub(crate) overflow_before_ids: Vec<u32>,
}

impl TaskPane {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        colours: ThemeColors,
    ) -> Result<Self> {
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, 200, 28),
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
            conn: Arc::clone(conn),
            window,
            buttons: Vec::new(),
            colours,
            pane_w: 200,
            pane_h: 28,
            hovered: None,
            vertical: false,
            scroll: 0,
            tooltip: None,
            preview: crate::winpreview::WindowPreview::new(),
            drag: None,
            focused_id: 0,
            visible_count: 0,
            overflow: None,
            overflow_ids: Vec::new(),
            overflow_before: None,
            overflow_before_ids: Vec::new(),
        })
    }

    fn show_preview(&mut self, local: (i16, i16, u16, u16), client: u32, icon: Option<PixmapData>) {
        if !crate::layout_preferences::taskbar_preview() {
            return;
        }
        let (near, beside) =
            crate::taskbar::applet_anchor(&*self.window, local, (self.pane_w, self.pane_h));
        self.preview.show(self.conn.as_ref(), client, icon, near, beside);
    }
}

impl Applet for TaskPane {
    fn set_theme_colours(&mut self, tc: &ThemeColors) {
        self.colours = *tc;
    }
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }
    fn paint(&self, g: &dyn GraphicsContext) {
        antibox_ui::theme::panel_surface(g, self.pane_w, self.pane_h, self.colours.task_bar_colour);
        let floating = self.drag.as_ref().filter(|d| d.moved);
        let float_idx = floating.map(|d| d.index);
        for (i, btn) in self.buttons.iter().enumerate() {
            if btn.rect.2 == 0 || Some(i) == float_idx {
                continue;
            }
            self.draw_button(g, btn, btn.rect.0, btn.rect.1, btn.active, btn.active);
        }
        self.draw_overflow(g);
        if let Some(d) = floating {
            self.draw_button(
                g,
                &self.buttons[d.index],
                self.float_x(d),
                self.float_y(d),
                false,
                true,
            );
        }
    }
    fn preferred_width(&self) -> u32 {
        let n = self.buttons.len() as i32;
        (n * Self::max_button_w() + (n - 1).max(0) * antibox_ui::metrics::item_gap()).max(0) as u32
    }
    fn preferred_height(&self) -> u32 {
        antibox_ui::metrics::panel_height() as u32
    }
    fn handle_click(&mut self, x: i32, _y: i32, button: u8) -> Option<u32> {
        if (button == 4 || button == 5) && self.vertical {
            let step = antibox_ui::metrics::panel_height() as i16;
            let content = self.buttons.len() as i16
                * (step + antibox_ui::metrics::item_gap() as i16);
            if content > self.pane_h as i16 {
                self.scroll += if button == 4 { -step } else { step };
                let (pw, ph) = (self.pane_w, self.pane_h);
                self.layout_buttons(pw, ph);
                self.repaint();
                return None;
            }
        }
        if (button == 4 || button == 5) && !crate::layout_preferences::taskbar_wheel_enabled() {
            return None;
        }
        if button == 1 {
            if let Some(i) = self.button_at(x, _y) {
                let (bx, by, bw, bh) = self.buttons[i].rect;
                self.drag = Some(Drag {
                    index: i,
                    start_x: x,
                    start_y: _y,
                    grab_dx: x - bx as i32,
                    grab_dy: _y - by as i32,
                    cur_x: x,
                    cur_y: _y,
                    bw,
                    bh,
                    window_id: self.buttons[i].window_id,
                    moved: false,
                });
            }
            return None;
        }
        self.find_by_pos(x, _y)
    }
    fn handle_release(&mut self, _x: i32, _y: i32, button: u8) -> Option<u32> {
        if button != 1 {
            return None;
        }
        let drag = self.drag.take()?;
        if drag.moved {
            return None;
        }
        let btn = self
            .buttons
            .iter()
            .find(|b| b.window_id == drag.window_id)?;
        Some(self.cycle_target(btn))
    }
    fn handle_motion(&mut self, x: i32, _y: i32) {
        if self.drag.is_some() {
            self.preview.hide(self.conn.as_ref());
            self.drag_motion(x, _y);
            return;
        }
        let idx = self.button_at(x, _y);
        if idx == self.hovered {
            return;
        }
        self.hovered = idx;
        match idx {
            Some(i) => {
                let btn = &self.buttons[i];
                let rect = btn.rect;
                let label = btn.label.clone();
                let icon = btn.icon.clone();
                self.show_preview(rect, btn.window_id, icon);
                let text_w = antibox_ui::metrics::text_w(label.len().max(1));
                let avail = (rect.2 as i32 - 6).max(1);
                if text_w > avail || label.contains("  (") {
                    crate::tooltip::show_rect_tip(
                        &mut self.tooltip,
                        self.conn.as_ref(),
                        &*self.window,
                        rect,
                        (self.pane_w, self.pane_h),
                        &label,
                    );
                } else {
                    crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
                }
            }
            None => {
                self.preview.hide(self.conn.as_ref());
                crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
            }
        }
    }

    fn handle_leave(&mut self) {
        self.hovered = None;
        self.preview.hide(self.conn.as_ref());
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }
    fn tick_tooltip(&mut self) {
        self.preview.pump(self.conn.as_ref());
        crate::tooltip::pump_tip(&mut self.tooltip, self.conn.as_ref());
    }
    fn shutdown(&mut self) {
        self.preview.hide(self.conn.as_ref());
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }
    fn tooltip_pending(&self) -> bool {
        crate::tooltip::tip_pending(&self.tooltip)
    }
    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.pane_w = w;
        self.pane_h = h;
        let _ = self
            .window
            .configure(Some(x as i32), Some(y as i32), Some(w), Some(h));
        self.layout_buttons(w, h);
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests;
