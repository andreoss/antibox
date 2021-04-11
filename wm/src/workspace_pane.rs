use crate::action::WorkspaceOp;
use crate::applet::Applet;
use crate::render::ThemeColors;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

pub struct MiniWin {
    pub rect: (i16, i16, u16, u16),
    pub focused: bool,
}

impl PartialEq for MiniWin {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect && self.focused == other.focused
    }
}

pub(crate) fn mini_rect(
    fr: Rect,
    sw: f32,
    sh: f32,
    cell: (i16, i16, u16, u16),
) -> (i16, i16, u16, u16) {
    let (bx, by, bw, bh) = cell;
    let iw = bw.saturating_sub(2) as f32;
    let ih = bh.saturating_sub(2) as f32;
    let mx = bx + 1 + (fr.x.max(0) as f32 / sw * iw) as i16;
    let my = by + 1 + (fr.y.max(0) as f32 / sh * ih) as i16;
    let mw = ((fr.w as f32 / sw * iw) as u16).clamp(2, bw.saturating_sub(2).max(2));
    let mh = ((fr.h as f32 / sh * ih) as u16).clamp(2, bh.saturating_sub(2).max(2));
    (mx, my, mw, mh)
}

pub(crate) fn mini_title_height(mh: u16) -> u16 {
    (mh / 4).max(1)
}

pub struct WorkspaceButton {
    pub label: String,
    pub active: bool,

    pub urgent: bool,
    pub rect: (i16, i16, u16, u16),
    pub minis: Vec<MiniWin>,
}

pub struct WorkspacesPane {
    pub(crate) window: Box<dyn WindowHandle>,
    pub(crate) buttons: Vec<WorkspaceButton>,
    pub(crate) active_workspace: u32,
    pub(crate) theme_colours: ThemeColors,
    pub(crate) total_width: u32,
    pub(crate) pending: Option<crate::action::Action>,
    pub(crate) conn: Arc<dyn DisplayBackend>,
    hovered: Option<usize>,
    tooltip: Option<crate::tooltip::ToolTip>,
}

impl WorkspacesPane {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        names: &[String],
        theme_colours: ThemeColors,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, 1, 28),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE
                | EventMask::BUTTON_PRESS
                | EventMask::POINTER_MOTION
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW,
        )?;
        let mut pane = WorkspacesPane {
            window,
            buttons: Vec::new(),
            active_workspace: 0,
            theme_colours,
            total_width: 1,
            pending: None,
            conn: Arc::clone(conn),
            hovered: None,
            tooltip: None,
        };
        pane.set_names(names);
        Ok(pane)
    }

    pub fn set_names(&mut self, names: &[String]) {
        let sw = self.conn.screen_width().max(1) as i32;
        let sh = self.conn.screen_height().max(1) as i32;
        let top = antibox_ui::metrics::button_inset() as i16;
        let cell_h = antibox_ui::metrics::button_height() as i16;
        let cw_min = antibox_ui::metrics::panel_height() * 2 / 3;
        let cw_max = antibox_ui::metrics::panel_height() * 5 / 2;
        let cell_w = (cell_h as i32 * sw / sh).clamp(cw_min, cw_max) as u16;
        let gap = antibox_ui::metrics::gap() as i16;
        let mut x = 0i16;
        let mut buttons = Vec::new();
        for (i, name) in names.iter().enumerate() {
            buttons.push(WorkspaceButton {
                label: name.clone(),
                active: i as u32 == self.active_workspace,
                urgent: false,
                rect: (x, top, cell_w, cell_h as u16),
                minis: Vec::new(),
            });
            x += cell_w as i16 + gap;
        }
        self.total_width = (x - gap).max(1) as u32;
        self.buttons = buttons;
        let h = antibox_ui::metrics::panel_height() as u16;
        let _ = self
            .window
            .configure(None, None, Some(self.total_width as u16), Some(h));
    }

    pub fn sync_from_frames(
        &mut self,
        frames: &crate::frame_store::FrameStore,
        focused: Option<crate::id::ClientId>,
        stack: &[crate::id::ClientId],
    ) -> bool {
        let sw = self.conn.screen_width().max(1) as f32;
        let sh = self.conn.screen_height().max(1) as f32;
        let mut changed = false;
        for (i, btn) in self.buttons.iter_mut().enumerate() {
            let mut wins: Vec<(&crate::id::ClientId, &crate::frame::FrameWindow)> = frames
                .iter()
                .filter(|(_, fw)| {
                    let ws = fw.workspace();
                    !((ws != i as u32 && ws != !0)
                        || fw.state().minimized
                        || fw.state().skip_taskbar)
                })
                .collect();
            wins.sort_by_key(|(id, fw)| {
                (
                    fw.layer(),
                    stack
                        .iter()
                        .position(|&x| x == **id)
                        .unwrap_or(usize::MAX),
                )
            });
            let mut minis = Vec::new();
            for (id, fw) in wins {
                let rect = mini_rect(fw.frame_rect(), sw, sh, btn.rect);
                minis.push(MiniWin {
                    rect,
                    focused: Some(*id) == focused,
                });
            }
            if minis != btn.minis {
                btn.minis = minis;
                changed = true;
            }

            let ws_urgent = frames.iter().any(|(id, fw)| {
                let ws = fw.workspace();
                (ws == i as u32 || ws == !0) && fw.state().urgent && Some(*id) != focused
            });
            if btn.urgent != ws_urgent {
                btn.urgent = ws_urgent;
                changed = true;
            }
        }
        changed
    }

    pub fn set_active(&mut self, idx: u32) {
        for (i, btn) in self.buttons.iter_mut().enumerate() {
            btn.active = i as u32 == idx;
        }
        self.active_workspace = idx;
    }
}

impl Applet for WorkspacesPane {
    fn set_theme_colours(&mut self, tc: &ThemeColors) {
        self.theme_colours = *tc;
    }
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }
    fn paint(&self, g: &dyn GraphicsContext) {
        antibox_ui::theme::panel_surface(
            g,
            self.total_width as u16,
            antibox_ui::metrics::panel_height() as u16,
            self.theme_colours.task_bar_colour,
        );
        let face = self.theme_colours.task_bar_colour;
        for (i, btn) in self.buttons.iter().enumerate() {
            let (x, y, w, h) = btn.rect;
            let bg = if btn.active {
                antibox_ui::theme::pressed_face(face)
            } else {
                face
            };
            antibox_ui::theme::panel_button_surface(
                g,
                Rect::px(x, y, w, h),
                antibox_ui::theme::Fill::new(bg, btn.active),
            );
            if btn.active {
                antibox_ui::theme::selection_overlay(g, x, y, w, h);
            }
            let (focus_fill, normal_fill, outline) = if crate::render::is_dark(bg) {
                (
                    antibox_ui::theme::light(),
                    crate::render::brighten_colour(bg, 0.45),
                    crate::render::darken_colour(bg, 0.4),
                )
            } else {
                (
                    crate::render::brighten_colour(bg, 0.6),
                    crate::render::darken_colour(bg, 0.6),
                    crate::render::darken_colour(bg, 0.3),
                )
            };
            if crate::layout_preferences::pager_preview() {
                for mini in &btn.minis {
                    let (mx, my, mw, mh) = mini.rect;
                    let _ = g.set_foreground(if mini.focused {
                        focus_fill
                    } else {
                        normal_fill
                    });
                    let _ = g.fill_rect(mx, my, mw, mh);
                    let th = mini_title_height(mh);
                    if mh > th + 1 {
                        let _ = g.set_foreground(if mini.focused {
                            self.theme_colours.active_title_top
                        } else {
                            self.theme_colours.inactive_title_top
                        });
                        let _ = g.fill_rect(mx, my, mw, th);
                    }
                    let _ = g.set_foreground(outline);
                    let _ = g.draw_rect(mx, my, mw, mh);
                }
            }
            if crate::layout_preferences::pager_numbers() {
                let text = (i + 1).to_string();
                let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
                let tw = g.text_width(&text).unwrap_or(0) as i16;
                let lx = x + ((w as i16 - tw) / 2).max(0);
                let baseline = antibox_ui::metrics::baseline(y as i32, h as i32) as i16;
                let fg = if btn.active {
                    self.theme_colours.workspace_active_fg
                } else {
                    self.theme_colours.workspace_normal_fg
                };
                let _ = g.set_foreground(fg);
                let _ = g.draw_text_transparent(lx, baseline, &text);
            }
            if btn.urgent && !btn.active {
                let _ = g.set_foreground(self.theme_colours.urgent_bg);
                let _ = g.draw_rect(x, y, w, h);
                let _ = g.draw_rect(x + 1, y + 1, w.saturating_sub(2), h.saturating_sub(2));
            }
        }
    }
    fn preferred_width(&self) -> u32 {
        self.total_width
    }
    fn preferred_height(&self) -> u32 {
        antibox_ui::metrics::panel_height() as u32
    }
    fn handle_click(&mut self, x: i32, _y: i32, button: u8) -> Option<u32> {
        if (button == 4 || button == 5) && !crate::layout_preferences::taskbar_wheel_enabled() {
            return None;
        }
        for (i, btn) in self.buttons.iter().enumerate() {
            let (bx, _, bw, _) = btn.rect;
            if x >= bx as i32 && x < (bx + bw as i16) as i32 {
                if button == 3 {
                    self.pending = Some(crate::action::Action::Workspace(
                        WorkspaceOp::WorkspaceMenu(i as u32),
                    ));
                } else {
                    self.active_workspace = i as u32;
                    self.pending = Some(crate::action::Action::Workspace(WorkspaceOp::Workspace(
                        i as u32,
                    )));
                }
                break;
            }
        }
        None
    }

    fn take_action(&mut self) -> Option<crate::action::Action> {
        self.pending.take()
    }
    fn handle_motion(&mut self, x: i32, _y: i32) {
        let idx = self.buttons.iter().position(|b| {
            let (bx, _, bw, _) = b.rect;
            x >= bx as i32 && x < (bx + bw as i16) as i32
        });
        if idx == self.hovered {
            return;
        }
        self.hovered = idx;
        match idx {
            Some(i) => {
                let rect = self.buttons[i].rect;
                let label = self.buttons[i].label.clone();
                crate::tooltip::show_rect_tip(
                    &mut self.tooltip,
                    self.conn.as_ref(),
                    &*self.window,
                    rect,
                    &label,
                );
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
#[path = "workspace_pane_tests.rs"]
mod tests;
