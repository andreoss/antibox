 use antibox_core::error::Result;
use crate::action::WorkspaceOp;
use crate::applet::Applet;
use crate::render::ThemeColors;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

pub struct MiniWin {
    pub rect: (i16, i16, u16, u16),
    pub client: u32,
    pub frame: Rect,
    pub focused: bool,
    pub icon: Option<PixmapData>,
}

impl PartialEq for MiniWin {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect
            && self.client == other.client
            && self.frame == other.frame
            && self.focused == other.focused
            && match (&self.icon, &other.icon) {
                (None, None) => true,
                (Some(a), Some(b)) => {
                    a.width == b.width && a.height == b.height && a.data == b.data
                }
                _ => false,
            }
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

pub(crate) fn mini_icon_rect(mx: i16, my: i16, mw: u16, mh: u16) -> Option<(i16, i16, u16)> {
    let th = mini_title_height(mh);
    let pad = antibox_core::scale::scaled(1).max(1) as u16;
    let body_h = mh.saturating_sub(th);
    let side = mw.min(body_h).saturating_sub(2 * pad);
    if (side as i32) < (antibox_ui::metrics::icon() / 2).max(4) {
        return None;
    }
    let ix = mx + ((mw - side) / 2) as i16;
    let iy = my + th as i16 + ((body_h - side) / 2) as i16;
    Some((ix, iy, side))
}

pub struct WorkspaceButton {
    pub label: String,
    pub active: bool,

    pub urgent: bool,
    pub rect: (i16, i16, u16, u16),
    pub minis: Vec<MiniWin>,
}

fn cell_at(buttons: &[WorkspaceButton], x: i32, y: i32, rows: u16) -> Option<usize> {
    buttons.iter().position(|b| {
        let (bx, by, bw, bh) = b.rect;
        let in_x = x >= i32::from(bx) && x < i32::from(bx) + i32::from(bw);
        let in_y = rows <= 1 || (y >= i32::from(by) && y < i32::from(by) + i32::from(bh));
        in_x && in_y
    })
}

fn row_gap() -> i16 {
    (antibox_ui::metrics::gap() + antibox_core::scale::scaled(2)) as i16
}

fn cell_width(cell_h: i16, sw: i32, sh: i32) -> u16 {
    let cw_min = antibox_ui::metrics::panel_height() * 2 / 3;
    let cw_max = antibox_ui::metrics::panel_height() * 5 / 2;
    (i32::from(cell_h) * sw / sh.max(1)).clamp(cw_min, cw_max) as u16
}

fn grid_columns(wrap: Option<u16>, cell_w: u16, gap: u16, count: usize) -> usize {
    let count = count.max(1);
    let Some(avail) = wrap else { return count };
    let step = i32::from(cell_w) + i32::from(gap);
    let fits = (((i32::from(avail) + i32::from(gap)) / step.max(1)).max(1) as usize).min(count);
    let rows = count.div_ceil(fits);
    count.div_ceil(rows)
}

pub struct WorkspacesPane {
    pub(crate) window: Box<dyn WindowHandle>,
    pub(crate) buttons: Vec<WorkspaceButton>,
    pub(crate) active_workspace: u32,
    pub(crate) theme_colours: ThemeColors,
    pub(crate) total_width: u32,
    pub(crate) pane_h: u16,
    wrap: Option<u16>,
    rows: u16,
    pub(crate) pending: Option<crate::action::Action>,
    pub(crate) conn: Arc<dyn DisplayBackend>,
    hovered: Option<usize>,
    tooltip: Option<crate::tooltip::ToolTip>,
    preview: crate::winpreview::WindowPreview,
}

impl WorkspacesPane {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        names: &[String],
        theme_colours: ThemeColors,
    ) -> Result<Self> {
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
        let mut pane = Self {
            window,
            buttons: Vec::new(),
            active_workspace: 0,
            theme_colours,
            total_width: 1,
            pane_h: 28,
            wrap: None,
            rows: 1,
            pending: None,
            conn: Arc::clone(conn),
            hovered: None,
            tooltip: None,
            preview: crate::winpreview::WindowPreview::new(),
        };
        pane.set_names(names);
        Ok(pane)
    }

    fn show_preview(&mut self, index: usize) {
        if !crate::layout_preferences::pager_hover_preview() {
            return;
        }
        let Some(btn) = self.buttons.get(index) else { return };
        let wins: Vec<crate::winpreview::DeskWin> = btn
            .minis
            .iter()
            .map(|m| crate::winpreview::DeskWin {
                client: m.client,
                rect: m.frame,
                icon: m.icon.clone(),
            })
            .collect();
        let (near, beside) = crate::taskbar::applet_anchor(
            &*self.window,
            btn.rect,
            (self.total_width as u16, self.pane_h),
        );
        let screen = Rect::new(
            0,
            0,
            self.conn.screen_width() as i32,
            self.conn.screen_height() as i32,
        );
        self.preview.show_desk(
            self.conn.as_ref(),
            u64::from(index as u32),
            wins,
            screen,
            near,
            beside,
        );
    }

    pub fn set_wrap_width(&mut self, wrap: Option<u16>) {
        if self.wrap == wrap {
            return;
        }
        self.wrap = wrap;
        let names: Vec<String> = self.buttons.iter().map(|b| b.label.clone()).collect();
        if !names.is_empty() {
            self.set_names(&names);
        }
    }

    pub const fn rows(&self) -> u16 {
        self.rows
    }

    pub fn set_names(&mut self, names: &[String]) {
        let sw = self.conn.screen_width().max(1) as i32;
        let sh = self.conn.screen_height().max(1) as i32;
        let top = antibox_ui::metrics::button_inset() as i16;
        let cell_h = antibox_ui::metrics::button_height() as i16;
        let grid = crate::layout_preferences::pager_grid();
        let cell_w = cell_width(cell_h, sw, sh);
        let gap = if grid { 0 } else { antibox_ui::metrics::gap() as i16 };
        let cols = grid_columns(self.wrap, cell_w, gap as u16, names.len());
        let full = cols as i16 * (cell_w as i16 + gap) - gap;
        let mut buttons = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let (col, row) = (i % cols, i / cols);
            let in_row = (names.len() - row * cols).min(cols) as i16;
            let used = in_row * (cell_w as i16 + gap) - gap;
            let indent = (full - used).max(0) / 2;
            let x = indent + col as i16 * (cell_w as i16 + gap);
            let vgap = if grid { 0 } else { row_gap() };
            let y = top + row as i16 * (cell_h + vgap);
            buttons.push(WorkspaceButton {
                label: name.clone(),
                active: i as u32 == self.active_workspace,
                urgent: false,
                rect: (x, y, cell_w, cell_h as u16),
                minis: Vec::new(),
            });
        }
        self.total_width = full.max(1) as u32;
        self.rows = names.len().div_ceil(cols).max(1) as u16;
        self.buttons = buttons;
        let h = self.preferred_height() as u16;
        self.pane_h = h;
        let _ = self
            .window
            .configure(None, None, Some(self.total_width as u16), Some(h));
    }

    fn paint_grid(&self, g: &dyn GraphicsContext) {
        if self.buttons.is_empty() {
            return;
        }
        let base = crate::render::darken_colour(self.theme_colours.task_bar_colour, 0.30);
        let active_bg = antibox_core::colour::distinct(
            antibox_core::colour::lerp(base, self.theme_colours.workspace_active_bg, 0.3),
            base,
        );
        let line = crate::render::darken_colour(self.theme_colours.task_bar_colour, 0.55);
        let win = crate::render::brighten_colour(base, 0.55);
        let win_focus = self.theme_colours.workspace_active_bg;
        for btn in &self.buttons {
            let (x, y, w, h) = btn.rect;
            let bg = if btn.urgent {
                self.theme_colours.urgent_bg
            } else if btn.active {
                active_bg
            } else {
                base
            };
            let _ = g.set_foreground(bg);
            let _ = g.fill_rect(x, y, w, h);
            for mini in &btn.minis {
                let (mx, my, mw, mh) = mini.rect;
                let _ = g.set_foreground(if mini.focused { win_focus } else { win });
                let _ = g.fill_rect(mx, my, mw, mh);
                let _ = g.set_foreground(line);
                let _ = g.draw_rect(mx, my, mw, mh);
            }
        }
        let _ = g.set_foreground(line);
        for btn in &self.buttons {
            let (x, y, w, h) = btn.rect;
            let _ = g.draw_rect(x, y, w, h);
        }
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
                let (mx, my, mw, mh) = rect;
                let icon = mini_icon_rect(mx, my, mw, mh).map(|(_, _, side)| {
                    crate::icon_render::resolve_client_icon(
                        fw.client().icons(),
                        side,
                        self.theme_colours.task_bar_colour,
                    )
                });
                minis.push(MiniWin {
                    rect,
                    client: fw.client_xid(),
                    frame: fw.frame_rect(),
                    focused: Some(*id) == focused,
                    icon,
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

    pub fn set_active(&mut self, idx: u32) -> bool {
        if self.active_workspace == idx && self.buttons.iter().enumerate().all(|(i, b)| b.active == (i as u32 == idx)) {
            return false;
        }
        for (i, btn) in self.buttons.iter_mut().enumerate() {
            btn.active = i as u32 == idx;
        }
        self.active_workspace = idx;
        true
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
            self.pane_h.max(antibox_ui::metrics::panel_height() as u16),
            self.theme_colours.task_bar_colour,
        );
        if crate::layout_preferences::pager_grid() {
            self.paint_grid(g);
            return;
        }
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
                    if let Some(icon) = &mini.icon {
                        if let Some((ix, iy, _)) = mini_icon_rect(mx, my, mw, mh) {
                            let _ = g.draw_pixmap(ix, iy, icon);
                        }
                    }
                    let _ = g.set_foreground(outline);
                    let _ = g.draw_rect(mx, my, mw, mh);
                }
            }
            if crate::layout_preferences::pager_numbers() {
                let text = roman(i + 1);
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
        let base = antibox_ui::metrics::panel_height() as u32;
        if self.rows <= 1 {
            return base;
        }
        let cell_h = antibox_ui::metrics::button_height() as u32;
        let gap = if crate::layout_preferences::pager_grid() {
            0
        } else {
            row_gap() as u32
        };
        let inset = antibox_ui::metrics::button_inset() as u32;
        u32::from(self.rows) * (cell_h + gap) - gap + inset * 2
    }
    fn handle_click(&mut self, x: i32, y: i32, button: u8) -> Option<u32> {
        if (button == 4 || button == 5) && !crate::layout_preferences::taskbar_wheel_enabled() {
            return None;
        }
        if let Some(i) = cell_at(&self.buttons, x, y, self.rows) {
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
        }
        None
    }

    fn take_action(&mut self) -> Option<crate::action::Action> {
        self.pending.take()
    }
    fn handle_motion(&mut self, x: i32, y: i32) {
        let idx = cell_at(&self.buttons, x, y, self.rows);
        if idx == self.hovered {
            return;
        }
        self.hovered = idx;
        match idx {
            Some(i) => {
                let rect = self.buttons[i].rect;
                let label = self.buttons[i].label.clone();
                self.show_preview(i);
                crate::tooltip::show_rect_tip(
                    &mut self.tooltip,
                    self.conn.as_ref(),
                    &*self.window,
                    rect,
                    (self.total_width as u16, self.pane_h),
                    &label,
                );
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
        self.pane_h = h;
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

fn roman(mut n: usize) -> String {
    const VALUES: [(usize, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    if n == 0 {
        return "N".to_string();
    }
    let mut out = String::new();
    for (v, s) in VALUES {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}
