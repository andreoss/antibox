use super::*;
use antibox_gfx::backend::GraphicsContext;
use antibox_gfx::colour::tint as tint_rgb;
use antibox_gfx::point::{Dimension, Point};
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

pub fn press_colours(face: Colour, fg: Colour, sunken: bool) -> (u32, u32) {
    if flat() && sunken {
        (fg, face)
    } else {
        (face, fg)
    }
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



pub fn has_themed_title_button(key: &str) -> bool {
    element_ops(&["title_", key].concat()).is_some()
}

pub fn themed_title_button(
    g: &dyn GraphicsContext,
    key: &str,
    r: Rect,
    bg: Colour,
    focused: bool,
    sunken: bool,
) -> bool {
    let (x, y, w, h) = r.as_px();
    let active = ["title_", key].concat();
    let name = if focused {
        active
    } else {
        let inactive = [&active, "_inactive"].concat();
        if element_ops(&inactive).is_some() {
            inactive
        } else {
            active
        }
    };
    let pressed = [&name, "_pressed"].concat();
    let ops = if sunken {
        element_ops(&pressed).or_else(|| element_ops(&name))
    } else {
        element_ops(&name)
    };
    let Some(ops) = ops else { return false };
    let off = if sunken && element_ops(&pressed).is_none() {
        hairline(w, h) as i16
    } else {
        0
    };
    paint_element(g, &ops, x + off, y + off, w, h, bg);
    true
}

pub fn themed_title_bar(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    bg: Colour,
    focused: bool,
) -> bool {
    let key = if focused {
        "title_bar"
    } else {
        "title_bar_inactive"
    };
    let Some(ops) = element_ops(key) else {
        return false;
    };
    paint_element(g, &ops, x, y, w, h, bg);
    true
}


pub fn themed_combo_button(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    pressed: bool,
) -> bool {
    let key = if pressed {
        "combo_button_pressed"
    } else {
        "combo_button"
    };
    let Some(ops) = element_ops(key).or_else(|| element_ops("combo_button")) else {
        return false;
    };
    paint_element(g, &ops, x, y, w, h, face());
    true
}

pub fn pressed_dither(sunken: bool) -> bool {
    sunken && flag_or("scrollbar_dither", false) && !flat() && !grad_buttons()
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

fn grad_button_frame(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, bg: Colour, sunken: bool) {
    let key = if sunken { "button_pressed" } else { "button" };
    if let Some(ops) = element_ops(key) {
        paint_element(g, &ops, x, y, w, h, bg);
        return;
    }
    let (grad_top, grad_bottom) = if sunken {
        (scale_rgb(bg, 0.77), scale_rgb(bg, 1.3))
    } else {
        (scale_rgb(bg, 1.3), scale_rgb(bg, 0.77))
    };
    let s = hairline(w, h);
    let si = s as i16;
    let x1 = x + w as i16;
    let y1 = y + h as i16;
    let _ = g.fill_gradient_v(
        x + si,
        y + si,
        w.saturating_sub(s * 2),
        h.saturating_sub(s * 2),
        grad_top,
        grad_bottom,
    );
    let (bev_tl, bev_br) = if sunken {
        (scale_rgb(bg, 0.67), scale_rgb(bg, 1.5))
    } else {
        (scale_rgb(bg, 1.5), scale_rgb(bg, 0.67))
    };
    let _ = g.set_foreground(bev_tl);
    let _ = g.fill_rect(x + si, y + si, w.saturating_sub(s * 2), s);
    let _ = g.fill_rect(x + si, y + si, s, h.saturating_sub(s * 2));
    let _ = g.set_foreground(bev_br);
    let _ = g.fill_rect(x + si, y1 - si * 2, w.saturating_sub(s * 2), s);
    let _ = g.fill_rect(x1 - si * 2, y + si, s, h.saturating_sub(s * 2));
    let _ = g.set_foreground(scale_rgb(bg, 0.45));
    let _ = g.draw_rect(x, y, w.saturating_sub(s), h.saturating_sub(s));
}

fn taskbar_surround() -> u32 {
    if colour_or("taskbar_face", 0x0) != 0 {
        colour_or("taskbar_face", 0x0)
    } else {
        face()
    }
}

pub fn panel_button_surface(g: &dyn GraphicsContext, r: Rect, style: Fill) {
    let (x, y, w, h) = r.as_px();
    let key = if style.sunken {
        "panel_button_pressed"
    } else {
        "panel_button"
    };
    if let Some(ops) = element_ops(key) {
        paint_element(g, &ops, x, y, w, h, style.colour);
        round_button_corners(g, x, y, w, h, taskbar_surround());
        return;
    }
    button_surface(g, r, style);
}

pub fn themed_button() -> bool {
    element_ops("button").is_some() || grad_buttons()
}

pub fn themed_panel_button() -> bool {
    element_ops("panel_button").is_some() || grad_buttons()
}

pub fn themed_bar_button() -> bool {
    themed_panel_button() || themed_button()
}

pub fn has_start_button_element() -> bool {
    element_ops("start_button").is_some()
}

pub fn has_start_glyph() -> bool {
    element_ops("start_glyph").is_some()
}

pub fn themed_start_glyph(g: &dyn GraphicsContext, x: i16, y: i16, size: u16) -> bool {
    if let Some(ops) = element_ops("start_glyph") {
        paint_element(g, &ops, x, y, size, size, face());
        return true;
    }
    false
}

pub fn themed_start_button(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    pressed: bool,
) -> bool {
    let ops = if pressed {
        element_ops("start_button_pressed").or_else(|| element_ops("start_button"))
    } else {
        element_ops("start_button")
    };
    if let Some(ops) = ops {
        paint_element(g, &ops, x, y, w, h, face());
        if start_radius_base() == 0 {
            round_button_corners(g, x, y, w, h, taskbar_surround());
        }
        return true;
    }
    false
}

pub fn start_cap_slivers(w: u16, h: u16) -> Option<Vec<(i16, i16, u16, u16)>> {
    let sr = start_radius_base();
    if sr == 0 {
        return None;
    }
    let r = (antibox_gfx::scale::scaled(i32::from(sr)) as i16)
        .min(((w.min(h) as i16 - 2) / 2).max(0));
    if r <= 0 {
        return None;
    }
    let mut out = Vec::with_capacity(r as usize * 2);
    let x1 = w as i16;
    let y1 = h as i16;
    for row in 0..r {
        let dy = f64::from(r - row) - 0.5;
        let reach = (f64::from(r * r) - dy * dy).max(0.0).sqrt();
        let inset = (f64::from(r) - reach).ceil() as i16;
        if inset <= 0 {
            continue;
        }
        out.push((x1 - inset, row, inset as u16, 1));
        out.push((x1 - inset, y1 - 1 - row, inset as u16, 1));
    }
    Some(out)
}

pub fn button_surface(g: &dyn GraphicsContext, r: Rect, style: Fill) {
    let (x, y, w, h) = r.as_px();
    let fill = style.colour;
    let sunken = style.sunken;
    if grad_buttons() && w >= 6 && h >= 6 {
        grad_button_frame(g, x, y, w, h, fill, sunken);
        return;
    }
    if chrome_override() && w >= 4 && h >= 4 {
        let key = if sunken { "button_pressed" } else { "button" };
        if let Some(ops) = element_ops(key) {
            paint_element(g, &ops, x, y, w, h, fill);
            round_button_corners(g, x, y, w, h, face());
            return;
        }
    }
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
    blit(&rows);

    if let Some(hi) = title_glyph(&format!("{key}_hi")) {
        let _ = g.set_foreground(face_light());
        blit(&hi);
    }
    if let Some(sh) = title_glyph(&format!("{key}_sh")) {
        let _ = g.set_foreground(shadow());
        blit(&sh);
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
    if flat() {
        let glyph = if _active { fg } else { title_text_inactive() };
        let (fill, label) = press_colours(bg, glyph, sunken);
        fill_surface(g, x, y, w, h, fill);
        bevel(g, x, y, w, h, sunken);
        return (label, 0);
    }
    if sculpted() {
        let face = if sunken { scale_rgb(bg, 0.85) } else { bg };
        let _ = g.set_foreground(face);
        let _ = g.fill_rect(x, y, w, h);
        let s = hairline(w, h);
        let _ = g.set_foreground(scale_rgb(bg, 0.55));
        dotted_line(g, x, y, false, h);
        let _ = g.set_foreground(tint_rgb(bg, 0.6));
        dotted_line(g, x + s as i16, y, false, h);
        return (fg, i16::from(sunken));
    }
    if grad_buttons() && !equal_tabs() {
        button_surface(g, Rect::px(x, y, w, h), Fill::new(bg, sunken));
        return (fg, i16::from(sunken));
    }
    if grad_buttons() {
        let (top, bot) = if sunken {
            (scale_rgb(bg, 0.9), scale_rgb(bg, 1.06))
        } else {
            (scale_rgb(bg, 1.12), scale_rgb(bg, 0.95))
        };
        let _ = g.fill_gradient_v(x, y, w, h, top, bot);
        let s = hairline(w, h);
        let si = s as i16;
        let (hi, sh) = if sunken {
            (scale_rgb(bg, 0.62), scale_rgb(bg, 1.2))
        } else {
            (scale_rgb(bg, 1.25), scale_rgb(bg, 0.72))
        };
        let _ = g.set_foreground(hi);
        let _ = g.fill_rect(x + si, y + si, w.saturating_sub(s * 2), s);
        let _ = g.fill_rect(x + si, y + si, s, h.saturating_sub(s * 2));
        let _ = g.set_foreground(sh);
        let _ = g.fill_rect(x + si, y + h as i16 - si * 2, w.saturating_sub(s * 2), s);
        let _ = g.fill_rect(x + w as i16 - si * 2, y + si, s, h.saturating_sub(s * 2));
        let _ = g.set_foreground(scale_rgb(bg, 0.5));
        let _ = g.draw_rect(x, y, w, h);
        return (fg, i16::from(sunken));
    }
    if coloured_frame() {
        let _ = g.set_foreground(bg);
        let _ = g.fill_rect(x, y, w, h);
        let (hi, sh) = if sunken {
            (scale_rgb(bg, 0.5), tint_rgb(bg, 0.5))
        } else {
            (tint_rgb(bg, 0.5), scale_rgb(bg, 0.5))
        };
        let s = hairline(w, h);
        let si = s as i16;
        let _ = g.set_foreground(hi);
        let _ = g.fill_rect(x, y, w, s);
        let _ = g.fill_rect(x, y, s, h);
        let _ = g.set_foreground(sh);
        let _ = g.fill_rect(x + w as i16 - si, y, s, h);
        let _ = g.fill_rect(x, y + h as i16 - si, w, s);
        return (fg, i16::from(sunken));
    }
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
    title: Colour,
    title_h: u16,
    focused: bool,
) {
    let (w, h) = size.as_px();
    let bh = h.saturating_sub(top.max(0) as u16);
    if outlined() && w >= 4 && bh >= 4 {
        outlined_window_frame(g, top, Dimension::px(w, bh), border, title, title_h, focused);
        return;
    }
    if sculpted() && w >= 8 && bh >= 8 {
        sculpted_window_frame(g, top, w, bh, border, title_h, focused);
        return;
    }
    if tab_title() && w >= 10 && bh >= 10 {
        tab_window_frame(g, top, w, bh, border, title_h);
        return;
    }
    let _ = g.set_foreground(if flat() { face() } else { border });
    let _ = g.fill_rect(0, top, w, bh);
    if w < 2 || bh < 2 {
        let _ = g.set_foreground(border);
        let _ = g.draw_rect(0, top, w, bh);
        return;
    }
    if flat() {
        let _ = g.set_foreground(border);
        outline_rect(g, 0, top, w, bh);
        return;
    }
    let s = hairline(w, bh);
    let si = s as i16;
    let wi = w as i16;
    let y0 = top;
    let yb = top + bh as i16;

    if coloured_frame() {
        let hi = tint_rgb(border, 0.54);
        let sh = scale_rgb(border, 0.49);
        let bw = (antibox_gfx::scale::scaled(i32::from(border_base())) as i16).max(si * 3);
        let bb = (antibox_gfx::scale::scaled(i32::from(border_bottom_base())) as i16).max(si * 3);
        let _ = g.set_foreground(hi);
        let _ = g.fill_rect(0, y0, w, s * 2);
        let _ = g.fill_rect(0, y0, s * 2, bh);
        let _ = g.fill_rect(0, yb - bb, w.saturating_sub(s * 2), s);
        let _ = g.fill_rect(wi - bw + si, y0 + si * 2, s, bh.saturating_sub(s * 4));
        let _ = g.set_foreground(sh);
        let _ = g.fill_rect(si * 2, yb - si * 2, w.saturating_sub(s * 2), s * 2);
        let _ = g.fill_rect(wi - si * 2, y0 + si * 2, s * 2, bh.saturating_sub(s * 2));
        let _ = g.fill_rect(si * 2, y0 + bw - si * 2, w.saturating_sub(s * 4), s);
        let _ = g.fill_rect(bw - si * 2, y0 + si * 2, s, bh.saturating_sub(s * 4));
        let cl = antibox_gfx::scale::scaled(24).max(i32::from(bw) + 6) as i16;
        if wi > cl * 2 + 2 {
            for nx in [cl, wi - cl] {
                let _ = g.set_foreground(sh);
                let _ = g.fill_rect(nx, y0 + si, s, bw.saturating_sub(si) as u16);
                let _ = g.fill_rect(nx, yb - bb, s, bb.saturating_sub(si) as u16);
                let _ = g.set_foreground(hi);
                let _ = g.fill_rect(nx + si, y0 + si, s, bw.saturating_sub(si) as u16);
                let _ = g.fill_rect(nx + si, yb - bb, s, bb.saturating_sub(si) as u16);
            }
        }
        if bh as i16 > cl * 2 + 2 {
            for ny in [y0 + cl, yb - cl] {
                let _ = g.set_foreground(sh);
                let _ = g.fill_rect(si, ny, bw.saturating_sub(si) as u16, s);
                let _ = g.fill_rect(wi - bw + si, ny, bw.saturating_sub(si) as u16, s);
                let _ = g.set_foreground(hi);
                let _ = g.fill_rect(si, ny + si, bw.saturating_sub(si) as u16, s);
                let _ = g.fill_rect(wi - bw + si, ny + si, bw.saturating_sub(si) as u16, s);
            }
        }
        return;
    }
    if element_ops("frame_left").is_some() {
        element_window_frame(g, top, w, bh, focused);
        return;
    }
    if frame_well() {
        let ring2 = scale_rgb(face(), 0.29);
        let slab_sh = scale_rgb(face(), 0.73);
        let recess = scale_rgb(face(), 0.35);
        let rim_sh = scale_rgb(face(), 0.9);
        let _ = g.set_foreground(face());
        let _ = g.fill_rect(0, y0, w, bh);
        let _ = g.set_foreground(ring2);
        let _ = g.fill_rect(si, y0 + si, w.saturating_sub(s * 2), s);
        let _ = g.fill_rect(si, y0 + si, s, bh.saturating_sub(s * 2));
        let _ = g.fill_rect(wi - si * 2, y0 + si, s, bh.saturating_sub(s * 2));
        let _ = g.fill_rect(si, yb - si * 2, w.saturating_sub(s * 2), s);
        let side_y = y0 + si * 2;
        let side_h = bh.saturating_sub(s * 4);
        let left_cols = [
            (si * 2, light()),
            (si * 4, slab_sh),
            (si * 5, recess),
            (si * 6, face_light()),
        ];
        for (cx, c) in left_cols {
            let _ = g.set_foreground(c);
            let _ = g.fill_rect(cx, side_y, s, side_h);
        }
        let right_cols = [
            (wi - si * 3, slab_sh),
            (wi - si * 5, light()),
            (wi - si * 6, recess),
            (wi - si * 7, rim_sh),
        ];
        for (cx, c) in right_cols {
            let _ = g.set_foreground(c);
            let _ = g.fill_rect(cx, side_y, s, side_h);
        }
        let row_x = si * 2;
        let row_w = w.saturating_sub(s * 4);
        let bottom_rows = [
            (yb - si * 3, slab_sh),
            (yb - si * 5, light()),
            (yb - si * 6, recess),
        ];
        for (ry, c) in bottom_rows {
            let _ = g.set_foreground(c);
            let _ = g.fill_rect(row_x, ry, row_w, s);
        }
    } else {
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
    if menu_glyph() || frame_outline() {
        let _ = g.set_foreground(dark());
        outline_rect(g, 0, y0, w, bh);
    }
    if corner_grip() {
        let bb = (antibox_gfx::scale::scaled(i32::from(border_base())) as i16).max(si);
        let glen = (antibox_gfx::scale::scaled(14) as i16).max(bb * 3);
        if wi > glen + bb && bh as i16 > glen + bb {
            let _ = g.set_foreground(title_active());
            let _ = g.fill_rect(wi - glen, yb - bb, glen as u16, bb as u16);
            let _ = g.fill_rect(wi - bb, yb - glen, bb as u16, glen as u16);
            let _ = g.set_foreground(tint_rgb(title_active(), 0.4));
            let _ = g.fill_rect(wi - glen, yb - bb, glen as u16, s);
            let _ = g.fill_rect(wi - bb, yb - glen, s, glen as u16);
        }
    }
}

fn outline_rect(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16) {
    let r = radius_px(w, h);
    if r > 0 {
        rounded_stroke(g, x, y, w, h, r);
    } else {
        let _ = g.draw_rect(x, y, w.saturating_sub(1), h.saturating_sub(1));
    }
}

fn chan_add(c: Colour, dr: u32, dg: u32, db: u32) -> Colour {
    let r = ((c >> 16) & 0xFF) + dr;
    let g = ((c >> 8) & 0xFF) + dg;
    let b = (c & 0xFF) + db;
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

const fn chan_sub(c: Colour, d: u32) -> Colour {
    let r = ((c >> 16) & 0xFF).saturating_sub(d);
    let g = ((c >> 8) & 0xFF).saturating_sub(d);
    let b = (c & 0xFF).saturating_sub(d);
    (r << 16) | (g << 8) | b
}

pub fn title_tab(g: &dyn GraphicsContext, w: u16, h: u16, bg: Colour, border: Colour) {
    let _ = g.set_foreground(bg);
    let _ = g.fill_rect(0, 0, w, h);
    if w < 3 || h < 2 {
        return;
    }
    let s = hairline(w, h);
    let si = s as i16;
    let wi = w as i16;
    let _ = g.set_foreground(chan_add(bg, 52, 52, 80));
    let _ = g.fill_rect(si, si, w.saturating_sub(s * 2), s);
    let _ = g.fill_rect(si, si, s, h.saturating_sub(s));
    let _ = g.set_foreground(chan_sub(bg, 80));
    let _ = g.fill_rect(wi - si * 2, si * 2, s, h.saturating_sub(s * 2));
    let _ = g.set_foreground(chan_sub(border, 72));
    let _ = g.fill_rect(0, 0, w, s);
    let _ = g.fill_rect(0, 0, s, h);
    let _ = g.set_foreground(chan_sub(border, 128));
    let _ = g.fill_rect(wi - si, 0, s, h);
}

fn tab_window_frame(
    g: &dyn GraphicsContext,
    top: i16,
    w: u16,
    bh: u16,
    border: Colour,
    tab_w: u16,
) {
    let s = hairline(w, bh);
    let si = s as i16;
    let wi = w as i16;
    let y0 = top;
    let yb = top + bh as i16;
    let _ = g.set_foreground(border);
    let _ = g.fill_rect(0, y0, w, bh);
    let tl = [
        chan_sub(border, 72),
        chan_add(border, 64, 64, 64),
        chan_sub(border, 8),
        chan_sub(border, 88),
        chan_sub(border, 72),
    ];
    let br = [
        chan_sub(border, 128),
        chan_sub(border, 88),
        chan_sub(border, 8),
        chan_add(border, 64, 64, 64),
        chan_sub(border, 72),
    ];
    for (i, (&ct, &cb)) in tl.iter().zip(br.iter()).enumerate() {
        let o = i as i16 * si;
        let span = w.saturating_sub(2 * i as u16 * s);
        let vspan = bh.saturating_sub(2 * i as u16 * s);
        let _ = g.set_foreground(ct);
        let _ = g.fill_rect(o, y0 + o, span, s);
        let _ = g.fill_rect(o, y0 + o, s, vspan);
        let _ = g.set_foreground(cb);
        let _ = g.fill_rect(o, yb - o - si, span, s);
        let _ = g.fill_rect(wi - o - si, y0 + o, s, vspan);
    }
    if tab_w as i16 > si * 4 {
        let _ = g.set_foreground(tl[2]);
        let _ = g.fill_rect(si * 2, y0 + si, tab_w.saturating_sub(s * 4), s);
    }
    let bw = (antibox_gfx::scale::scaled(i32::from(border_base())) as i16).max(si);
    let cl = antibox_gfx::scale::scaled(22) as i16;
    if wi > cl * 3 && (yb - y0) > cl * 3 {
        let _ = g.set_foreground(tl[0]);
        let _ = g.fill_rect(wi - cl, yb - bw + si, s, (bw - si * 2).max(si) as u16);
        let _ = g.fill_rect(wi - bw + si, yb - cl, (bw - si * 2).max(si) as u16, s);
    }
}

pub fn dotted_line(g: &dyn GraphicsContext, x: i16, y: i16, hor: bool, len: u16) {
    let s = antibox_gfx::scale::scaled(1).max(1) as i16;
    let mut i = 0i16;
    while (i as u16) < len {
        let (px, py) = if hor { (x + i, y) } else { (x, y + i) };
        if ((px / s) + (py / s)) % 2 != 0 {
            let _ = g.fill_rect(px, py, s as u16, s as u16);
        }
        i += s;
    }
}

fn sculpted_window_frame(
    g: &dyn GraphicsContext,
    top: i16,
    w: u16,
    bh: u16,
    border: Colour,
    _title_h: u16,
    focused: bool,
) {
    let s = hairline(w, bh);
    let si = s as i16;
    let wi = w as i16;
    let y0 = top;
    let yb = top + bh as i16;
    let bw = (antibox_gfx::scale::scaled(i32::from(border_base())) as i16).max(si * 3);
    let bb = (antibox_gfx::scale::scaled(i32::from(border_bottom_base())) as i16).max(bw);
    let lt = tint_rgb(border, 0.6);
    let dk = scale_rgb(border, 0.55);

    let _ = g.set_foreground(border);
    let _ = g.fill_rect(0, y0, w, bh);

    let zone = |g: &dyn GraphicsContext,
                x: i16,
                y: i16,
                hor: bool,
                len: u16,
                off: i16,
                colour: Colour,
                solid_first: bool| {
        let _ = g.set_foreground(colour);
        if !focused {
            let o = if solid_first { off } else { off + si };
            if hor {
                let _ = g.fill_rect(x, y + o, len, s * 2);
            } else {
                let _ = g.fill_rect(x + o, y, s * 2, len);
            }
            return;
        }
        if solid_first {
            if hor {
                let _ = g.fill_rect(x, y + off, len, s);
                dotted_line(g, x, y + off + si, true, len);
            } else {
                let _ = g.fill_rect(x + off, y, s, len);
                dotted_line(g, x + off + si, y, false, len);
            }
            return;
        }
        for c in [off, off + si] {
            if hor {
                dotted_line(g, x, y + c, true, len);
            } else {
                dotted_line(g, x + c, y, false, len);
            }
        }
    };

    let inner_w = w.saturating_sub(s * 2);
    let inner_h = bh.saturating_sub(s * 2);
    zone(g, si, y0 + si, true, inner_w, 0, lt, true);
    zone(g, si, y0 + si, true, inner_w, bw - si * 4, dk, false);
    let _ = g.set_foreground(0x000000);
    let _ = g.fill_rect(bw, y0 + bw - si, w.saturating_sub(bw as u16 * 2), s);

    let _ = g.set_foreground(0x000000);
    let _ = g.fill_rect(si, yb - bb, inner_w, s);
    zone(g, si, yb - bb + si, true, inner_w, 0, lt, true);
    let _ = g.set_foreground(dk);
    dotted_line(g, si, yb - si * 3, true, inner_w);
    let _ = g.fill_rect(si, yb - si * 2, inner_w, s);

    zone(g, si, y0 + si, false, inner_h, 0, lt, true);
    let _ = g.set_foreground(lt);
    dotted_line(g, si * 3, y0 + si, false, inner_h);
    zone(g, si, y0 + si, false, inner_h, bw - si * 4, dk, false);
    let _ = g.set_foreground(dk);
    let _ = g.fill_rect(bw - si, y0 + si, s, inner_h);

    let _ = g.set_foreground(0x000000);
    let _ = g.fill_rect(wi - bw, y0 + si, s, inner_h);
    zone(g, wi - bw + si, y0 + si, false, inner_h, 0, lt, true);
    let _ = g.set_foreground(lt);
    dotted_line(g, wi - bw + si * 3, y0 + si, false, inner_h);
    zone(
        g,
        wi - bw + si,
        y0 + si,
        false,
        inner_h,
        bw - si * 4,
        dk,
        false,
    );

    let cl = antibox_gfx::scale::scaled(24) as i16;
    if wi > cl * 3 && (yb - y0) > cl * 3 {
        for (x, y, hor) in [
            (cl, y0 + si, true),
            (wi - cl - si * 4, y0 + si, true),
            (cl, yb - bb, true),
            (wi - cl - si * 4, yb - bb, true),
            (si, y0 + cl, false),
            (si, yb - cl - si * 4, false),
            (wi - bw, y0 + cl, false),
            (wi - bw, yb - cl - si * 4, false),
        ] {
            let (nw, nh, dx, dy) = if hor {
                let band = if y == y0 + si { bw } else { bb };
                (s * 2, (band - si) as u16, si * 2, 0)
            } else {
                ((bw - si) as u16, s * 2, 0, si * 2)
            };
            let _ = g.set_foreground(dk);
            let _ = g.fill_rect(x, y, nw, nh);
            let _ = g.set_foreground(lt);
            let _ = g.fill_rect(x + dx, y + dy, nw, nh);
        }
    }

    let _ = g.set_foreground(0x000000);
    let _ = g.fill_rect(0, y0, w, s);
    let _ = g.fill_rect(0, yb - si, w, s);
    let _ = g.fill_rect(0, y0, s, bh);
    let _ = g.fill_rect(wi - si, y0, s, bh);
}

fn element_window_frame(g: &dyn GraphicsContext, top: i16, w: u16, bh: u16, focused: bool) {
    let s = hairline(w, bh);
    let wi = w as i16;
    let y0 = top;
    let yb = top + bh as i16;
    let bw = antibox_gfx::scale::scaled(i32::from(border_base())).max(i32::from(s)) as u16;
    let bb = antibox_gfx::scale::scaled(i32::from(border_bottom_base())).max(i32::from(s)) as u16;
    let _ = g.set_foreground(face());
    let _ = g.fill_rect(0, y0, w, bh);
    let strips: [(&str, i16, i16, u16, u16); 4] = [
        ("frame_left", 0, y0, bw, bh),
        ("frame_right", wi - bw as i16, y0, bw, bh),
        ("frame_top", 0, y0, w, bw),
        ("frame_bottom", 0, yb - bb as i16, w, bb),
    ];
    for (key, sx, sy, sw, sh) in strips {
        let ops = if focused {
            element_ops(key)
        } else {
            element_ops(&[key, "_inactive"].concat()).or_else(|| element_ops(key))
        };
        if let Some(ops) = ops {
            paint_element(g, &ops, sx, sy, sw, sh, face());
        }
    }
}

fn outlined_window_frame(
    g: &dyn GraphicsContext,
    top: i16,
    size: Dimension,
    border: Colour,
    title: Colour,
    title_h: u16,
    focused: bool,
) {
    let (w, bh) = size.as_px();
    let s = hairline(w, bh);
    let si = s as i16;
    let wi = w as i16;
    let y0 = top;
    let yb = top + bh as i16;
    let bw = (antibox_gfx::scale::scaled(i32::from(border_base())) as i16).max(si * 2);
    let th = title_h as i16;
    let dk = scale_rgb(border, 0.5);
    let mid = scale_rgb(border, 0.83);

    let _ = g.set_foreground(border);
    let _ = g.fill_rect(0, y0, w, bh);

    let tail = antibox_gfx::scale::scaled(26) as i16;
    let lfs = if yb - y0 > bw * 2 + th + tail {
        y0 + bw + th + tail
    } else {
        y0 + bw + th
    };
    let _ = g.set_foreground(scale_rgb(title, 1.5));
    let _ = g.fill_rect(si, y0 + si, w.saturating_sub(s * 2), s);
    let _ = g.fill_rect(si, y0 + si, s, (lfs - y0 - si).max(0) as u16);
    let _ = g.set_foreground(scale_rgb(title, 0.83));
    let _ = g.fill_rect(si * 2, y0 + bw + th, s, (lfs - si - (y0 + bw + th)).max(0) as u16);
    let _ = g.set_foreground(light());
    let _ = g.fill_rect(si, lfs + si, s, (yb - si - lfs - si).max(0) as u16);
    let bb = (antibox_gfx::scale::scaled(i32::from(border_bottom_base())) as i16).max(bw);
    let _ = g.set_foreground(dk);
    let _ = g.fill_rect(wi - si * 2, y0 + si, s, (yb - bb - y0 - si).max(0) as u16);

    let cy = y0 + bw + th - si;
    let cw = w.saturating_sub((bw - si) as u16 * 2);
    let _ = g.set_foreground(dk);
    let _ = g.fill_rect(bw - si, cy, cw, s);
    let _ = g.fill_rect(bw - si, yb - bb, cw, s);
    let _ = g.fill_rect(bw - si, cy, s, (yb - bb - cy + si).max(0) as u16);
    let _ = g.fill_rect(wi - bw, cy, s, (yb - bb - cy + si).max(0) as u16);

    let gy = yb - bb + si;
    let gh = (bb - si * 2).max(si) as u16;
    let cl = antibox_gfx::scale::scaled(20) as i16;
    let sections: &[(i16, i16, bool)] = &if wi > cl * 4 {
        [
            (si, cl, false),
            (cl + si, wi - cl - si, true),
            (wi - cl, wi - si, false),
        ]
    } else {
        [(si, wi - si, true), (0, 0, false), (0, 0, false)]
    };
    for &(sx, ex, middle) in sections {
        let sw = (ex - sx).max(0) as u16;
        if sw == 0 {
            continue;
        }
        let fill = if middle && focused { border } else { mid };
        let _ = g.set_foreground(fill);
        let _ = g.fill_rect(sx, gy, sw, gh);
        let _ = g.set_foreground(light());
        let _ = g.fill_rect(sx, gy, sw, s);
        let _ = g.fill_rect(sx, gy, s, gh);
        let _ = g.set_foreground(dk);
        let _ = g.fill_rect(sx, gy + gh as i16 - si, sw, s);
        let _ = g.fill_rect(ex - si, gy, s, gh);
    }

    let _ = g.set_foreground(0x0000_0000);
    let _ = g.fill_rect(0, y0, w, s);
    let _ = g.fill_rect(0, y0, s, bh);
    let _ = g.fill_rect(wi - si, y0, s, bh);
    let _ = g.fill_rect(0, yb - si, w, s);
    let _ = g.fill_rect(si, lfs, s, s);
    let _ = g.fill_rect(si * 2, lfs - si, s, s);
}

pub fn title_stipple(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    base: u32,
    focused: bool,
) {
    let s = antibox_gfx::scale::scaled(1).max(1) as i16;
    if (w as i16) < s * 4 || (h as i16) < s * 6 {
        return;
    }
    if title_stripes() {
        let (hi_key, sh_key) = if focused {
            (title_stripe_hi(), title_stripe_sh())
        } else {
            (title_stripe_hi_inactive(), title_stripe_sh_inactive())
        };
        let lite = match hi_key {
            0 => tint_rgb(base, 0.9),
            c => c,
        };
        let dk = match sh_key {
            0 => scale_rgb(base, 0.5),
            c => c,
        };
        let mut yy = y + s * 2;
        let y1 = y + h as i16 - s * 2;
        let mut i = 0;
        while yy <= y1 {
            let _ = g.set_foreground(if i % 2 == 0 { lite } else { dk });
            let _ = g.fill_rect(x, yy, w, s as u16);
            yy += s;
            i += 1;
        }
        return;
    }
    if !title_stipple_enabled() {
        return;
    }
    let x1 = x + w as i16 - s * 2;
    let y1 = y + h as i16 - s * 2;
    for (colour, dx) in [(scale_rgb(base, 1.5), 0), (scale_rgb(base, 0.667), s)] {
        let _ = g.set_foreground(colour);
        let mut yy = y + s * 2;
        while yy <= y1 {
            let mut xx = x + dx;
            while xx <= x1 {
                let _ = g.fill_rect(xx, yy + dx, s as u16, s as u16);
                xx += s * 3;
            }
            yy += s * 4;
        }
    }
}

pub fn panel_edge_height() -> u16 {
    antibox_gfx::scale::scaled(1).max(1) as u16 * 2
}

pub fn panel_edge(g: &dyn GraphicsContext, w: u16) {
    let s = antibox_gfx::scale::scaled(1).max(1) as u16;
    if let Some(ops) = element_ops("panel_edge") {
        paint_element(g, &ops, 0, 0, w, s * 2, face());
        return;
    }
    let _ = g.set_foreground(light());
    let _ = g.fill_rect(0, 0, w, s);
    let _ = g.set_foreground(face_light());
    let _ = g.fill_rect(0, s as i16, w, s);
}

pub fn panel_edge_left(g: &dyn GraphicsContext, h: u16) {
    let s = antibox_gfx::scale::scaled(1).max(1) as u16;
    let _ = g.set_foreground(light());
    let _ = g.fill_rect(0, 0, s, h);
    let _ = g.set_foreground(face_light());
    let _ = g.fill_rect(s as i16, 0, s, h);
}

pub fn panel_edge_right(g: &dyn GraphicsContext, w: u16, h: u16) {
    let s = antibox_gfx::scale::scaled(1).max(1) as u16;
    let x = w.saturating_sub(s * 2) as i16;
    let _ = g.set_foreground(light());
    let _ = g.fill_rect(x, 0, s, h);
    let _ = g.set_foreground(face_light());
    let _ = g.fill_rect(x + s as i16, 0, s, h);
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
    if let Some(ops) = element_ops("well") {
        paint_element(g, &ops, x, y, w, h, face());
        return;
    }
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
    if let Some(ops) = element_ops("menu_sel") {
        paint_element(g, &ops, x, y, w, h, sel_bg);
        return;
    }
    let _ = g.set_foreground(sel_bg);
    let _ = g.fill_rect(x, y, w, h);
}

pub fn menu_row_rule(g: &dyn GraphicsContext, x: i16, y: i16, w: u16) {
    if let Some(ops) = element_ops("menu_rule") {
        let s = antibox_gfx::scale::scaled(1).max(1) as u16;
        paint_element(g, &ops, x, y, w, 2 * s, 0);
    }
}

pub fn submenu_indicator(g: &dyn GraphicsContext, x: i16, cy: i16, size: i16, colour: Colour) {
    if size <= 0 {
        return;
    }
    if let Some(ops) = element_ops("submenu_arrow") {
        let half = (size / 2).max(0) as u16;
        paint_element(g, &ops, x, cy - size / 2, half, size.max(0) as u16, colour);
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

const GLINT_BITS: [u8; 9] = [0, 1, 0, 1, 0, 1, 0, 1, 1];
const INNER_SHADOW_BITS: [u8; 100] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1,
    1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
    0, 0, 0, 0,
];
const OUTER_SHADOW_BITS: [u8; 100] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 0,
];
const BIG_INNER_SHADOW_BITS: [u8; 49] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0,
    1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
];
const BIG_OUTER_SHADOW_BITS: [u8; 49] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1,
    1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0,
];
const SMALL_INNER_SHADOW_BITS: [u8; 25] = [
    1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
const SMALL_OUTER_SHADOW_BITS: [u8; 25] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0,
];

struct Pattern<'a> {
    outer: &'a [u8],
    inner: Option<&'a [u8]>,
    c_outer: u32,
    c_inner: u32,
}

fn blit_pattern(
    g: &dyn GraphicsContext,
    at: Point,
    n: i16,
    s: i16,
    pat: &Pattern<'_>,
    rotated: bool,
) {
    let (x, y) = (at.x as i16, at.y as i16);
    for row in 0..n {
        for col in 0..n {
            let (pr, pc) = if rotated {
                (n - 1 - row, n - 1 - col)
            } else {
                (row, col)
            };
            let i = (pr * n + pc) as usize;
            let colour = if pat.outer[i] == 0 {
                pat.c_outer
            } else if pat.inner.is_some_and(|p| p[i] == 0) {
                pat.c_inner
            } else {
                continue;
            };
            let _ = g.set_foreground(colour);
            let _ = g.fill_rect(x + col * s, y + row * s, s as u16, s as u16);
        }
    }
}

fn bevel_ring(g: &dyn GraphicsContext, r: Rect, s: u16, light: Colour, shadow: Colour, down: bool) {
    let (x, y, w, h) = r.as_px();
    let si = s as i16;
    let x1 = x + w as i16 - si;
    let y1 = y + h as i16 - si;
    if down {
        let _ = g.set_foreground(shadow);
        let _ = g.fill_rect(x, y, w, s);
        let _ = g.fill_rect(x, y, s, h);
        let _ = g.fill_rect(x + si, y + si, w.saturating_sub(s), s);
        let _ = g.fill_rect(x + si, y + si, s, h.saturating_sub(s));
        let _ = g.set_foreground(light);
        let _ = g.fill_rect(x, y1, w, s);
        let _ = g.fill_rect(x1, y, s, h);
        let _ = g.fill_rect(x + si, y1 - si, w.saturating_sub(s), s);
        let _ = g.fill_rect(x1 - si, y + si, s, h.saturating_sub(s));
        return;
    }
    let _ = g.set_foreground(shadow);
    let _ = g.fill_rect(x, y, w.saturating_sub(s), s);
    let _ = g.fill_rect(x, y, s, h.saturating_sub(s));
    let _ = g.fill_rect(x, y1 - si, w.saturating_sub(s), s);
    let _ = g.fill_rect(x1 - si, y, s, h.saturating_sub(s));
    let _ = g.fill_rect(x, y, w, s);
    let _ = g.fill_rect(x, y, s, h);
    let _ = g.set_foreground(light);
    let _ = g.fill_rect(x + si, y + si, w.saturating_sub(s), s);
    let _ = g.fill_rect(x + si, y + si, s, h.saturating_sub(s));
    let _ = g.fill_rect(x + si, y1, w.saturating_sub(s), s);
    let _ = g.fill_rect(x1, y + si, s, h.saturating_sub(s));
}

pub fn sculpted_title_plate(
    g: &dyn GraphicsContext,
    r: Rect,
    bg: Colour,
    sunken: bool,
    right: bool,
) {
    let (x, y, w, h) = r.as_px();
    let face = if sunken { scale_rgb(bg, 0.85) } else { bg };
    let _ = g.set_foreground(face);
    let _ = g.fill_rect(x, y, w, h);
    let s = hairline(w, h);
    let kx = if right {
        x + w as i16 - s as i16 * 2
    } else {
        x
    };
    let _ = g.set_foreground(scale_rgb(bg, 0.55));
    dotted_line(g, kx, y, false, h);
    let _ = g.set_foreground(tint_rgb(bg, 0.6));
    dotted_line(g, kx + s as i16, y, false, h);
}

pub fn tab_button(g: &dyn GraphicsContext, r: Rect, bg: Colour, zoom: bool, sunken: bool) {
    let (x, y, w, h) = r.as_px();
    let side = w.min(h);
    let x = x + (w as i16 - side as i16) / 2;
    let y = y + (h as i16 - side as i16) / 2;
    let w = side;
    let h = side;
    let s = hairline(w, h);
    let si = s as i16;
    let grey = {
        let (r, gc, b) = ((bg >> 16) & 0xFF, (bg >> 8) & 0xFF, bg & 0xFF);
        r == gc && gc == b
    };
    let light1 = chan_add(bg, 33, 33, 33);
    let light2 = chan_add(bg, 52, 52, if grey { 52 } else { 63 });
    let shadow1 = chan_sub(bg, 21);
    let _ = g.set_foreground(bg);
    let _ = g.fill_rect(x, y, w, h);
    if !zoom {
        let shadow2 = chan_sub(bg, 72);
        bevel_ring(g, Rect::px(x, y, w, h), s, light2, shadow2, sunken);
        let fx = x + si * 2;
        let fy = y + si * 2;
        let fw = w.saturating_sub(s * 4);
        let _ = g.set_foreground(light1);
        let _ = g.fill_rect(fx, fy, fw, fw);
        let fi = fw as i16;
        if sunken {
            blit_pattern(
                g,
                Point::new((fx + fi - si * 3) as i32, (fy + fi - si * 3) as i32),
                3,
                si,
                &Pattern {
                    outer: &GLINT_BITS,
                    inner: None,
                    c_outer: light2,
                    c_inner: 0,
                },
                true,
            );
            blit_pattern(
                g,
                Point::new(fx as i32, fy as i32),
                10,
                si,
                &Pattern {
                    outer: &OUTER_SHADOW_BITS,
                    inner: Some(&INNER_SHADOW_BITS),
                    c_outer: shadow1,
                    c_inner: bg,
                },
                true,
            );
        } else {
            blit_pattern(
                g,
                Point::new(fx as i32, fy as i32),
                3,
                si,
                &Pattern {
                    outer: &GLINT_BITS,
                    inner: None,
                    c_outer: light2,
                    c_inner: 0,
                },
                false,
            );
            blit_pattern(
                g,
                Point::new((fx + fi - si * 10) as i32, (fy + fi - si * 10) as i32),
                10,
                si,
                &Pattern {
                    outer: &OUTER_SHADOW_BITS,
                    inner: Some(&INNER_SHADOW_BITS),
                    c_outer: shadow1,
                    c_inner: bg,
                },
                false,
            );
        }
        return;
    }
    let shadow2 = chan_sub(bg, 45);
    let off3 = ((w as i32 * 3) / 14) as i16;
    let off5 = ((w as i32 * 5) / 14) as u16;
    let bx = x + off3;
    let by = y + off3;
    let bw = w.saturating_sub(off3 as u16);
    bevel_ring(g, Rect::px(bx, by, bw, bw), s, light2, shadow2, sunken);
    let bfx = bx + si * 2;
    let bfy = by + si * 2;
    let bfw = bw.saturating_sub(s * 4);
    let _ = g.set_foreground(light1);
    let _ = g.fill_rect(bfx, bfy, bfw, bfw);
    let bfi = bfw as i16;
    if sunken {
        blit_pattern(
            g,
            Point::new((bfx + bfi - si * 3) as i32, (bfy + bfi - si * 3) as i32),
            3,
            si,
            &Pattern {
                outer: &GLINT_BITS,
                inner: None,
                c_outer: light2,
                c_inner: 0,
            },
            true,
        );
    } else {
        blit_pattern(
            g,
            Point::new((bfx + bfi - si * 7) as i32, (bfy + bfi - si * 7) as i32),
            7,
            si,
            &Pattern {
                outer: &BIG_OUTER_SHADOW_BITS,
                inner: Some(&BIG_INNER_SHADOW_BITS),
                c_outer: shadow1,
                c_inner: bg,
            },
            false,
        );
    }
    let sw = w.saturating_sub(off5);
    bevel_ring(g, Rect::px(x, y, sw, sw), s, light2, shadow2, sunken);
    if !sunken {
        let _ = g.set_foreground(bg);
        let swi = sw as i16;
        let _ = g.fill_rect(x, y + swi - si, s, s);
        let _ = g.fill_rect(x + swi - si, y, s, s);
    }
    let sfx = x + si * 2;
    let sfy = y + si * 2;
    let sfw = sw.saturating_sub(s * 4);
    let _ = g.set_foreground(light1);
    let _ = g.fill_rect(sfx, sfy, sfw, sfw);
    let sfi = sfw as i16;
    if sunken {
        blit_pattern(
            g,
            Point::new(sfx as i32, sfy as i32),
            5,
            si,
            &Pattern {
                outer: &SMALL_OUTER_SHADOW_BITS,
                inner: Some(&SMALL_INNER_SHADOW_BITS),
                c_outer: shadow1,
                c_inner: bg,
            },
            true,
        );
    } else {
        blit_pattern(
            g,
            Point::new((sfx + sfi - si * 5) as i32, (sfy + sfi - si * 5) as i32),
            5,
            si,
            &Pattern {
                outer: &SMALL_OUTER_SHADOW_BITS,
                inner: Some(&SMALL_INNER_SHADOW_BITS),
                c_outer: shadow1,
                c_inner: bg,
            },
            false,
        );
    }
}
