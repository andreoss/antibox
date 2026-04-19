pub(super) use super::TaskPane;
use crate::applet::Applet;
use antibox_core::backend::GraphicsContext;
use antibox_core::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

static URGENT_PHASE: AtomicU32 = AtomicU32::new(0);

pub fn tick_urgent_phase() {
    URGENT_PHASE.fetch_add(1, Ordering::Relaxed);
}

impl TaskPane {
    pub(super) fn draw_button(
        &self,
        g: &dyn GraphicsContext,
        btn: &super::data::TaskButton,
        x: i16,
        y: i16,
        sunken: bool,
        bold: bool,
    ) {
        let (_, _, w, h) = btn.rect;
        let flash = btn.urgent && URGENT_PHASE.load(Ordering::Relaxed) & 1 != 0;
        let bg = if flash {
            antibox_ui::theme::sel_bg()
        } else if btn.urgent {
            self.colours.urgent_bg
        } else {
            Self::button_bg(self.colours.task_bar_colour, btn.active)
        };
        let fg = if flash {
            antibox_ui::theme::sel_fg()
        } else if btn.urgent {
            self.colours.urgent_fg
        } else if btn.minimized {
            antibox_core::colour::lerp(self.colours.button_fg, bg, 0.4)
        } else {
            self.colours.button_fg
        };
        let role = if btn.active {
            antibox_core::backend::FontRole::ActiveTaskBar
        } else {
            antibox_core::backend::FontRole::NormalTaskBar
        };
        let font = antibox_core::backend::FontSpec::role_styled(
            role,
            antibox_ui::metrics::font_pt(),
            bold,
            false,
        );
        let progress = btn.progress.filter(|&p| p > 0).map(|p| {
            (
                p,
                crate::render::brighten_colour(self.colours.workspace_active_bg, 0.55),
            )
        });
        let label = if crate::layout_preferences::taskbar_show_titles() {
            btn.label.as_str()
        } else {
            ""
        };
        let mut b = antibox_ui::widget::PanelButton::new(
            antibox_core::rect::Rect::px(x, y, w, h),
            bg,
            fg,
            &font,
            label,
        );
        b.sunken = sunken;
        b.icon = btn.icon.as_ref();
        b.progress = progress;
        b.align = crate::layout_preferences::taskbar_align();
        b.on_bar = true;
        antibox_ui::widget::panel_button(g, &b, self.window.id());
    }

    pub(super) fn draw_overflow(&self, g: &dyn GraphicsContext) {
        let Some((x, y, w, h)) = self.overflow else {
            return;
        };
        let n = self.buttons.len().saturating_sub(self.visible_count);
        if n == 0 {
            return;
        }
        let bg = Self::button_bg(self.colours.task_bar_colour, false);
        let font = antibox_core::backend::FontSpec::role_styled(
            antibox_core::backend::FontRole::NormalTaskBar,
            antibox_ui::metrics::font_pt(),
            true,
            false,
        );
        let fg = self.colours.button_fg;
        let count = n.to_string();
        let mut b = antibox_ui::widget::PanelButton::new(
            antibox_core::rect::Rect::px(x, y, w, h),
            bg,
            fg,
            &font,
            "",
        );
        b.on_bar = true;
        antibox_ui::widget::panel_button(g, &b, self.window.id());
        let _ = g.set_font(&font);
        let tw = g.text_width(&count).unwrap_or(0) as i16;
        let asize = (h as i16 * 2 / 3).clamp(6, 14);
        let agap = antibox_ui::metrics::gap() as i16;
        let bx = x + ((w as i16 - asize - agap - tw) / 2).max(0);
        let cy = y + h as i16 / 2;
        antibox_ui::theme::arrow_glyph(g, bx + asize / 2, cy, asize, overflow_arrow(), fg);
        let baseline = antibox_ui::metrics::baseline(y as i32, h as i32) as i16;
        let _ = g.set_foreground(fg);
        let _ = g.draw_text_transparent(bx + asize + agap, baseline, &count);
    }

    pub(super) fn repaint(&self) {
        crate::paintbuf::buffered(
            &*self.conn,
            self.window.id(),
            self.pane_w,
            self.pane_h,
            |g| self.paint(g),
        );
        let _ = self.conn.flush();
    }
}

fn overflow_arrow() -> antibox_ui::theme::Arrow {
    use antibox_ui::theme::Arrow;
    match crate::layout_preferences::taskbar_position() {
        crate::taskbar::TaskBarPosition::Top => Arrow::Down,
        crate::taskbar::TaskBarPosition::LeftVertical => Arrow::Right,
        crate::taskbar::TaskBarPosition::RightVertical => Arrow::Left,
        _ => Arrow::Up,
    }
}
