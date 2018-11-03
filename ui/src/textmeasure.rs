use crate::metrics;
use antibox_gfx::backend::{GraphicsContext, RenderBackend};
use antibox_gfx::scale::scaled;

pub fn with_measure_context<B: RenderBackend + ?Sized, R>(
    backend: &B,
    f: impl FnOnce(&dyn GraphicsContext) -> R,
) -> Option<R> {
    let pixmap = backend.create_pixmap(1, 1, backend.screen_depth()).ok()?;
    let out = backend.create_graphics(pixmap).ok().map(|g| f(&*g));
    let _ = backend.free_pixmap(pixmap);
    out
}

fn text_w(g: &dyn GraphicsContext, s: &str) -> i32 {
    g.text_width(s).unwrap_or(0) as i32
}

pub fn font_height(g: &dyn GraphicsContext) -> i32 {
    let h = g.font_metrics().2 as i32;
    if h > 0 {
        h
    } else {
        metrics::font_px()
    }
}

pub fn font_ascent(g: &dyn GraphicsContext) -> i32 {
    let a = g.font_metrics().0 as i32;
    if a > 0 {
        a
    } else {
        metrics::font_px() * 3 / 4
    }
}

pub fn line_height(g: &dyn GraphicsContext) -> i32 {
    font_height(g) + scaled(2)
}

fn push_word(
    g: &dyn GraphicsContext,
    out: &mut Vec<String>,
    cur: &mut String,
    word: &str,
    maxw: i32,
) {
    let joined = if cur.is_empty() {
        word.to_string()
    } else {
        format!("{} {}", cur, word)
    };
    if text_w(g, &joined) <= maxw {
        *cur = joined;
        return;
    }
    if !cur.is_empty() {
        out.push(std::mem::replace(cur, String::new()));
    }
    if text_w(g, word) <= maxw {
        *cur = word.to_string();
        return;
    }
    let mut piece = String::new();
    for c in word.chars() {
        piece.push(c);
        if text_w(g, &piece) > maxw && piece.chars().count() > 1 {
            piece.pop();
            out.push(piece.clone());
            piece.clear();
            piece.push(c);
        }
    }
    *cur = piece;
}

fn wrap_line(g: &dyn GraphicsContext, line: &str, maxw: i32, out: &mut Vec<String>) {
    let mut cur = String::new();
    for word in line.split_whitespace() {
        push_word(g, out, &mut cur, word, maxw);
    }
    out.push(cur);
}

pub fn break_lines(g: &dyn GraphicsContext, text: &str, wrap_width: Option<u16>) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.split('\n') {
        match wrap_width {
            Some(w) if text_w(g, raw) > w as i32 => wrap_line(g, raw, w as i32, &mut out),
            _ => out.push(raw.to_string()),
        }
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

pub fn measure(
    g: &dyn GraphicsContext,
    text: &str,
    wrap_width: Option<u16>,
) -> (u16, u16, Vec<String>) {
    let lines = break_lines(g, text, wrap_width);
    let w = lines.iter().map(|l| text_w(g, l)).max().unwrap_or(0);
    let h = lines.len() as i32 * line_height(g);
    (w.max(0) as u16, h.max(0) as u16, lines)
}

pub fn draw_lines(g: &dyn GraphicsContext, x: i16, y: i16, lines: &[String]) {
    let lh = line_height(g);
    let ascent = font_ascent(g) + (lh - font_height(g)) / 2;
    for (i, line) in lines.iter().enumerate() {
        let by = y as i32 + i as i32 * lh + ascent;
        let _ = g.draw_text(x, by as i16, line);
    }
}

pub fn draw(g: &dyn GraphicsContext, x: i16, y: i16, text: &str, wrap_width: Option<u16>) {
    let lines = break_lines(g, text, wrap_width);
    draw_lines(g, x, y, &lines);
}

#[cfg(test)]
#[path = "textmeasure_tests.rs"]
mod tests;
