use crate::manager::WindowManager;
use crate::wmstate::ResizeEdge;
use antibox_core::backend::{
    DisplayBackend, EventMask, FontRole, FontSpec, SizeHints, WmWindowClass,
};
use antibox_core::point::Point;
use antibox_core::rect::Rect;

fn pad_x() -> i32 {
    antibox_ui::metrics::pad() + antibox_ui::metrics::gap()
}

fn pad_y() -> i32 {
    antibox_ui::metrics::gap() + antibox_core::scale::scaled(1)
}

fn font() -> FontSpec {
    FontSpec::role(FontRole::ToolTip, antibox_ui::metrics::font_pt())
}

pub fn readout(fr: Rect, edge: ResizeEdge, hints: Option<&SizeHints>) -> String {
    if edge == ResizeEdge::None {
        return format!("{}, {}", fr.x, fr.y);
    }
    use crate::frame::{border_width, bottom_border_width, title_block_height};
    let cw = (fr.w - border_width() * 2).max(1);
    let ch = (fr.h - title_block_height() - bottom_border_width()).max(1);
    if let Some(sh) = hints {
        use antibox_core::backend::hints::size_hints_flags::{P_BASE_SIZE, P_RESIZE_INC};
        if sh.flags & P_RESIZE_INC != 0 && sh.width_inc > 1 && sh.height_inc > 1 {
            let bw = if sh.flags & P_BASE_SIZE != 0 {
                sh.base_width
            } else {
                sh.min_width
            } as i32;
            let bh = if sh.flags & P_BASE_SIZE != 0 {
                sh.base_height
            } else {
                sh.min_height
            } as i32;
            let cols = (cw - bw).max(0) / sh.width_inc as i32;
            let rows = (ch - bh).max(0) / sh.height_inc as i32;
            return format!("{cols} x {rows}");
        }
    }
    format!("{cw} x {ch}")
}

pub fn show<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    text: &str,
    center: Point,
) {
    let Some(b) = wm.backend.clone() else { return };
    let Some((tw, th)) = antibox_ui::textmeasure::with_measure_context(&*b, |g| {
        let _ = g.set_font(&font());
        let (tw, th, _) = antibox_ui::textmeasure::measure(g, text, None);
        (tw as i32, th as i32)
    }) else { return };
    let w = (tw + pad_x() * 2).max(antibox_ui::metrics::text_w(5));
    let h = th + pad_y() * 2;
    let sw = b.screen_width() as i32;
    let sh = b.screen_height() as i32;
    let x = (center.x - w / 2).clamp(0, (sw - w).max(0));
    let y = (center.y - h / 2).clamp(0, (sh - h).max(0));

    if wm.moveresize_popup.is_none() {
        let Ok(win) = b.create_window(
            b.root().as_parent(),
            Rect::new(x, y, w, h),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE,
        ) else { return };
        let _ = win.map();
        wm.moveresize_popup = Some(win);
    }
    if let Some(win) = &wm.moveresize_popup {
        let _ = win.configure(Some(x), Some(y), Some(w as u16), Some(h as u16));
        let _ = win.raise();
        paint(&*b, win.id(), w as u16, h as u16, text);
        let _ = b.flush();
    }
}

fn paint<H: DisplayBackend + 'static + ?Sized>(b: &H, win: u32, w: u16, h: u16, text: &str) {
    let Ok(g) = b.create_graphics(win) else { return };
    let _ = g.set_font(&font());
    let bg = antibox_ui::theme::tooltip_bg();
    let fg = antibox_ui::theme::tooltip_fg();
    let _ = g.set_foreground(bg);
    let _ = g.fill_rect(0, 0, w, h);
    let _ = g.set_foreground(fg);
    let _ = g.draw_rect(0, 0, w.saturating_sub(1), h.saturating_sub(1));
    let _ = g.set_background(bg);
    antibox_ui::textmeasure::draw(&*g, pad_x() as i16, pad_y() as i16, text, None);
}

pub fn hide<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(win) = wm.moveresize_popup.take() {
        let _ = win.unmap();
        let _ = win.destroy();
        if let Some(b) = wm.backend() {
            let _ = b.flush();
        }
    }
}
