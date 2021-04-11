mod draw;
mod nt;

mod colour;

pub use self::draw::*;
use self::nt::*;
use antibox_gfx::colour::Colour;

pub use self::colour::contrast;
pub(crate) use self::colour::mix_rgb;
pub(crate) use self::colour::scale_rgb;
pub(crate) use self::colour::tint_rgb;

pub const fn face() -> Colour {
    FACE
}
pub const fn field() -> Colour {
    FIELD
}
pub fn list_bg() -> Colour {
    if LIST_BG != 0 {
        LIST_BG
    } else {
        FIELD
    }
}
pub fn menu_bg() -> Colour {
    if MENU_BG != 0 {
        MENU_BG
    } else {
        FACE
    }
}
pub fn arrow_colour() -> Colour {
    if ARROW_COLOUR != 0 {
        ARROW_COLOUR
    } else {
        TEXT
    }
}

pub fn text() -> Colour {
    contrast(TEXT, FACE)
}
pub const fn disabled() -> Colour {
    DISABLED
}
pub const fn sel_bg() -> Colour {
    SEL_BG
}
pub fn sel_fg() -> Colour {
    contrast(SEL_FG, SEL_BG)
}
pub const fn light() -> Colour {
    LIGHT
}
pub const fn face_light() -> Colour {
    FACE_LIGHT
}
pub const fn button_face() -> Colour {
    BUTTON_FACE
}
pub const fn shadow() -> Colour {
    SHADOW
}
pub const fn dark() -> Colour {
    DARK
}
pub const fn tooltip_bg() -> Colour {
    TOOLTIP_BG
}
pub fn tooltip_fg() -> Colour {
    contrast(TOOLTIP_FG, TOOLTIP_BG)
}
pub const fn sel_line() -> Colour {
    SEL_LINE
}
pub const fn title_active() -> Colour {
    TITLE_ACTIVE
}
pub const fn title_inactive() -> Colour {
    TITLE_INACTIVE
}

pub const fn title_inset_base() -> u16 {
    TITLE_INSET
}

pub const fn title_overlap_base() -> u16 {
    TITLE_OVERLAP
}

pub const fn close_gap_base() -> u16 {
    CLOSE_GAP
}

pub const fn title_end_pad_base() -> u16 {
    TITLE_END_PAD
}

pub const fn chrome_font() -> &'static str {
    CHROME_FONT
}

pub const fn ui_font_name() -> &'static str {
    UI_FONT_NAME
}

pub const fn clock_format() -> &'static str {
    CLOCK_FORMAT
}

pub fn tray_face() -> Colour {
    if TRAY_FACE != 0 {
        TRAY_FACE
    } else if TASKBAR_FACE != 0 {
        TASKBAR_FACE
    } else {
        FACE
    }
}

pub const fn title_buttons() -> &'static str {
    TITLE_BUTTONS
}

const NT_GLYPHS: &[(&str, &[u16])] = &[
    (
        "minimize",
        &[
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x007E, 0x007E, 0x0000,
        ],
    ),
    (
        "maximize",
        &[
            0x01FF, 0x01FF, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x01FF, 0x0000,
        ],
    ),
    (
        "restore",
        &[
            0x03FC, 0x03FC, 0x0204, 0x02FF, 0x02FF, 0x0381, 0x0081, 0x0081, 0x00FF, 0x0000,
        ],
    ),
    (
        "close",
        &[
            0x0000, 0x0186, 0x00CC, 0x0078, 0x0030, 0x0078, 0x00CC, 0x0186, 0x0000, 0x0000,
        ],
    ),
];

pub fn title_glyph(key: &str) -> Option<&'static [u16]> {
    NT_GLYPHS.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

pub const fn pad_base() -> u16 {
    PAD
}

pub const fn gap_base() -> u16 {
    GAP
}

pub const fn item_gap_base() -> i16 {
    ITEM_GAP
}

pub const fn title_height_base() -> u16 {
    TITLE_HEIGHT
}

pub const fn border_base() -> u16 {
    BORDER
}

pub const fn border_bottom_base() -> u16 {
    BORDER_BOTTOM
}

pub const fn button_base() -> u16 {
    BUTTON_BASE
}

pub fn title_button_inset_base() -> u16 {
    if TITLE_BUTTON_INSET != 0 {
        TITLE_BUTTON_INSET
    } else {
        BUTTON_INSET
    }
}

pub const fn title_justify_default() -> u8 {
    TITLE_JUSTIFY
}

pub const fn taskbar_justify_default() -> &'static str {
    TASKBAR_JUSTIFY
}

fn normalize_side(s: &str) -> Option<&'static str> {
    if s.eq_ignore_ascii_case("top") {
        Some("top")
    } else if s.eq_ignore_ascii_case("bottom") {
        Some("bottom")
    } else if s.eq_ignore_ascii_case("left") {
        Some("left")
    } else if s.eq_ignore_ascii_case("right") {
        Some("right")
    } else {
        None
    }
}

static TITLE_SIDE_OVERRIDE: antibox_gfx::sync::atomic::AtomicU8 =
    antibox_gfx::sync::atomic::AtomicU8::new(0);

pub fn set_title_side_override(name: &str) {
    use std::sync::atomic::Ordering;
    let code = match normalize_side(name) {
        Some("top") => 1,
        Some("bottom") => 2,
        Some("left") => 3,
        Some("right") => 4,
        _ => 0,
    };
    TITLE_SIDE_OVERRIDE.store(code, Ordering::Relaxed);
}

pub fn title_side_name() -> &'static str {
    use std::sync::atomic::Ordering;
    match TITLE_SIDE_OVERRIDE.load(Ordering::Relaxed) {
        1 => return "top",
        2 => return "bottom",
        3 => return "left",
        4 => return "right",
        _ => {}
    }
    normalize_side(TITLE_SIDE).unwrap_or("top")
}

pub fn title_on_left() -> bool {
    title_side_name() == "left"
}

pub fn title_on_right() -> bool {
    title_side_name() == "right"
}

pub fn title_on_bottom() -> bool {
    title_side_name() == "bottom"
}

pub fn title_vertical() -> bool {
    matches!(title_side_name(), "left" | "right")
}

pub fn title_offset_side() -> bool {
    title_side_name() != "top"
}

pub fn taskbar_item_chars() -> u16 {
    match TASKBAR_ITEM_CHARS {
        0 => 24,
        v => v,
    }
}

pub fn taskbar_item_pct() -> u16 {
    TASKBAR_ITEM_PCT
}

pub fn menu_sel_bg() -> Colour {
    if MENU_SEL_BG != 0 {
        MENU_SEL_BG
    } else {
        TITLE_ACTIVE
    }
}

pub fn menu_sel_fg() -> Colour {
    if MENU_SEL_BG != 0 {
        contrast(MENU_SEL_FG, MENU_SEL_BG)
    } else {
        contrast(TITLE_TEXT, TITLE_ACTIVE)
    }
}

pub fn title_layout() -> (&'static str, &'static str) {
    (TITLE_LAYOUT_LEFT, TITLE_LAYOUT_RIGHT)
}

pub fn sunken_depth() -> u16 {
    SUNKEN_DEPTH.clamp(1, 2)
}

pub fn corner_radius_px() -> u16 {
    let r = CORNER_RADIUS;
    if r == 0 {
        0
    } else {
        antibox_gfx::scale::scaled(r as i32).max(1) as u16
    }
}

pub fn frame_radius_px() -> u16 {
    let r = FRAME_RADIUS;
    if r == 0 {
        corner_radius_px()
    } else {
        antibox_gfx::scale::scaled(r as i32).max(1) as u16
    }
}

pub fn button_radius_px() -> u16 {
    let r = BUTTON_RADIUS;
    if r == 0 {
        corner_radius_px()
    } else {
        antibox_gfx::scale::scaled(r as i32).max(1) as u16
    }
}

pub fn graph_bg() -> Colour {
    face()
}

pub fn graph_series(i: usize) -> Colour {
    let derived = [sel_bg(), title_active(), tint_rgb(sel_bg(), 0.5), shadow()];
    derived[i % 4]
}

#[cfg(test)]
mod theme_gen;
