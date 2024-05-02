use super::*;
use antibox_gfx::backend::GraphicsContext;
use antibox_gfx::point::Dimension;
use antibox_gfx::rect::Rect;


fn radius_px(w: u16, h: u16) -> i16 {
    let r = corner_radius_px() as i16;
    if r == 0 {
        return 0;
    }
    r.min((w.min(h) as i16 - 2) / 2).max(0)
}

fn button_radius_px_clamped(w: u16, h: u16) -> i16 {
    let r = button_radius_px() as i16;
    if r == 0 {
        return 0;
    }
    r.min((w.min(h) as i16 - 2) / 2).max(0)
}

pub fn rounded_region(w: u16, h: u16) -> Option<Vec<(i16, i16, u16, u16)>> {
    let r = frame_radius_px() as i32;
    let r = r.min(((w.min(h) as i32 - 2) / 2).max(0));
    if r == 0 {
        return None;
    }

    let (wi, hi) = (w as i32, h as i32);
    let mut out = Vec::with_capacity(r as usize + 1);
    for y in 0..r {
        let dy = (r - y) as f64 - 0.5;
        let reach = ((r as f64) * (r as f64) - dy * dy).max(0.0).sqrt();
        let inset = ((r as f64) - reach).ceil() as i32;
        let row_w = (wi - 2 * inset).max(1) as u16;
        out.push((inset as i16, y as i16, row_w, 1));
    }
    out.push((0, r as i16, w, (hi - r).max(1) as u16));
    Some(out)
}

fn rounded_stroke(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, r: i16) {
    let x1 = x + w as i16 - 1;
    let y1 = y + h as i16 - 1;
    let d = (r * 2) as u16;
    let _ = g.draw_line(x + r, y, x1 - r, y);
    let _ = g.draw_line(x + r, y1, x1 - r, y1);
    let _ = g.draw_line(x, y + r, x, y1 - r);
    let _ = g.draw_line(x1, y + r, x1, y1 - r);
    let _ = g.draw_arc(x, y, d, d, 90 * 64, 90 * 64);
    let _ = g.draw_arc(x1 - d as i16, y, d, d, 0, 90 * 64);
    let _ = g.draw_arc(x, y1 - d as i16, d, d, 180 * 64, 90 * 64);
    let _ = g.draw_arc(x1 - d as i16, y1 - d as i16, d, d, 270 * 64, 90 * 64);
}

fn rounded_fill(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, r: i16) {
    let x1 = x + w as i16 - 1;
    let y1 = y + h as i16 - 1;
    let d = (r * 2) as u16;
    let _ = g.fill_rect(x + r, y, w.saturating_sub(d), h);
    let _ = g.fill_rect(x, y + r, w, h.saturating_sub(d));
    let _ = g.fill_arc(x, y, d, d, 90 * 64, 90 * 64);
    let _ = g.fill_arc(x1 - d as i16, y, d, d, 0, 90 * 64);
    let _ = g.fill_arc(x, y1 - d as i16, d, d, 180 * 64, 90 * 64);
    let _ = g.fill_arc(x1 - d as i16, y1 - d as i16, d, d, 270 * 64, 90 * 64);
}

fn fill_corner_arcs(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    surround: Colour,
    r: i16,
) {
    let _ = g.set_foreground(surround);
    let x1 = x + w as i16;
    let y1 = y + h as i16;
    for row in 0..r {
        let dy = (r - row) as f64 - 0.5;
        let reach = ((r * r) as f64 - dy * dy).max(0.0).sqrt();
        let inset = ((r as f64) - reach).ceil() as i16;
        if inset <= 0 {
            continue;
        }
        let iw = inset as u16;
        let _ = g.fill_rect(x, y + row, iw, 1);
        let _ = g.fill_rect(x1 - inset, y + row, iw, 1);
        let _ = g.fill_rect(x, y1 - 1 - row, iw, 1);
        let _ = g.fill_rect(x1 - inset, y1 - 1 - row, iw, 1);
    }
}

pub fn round_button_corners(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    surround: Colour,
) {
    let r = button_radius_px_clamped(w, h);
    if r <= 0 {
        return;
    }
    fill_corner_arcs(g, x, y, w, h, surround, r);

    if button_radius_px() as i16 > radius_px(w, h) + antibox_gfx::scale::scaled(2) as i16 {
        let ring = metric_or("button_ring", 0);
        let _ = g.set_foreground(if ring != 0 { ring } else { dark() });
        rounded_stroke(g, x, y, w, h, r);
    }
}

pub fn round_title_button_corners(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    surround: Colour,
) {
    let r = radius_px(w, h);
    if r <= 0 {
        return;
    }
    fill_corner_arcs(g, x, y, w, h, surround, r);
}

pub const fn press_colours(face: Colour, fg: Colour, _sunken: bool) -> (u32, u32) {
    (face, fg)
}

pub fn disabled_colours(face: Colour, fg: Colour) -> (u32, u32) {
    (light(), mix_rgb(fg, face, 2.0 / 3.0))
}

pub fn button_text(g: &dyn GraphicsContext, x: i16, y: i16, fg: Colour, label: Colour, text: &str) {
    if fg == disabled() {
        etched_text(g, x, y, text);
        return;
    }
    let _ = g.set_foreground(label);
    let _ = g.draw_text(x, y, text);
}

pub fn fill_surface(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, colour: Colour) {
    let _ = g.set_foreground(colour);
    let r = radius_px(w, h);
    if r > 0 {
        rounded_fill(g, x, y, w, h, r);
    } else {
        let _ = g.fill_rect(x, y, w, h);
    }
}

pub fn fill_selection(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16) {
    let _ = g.set_foreground(sel_bg());
    let r = radius_px(w, h);
    if r > 0 {
        rounded_fill(g, x, y, w, h, r);
    } else {
        let _ = g.fill_rect(x, y, w, h);
    }
    selection_overlay(g, x, y, w, h);
}

pub fn selection_overlay(_g: &dyn GraphicsContext, _x: i16, _y: i16, _w: u16, _h: u16) {}

pub fn dot_rect(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, colour: Colour) {
    if w < 2 || h < 2 {
        return;
    }
    let _ = g.set_foreground(colour);
    let x1 = x + w as i16 - 1;
    let y1 = y + h as i16 - 1;
    let mut px = x;
    while px <= x1 {
        let _ = g.draw_point(px, y);
        let _ = g.draw_point(px, y1);
        px += 2;
    }
    let mut py = y;
    while py <= y1 {
        let _ = g.draw_point(x, py);
        let _ = g.draw_point(x1, py);
        py += 2;
    }
}

fn hairline(w: u16, h: u16) -> u16 {
    let s = antibox_gfx::scale::scaled(1).max(1) as u16;
    s.min(w / 4).min(h / 4).max(1)
}

fn bevel_depth(sunken: bool) -> i32 {
    let s = antibox_gfx::scale::scaled(1).max(1);
    if sunken {
        s * sunken_depth() as i32
    } else {
        s * 2
    }
}

pub fn bevel_inset(sunken: bool) -> i32 {
    bevel_depth(sunken).max(crate::metrics::pad())
}

pub fn field_inset() -> i32 {
    bevel_inset(true)
}

pub fn bevel(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, sunken: bool) {
    if w < 2 || h < 2 {
        return;
    }
    let s = hairline(w, h);
    let si = s as i16;
    let x1 = x + w as i16;
    let y1 = y + h as i16;
    let (outer_tl, outer_br, inner_tl, inner_br) = if sunken {
        (shadow(), light(), dark(), face())
    } else {
        (light(), dark(), face_light(), shadow())
    };
    let _ = g.set_foreground(outer_tl);
    let _ = g.fill_rect(x, y, w, s);
    let _ = g.fill_rect(x, y, s, h);
    let _ = g.set_foreground(outer_br);
    let _ = g.fill_rect(x, y1 - si, w, s);
    let _ = g.fill_rect(x1 - si, y, s, h);
    let _ = g.set_foreground(inner_tl);
    let _ = g.fill_rect(x + si, y + si, w.saturating_sub(2 * s), s);
    let _ = g.fill_rect(x + si, y + si, s, h.saturating_sub(2 * s));
    let _ = g.set_foreground(inner_br);
    let _ = g.fill_rect(x + si, y1 - si * 2, w.saturating_sub(2 * s), s);
    let _ = g.fill_rect(x1 - si * 2, y + si, s, h.saturating_sub(2 * s));
}



pub fn themed_title_button(
    _g: &dyn GraphicsContext,
    _key: &str,
    _r: Rect,
    _bg: Colour,
    _focused: bool,
    _sunken: bool,
) -> bool {
    false
}

pub fn themed_title_bar(
    _g: &dyn GraphicsContext,
    _x: i16,
    _y: i16,
    _w: u16,
    _h: u16,
    _bg: Colour,
    _focused: bool,
) -> bool {
    false
}


pub fn themed_combo_button(
    _g: &dyn GraphicsContext,
    _x: i16,
    _y: i16,
    _w: u16,
    _h: u16,
    _pressed: bool,
) -> bool {
    false
}

pub const fn pressed_dither(sunken: bool) -> bool {
    sunken
}

#[derive(Clone, Copy)]
pub struct Fill {
    pub colour: Colour,
    pub sunken: bool,
}

impl Fill {
    pub const fn new(colour: Colour, sunken: bool) -> Self {
        Self { colour, sunken }
    }

    pub const fn raised(colour: Colour) -> Self {
        Self {
            colour,
            sunken: false,
        }
    }
}

pub fn button_surface(g: &dyn GraphicsContext, r: Rect, style: Fill) {
    let (x, y, w, h) = r.as_px();
    let fill = style.colour;
    let sunken = style.sunken;
    if pressed_dither(sunken) {
        let _ = g.set_foreground(face());
        let _ = g.fill_rect(x, y, w, h);
        let _ = g.set_foreground(light());
        for ry in 0..h as i16 {
            let mut rx = (x + y + ry) & 1;
            while rx < w as i16 {
                let _ = g.draw_point(x + rx, y + ry);
                rx += 2;
            }
        }
        bevel(g, x, y, w, h, true);
        return;
    }
    fill_surface(g, x, y, w, h, fill);
    bevel(g, x, y, w, h, sunken);
}

pub fn title_glyph_bitmap(
    g: &dyn GraphicsContext,
    key: &str,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
) -> bool {
    let Some(rows) = title_glyph(key) else { return false };
    let s = hairline(w, h) as i16;
    let gsz = 10 * s;
    let ox = x + (w as i16 - gsz) / 2;
    let oy = y + (h as i16 - gsz) / 2;
    let blit = |rows: &[u16]| {
        for (ry, row) in rows.iter().enumerate() {
            for bx in 0..10 {
                if row >> bx & 1u16 == 1u16 {
                    let _ = g.fill_rect(ox + bx as i16 * s, oy + ry as i16 * s, s as u16, s as u16);
                }
            }
        }
    };
    blit(rows);

    if let Some(hi) = title_glyph(&format!("{key}_hi")) {
        let _ = g.set_foreground(face_light());
        blit(hi);
    }
    if let Some(sh) = title_glyph(&format!("{key}_sh")) {
        let _ = g.set_foreground(shadow());
        blit(sh);
    }
    true
}

pub fn title_button(
    g: &dyn GraphicsContext,
    r: Rect,
    bg: Colour,
    fg: Colour,
    _active: bool,
    sunken: bool,
) -> (u32, i16) {
    let (x, y, w, h) = r.as_px();
    let face = if sunken { scale_rgb(bg, 0.85) } else { bg };
    let _ = g.set_foreground(face);
    let _ = g.fill_rect(x, y, w, h);
    bevel(g, x, y, w, h, sunken);
    if sunken {
        (scale_rgb(fg, 0.5), 1)
    } else {
        (fg, 0)
    }
}

pub fn window_frame(
    g: &dyn GraphicsContext,
    top: i16,
    size: Dimension,
    border: Colour,
    _title: Colour,
    _title_h: u16,
    _focused: bool,
) {
    let (w, h) = size.as_px();
    let bh = h.saturating_sub(top.max(0) as u16);
    let _ = g.set_foreground(border);
    let _ = g.fill_rect(0, top, w, bh);
    if w < 2 || bh < 2 {
        let _ = g.set_foreground(border);
        let _ = g.draw_rect(0, top, w, bh);
        return;
    }
    let s = hairline(w, bh);
    let si = s as i16;
    let wi = w as i16;
    let y0 = top;
    let yb = top + bh as i16;

    let _ = g.set_foreground(light());
    let _ = g.fill_rect(si, y0 + si, w.saturating_sub(s * 2), s);
    let _ = g.fill_rect(si, y0 + si, s, bh.saturating_sub(s * 2));
    let _ = g.set_foreground(dark());
    let _ = g.fill_rect(wi - si, y0, s, bh);
    let _ = g.fill_rect(0, yb - si, w, s);
    let _ = g.set_foreground(shadow());
    let _ = g.fill_rect(wi - si * 2, y0 + si, s, bh.saturating_sub(s * 2));
    let _ = g.fill_rect(si, yb - si * 2, w.saturating_sub(s * 2), s);
}

pub fn title_stipple(_g: &dyn GraphicsContext, _x: i16, _y: i16, _w: u16, _h: u16, _base: u32) {}

pub fn panel_edge_height() -> u16 {
    antibox_gfx::scale::scaled(1).max(1) as u16 * 2
}

pub fn panel_edge(g: &dyn GraphicsContext, w: u16) {
    let s = antibox_gfx::scale::scaled(1).max(1) as u16;
    let _ = g.set_foreground(light());
    let _ = g.fill_rect(0, 0, w, s);
    let _ = g.set_foreground(face_light());
    let _ = g.fill_rect(0, s as i16, w, s);
}

pub fn panel_surface(g: &dyn GraphicsContext, w: u16, h: u16, fallback: Colour) {
    let _ = g.set_foreground(fallback);
    let _ = g.fill_rect(0, 0, w, h);
}

pub const fn tray_edge_width() -> u16 {
    0
}

pub fn tray_edge(_g: &dyn GraphicsContext, _x: i16, _y: i16, _h: u16) {}

pub fn pressed_face(face: Colour) -> u32 {
    tint_rgb(face, 0.5)
}

pub fn well(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16) {
    bevel(g, x, y, w, h, true);
}

pub fn raised(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16) {
    let _ = g.set_foreground(face());
    let _ = g.fill_rect(x, y, w, h);
    bevel(g, x, y, w, h, false);
}

pub fn sunken_field(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16) {
    let _ = g.set_foreground(field());
    let _ = g.fill_rect(x, y, w, h);
    bevel(g, x, y, w, h, true);
}

#[derive(Clone, Copy)]
pub enum Arrow {
    Up,
    Down,
    Left,
    Right,
}

pub fn arrow_glyph(
    g: &dyn GraphicsContext,
    cx: i16,
    cy: i16,
    size: i16,
    dir: Arrow,
    colour: Colour,
) {
    let _ = g.set_foreground(colour);
    let s = size / 2;
    for d in 0..size {
        let reach = (s - d / 2).max(0);
        match dir {
            Arrow::Up => {
                let ly = cy - s + (size - 1 - d);
                let _ = g.draw_line(cx - reach, ly, cx + reach, ly);
            }
            Arrow::Down => {
                let ly = cy - s + d;
                let _ = g.draw_line(cx - reach, ly, cx + reach, ly);
            }
            Arrow::Left => {
                let lx = cx - s + (size - 1 - d);
                let _ = g.draw_line(lx, cy - reach, lx, cy + reach);
            }
            Arrow::Right => {
                let lx = cx - s + d;
                let _ = g.draw_line(lx, cy - reach, lx, cy + reach);
            }
        }
    }
}

pub fn menu_border(g: &dyn GraphicsContext, w: u16, h: u16, _face: Colour) -> (u32, u32) {
    bevel(g, 0, 0, w, h, false);
    (light(), shadow())
}

pub fn menu_selection(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, sel_bg: Colour) {
    let _ = g.set_foreground(sel_bg);
    let _ = g.fill_rect(x, y, w, h);
}

pub fn menu_row_rule(_g: &dyn GraphicsContext, _x: i16, _y: i16, _w: u16) {}

pub fn submenu_indicator(g: &dyn GraphicsContext, x: i16, cy: i16, size: i16, colour: Colour) {
    if size <= 0 {
        return;
    }
    let a = (size / 2).max(1);
    let _ = g.set_foreground(colour);
    let mut prev: Option<(i16, i16)> = None;
    for dy in -a..=a {
        let half = (a as f64 * (1.0 - (dy as f64 / a as f64).abs())).round() as i16;
        let cur = (x + half, cy + dy);
        if let Some(p) = prev {
            let _ = g.draw_line(p.0, p.1, cur.0, cur.1);
        }
        prev = Some(cur);
    }
}

pub fn etched_text(g: &dyn GraphicsContext, x: i16, y: i16, text: &str) {
    let (hi, dim) = disabled_colours(face(), super::text());
    let _ = g.set_foreground(hi);
    let _ = g.draw_text(x + 1, y + 1, text);
    let _ = g.set_foreground(dim);
    let _ = g.draw_text(x, y, text);
}

#[cfg(test)]
#[path = "draw_tests.rs"]
mod tests;
