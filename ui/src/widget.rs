use crate::metrics;
use crate::theme;
use antibox_gfx::backend::{FontSpec, GraphicsContext};
use antibox_gfx::rect::Rect;
use std::borrow::Cow;

pub fn fit_label<'a>(g: &dyn GraphicsContext, text: &'a str, max_w: u16) -> Cow<'a, str> {
    let max_w = max_w as u32;
    if max_w == 0 {
        return Cow::Borrowed("");
    }
    if g.text_width(text).unwrap_or(0) <= max_w {
        return Cow::Borrowed(text);
    }
    let ell_w = g.text_width("\u{2026}").unwrap_or(0);
    if ell_w >= max_w {
        return Cow::Borrowed("");
    }
    let avail = max_w - ell_w;
    let mut out = String::new();
    for c in text.chars() {
        out.push(c);
        if g.text_width(&out).unwrap_or(0) > avail {
            out.pop();
            break;
        }
    }
    out.push('\u{2026}');
    Cow::Owned(out)
}

pub fn down_arrow(g: &dyn GraphicsContext, cx: i16, cy: i16, colour: antibox_gfx::colour::Colour) {
    let _ = g.set_foreground(colour);
    theme::arrow_glyph(g, cx, cy + 2, 7, theme::Arrow::Down, colour);
}

pub fn draw_cross(g: &dyn GraphicsContext, l: i16, t: i16, r: i16, b: i16, stroke: i16) {
    for d in 0..stroke {
        let _ = g.draw_line(l + d, t, r, b - d);
        let _ = g.draw_line(l, t + d, r - d, b);
        let _ = g.draw_line(l + d, b, r, t + d);
        let _ = g.draw_line(l, b - d, r - d, t);
    }
}

pub fn combo_button(
    g: &dyn GraphicsContext,
    field_x: i16,
    y: i16,
    field_w: u16,
    h: u16,
    pressed: bool,
) -> i16 {
    let bw = h;
    let bx = field_x + field_w as i16 - bw as i16 - 1;
    let by = y + 1;
    let bh = h.saturating_sub(2);
    let off = if theme::themed_combo_button(g, bx, by, bw, bh, pressed) {
        pressed as i16
    } else {
        let _ = g.set_foreground(theme::face());
        let _ = g.fill_rect(bx, by, bw, bh);
        theme::bevel(g, bx, by, bw, bh, pressed);
        pressed as i16
    };
    let cx = bx + bw as i16 / 2 + off;
    let cy = by + bh as i16 / 2 - 2 + off;
    down_arrow(g, cx, cy, theme::arrow_colour());
    bx
}

pub fn focus_rect(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16) {
    if w < 2 || h < 2 {
        return;
    }
    theme::dot_rect(g, x, y, w, h, theme::text());
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LabelAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
pub struct PanelButton<'a> {
    pub rect: Rect,
    pub face: antibox_gfx::colour::Colour,
    pub fg: antibox_gfx::colour::Colour,
    pub sunken: bool,
    pub font: &'a FontSpec,
    pub label: &'a str,
    pub progress: Option<(u8, u32)>,
    pub align: LabelAlign,
    pub on_bar: bool,
}

impl<'a> PanelButton<'a> {
    pub const fn new(
        rect: Rect,
        face: antibox_gfx::colour::Colour,
        fg: antibox_gfx::colour::Colour,
        font: &'a FontSpec,
        label: &'a str,
    ) -> Self {
        PanelButton {
            rect,
            face,
            fg,
            sunken: false,
            font,
            label,
            progress: None,
            align: LabelAlign::Center,
            on_bar: false,
        }
    }
}

pub fn panel_button(g: &dyn GraphicsContext, b: &PanelButton<'_>) {
    let (x, y, w, h) = b.rect.as_px();
    let PanelButton {
        face,
        fg,
        sunken,
        font,
        label,
        progress,
        align,
        on_bar,
        ..
    } = *b;
    let (fill, label_fg) = theme::press_colours(face, fg, sunken);
    if on_bar {
        theme::panel_button_surface(g, Rect::px(x, y, w, h), theme::Fill::new(fill, sunken));
    } else {
        theme::button_surface(g, Rect::px(x, y, w, h), theme::Fill::new(fill, sunken));
        theme::round_button_corners(g, x, y, w, h, theme::face());
    }
    if let Some((percent, colour)) = progress {
        let interior = w.saturating_sub(4);
        let fillw = (interior as u32 * percent.min(100) as u32 / 100) as u16;
        if fillw > 0 {
            let _ = g.set_foreground(colour);
            let _ = g.fill_rect(x + 2, y + 2, fillw, h.saturating_sub(4).max(1));
        }
    }
    let off = i16::from(sunken && fill == face);

    let inset = metrics::pad() as i16 + 2;
    let tx = x + inset + off;
    let _ = g.set_font(font);
    let _ = g.set_foreground(label_fg);
    let _ = g.set_background(fill);
    let baseline = metrics::baseline(y as i32, h as i32) as i16 + off;

    let transparent = theme::pressed_dither(sunken);
    let draw = |lx: i16, text: &str| {
        if transparent {
            let _ = g.set_foreground(label_fg);
            let _ = g.draw_text_transparent(lx, baseline, text);
        } else {
            theme::button_text(g, lx, baseline, fg, label_fg, text);
        }
    };
    let avail = (x + w as i16 - tx - inset).max(0) as u16;
    let text = fit_label(g, label, avail);
    let tw = g.text_width(&text).unwrap_or(0) as i16;
    let slack = (avail as i16 - tw).max(0);
    let lx = match align {
        LabelAlign::Left => tx,
        LabelAlign::Center => tx + slack / 2,
        LabelAlign::Right => tx + slack,
    };
    draw(lx, &text);
}

#[cfg(test)]
#[path = "widget_tests.rs"]
mod tests;
