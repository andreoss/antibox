use crate::option::*;

pub fn parse_geometry(s: &str) -> (GeoFlags, i32, i32, u32, u32) {
    let s = s.trim();
    let (mut flags, mut x, mut y, mut w, mut h) = (GeoFlags::default(), 0i32, 0i32, 0u32, 0u32);
    if let Some(xi) = s.find(|c: char| c.eq_ignore_ascii_case(&'x')) {
        w = s[..xi].parse().unwrap_or(0);
        let after_x = &s[xi + 1..];
        if let Some(si) = after_x.find(&['+', '-'][..]) {
            h = after_x[..si].parse().unwrap_or(0);
            parse_coords(&after_x[si..], &mut flags, &mut x, &mut y);
        } else {
            h = after_x.parse().unwrap_or(0);
        }
        flags |= GeoFlags::W | GeoFlags::H;
    } else if let Some(si) = s.find(&['+', '-'][..]) {
        parse_coords(&s[si..], &mut flags, &mut x, &mut y);
    }
    (flags, x, y, w, h)
}

fn parse_coords(s: &str, flags: &mut GeoFlags, x: &mut i32, y: &mut i32) {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return;
    }
    let first = bytes[0] as char;
    let rest = &s[1..];
    if let Some(idx) = rest.find(&['+', '-'][..]) {
        *x = parse_int_sign(rest[..idx].parse().ok(), first);
        *flags |= GeoFlags::X;
        let second = rest.as_bytes()[idx] as char;
        *y = parse_int_sign(rest[idx + 1..].parse().ok(), second);
        *flags |= GeoFlags::Y;
    } else {
        *x = parse_int_sign(rest.parse().ok(), first);
        *flags |= GeoFlags::X;
    }
}

fn parse_int_sign(val: Option<i32>, sign: char) -> i32 {
    val.unwrap_or(0) * if sign == '-' { -1 } else { 1 }
}

pub fn parse_layer(s: &str) -> Option<WinLayer> {
    if let Ok(num) = s.parse::<i32>() {
        return layer_from_code(num);
    }
    match s {
        "Desktop" => Some(WinLayer::Desktop),
        "Below" => Some(WinLayer::Below),
        "Normal" => Some(WinLayer::Normal),
        "Above" | "OnTop" => Some(WinLayer::OnTop),
        "Dock" => Some(WinLayer::Dock),
        "AboveDock" => Some(WinLayer::AboveDock),
        "Menu" => Some(WinLayer::Menu),
        "Fullscreen" => Some(WinLayer::Fullscreen),
        "AboveAll" => Some(WinLayer::AboveAll),
        _ => None,
    }
}

const fn layer_from_code(n: i32) -> Option<WinLayer> {
    match n {
        0 | 1 => Some(WinLayer::Desktop),
        2 | 3 => Some(WinLayer::Below),
        4 | 5 => Some(WinLayer::Normal),
        6 | 7 => Some(WinLayer::OnTop),
        8 | 9 => Some(WinLayer::Dock),
        10 | 11 => Some(WinLayer::AboveDock),
        12 | 13 => Some(WinLayer::Menu),
        14 => Some(WinLayer::Fullscreen),
        15 => Some(WinLayer::AboveAll),
        _ => None,
    }
}

pub fn lookup_option_flag(opt: &str) -> Option<WindowFlags> {
    let table: [(&str, WindowFlags); 11] = [
        ("allWorkspaces", WindowFlags::ALL_WORKSPACES),
        ("doNotFocus", WindowFlags::DO_NOT_FOCUS),
        ("doNotManage", WindowFlags::DO_NOT_MANAGE),
        (
            "ignoreOverrideRedirect",
            WindowFlags::IGNORE_OVERRIDE_REDIRECT,
        ),
        ("ignoreTaskBar", WindowFlags::IGNORE_TASKBAR),
        ("noFocusOnMap", WindowFlags::NO_FOCUS_ON_MAP),
        ("startFullscreen", WindowFlags::FULLSCREEN),
        ("startMaximized", WindowFlags::MAXIMIZED_BOTH),
        ("startMaximizedHorz", WindowFlags::MAXIMIZED_HORZ),
        ("startMaximizedVert", WindowFlags::MAXIMIZED_VERT),
        ("startMinimized", WindowFlags::MINIMIZED),
    ];
    table
        .iter()
        .find(|(name, _)| *name == opt)
        .map(|(_, flag)| *flag)
}

#[cfg(test)]
#[path = "parsing_tests.rs"]
mod tests;
