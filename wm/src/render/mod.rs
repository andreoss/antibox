mod menu;
pub use self::menu::{
    draw_menu_border, draw_menu_frame, draw_menu_row_rule, draw_menu_separator, draw_text_mnemonic,
    draw_text_underline, fill_menu_selection, hot_char_at, menu_clamp_pos, menu_content_width,
    menu_destroy_window, menu_hot_match, menu_item_at, mnemonic_key, parse_mnemonic,
    toward_submenu,
};

use crate::frame::{title_bar_height, FrameWindow};
use antibox_core::backend::{FontSpec, GraphicsContext};
use antibox_core::colour::RgbColour;
use antibox_core::rect::Rect;

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub active_title_top: u32,
    pub active_title_bottom: u32,
    pub inactive_title_top: antibox_core::colour::Colour,
    pub inactive_title_bottom: antibox_core::colour::Colour,
    pub active_text: u32,
    pub inactive_text: antibox_core::colour::Colour,
    pub border_active: u32,
    pub border_inactive: u32,
    pub title_line: antibox_core::colour::Colour,
    pub button_bg: u32,
    pub button_fg: u32,
    pub task_bar_colour: antibox_core::colour::Colour,
    pub menu_bg: antibox_core::colour::Colour,
    pub workspace_active_bg: antibox_core::colour::Colour,
    pub workspace_active_fg: antibox_core::colour::Colour,
    pub workspace_normal_bg: antibox_core::colour::Colour,
    pub workspace_normal_fg: antibox_core::colour::Colour,
    pub urgent_bg: antibox_core::colour::Colour,
    pub urgent_fg: antibox_core::colour::Colour,
    pub battery_bg: u32,
    pub battery_fg: u32,
    pub battery_line: u32,
}

pub(crate) fn parse_colour(s: &str, default: &str) -> u32 {
    s.parse::<RgbColour>()
        .ok()
        .or_else(|| default.parse::<RgbColour>().ok())
        .map_or(0x808080, |c| c.to_pixel())
}

impl Default for ThemeColors {
    fn default() -> ThemeColors {
        let active_top = parse_colour("rgb:00/00/80", "rgb:00/00/80");
        let active_bottom = active_top;
        let inactive_top = parse_colour("rgb:80/80/80", "rgb:80/80/80");
        let inactive_bottom = inactive_top;
        let active_text = parse_colour("rgb:FF/FF/FF", "rgb:FF/FF/FF");
        let inactive_text = parse_colour("rgb:C0/C0/C0", "rgb:C0/C0/C0");
        let border_active = parse_colour("rgb:C0/C0/C0", "rgb:C0/C0/C0");
        let border_inactive = parse_colour("rgb:C0/C0/C0", "rgb:C0/C0/C0");
        let button_bg = parse_colour("rgb:C0/C0/C0", "rgb:C0/C0/C0");
        let button_fg = parse_colour("rgb:00/00/00", "rgb:00/00/00");
        let task_bar_colour = parse_colour("rgb:C0/C0/C0", "rgb:C0/C0/C0");
        let ws_active_bg = parse_colour("rgb:00/00/80", "rgb:00/00/80");
        let ws_active_fg = parse_colour("rgb:FF/FF/FF", "rgb:FF/FF/FF");
        let ws_normal_bg = parse_colour("rgb:C0/C0/C0", "rgb:C0/C0/C0");
        let ws_normal_fg = parse_colour("rgb:00/00/00", "rgb:00/00/00");
        let urg_bg = parse_colour("rgb:C0/30/30", "rgb:C0/30/30");
        let urg_fg = parse_colour("rgb:FF/FF/FF", "rgb:FF/FF/FF");
        let bat_bg = parse_colour("rgb:FF/FF/00", "rgb:FF/FF/00");
        let bat_fg = parse_colour("rgb:00/FF/00", "rgb:00/FF/00");
        let bat_line = parse_colour("rgb:00/FF/00", "rgb:00/FF/00");
        ThemeColors {
            active_title_top: active_top,
            active_title_bottom: active_bottom,
            inactive_title_top: inactive_top,
            inactive_title_bottom: inactive_bottom,
            active_text,
            inactive_text,
            border_active,
            border_inactive,
            title_line: task_bar_colour,
            button_bg,
            button_fg,
            task_bar_colour,
            menu_bg: task_bar_colour,
            workspace_active_bg: ws_active_bg,
            workspace_active_fg: ws_active_fg,
            workspace_normal_bg: ws_normal_bg,
            workspace_normal_fg: ws_normal_fg,
            urgent_bg: urg_bg,
            urgent_fg: urg_fg,
            battery_bg: bat_bg,
            battery_fg: bat_fg,
            battery_line: bat_line,
        }
    }
}

pub fn bevel_light(face: antibox_core::colour::Colour) -> u32 {
    brighten_colour(face, 0.6)
}

pub fn draw_submenu_arrow(
    g: &dyn GraphicsContext,
    x: i16,
    cy: i16,
    size: i16,
    colour: antibox_core::colour::Colour,
) {
    antibox_ui::theme::submenu_indicator(g, x, cy, size, colour);
}

pub fn draw_button_bevel(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    face: antibox_core::colour::Colour,
    sunken: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if w < 2 || h < 2 {
        return Ok(());
    }
    let white = brighten_colour(face, 0.9);
    let light = brighten_colour(face, 0.4);
    let shadow = darken_colour(face, 0.5);
    let black = darken_colour(face, 0.8);
    let (tl_out, tl_in, br_in, br_out) = if sunken {
        (shadow, black, light, white)
    } else {
        (white, light, shadow, black)
    };
    let x1 = x + w as i16 - 1;
    let y1 = y + h as i16 - 1;
    g.set_foreground(tl_out)?;
    g.draw_line(x, y, x1, y)?;
    g.draw_line(x, y, x, y1)?;
    g.set_foreground(br_out)?;
    g.draw_line(x, y1, x1, y1)?;
    g.draw_line(x1, y, x1, y1)?;
    g.set_foreground(tl_in)?;
    g.draw_line(x + 1, y + 1, x1 - 1, y + 1)?;
    g.draw_line(x + 1, y + 1, x + 1, y1 - 1)?;
    g.set_foreground(br_in)?;
    g.draw_line(x + 1, y1 - 1, x1 - 1, y1 - 1)?;
    g.draw_line(x1 - 1, y + 1, x1 - 1, y1 - 1)?;
    Ok(())
}

pub(crate) use antibox_core::colour::is_dark;

pub(crate) fn darken_colour(
    c: antibox_core::colour::Colour,
    factor: f32,
) -> antibox_core::colour::Colour {
    antibox_core::colour::scale(c, factor)
}

pub(crate) fn shift_colour(
    c: antibox_core::colour::Colour,
    d: i32,
) -> antibox_core::colour::Colour {
    antibox_core::colour::shift(c, d)
}

pub(crate) fn brighten_colour(
    c: antibox_core::colour::Colour,
    factor: f32,
) -> antibox_core::colour::Colour {
    antibox_core::colour::tint(c, factor)
}

pub fn draw_frame(
    fw: &FrameWindow,
    g: &dyn GraphicsContext,
    focused: bool,
    colours: &ThemeColors,
    gradients_enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !fw.decorated() {
        return Ok(());
    }
    if fw.title_vertical() {
        return draw_frame_left_title(fw, g, focused, colours);
    }
    if fw.title_on_bottom() {
        return draw_frame_bottom_title(fw, g, focused, colours);
    }
    let fw_w = fw.frame_rect().w as u16;
    let fw_h = fw.frame_rect().h as u16;
    let bw = fw.effective_border();
    let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
    if bw > 0 {
        draw_border(g, focused, 0, fw_w, fw_h, colours)?;
    }
    draw_title_bar(
        g,
        focused,
        fw.state().urgent,
        TitleBarDims { fw_w, bw },
        colours,
        gradients_enabled,
    )?;
    draw_buttons(fw, g, focused, colours)?;
    let (text_x, title_right) = fw.title_text_span();
    draw_title_text(fw, g, focused, colours, text_x as i16, title_right as i16)?;
    Ok(())
}

#[derive(Clone, Copy)]
struct TitleBarDims {
    fw_w: u16,
    bw: i32,
}

fn draw_title_bar(
    g: &dyn GraphicsContext,
    focused: bool,
    urgent: bool,
    dims: TitleBarDims,
    colours: &ThemeColors,
    gradients_enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let TitleBarDims { fw_w, bw, .. } = dims;
    let bar_h = title_bar_height() as u16;
    let (bx, by, bar_w, grad_h) = if (crate::frame::title_top_inset() > 0
        || antibox_ui::theme::title_overlap_base() > 0)
        && bw > 0
    {
        let ti = crate::frame::title_side_inset();
        (
            ti as i16,
            ti as i16,
            fw_w.saturating_sub(ti as u16 * 2),
            bar_h,
        )
    } else {
        (
            bw as i16,
            bw as i16,
            fw_w.saturating_sub(bw as u16 * 2),
            bar_h,
        )
    };
    let (left, right) = if urgent && !focused {
        (colours.urgent_bg, darken_colour(colours.urgent_bg, 0.33))
    } else if focused {
        (colours.active_title_top, colours.active_title_bottom)
    } else {
        (colours.inactive_title_top, colours.inactive_title_bottom)
    };
    if gradients_enabled && left != right {
        g.fill_gradient_h(bx, by, bar_w, grad_h, left, right)?;
    } else {
        g.set_foreground(left)?;
        g.fill_rect(bx, by, bar_w, grad_h)?;
    }
    let _ = antibox_ui::theme::themed_title_bar(g, bx, by, bar_w, grad_h, left, focused);
    Ok(())
}

fn draw_menu_button(
    g: &dyn GraphicsContext,
    focused: bool,
    colours: &ThemeColors,
    rect: &Rect,
    sunken: bool,
) {
    if antibox_ui::theme::title_glyph("menu").is_some() {
        let (x, y, w, h) = (rect.x as i16, rect.y as i16, rect.w as u16, rect.h as u16);
        let (fg, off) = antibox_ui::theme::title_button(
            g,
            Rect::px(x, y, w, h),
            colours.button_bg,
            colours.button_fg,
            focused,
            sunken,
        );
        let surround = if focused {
            colours.active_title_top
        } else {
            colours.inactive_title_top
        };
        antibox_ui::theme::round_title_button_corners(g, x, y, w, h, surround);
        let _ = g.set_foreground(fg);
        let _ = antibox_ui::theme::title_glyph_bitmap(g, "menu", x + off, y + off, w, h);
        return;
    }
    let (x, y, w, h) = (rect.x as i16, rect.y as i16, rect.w as u16, rect.h as u16);
    let fg = if focused {
        colours.active_text
    } else {
        colours.inactive_text
    };
    g.set_foreground(fg).ok();
    let off = i16::from(sunken);
    let _ = draw_button_glyph(g, "menu", x + off, y + off, w, h);
}

fn draw_title_text(
    fw: &FrameWindow,
    g: &dyn GraphicsContext,
    focused: bool,
    colours: &ThemeColors,
    text_x: i16,
    text_right: i16,
) -> Result<(), Box<dyn std::error::Error>> {
    let colour = if focused {
        colours.active_text
    } else {
        colours.inactive_text
    };
    let (top, bottom) = if focused {
        (colours.active_title_top, colours.active_title_bottom)
    } else {
        (colours.inactive_title_top, colours.inactive_title_bottom)
    };
    g.set_foreground(colour)?;
    let _ = g.set_background(antibox_core::backend::blend_colour(top, bottom, 0.5));
    let _ = g.set_font(&FontSpec::role_styled(
        antibox_core::backend::FontRole::Title,
        antibox_ui::metrics::font_pt(),
        true,
        false,
    ));
    let avail = (text_right - text_x - 4).max(0) as u16;
    let bar_top = if fw.effective_border() > 0 {
        crate::frame::title_side_inset()
    } else {
        0
    };
    let bar_baseline = antibox_ui::metrics::baseline(0, title_bar_height());
    let baseline = bar_top + bar_baseline;

    let text_avail = avail;
    let title = crate::applet::fit_label(g, fw.client().title(), text_avail);
    let tw = g.text_width(&title).unwrap_or(0) as i32;
    let slack = (avail as i32 - tw).max(0);
    let justify = crate::layout_preferences::title_justify() as i32;
    let block_x = text_x + (slack * justify / 100) as i16;
    let tx = block_x;
    g.draw_text_transparent(tx, baseline as i16, &title)?;
    if focused && !fw.state().urgent {
        let gap = antibox_core::scale::scaled(3) as i16;
        let s = antibox_core::scale::scaled(1).max(1);
        let sx = tx + tw as i16 + gap;
        let sw = (text_right - gap - sx).max(0) as u16;
        let (sy, sh) = if crate::frame::title_top_inset() > 0 && fw.effective_border() > 0 {
            (bar_top as i16, (title_bar_height() - s * 5).max(1) as u16)
        } else {
            (fw.effective_border() as i16, title_bar_height() as u16 - 1)
        };
        antibox_ui::theme::title_stipple(g, sx, sy, sw, sh, colours.active_title_top);
        let lsw = (block_x - gap - text_x).max(0) as u16;
        antibox_ui::theme::title_stipple(g, text_x, sy, lsw, sh, colours.active_title_top);
    }
    Ok(())
}

fn draw_frame_left_title(
    fw: &FrameWindow,
    g: &dyn GraphicsContext,
    focused: bool,
    colours: &ThemeColors,
) -> Result<(), Box<dyn std::error::Error>> {
    let fw_w = fw.frame_rect().w as u16;
    let fw_h = fw.frame_rect().h as u16;
    let bw = fw.effective_border();
    let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
    if bw > 0 {
        draw_border(g, focused, 0, fw_w, fw_h, colours)?;
    }
    let bandr = fw.band_rect();
    let band = bandr.w;
    let band_x = bandr.x as i16;
    let band_y = bandr.y as i16;
    let band_h = bandr.h.max(1) as u16;
    let (fill, urgent) = if fw.state().urgent && !focused {
        (colours.urgent_bg, true)
    } else if focused {
        (colours.active_title_top, false)
    } else {
        (colours.inactive_title_top, false)
    };
    let _ = g.set_foreground(fill);
    let _ = g.fill_rect(band_x, band_y, band as u16, band_h);
    let s = antibox_core::scale::scaled(1).max(1) as u16;
    let _ = g.set_foreground(shift_colour(fill, 40));
    let _ = g.fill_rect(band_x, band_y, s, band_h);
    let _ = g.set_foreground(shift_colour(fill, -40));
    let _ = g.fill_rect(band_x + band as i16 - s as i16, band_y, s, band_h);

    draw_buttons(fw, g, focused, colours)?;

    let (text_start, text_end) = fw.title_text_span();
    let avail = (text_end - text_start).max(0);
    if avail > antibox_core::scale::scaled(8) {
        let colour = if urgent {
            colours.urgent_fg
        } else if focused {
            colours.active_text
        } else {
            colours.inactive_text
        };
        let _ = g.set_font(&FontSpec::role_styled(
            antibox_core::backend::FontRole::Title,
            antibox_ui::metrics::font_pt(),
            true,
            false,
        ));
        let _ = g.set_foreground(colour);
        let _ = g.set_background(fill);
        let fh = antibox_ui::metrics::font_px();
        let tx = band_x + ((band - fh) / 2).max(0) as i16;
        let title = crate::applet::fit_label(g, fw.client().title(), avail as u16);
        let _ = g.draw_text_rotated_ccw(tx, text_end as i16, &title);
    }
    Ok(())
}

fn draw_frame_bottom_title(
    fw: &FrameWindow,
    g: &dyn GraphicsContext,
    focused: bool,
    colours: &ThemeColors,
) -> Result<(), Box<dyn std::error::Error>> {
    let fw_w = fw.frame_rect().w as u16;
    let fw_h = fw.frame_rect().h as u16;
    let bw = fw.effective_border();
    let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
    if bw > 0 {
        draw_border(g, focused, 0, fw_w, fw_h, colours)?;
    }
    let bandr = fw.band_rect();
    let (fill, urgent) = if fw.state().urgent && !focused {
        (colours.urgent_bg, true)
    } else if focused {
        (colours.active_title_top, false)
    } else {
        (colours.inactive_title_top, false)
    };
    let s = antibox_core::scale::scaled(1).max(1) as u16;
    let _ = g.set_foreground(fill);
    let _ = g.fill_rect(
        bandr.x as i16,
        bandr.y as i16,
        bandr.w as u16,
        bandr.h as u16,
    );
    let _ = g.set_foreground(shift_colour(fill, 40));
    let _ = g.fill_rect(bandr.x as i16, bandr.y as i16, bandr.w as u16, s);
    let _ = g.set_foreground(shift_colour(fill, -40));
    let _ = g.fill_rect(
        bandr.x as i16,
        (bandr.y + bandr.h) as i16 - s as i16,
        bandr.w as u16,
        s,
    );

    draw_buttons(fw, g, focused, colours)?;

    let (text_x, text_right) = fw.title_text_span();
    let avail = (text_right - text_x - 4).max(0) as u16;
    if avail > antibox_core::scale::scaled(8) as u16 {
        let colour = if urgent {
            colours.urgent_fg
        } else if focused {
            colours.active_text
        } else {
            colours.inactive_text
        };
        let _ = g.set_font(&FontSpec::role_styled(
            antibox_core::backend::FontRole::Title,
            antibox_ui::metrics::font_pt(),
            true,
            false,
        ));
        let _ = g.set_foreground(colour);
        let _ = g.set_background(fill);
        let baseline = bandr.y + antibox_ui::metrics::baseline(0, bandr.h);
        let title = crate::applet::fit_label(g, fw.client().title(), avail);
        let _ = g.draw_text_transparent(text_x as i16, baseline as i16, &title);
    }
    Ok(())
}

fn draw_buttons(
    fw: &FrameWindow,
    g: &dyn GraphicsContext,
    focused: bool,
    colours: &ThemeColors,
) -> Result<(), Box<dyn std::error::Error>> {
    let pressed = fw.pressed_button();
    let (bg, glyph_fg) = (colours.button_bg, colours.button_fg);

    for (id, _sym, pix_key, rect) in fw.title_button_layout() {
        let sunken = pressed == Some(id);
        let (x, y, w, h) = (rect.x as i16, rect.y as i16, rect.w as u16, rect.h as u16);
        if antibox_ui::theme::themed_title_button(
            g,
            pix_key,
            Rect::px(x, y, w, h),
            bg,
            focused,
            sunken,
        ) {
            let surround = if focused {
                colours.active_title_top
            } else {
                colours.inactive_title_top
            };
            antibox_ui::theme::round_title_button_corners(g, x, y, w, h, surround);
            continue;
        }
        if pix_key == "menu" {
            draw_menu_button(g, focused, colours, &rect, sunken);
            continue;
        }
        let (fg, off) =
            antibox_ui::theme::title_button(g, Rect::px(x, y, w, h), bg, glyph_fg, focused, sunken);
        let surround = if focused {
            colours.active_title_top
        } else {
            colours.inactive_title_top
        };
        antibox_ui::theme::round_title_button_corners(g, x, y, w, h, surround);
        g.set_foreground(fg)?;
        draw_button_glyph(g, pix_key, x + off, y + off, w, h)?;
    }
    Ok(())
}

fn draw_button_glyph(
    g: &dyn GraphicsContext,
    key: &str,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    if antibox_ui::theme::title_glyph_bitmap(g, key, x, y, w, h) {
        return Ok(());
    }

    let m = (w.min(h) as i16 / 6).max(antibox_core::scale::scaled(1) as i16);
    let l = x + m;
    let r = x + w as i16 - 1 - m;
    let t = y + m;
    let b = y + h as i16 - 1 - m;
    let bw = (r - l + 1).max(1) as u16;
    let bh = (b - t + 1).max(1) as u16;
    let cy = ((t as i32 + b as i32) / 2) as i16;
    let line2 = antibox_core::scale::scaled(2).max(2) as i16;
    let line1 = antibox_core::scale::scaled(1).max(1) as i16;
    match key {
        "close" => {
            let lx = l + line1;
            let rx = r - line1;
            for d in 0..line2 {
                g.draw_line(lx + d, t, rx - (line2 - 1) + d, b)?;
                g.draw_line(lx + d, b, rx - (line2 - 1) + d, t)?;
            }
        }
        "minimize" => {
            let bar_w = ((bw as i16 * 3 / 5).max(4)) as u16;
            g.fill_rect(l + line1, b - line2 + 1, bar_w, line2 as u16)?;
        }
        "maximize" => {
            for d in 0..line1 {
                g.draw_rect(
                    l + d,
                    t + d,
                    (bw as i16 - 1 - 2 * d).max(1) as u16,
                    (bh as i16 - 1 - 2 * d).max(1) as u16,
                )?;
            }
            g.fill_rect(l, t, bw, line2 as u16)?;
        }
        "restore" => {
            let s = (bw as i16 - 4).max(5);
            let off = 3i16;
            for d in 0..line1 {
                let sd = (s - 2 * d).max(1) as u16;
                g.draw_rect(l + off + d, t + d, sd, sd)?;
                g.draw_rect(l + d, t + off + d, sd, sd)?;
            }
            g.draw_line(l + off, t + 1, l + off + s, t + 1)?;
            g.draw_line(l, t + off + 1, l + s, t + off + 1)?;
        }
        "rollup" => {
            let cx = ((l as i32 + r as i32) / 2) as i16;
            let pts: [(i16, i16); 3] = [(cx, t + 1), (r, b - 1), (l, b - 1)];
            g.fill_polygon(&pts)?;
        }
        "rolldown" => {
            let cx = ((l as i32 + r as i32) / 2) as i16;
            let pts: [(i16, i16); 3] = [(cx, b - 1), (r, t + 1), (l, t + 1)];
            g.fill_polygon(&pts)?;
        }
        "hide" => {
            let qw = (bw / 4).max(1) as i16;
            g.fill_rect(
                l + qw,
                cy - line2 / 2,
                (bw - 2 * qw as u16).max(2),
                line2 as u16,
            )?;
        }
        "pin" => {
            let cx = ((l as i32 + r as i32) / 2) as i16;
            let head = (bw as i16 / 2).max(3);
            g.fill_arc(cx - head / 2, t, head as u16, head as u16, 0, 360 * 64)?;
            g.fill_rect(
                cx - (line1 / 2).max(0),
                t + head / 2,
                line1.max(1) as u16,
                (b - t - head / 2).max(1) as u16,
            )?;
        }
        "pinned" => {
            let cx = ((l as i32 + r as i32) / 2) as i16;
            let head = (bw as i16 / 2).max(3);
            g.fill_arc(cx - head / 2, t, head as u16, head as u16, 0, 360 * 64)?;
            g.fill_rect(
                cx - (line1 / 2).max(0),
                t + head / 2,
                line1.max(1) as u16,
                (b - t - head / 2 - line2).max(1) as u16,
            )?;
            g.fill_rect(l, b - line2 + 1, bw, line2 as u16)?;
        }
        "menu" => {
            let bar_w = (bw as i16 * 3 / 5).max(4) as u16;
            let bx = ((l as i32 + r as i32) / 2) as i16 - bar_w as i16 / 2;
            let by = ((t as i32 + b as i32) / 2) as i16;
            g.fill_rect(bx, by - line1 - 1, bar_w, line1 as u16)?;
            g.fill_rect(bx, by, bar_w, line1 as u16)?;
            g.fill_rect(bx, by + line1 + 1, bar_w, line1 as u16)?;
        }
        _ => {}
    }
    Ok(())
}

fn draw_border(
    g: &dyn GraphicsContext,
    focused: bool,
    top: i16,
    w: u16,
    h: u16,
    colours: &ThemeColors,
) -> Result<(), Box<dyn std::error::Error>> {
    let border = if focused {
        colours.border_active
    } else {
        colours.border_inactive
    };
    let title = if focused {
        colours.active_title_top
    } else {
        colours.inactive_title_top
    };
    let extent = title_bar_height() as u16;
    antibox_ui::theme::window_frame(
        g,
        top,
        antibox_core::point::Dimension::px(w, h),
        border,
        title,
        extent,
        focused,
    );
    Ok(())
}

#[cfg(test)]
mod tests;
