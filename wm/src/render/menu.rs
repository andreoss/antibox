use antibox_core::backend::FontSpec;
use antibox_core::backend::GraphicsContext;
use antibox_core::backend::WindowHandle;
use antibox_core::point::Point;

pub fn parse_mnemonic(label: &str) -> (String, Option<usize>) {
    match label.find('_') {
        Some(pos) => {
            let hot = label[..pos].chars().count();
            let mut text = String::with_capacity(label.len() - 1);
            text.push_str(&label[..pos]);
            text.push_str(&label[pos + 1..]);
            (text, Some(hot))
        }
        None => (label.to_string(), None),
    }
}

pub fn mnemonic_key(label: &str) -> Option<char> {
    let (text, hot) = parse_mnemonic(label);
    hot.and_then(|h| text.chars().nth(h))
        .map(|c| c.to_ascii_uppercase())
}

pub fn menu_content_width<'a>(labels: impl Iterator<Item = &'a str>) -> u16 {
    let spec = FontSpec::role_styled(
        antibox_core::backend::FontRole::Menu,
        antibox_ui::metrics::font_pt(),
        false,
        false,
    );
    let arrow_col = antibox_core::scale::scaled(28);
    let mut w = antibox_core::scale::scaled(96);
    for label in labels {
        let (text, _) = parse_mnemonic(label);
        if let Some(tw) = antibox_core::backend::global_text_width(&spec, &text) {
            w = w.max(tw as i32 + arrow_col);
        }
    }
    w.clamp(
        antibox_core::scale::scaled(96),
        antibox_core::scale::scaled(420),
    ) as u16
}

pub fn draw_menu_frame(
    g: &dyn GraphicsContext,
    w: u16,
    h: u16,
    face: antibox_core::colour::Colour,
) -> (u32, u32) {
    let _ = g.set_foreground(face);
    let _ = g.fill_rect(0, 0, w, h);
    draw_menu_border(g, w, h, face)
}

pub fn draw_menu_border(
    g: &dyn GraphicsContext,
    w: u16,
    h: u16,
    face: antibox_core::colour::Colour,
) -> (u32, u32) {
    antibox_ui::theme::menu_border(g, w, h, face)
}

pub fn fill_menu_selection(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    sel_bg: antibox_core::colour::Colour,
) {
    antibox_ui::theme::menu_selection(g, x, y, w, h, sel_bg);
}

pub fn draw_menu_row_rule(g: &dyn GraphicsContext, x: i16, y: i16, w: u16) {
    antibox_ui::theme::menu_row_rule(g, x, y, w);
}

pub fn draw_menu_separator(
    g: &dyn GraphicsContext,
    x: i16,
    y: i16,
    w: u16,
    light: antibox_core::colour::Colour,
    dark: u32,
) {
    let s = antibox_core::scale::scaled(1).max(1) as u16;
    let _ = g.set_foreground(dark);
    let _ = g.fill_rect(x, y, w, s);
    let _ = g.set_foreground(light);
    let _ = g.fill_rect(x, y + s as i16, w, s);
}

pub fn draw_text_underline(
    g: &dyn GraphicsContext,
    x: i16,
    baseline: i16,
    text: &str,
    hot: Option<usize>,
    fg: antibox_core::colour::Colour,
) {
    let _ = g.set_foreground(fg);
    let _ = g.draw_text(x, baseline, text);
    if let Some(hi) = hot {
        let prefix: String = text.chars().take(hi).collect();
        let ch: String = text.chars().skip(hi).take(1).collect();
        let px = x + g.text_width(&prefix).unwrap_or(0) as i16;
        let cw = g.text_width(&ch).unwrap_or(6) as i16;
        let uy = baseline + 2;
        let _ = g.draw_line(px, uy, px + cw - 1, uy);
    }
}

pub fn draw_text_mnemonic(
    g: &dyn GraphicsContext,
    x: i16,
    baseline: i16,
    label: &str,
    fg: antibox_core::colour::Colour,
) {
    let (text, hot) = parse_mnemonic(label);
    draw_text_underline(g, x, baseline, &text, hot, fg);
}

pub fn hot_char_at(text: &str, hot: Option<usize>) -> Option<char> {
    hot.and_then(|h| text.chars().nth(h))
        .map(|c| c.to_ascii_uppercase())
}

pub fn menu_hot_match(
    hots: &[Option<char>],
    selected: Option<usize>,
    key: char,
) -> Option<(usize, usize)> {
    let key = key.to_ascii_uppercase();
    let count = hots.iter().filter(|h| **h == Some(key)).count();
    if count == 0 {
        return None;
    }
    let n = hots.len();
    let cur = selected.unwrap_or(n - 1);
    for step in 1..=n {
        let c = (cur + step) % n;
        if hots[c] == Some(key) {
            return Some((c, count));
        }
    }
    None
}

pub fn menu_clamp_pos(pos: Point, w: i32, h: i32, sw: i32, sh: i32, flip_up: bool) -> Point {
    let x = pos.x.clamp(0, (sw - w).max(0));
    let y = if flip_up {
        if pos.y + h > sh {
            (pos.y - h).max(0)
        } else {
            pos.y.max(0)
        }
    } else {
        pos.y.min((sh - h).max(0)).max(0)
    };
    Point::new(x, y)
}

pub fn toward_submenu(prev: Point, cur: Point, sub_pos: Point, sub_w: i32, sub_h: i32) -> bool {
    let (ex, moving_toward) = if prev.x <= sub_pos.x {
        (sub_pos.x, cur.x > prev.x)
    } else if prev.x >= sub_pos.x + sub_w {
        (sub_pos.x + sub_w, cur.x < prev.x)
    } else {
        return false;
    };
    if !moving_toward {
        return false;
    }
    let sign = |p1: Point, p2: Point, p3: Point| {
        i64::from(p1.x - p3.x) * i64::from(p2.y - p3.y)
            - i64::from(p2.x - p3.x) * i64::from(p1.y - p3.y)
    };
    let pad = antibox_core::scale::scaled(16).max(12);
    let a = prev;
    let b = Point::new(ex, sub_pos.y - pad);
    let c = Point::new(ex, sub_pos.y + sub_h + pad);
    let d1 = sign(cur, a, b);
    let d2 = sign(cur, b, c);
    let d3 = sign(cur, c, a);
    let neg = d1 < 0 || d2 < 0 || d3 < 0;
    let pos = d1 > 0 || d2 > 0 || d3 > 0;
    !(neg && pos)
}

pub fn menu_destroy_window(window: &mut Option<Box<dyn WindowHandle>>, visible: &mut bool) {
    if let Some(ref win) = window {
        let _ = win.unmap();
        let _ = win.destroy();
    }
    *window = None;
    *visible = false;
}

pub fn menu_item_at(
    p: Point,
    pos: Point,
    top_offset: i32,
    item_h: i32,
    count: usize,
) -> Option<usize> {
    let wy = p.y - pos.y;
    if wy < top_offset {
        return None;
    }
    let idx = ((wy - top_offset) / item_h) as usize;
    if idx < count {
        Some(idx)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "menu_triangle_tests.rs"]
mod triangle_tests;
