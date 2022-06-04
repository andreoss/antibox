#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmWindowClass {
    InputOutput,
    InputOnly,
    CopyFromParent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootWindow(u32);

impl RootWindow {
    pub const fn new(xid: u32) -> RootWindow {
        RootWindow(xid)
    }

    pub const fn as_parent(self) -> u32 {
        self.0
    }

    pub const fn read_id(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropMode {
    Replace,
    Prepend,
    Append,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackMode {
    Above,
    Below,
    TopIf,
    BottomIf,
    Opposite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabMode {
    Sync,
    Async,
}

#[derive(Debug, Clone, Copy)]
pub struct PointerGrab {
    pub owner_events: bool,
    pub window: u32,
    pub event_mask: EventMask,
    pub pointer_mode: GrabMode,
    pub keyboard_mode: GrabMode,
    pub confine_to: u32,
    pub cursor: u32,
    pub time: u32,
}

impl PointerGrab {
    pub const fn new(window: u32, event_mask: EventMask) -> PointerGrab {
        PointerGrab {
            owner_events: false,
            window,
            event_mask,
            pointer_mode: GrabMode::Async,
            keyboard_mode: GrabMode::Async,
            confine_to: 0,
            cursor: 0,
            time: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonGrabSpec {
    pub button: u8,
    pub modifiers: u16,
    pub window: u32,
    pub owner_events: bool,
    pub event_mask: EventMask,
    pub pointer_mode: GrabMode,
    pub keyboard_mode: GrabMode,
    pub confine_to: u32,
    pub cursor: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMask(u64);

impl EventMask {
    pub const NO_EVENT: Self = EventMask(0);
    pub const KEY_PRESS: Self = EventMask(1 << 0);
    pub const KEY_RELEASE: Self = EventMask(1 << 1);
    pub const BUTTON_PRESS: Self = EventMask(1 << 2);
    pub const BUTTON_RELEASE: Self = EventMask(1 << 3);
    pub const POINTER_MOTION: Self = EventMask(1 << 6);
    pub const ENTER_WINDOW: Self = EventMask(1 << 4);
    pub const LEAVE_WINDOW: Self = EventMask(1 << 5);
    pub const FOCUS_CHANGE: Self = EventMask(1 << 21);
    pub const EXPOSURE: Self = EventMask(1 << 15);
    pub const PROPERTY_CHANGE: Self = EventMask(1 << 22);
    pub const BUTTON_MOTION: Self = EventMask(1 << 13);
    pub const STRUCTURE_NOTIFY: Self = EventMask(1 << 17);
    pub const SUBSTRUCTURE_NOTIFY: Self = EventMask(1 << 19);
    pub const SUBSTRUCTURE_REDIRECT: Self = EventMask(1 << 20);

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn bits(&self) -> u64 {
        self.0
    }
}

impl std::ops::BitOr for EventMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> EventMask {
        EventMask(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for EventMask {
    type Output = Self;
    fn bitand(self, rhs: Self) -> EventMask {
        EventMask(self.0 & rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyButMask(pub u16);

impl KeyButMask {
    pub const MOD1: Self = KeyButMask(1 << 3);
    pub const CONTROL: Self = KeyButMask(1 << 2);
    pub const SHIFT: Self = KeyButMask(1 << 0);

    pub const fn new(bits: u16) -> KeyButMask {
        KeyButMask(bits)
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub const fn bits(&self) -> u16 {
        self.0
    }
}

impl std::ops::BitOr for KeyButMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> KeyButMask {
        KeyButMask(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerState {
    pub root_x: i16,
    pub root_y: i16,
    pub mask: KeyButMask,
}

#[derive(Debug, Clone)]
pub struct KeyboardMapping {
    pub keysyms_per_keycode: u8,
    pub keysyms: Vec<u32>,
}

pub struct ModifierMapping {
    pub keycodes_per_modifier: [Vec<u8>; 8],
}

#[derive(Debug, Clone)]
pub struct QueryTreeResult {
    pub root: u32,
    pub parent: u32,
    pub children: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapState {
    Unmapped,
    Unviewable,
    Viewable,
}

#[derive(Debug, Clone)]
pub struct WindowAttributes {
    pub override_redirect: bool,
    pub map_state: MapState,
    pub depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorInfo {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyboardInfo {
    pub rules: String,
    pub model: String,
    pub layouts: String,
    pub variants: String,
    pub options: String,
    pub group: usize,
}

impl KeyboardInfo {
    pub fn active_layout(&self) -> Option<&str> {
        self.layouts.split(',').nth(self.group).map(str::trim)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeOp {
    Set,
    Union,
    Intersect,
    Subtract,
}

pub const DEFAULT_UI_FONT: &str = "fixed";

static UI_FONT: crate::sync::atomic::LazyRwLock<Option<String>> = crate::sync::atomic::LazyRwLock::new();

pub fn set_ui_font(family: &str) {
    let f = family.trim();
    if let Ok(mut g) = UI_FONT.write() {
        *g = if f.is_empty() {
            None
        } else {
            Some(f.to_string())
        };
    }
}

pub fn ui_font() -> String {
    UI_FONT
        .read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| DEFAULT_UI_FONT.to_string())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FontRole {
    Title = 0,
    Menu,
    Switch,
    NormalTaskBar,
    ActiveTaskBar,
    Clock,
    Input,
    ToolTip,
}

const FONT_ROLE_COUNT: usize = 8;

#[derive(Clone)]
struct ElementFont {
    family: String,
    size: u16,
    bold: bool,
    italic: bool,
}

use std::cell::RefCell;

thread_local! {
    static ELEMENT_FONTS: RefCell<[Option<ElementFont>; FONT_ROLE_COUNT]> =
        const { RefCell::new([None, None, None, None, None, None, None, None]) };
}
pub fn parse_font_desc(s: &str) -> Option<(String, u16, bool, bool)> {
    let toks: Vec<&str> = s.split_whitespace().collect();
    if toks.is_empty() {
        return None;
    }
    let mut bold = false;
    let mut italic = false;
    let mut size = 0u16;
    let mut end = toks.len();
    while end > 0 {
        let t = toks[end - 1];
        if t.eq_ignore_ascii_case("bold") {
            bold = true;
            end -= 1;
        } else if t.eq_ignore_ascii_case("italic") {
            italic = true;
            end -= 1;
        } else if let Ok(n) = t.parse::<u16>() {
            size = n;
            end -= 1;
        } else {
            break;
        }
    }
    let family = toks[..end].join(" ");
    if family.is_empty() {
        return None;
    }
    Some((family, size, bold, italic))
}

pub fn set_element_font(role: FontRole, desc: &str) {
    let parsed = parse_font_desc(desc.trim()).map(|(family, size, bold, italic)| ElementFont {
        family,
        size,
        bold,
        italic,
    });
    ELEMENT_FONTS.with(|g| g.borrow_mut()[role as usize] = parsed);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontSpec {
    pub family: String,
    pub size: u16,
    pub bold: bool,
    pub italic: bool,
}

impl FontSpec {
    pub fn new(family: &str, size: u16) -> FontSpec {
        FontSpec {
            family: family.to_string(),
            size,
            bold: false,
            italic: false,
        }
    }

    pub fn ui(size: u16) -> FontSpec {
        let scaled = crate::scale::scaled(size as i32).clamp(1, 255) as u16;
        Self::new(&ui_font(), scaled)
    }

    pub fn role(role: FontRole, size: u16) -> FontSpec {
        Self::role_styled(role, size, false, false)
    }

    pub fn role_styled(role: FontRole, size: u16, bold: bool, italic: bool) -> FontSpec {
        let overridden = ELEMENT_FONTS.with(|g| g.borrow()[role as usize].clone());
        if let Some(ef) = overridden {
            let pt = if ef.size > 0 { ef.size } else { size };
            let scaled = crate::scale::scaled(pt as i32).clamp(1, 255) as u16;
            return FontSpec {
                family: ef.family.clone(),
                size: scaled,
                bold: ef.bold,
                italic: ef.italic,
            };
        }
        let mut f = Self::ui(size);
        f.bold = bold;
        f.italic = italic;
        f
    }

    #[must_use]
    pub fn bold(mut self) -> FontSpec {
        self.bold = true;
        self
    }

    #[must_use]
    pub fn italic(mut self) -> FontSpec {
        self.italic = true;
        self
    }
}

pub type TimerCallback = Box<dyn FnMut() + Send>;




#[cfg(test)]
#[path = "types_ui_font_tests.rs"]
mod ui_font_tests;
