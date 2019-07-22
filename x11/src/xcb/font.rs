
use antibox_core::libc;
use super::bindings::*;
use super::connection::XcbConnection;
use std::sync::Mutex;

#[derive(Clone)]
pub struct XcbFont {
    pub id: u32,
    ascent: i16,
    descent: i16,
    min_char: u16,
    max_char: u16,
    min_byte1: u8,
    max_byte1: u8,
    widths: Vec<i16>,
    default_width: i16,
}

const FONTPROP_SIZE: usize = 8;
const REPLY_HEADER_LEN: usize = 32;

impl XcbFont {
    fn glyph_index(&self, code: u32) -> Option<usize> {
        if code > 0xFFFF {
            return None;
        }
        let code = code as u16;
        if self.max_byte1 == 0 {
            if code < self.min_char || code > self.max_char {
                return None;
            }
            Some((code - self.min_char) as usize)
        } else {
            let row = (code >> 8) as u8;
            let col = code & 0xFF;
            if row < self.min_byte1
                || row > self.max_byte1
                || col < self.min_char
                || col > self.max_char
            {
                return None;
            }
            let ncols = (self.max_char - self.min_char + 1) as usize;
            Some((row - self.min_byte1) as usize * ncols + (col - self.min_char) as usize)
        }
    }

    fn char_width(&self, c: char) -> i32 {
        if let Some(idx) = self.glyph_index(c as u32) {
            if idx < self.widths.len() {
                let w = self.widths[idx];
                if w != 0 {
                    return w as i32;
                }
            }
        }
        self.default_width as i32
    }

    pub fn text_width(&self, text: &str) -> u32 {
        text.chars().map(|c| self.char_width(c)).sum::<i32>().max(0) as u32
    }

    pub fn metrics(&self) -> (u16, u16, u16) {
        let a = self.ascent.max(0) as u16;
        let d = self.descent.max(0) as u16;
        (a, d, a.saturating_add(d))
    }
}

pub fn register_global_width_provider(conn: &std::sync::Arc<XcbConnection>) {
    use antibox_core::backend::{set_global_font_providers, FontSpec};
    let conn = std::sync::Arc::clone(conn);
    let provider = std::sync::Arc::new(move |spec: &FontSpec, text: &str| {
        let px = (spec.size as f32 * 96.0 / 72.0).round() as u16;
        resolve_font(&conn, &spec.family, px).map_or(0, |cf| cf.text_width(text))
    });
    let _ = set_global_font_providers(provider);
}

fn strike_px(name: &str) -> Option<u16> {
    name.split('-').nth(7)?.parse().ok()
}

fn nearest_strike(names: &[String], px: u16) -> Option<(u16, String)> {
    names
        .iter()
        .filter_map(|s| {
            let strike = strike_px(s)?;
            if strike == 0 {
                return None;
            }
            let ad = if strike > px { strike - px } else { px - strike };
            let dist = ad as u32 * 2 + u32::from(strike < px);
            Some((dist, strike, s.clone()))
        })
        .min_by_key(|(dist, _, _)| *dist)
        .map(|(_, strike, name)| (strike, name))
}

fn with_pixel_size(xlfd: &str, px: u16) -> String {
    let mut parts: Vec<String> = xlfd.split('-').map(ToString::to_string).collect();
    if parts.len() >= 15 {
        parts[7] = px.to_string();
        parts[8] = "*".to_string();
        parts[12] = "*".to_string();
    }
    parts.join("-")
}

fn list_font_names(conn: *mut xcb_connection_t, pattern: &str) -> Option<Vec<String>> {
    let cpat = std::ffi::CString::new(pattern).ok()?;
    let cookie = unsafe { xcb_list_fonts(conn, 64, pattern.len() as u16, cpat.as_ptr()) };
    let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
    let r = unsafe { xcb_list_fonts_reply(conn, cookie, &mut e) };
    if r.is_null() {
        return None;
    }
    let count = unsafe { (*r).names_len } as usize;
    let reply_len = REPLY_HEADER_LEN + (unsafe { (*r).length } as usize) * 4;
    let start = std::mem::size_of::<xcb_list_fonts_reply_t>();
    let avail = reply_len.saturating_sub(start);
    let data = unsafe { std::slice::from_raw_parts((r as *const u8).add(start), avail) };
    let names = parse_names(data, count);
    unsafe { libc::free(r as *mut libc::c_void) };
    Some(names)
}

fn parse_names(data: &[u8], count: usize) -> Vec<String> {
    let mut names = Vec::new();
    let mut pos = 0;
    for _ in 0..count {
        if pos >= data.len() {
            break;
        }
        let len = data[pos] as usize;
        pos += 1;
        if pos + len > data.len() {
            break;
        }
        names.push(String::from_utf8_lossy(&data[pos..pos + len]).into_owned());
        pos += len;
    }
    names
}

fn open_font(conn: *mut xcb_connection_t, id: u32, name: &str) -> bool {
    let cname = match std::ffi::CString::new(name) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let len = (name.len()).min(std::u16::MAX as usize) as u16;
    unsafe { xcb_open_font(conn, id, len, cname.as_ptr()) };
    true
}

struct FontMetrics {
    ascent: i16,
    descent: i16,
    min_char: u16,
    max_char: u16,
    min_byte1: u8,
    max_byte1: u8,
    default_width: i16,
    widths: Vec<i16>,
}

fn query_font_metrics(conn: *mut xcb_connection_t, id: u32) -> Option<FontMetrics> {
    let cookie = unsafe { xcb_query_font(conn, id) };
    let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
    let r = unsafe { xcb_query_font_reply(conn, cookie, &mut e) };
    if r.is_null() {
        return None;
    }
    let ascent = unsafe { (*r).font_ascent };
    let descent = unsafe { (*r).font_descent };
    let min_char = unsafe { (*r).min_char_or_byte2 };
    let max_char = unsafe { (*r).max_char_or_byte2 };
    let min_byte1 = unsafe { (*r).min_byte1 };
    let max_byte1 = unsafe { (*r).max_byte1 };
    let char_infos_len = unsafe { (*r).char_infos_len } as usize;
    let properties_len = unsafe { (*r).properties_len } as usize;
    let default_width = unsafe { (*r).max_bounds.character_width };
    let reply_len = REPLY_HEADER_LEN + (unsafe { (*r).length } as usize) * 4;
    let offset = std::mem::size_of::<xcb_query_font_reply_t>() + properties_len * FONTPROP_SIZE;
    if offset + char_infos_len * std::mem::size_of::<xcb_charinfo_t>() > reply_len {
        unsafe { libc::free(r as *mut libc::c_void) };
        return None;
    }
    let ci_ptr = unsafe { (r as *const u8).add(offset) as *const xcb_charinfo_t };
    let mut widths = Vec::with_capacity(char_infos_len);
    for i in 0..char_infos_len {
        widths.push(unsafe { (*ci_ptr.add(i)).character_width });
    }
    unsafe { libc::free(r as *mut libc::c_void) };
    Some(FontMetrics {
        ascent,
        descent,
        min_char,
        max_char,
        min_byte1,
        max_byte1,
        default_width,
        widths,
    })
}

pub fn resolve_font(conn: &XcbConnection, family: &str, px: u16) -> Option<XcbFont> {
    if let Some(f) = resolve(conn, family, px) {
        return Some(f);
    }
    let primary = resolve(conn, "fixed", px);
    if let Some(f) = &primary {
        if f.metrics().2 + 2 >= px {
            return primary;
        }
    }
    let scaled = resolve(conn, "helvetica", px);
    let overshoot = scaled.as_ref().map_or(0, |f| f.metrics().2);
    if overshoot > px + px / 8 {
        let px2 = ((px as u32 * px as u32) / overshoot as u32).max(1) as u16;
        return resolve(conn, "helvetica", px2).or(scaled).or(primary);
    }
    scaled.or(primary)
}

pub fn resolve(conn: &XcbConnection, family: &str, px: u16) -> Option<XcbFont> {
    let key = (family.to_ascii_lowercase(), px);
    if let Some(hit) = cache().lock().ok().and_then(|g| g.get(&key).cloned()) {
        return hit;
    }
    let resolved = resolve_uncached(conn, family, px);
    if let Ok(mut g) = cache().lock() {
        g.insert(key, resolved.clone());
    }
    resolved
}

type FontCache = Mutex<std::collections::HashMap<(String, u16), Option<XcbFont>>>;

fn cache() -> &'static FontCache {
    use std::sync::Once;
    static INIT: Once = Once::new();
    static mut CACHE: *const FontCache = std::ptr::null();
    INIT.call_once(|| unsafe {
        CACHE = Box::into_raw(Box::new(Mutex::new(std::collections::HashMap::new())));
    });
    unsafe { &*CACHE }
}

fn resolve_uncached(conn: &XcbConnection, family: &str, px: u16) -> Option<XcbFont> {
    let raw = conn.raw();
    let unicode = format!("-*-{}-*-*-*-*-*-*-*-*-*-*-iso10646-1", family);
    let mut names = list_font_names(raw, &unicode).unwrap_or_default();
    if names.is_empty() {
        let pattern = format!("-*-{}-*-*-*-*-*-*-*-*-*-*-*-*", family);
        names = list_font_names(raw, &pattern).unwrap_or_default();
    }
    if names.is_empty() {
        return None;
    }
    let first = names[0].clone();
    let nearest = nearest_strike(&names, px);
    let scalable = names.iter().find(|n| strike_px(n) == Some(0)).cloned();
    let (target, fallback) = match &nearest {
        Some((strike, name)) if *strike + 2 >= px => (name.clone(), first.clone()),
        _ => {
            let fb = nearest
                .as_ref()
                .map(|(_, n)| n.clone())
                .unwrap_or_else(|| first.clone());
            match scalable {
                Some(s) => (with_pixel_size(&s, px), fb),
                None => (fb.clone(), fb),
            }
        }
    };

    let id = unsafe { xcb_generate_id(raw) };
    open_font(raw, id, &target);
    let metrics = query_font_metrics(raw, id).or_else(|| {
        if target != fallback {
            open_font(raw, id, &fallback);
            query_font_metrics(raw, id)
        } else {
            None
        }
    })?;

    Some(XcbFont {
        id,
        ascent: metrics.ascent,
        descent: metrics.descent,
        min_char: metrics.min_char,
        max_char: metrics.max_char,
        min_byte1: metrics.min_byte1,
        max_byte1: metrics.max_byte1,
        widths: metrics.widths,
        default_width: metrics.default_width,
    })
}

#[cfg(test)]
#[path = "font_tests.rs"]
mod tests;
