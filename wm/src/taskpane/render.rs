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
        sunken: bool,
        bold: bool,
    ) {
        let (_, y, w, h) = btn.rect;
        let flash = btn.urgent && URGENT_PHASE.load(Ordering::Relaxed) & 1 != 0;
        let bg = if flash {
            antibox_ui::theme::contrast(self.colours.urgent_bg, self.colours.urgent_bg)
        } else if btn.urgent {
            self.colours.urgent_bg
        } else {
            Self::button_bg(self.colours.task_bar_colour, btn.active)
        };
        let fg = if btn.urgent {
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
        antibox_ui::widget::panel_button(g, &b);
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
