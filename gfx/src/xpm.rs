use crate::backend::PixmapData;
use std::path::Path;

pub fn parse_xpm_string(s: &str) -> Option<PixmapData> {
    let is_xpm2 = s.as_bytes().starts_with(b"! XPM2");
    if !s.starts_with("/* XPM */") && !is_xpm2 {
        return None;
    }
    let entries: Vec<String> = if is_xpm2 {
        s.lines()
            .skip(1)
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("/*") && !l.starts_with('!'))
            .map(str::to_string)
            .collect()
    } else {
        extract_quoted_strings(s)
    };

    let hdr = entries.first()?;
    let hv: Vec<&str> = hdr.split_whitespace().collect();
    if hv.len() < 4 {
        return None;
    }
    let w: u32 = hv[0].parse().ok()?;
    let h: u32 = hv[1].parse().ok()?;
    let ncolours: usize = hv[2].parse().ok()?;
    let cpp: usize = hv[3].parse().ok()?;
    if w == 0 || h == 0 || w > 2048 || h > 2048 || cpp == 0 {
        return None;
    }
    if entries.len() < 1 + ncolours + h as usize {
        return None;
    }

    let mut palette: Vec<(String, [u8; 4])> = Vec::with_capacity(ncolours);
    for entry in entries.iter().skip(1).take(ncolours) {
        let key: String = entry.chars().take(cpp).collect();
        let rest: String = entry.chars().skip(cpp).collect();
        palette.push((key, parse_colour(&rest)));
    }

    let data_start = 1 + ncolours;
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for yi in 0..h as usize {
        let row: Vec<char> = entries[data_start + yi].chars().collect();
        for xi in 0..w as usize {
            let start = xi * cpp;
            let end = (start + cpp).min(row.len());
            let key: String = if start < end {
                row[start..end].iter().collect()
            } else {
                String::new()
            };
            let rgba = palette
                .iter()
                .find(|(k, _)| *k == key)
                .map_or([0, 0, 0, 255], |(_, p)| *p);
            pixels.extend_from_slice(&rgba);
        }
    }
    Some(PixmapData::new(w as u16, h as u16, pixels))
}

fn extract_quoted_strings(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            match s[i + 1..].find('"') {
                Some(rel) => {
                    out.push(s[i + 1..i + 1 + rel].to_string());
                    i = i + 1 + rel + 1;
                }
                None => break,
            }
        } else {
            i += 1;
        }
    }
    out
}

fn parse_colour(s: &str) -> [u8; 4] {
    let cleaned: String = s
        .chars()
        .filter(|&c| c != ',' && c != ';' && c != '"')
        .collect();
    for pair in cleaned.split_whitespace().collect::<Vec<_>>().chunks(2) {
        if pair.len() < 2 {
            continue;
        }
        let (key, val) = (pair[0].trim(), pair[1].trim());
        if key == "c" || key == "g" {
            if let Some(hex) = val.strip_prefix('#') {
                let clean: String = hex.chars().filter(|&c| c.is_ascii_hexdigit()).collect();
                if clean.len() >= 6 {
                    if let Ok(rgb) = u32::from_str_radix(&clean[..6], 16) {
                        return [
                            ((rgb >> 16) & 0xFF) as u8,
                            ((rgb >> 8) & 0xFF) as u8,
                            (rgb & 0xFF) as u8,
                            255,
                        ];
                    }
                }
            }
            if val == "None" {
                return [0, 0, 0, 0];
            }
        }
    }
    [0, 0, 0, 255]
}

fn is_indirect_xpm_ref(first_line: &str) -> Option<String> {
    let trimmed = first_line.trim();
    if trimmed.len() < 5 || trimmed.len() > 64 {
        return None;
    }
    let bytes = trimmed.as_bytes();
    if !bytes[0].is_ascii_lowercase() {
        return None;
    }
    if !trimmed.ends_with(".xpm") && !trimmed.ends_with(".XPM") && !trimmed.ends_with(".Xpm") {
        let lower = trimmed.to_ascii_lowercase();
        if !lower.ends_with(".xpm") {
            return None;
        }
    }
    for &b in &bytes[1..bytes.len() - 4] {
        if !b.is_ascii_alphanumeric() && b != b'-' && b != b'_' {
            return None;
        }
    }
    Some(trimmed.to_string())
}

pub fn load_xpm_file(path: &Path) -> Option<PixmapData> {
    if let Some(data) = parse_xpm_path(path) {
        return Some(data);
    }

    let mut current = path.to_path_buf();
    for _ in 0..9 {
        let bytes = std::fs::read(&current).ok()?;
        if bytes.len() > 64 {
            return None;
        }
        let first_line = std::str::from_utf8(&bytes).ok()?.lines().next()?;
        let ref_name = is_indirect_xpm_ref(first_line)?;

        if let Some(parent) = current.parent() {
            current = parent.join(&ref_name);
            if let Some(data) = parse_xpm_path(&current) {
                return Some(data);
            }
        } else {
            break;
        }
    }

    None
}

pub(crate) fn parse_xpm_path(path: &Path) -> Option<PixmapData> {
    let bytes = std::fs::read(path).ok()?;
    let content = std::str::from_utf8(&bytes).ok()?;
    parse_xpm_string(content)
}

#[cfg(test)]
#[path = "xpm_tests.rs"]
mod tests;
