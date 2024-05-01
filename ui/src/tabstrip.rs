use crate::metrics;
use crate::theme;
use crate::widget;
use antibox_gfx::backend::{FontSpec, GraphicsContext};
use antibox_gfx::rect::Rect;

fn tab_pad() -> i16 {
    (metrics::pad() * 2).max(6) as i16
}

fn tab_font(active: bool) -> FontSpec {
    FontSpec::role_styled(
        antibox_gfx::backend::FontRole::NormalTaskBar,
        metrics::font_pt(),
        active,
        false,
    )
}

pub fn close_extent(h: u16) -> i16 {
    (h as i16 * 3 / 5).max(antibox_gfx::scale::scaled(9) as i16)
}

pub fn close_rect(rect: (i16, i16, u16, u16)) -> (i16, i16, u16, u16) {
    let (x, y, w, h) = rect;
    let side = close_extent(h).min(h as i16) as u16;
    (
        x + w as i16 - side as i16 - tab_pad() / 2,
        y + (h as i16 - side as i16) / 2,
        side,
        side,
    )
}

pub fn tab_rects(labels: &[String], area: Rect) -> Vec<(i16, i16, u16, u16)> {
    let (x0, y, max_w, h) = area.as_px();
    let gap = metrics::item_gap() as i16;
    let n = labels.len() as i32;
    if n == 0 {
        return Vec::new();
    }
    let share = ((max_w as i32 - gap as i32 * (n - 1)) / n).max(1);
    let mut out = Vec::with_capacity(labels.len());
    let mut x = x0;
    for _ in labels {
        out.push((x, y, share as u16, h));
        x += share as i16 + gap;
    }
    out
}

fn draw_close(g: &dyn GraphicsContext, rect: (i16, i16, u16, u16), colour: antibox_gfx::colour::Colour) {
    let (cx, cy, cw, ch) = close_rect(rect);
    let s = antibox_gfx::scale::scaled(1).max(1) as i16;
    let inset = (cw as i16 / 4).max(s);
    let l = cx + inset;
    let t = cy + inset;
    let r = cx + cw as i16 - 1 - inset;
    let b = cy + ch as i16 - 1 - inset;
    let _ = g.set_foreground(colour);
    widget::draw_cross(g, l, t, r, b, s);
}

fn draw_one(g: &dyn GraphicsContext, rect: Rect, label: &str, active: bool, closable: bool) {
    let (x, y, w, h) = rect.as_px();
    let face = if active {
        theme::face()
    } else {
        antibox_gfx::colour::scale(theme::face(), 0.85)
    };
    let _ = g.set_foreground(face);
    let _ = g.fill_rect(x, y, w, h);
    theme::bevel(g, x, y, w, h, false);
    let fg = theme::text();
    let _ = g.set_font(&tab_font(active));
    let _ = g.set_foreground(fg);
    let _ = g.set_background(face);
    let extra = if closable { close_extent(h) } else { 0 };
    let avail = (w as i16 - tab_pad() - extra).max(0) as u16;
    let shown = widget::fit_label(g, label, avail);
    let tw_text = g.text_width(&shown).unwrap_or(0) as i16;
    let lx = x + ((w as i16 - extra - tw_text) / 2).max(0);
    let bl = metrics::baseline(y as i32, h as i32) as i16;
    let _ = g.draw_text_transparent(lx, bl, &shown);
    if closable {
        draw_close(g, (x, y, w, h), fg);
    }
}

pub fn draw_tabstrip(
    g: &dyn GraphicsContext,
    labels: &[String],
    active: usize,
    area: Rect,
) -> Vec<(i16, i16, u16, u16)> {
    let (x0, y, max_w, h) = area.as_px();
    let _ = g.set_foreground(theme::face());
    let _ = g.fill_rect(x0, y, max_w, h);
    let rects = tab_rects(labels, area);
    for (i, (&(tx, ty, tw, th), label)) in rects.iter().zip(labels).enumerate() {
        if tw == 0 {
            continue;
        }
        draw_one(g, Rect::px(tx, ty, tw, th), label, i == active, true);
    }
    rects
}
