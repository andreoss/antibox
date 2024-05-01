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
    tooltip: Option<crate::tooltip::ToolTip>,
    drag: Option<Drag>,
    focused_id: u32,
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
            tooltip: None,
            drag: None,
            focused_id: 0,
        })
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
            if Some(i) == float_idx {
                continue;
            }
            self.draw_button(g, btn, btn.rect.0, btn.active, btn.active);
        }
        if let Some(d) = floating {
            self.draw_button(g, &self.buttons[d.index], self.float_x(d), false, true);
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
        if (button == 4 || button == 5) && !crate::layout_preferences::taskbar_wheel_enabled() {
            return None;
        }
        if button == 1 {
            if let Some(i) = self.button_index_at(x) {
                let (bx, _, bw, _) = self.buttons[i].rect;
                self.drag = Some(Drag {
                    index: i,
                    start_x: x,
                    grab_dx: x - bx as i32,
                    cur_x: x,
                    bw,
                    window_id: self.buttons[i].window_id,
                    moved: false,
                });
            }
            return None;
        }
        self.find_by_pos(x)
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
            self.drag_motion(x);
            return;
        }
        let idx = self.button_index_at(x);
        if idx == self.hovered {
            return;
        }
        self.hovered = idx;
        match idx {
            Some(i) => {
                let btn = &self.buttons[i];
                let rect = btn.rect;
                let label = btn.label.clone();
                let text_w = antibox_ui::metrics::text_w(label.len().max(1));
                let avail = (rect.2 as i32 - 6).max(1);
                if text_w > avail || label.contains("  (") {
                    crate::tooltip::show_rect_tip(
                        &mut self.tooltip,
                        self.conn.as_ref(),
                        &*self.window,
                        rect,
                        &label,
                    );
                } else {
                    crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
                }
            }
            None => crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref()),
        }
    }
    fn handle_leave(&mut self) {
        self.hovered = None;
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }
    fn tick_tooltip(&mut self) {
        crate::tooltip::pump_tip(&mut self.tooltip, self.conn.as_ref());
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
