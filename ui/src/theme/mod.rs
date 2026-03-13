pub mod dsl;
pub mod exec;
pub mod load;

mod draw;

mod colour;

pub use self::draw::*;
use self::dsl::ThemeDef;
use antibox_gfx::colour::Colour;
use antibox_gfx::sync::atomic::LazyRwLock;

static LOADED: LazyRwLock<Option<ThemeDef>> = LazyRwLock::new();

pub const NT_THEME: &str = include_str!("../../../share/themes/nt.toml");
pub const K3_THEME: &str = include_str!("../../../share/themes/2k3.toml");
pub const KDE_THEME: &str = include_str!("../../../share/themes/kde.toml");
pub const MACOS9_THEME: &str = include_str!("../../../share/themes/macos9.toml");
pub const SERENITY_THEME: &str = include_str!("../../../share/themes/serenity.toml");

pub const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("nt", NT_THEME),
    ("2k3", K3_THEME),
    ("kde", KDE_THEME),
    ("macos9", MACOS9_THEME),
    ("serenity", SERENITY_THEME),
];

pub const BUILTIN_THEME_NAMES: &[&str] = &["nt", "2k3", "kde", "macos9", "serenity"];

pub fn install(def: ThemeDef) {
    if let Ok(mut g) = LOADED.write() {
        *g = Some(def);
    }
}

pub fn install_named(name: &str) -> bool {
    let text = BUILTIN_THEMES
        .iter()
        .find(|(n, _)| *n == name)
        .map_or(NT_THEME, |(_, t)| *t);
    match load::from_toml(text) {
        Ok(def) => {
            install(def);
            true
        }
        Err(_) => false,
    }
}

pub fn current() -> Option<ThemeDef> {
    if let Ok(g) = LOADED.read() {
        if let Some(d) = g.clone() {
            return Some(d);
        }
    }
    match load::from_toml(NT_THEME) {
        Ok(def) => {
            install(def.clone());
            Some(def)
        }
        Err(_) => None,
    }
}

pub fn draw_element(g: &dyn antibox_gfx::backend::GraphicsContext, name: &str, x: i16, y: i16, w: u16, h: u16) -> bool {
    let Some(def) = current() else { return false };
    let Some(ops) = def.element(name) else { return false };
    let s = antibox_gfx::scale::scaled(1).max(1) as i16;
    exec::draw_ops(g, &def, ops, x, y, w as i16, h as i16, s)
}

fn colour_or(name: &str, fallback: u32) -> u32 {
    match current() {
        Some(d) => d.colour(name).unwrap_or(fallback),
        None => fallback,
    }
}

trait FromI64: Sized {
    fn from_i64(v: i64) -> Option<Self>;
}

impl FromI64 for u16 {
    fn from_i64(v: i64) -> Option<Self> {
        Self::try_from(v).ok()
    }
}

impl FromI64 for i16 {
    fn from_i64(v: i64) -> Option<Self> {
        Self::try_from(v).ok()
    }
}

impl FromI64 for u8 {
    fn from_i64(v: i64) -> Option<Self> {
        Self::try_from(v).ok()
    }
}

impl FromI64 for i32 {
    fn from_i64(v: i64) -> Option<Self> {
        Self::try_from(v).ok()
    }
}

impl FromI64 for u32 {
    fn from_i64(v: i64) -> Option<Self> {
        Self::try_from(v).ok()
    }
}

fn metric_or<T: FromI64>(name: &str, fallback: T) -> T {
    match current() {
        Some(d) => d.metric(name).and_then(T::from_i64).unwrap_or(fallback),
        None => fallback,
    }
}

fn flag_or(name: &str, fallback: bool) -> bool {
    match current() {
        Some(d) => d.flag(name).unwrap_or(fallback),
        None => fallback,
    }
}

pub fn chrome_override() -> bool {
    flag_or("override_chrome", false)
}

pub fn outlined() -> bool {
    flag_or("outlined", false)
}

pub fn grad_buttons() -> bool {
    flag_or("grad_buttons", false)
}

pub fn equal_tabs() -> bool {
    flag_or("equal_tabs", false)
}

pub fn symmetric_title_buttons() -> bool {
    flag_or("symmetric_buttons", false)
}

pub fn title_gradient_vertical() -> bool {
    flag_or("title_grad_vertical", false)
}

pub fn title_stipple_enabled() -> bool {
    flag_or("title_stipple", false)
}

pub fn title_stripes() -> bool {
    flag_or("title_stripes", false)
}

pub fn title_icon() -> bool {
    flag_or("title_icon", false)
}

pub fn hide_buttons_inactive() -> bool {
    flag_or("hide_buttons_inactive", false)
}

pub fn frame_outline() -> bool {
    flag_or("frame_outline", false)
}

pub fn title_stripe_hi() -> Colour {
    colour_or("title_stripe_hi", 0)
}

pub fn title_stripe_sh() -> Colour {
    colour_or("title_stripe_sh", 0)
}

fn string_or(name: &str, fallback: &str) -> String {
    match current() {
        Some(d) => d.string(name).unwrap_or(fallback).to_string(),
        None => fallback.to_string(),
    }
}

pub use self::colour::contrast;
pub(crate) use self::colour::mix_rgb;
pub(crate) use self::colour::scale_rgb;
pub(crate) use self::colour::tint_rgb;

pub fn face() -> Colour {
    colour_or("face", 0xC0C0C0)
}
pub fn field() -> Colour {
    colour_or("field", 0xFFFFFF)
}
pub fn list_bg() -> Colour {
    if colour_or("list_bg", 0x0) != 0 {
        colour_or("list_bg", 0x0)
    } else {
        colour_or("field", 0xFFFFFF)
    }
}
pub fn menu_bg() -> Colour {
    if colour_or("menu_bg", 0x0) != 0 {
        colour_or("menu_bg", 0x0)
    } else {
        colour_or("face", 0xC0C0C0)
    }
}
pub fn arrow_colour() -> Colour {
    if colour_or("arrow_colour", 0x0) != 0 {
        colour_or("arrow_colour", 0x0)
    } else {
        colour_or("text", 0x000000)
    }
}

pub fn text() -> Colour {
    contrast(colour_or("text", 0x000000), colour_or("face", 0xC0C0C0))
}
pub fn disabled() -> Colour {
    colour_or("disabled", 0x808080)
}
pub fn sel_bg() -> Colour {
    colour_or("sel_bg", 0x000080)
}
pub fn sel_fg() -> Colour {
    contrast(colour_or("sel_fg", 0xFFFFFF), colour_or("sel_bg", 0x000080))
}
pub fn light() -> Colour {
    colour_or("light", 0xFFFFFF)
}
pub fn face_light() -> Colour {
    colour_or("face_light", 0xC0C0C0)
}
pub fn button_face() -> Colour {
    colour_or("button_face", 0xC0C0C0)
}
pub fn shadow() -> Colour {
    colour_or("shadow", 0x808080)
}
pub fn dark() -> Colour {
    colour_or("dark", 0x000000)
}
pub fn tooltip_bg() -> Colour {
    colour_or("tooltip_bg", 0xFFFFE1)
}
pub fn tooltip_fg() -> Colour {
    contrast(colour_or("tooltip_fg", 0x000000), colour_or("tooltip_bg", 0xFFFFE1))
}
pub fn sel_line() -> Colour {
    colour_or("sel_line", 0x000080)
}
pub fn title_active() -> Colour {
    colour_or("title_active", 0x000080)
}
pub fn title_inactive() -> Colour {
    colour_or("title_inactive", 0x808080)
}
pub fn title_gradient() -> Colour {
    if colour_or("title_gradient", 0x0) != 0 {
        colour_or("title_gradient", 0x0)
    } else {
        title_active()
    }
}
pub fn title_gradient_inactive() -> Colour {
    if colour_or("title_gradient_inactive", 0x0) != 0 {
        colour_or("title_gradient_inactive", 0x0)
    } else {
        title_inactive()
    }
}
pub fn title_text() -> Colour {
    colour_or("title_text", 0xFFFFFF)
}
pub fn title_text_inactive() -> Colour {
    colour_or("title_text_inactive", 0xC0C0C0)
}

pub fn title_inset_base() -> u16 {
    metric_or("title_inset", 0)
}

pub fn title_overlap_base() -> u16 {
    metric_or("title_overlap", 0)
}

pub fn close_gap_base() -> u16 {
    metric_or("close_gap", 0)
}

pub fn title_end_pad_base() -> u16 {
    metric_or("title_end_pad", 0)
}

pub fn ui_font_name() -> String {
    string_or("ui_font_name", "")
}

pub fn clock_format() -> String {
    string_or("clock_format", "")
}

pub fn tray_face() -> Colour {
    if colour_or("tray_face", 0x0) != 0 {
        colour_or("tray_face", 0x0)
    } else if colour_or("taskbar_face", 0x0) != 0 {
        colour_or("taskbar_face", 0x0)
    } else {
        colour_or("face", 0xC0C0C0)
    }
}

pub fn title_buttons() -> String {
    string_or("title_buttons", "xmi")
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

pub fn title_glyph(key: &str) -> Option<Vec<u16>> {
    if let Some(rows) = current().and_then(|d| d.glyph(key).map(<[u16]>::to_vec)) {
        return Some(rows);
    }
    NT_GLYPHS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_vec())
}

pub(crate) fn element_ops(name: &str) -> Option<Vec<dsl::Op>> {
    current().and_then(|d| d.element(name).map(<[dsl::Op]>::to_vec))
}

pub(crate) fn paint_element(
    g: &dyn antibox_gfx::backend::GraphicsContext,
    ops: &[dsl::Op],
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    bg: Colour,
) {
    let Some(def) = current() else { return };
    let s = antibox_gfx::scale::scaled(1).max(1) as i16;
    exec::draw_ops_on(g, &def, ops, x, y, w as i16, h as i16, s, Some(bg));
}

pub fn pad_base() -> u16 {
    metric_or("pad", 4)
}

pub fn gap_base() -> u16 {
    metric_or("gap", 2)
}

pub fn item_gap_base() -> i16 {
    metric_or("item_gap", 2)
}

pub fn title_height_base() -> u16 {
    metric_or("title_height", 18)
}

pub fn border_base() -> u16 {
    metric_or("border", 4)
}

pub fn border_bottom_base() -> u16 {
    metric_or("border_bottom", 4)
}

pub fn button_base() -> u16 {
    metric_or("button_base", 16)
}

pub fn title_button_inset_base() -> u16 {
    if metric_or("title_button_inset", 0) != 0 {
        metric_or("title_button_inset", 0)
    } else {
        metric_or("button_inset", 4)
    }
}

pub fn title_justify_default() -> u8 {
    metric_or("title_justify", 0)
}

pub fn taskbar_justify_default() -> String {
    string_or("taskbar_justify", "left")
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
    normalize_side(&string_or("title_side", "top")).unwrap_or("top")
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
    match metric_or("taskbar_item_chars", 0) {
        0 => 24,
        v => v,
    }
}

pub fn taskbar_item_pct() -> u16 {
    metric_or("taskbar_item_pct", 0)
}

pub fn menu_sel_bg() -> Colour {
    if colour_or("menu_sel_bg", 0x0) != 0 {
        colour_or("menu_sel_bg", 0x0)
    } else {
        colour_or("title_active", 0x000080)
    }
}

pub fn menu_sel_fg() -> Colour {
    if colour_or("menu_sel_bg", 0x0) != 0 {
        contrast(colour_or("menu_sel_fg", 0x0), colour_or("menu_sel_bg", 0x0))
    } else {
        contrast(colour_or("title_text", 0xFFFFFF), colour_or("title_active", 0x000080))
    }
}

pub fn title_layout() -> (String, String) {
    (
        string_or("title_layout_left", "sp"),
        string_or("title_layout_right", "xmir"),
    )
}

pub fn sunken_depth() -> u16 {
    metric_or("sunken_depth", 2).clamp(1, 2)
}

pub fn corner_radius_px() -> u16 {
    let r = metric_or("corner_radius", 0);
    if r == 0 {
        0
    } else {
        antibox_gfx::scale::scaled(r as i32).max(1) as u16
    }
}

pub fn frame_radius_px() -> u16 {
    let r = metric_or("frame_radius", 0);
    if r == 0 {
        corner_radius_px()
    } else {
        antibox_gfx::scale::scaled(r as i32).max(1) as u16
    }
}

pub fn button_radius_px() -> u16 {
    let r = metric_or("button_radius", 0);
    if r == 0 {
        corner_radius_px()
    } else {
        antibox_gfx::scale::scaled(r as i32).max(1) as u16
    }
}

pub fn graph_bg() -> Colour {
    face()
}

static GRAPH_COLOURS: LazyRwLock<(Vec<Colour>, Option<Colour>)> =
    LazyRwLock::new();

pub fn set_graph_colours(series: Vec<Colour>, heat: Option<Colour>) {
    if let Ok(mut g) = GRAPH_COLOURS.write() {
        *g = (series, heat);
    }
}

pub fn graph_series(i: usize) -> Colour {
    if let Ok(g) = GRAPH_COLOURS.read() {
        if !g.0.is_empty() {
            return g.0[i % g.0.len()];
        }
    }
    let derived = [sel_bg(), title_active(), tint_rgb(sel_bg(), 0.5), shadow()];
    derived[i % 4]
}

pub fn graph_heat() -> Colour {
    if let Ok(g) = GRAPH_COLOURS.read() {
        if let Some(h) = g.1 {
            return h;
        }
    }
    0xC82020
}

#[cfg(test)]
mod theme_gen;
