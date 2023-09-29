use std::cell::RefCell;
use antibox_core::sync::atomic::{AtomicI16, AtomicU8, LazyRwLock};
use std::sync::atomic::{AtomicBool, Ordering};

static TITLE_LEFT: LazyRwLock<Option<String>> = LazyRwLock::new();
static TITLE_RIGHT: LazyRwLock<Option<String>> = LazyRwLock::new();
static TITLE_JUSTIFY: AtomicI16 = AtomicI16::new(-1);
static TITLE_HEIGHT: AtomicI16 = AtomicI16::new(-1);
static TASKBAR_ALIGN: AtomicU8 = AtomicU8::new(255);
static TASKBAR_GROUPING: AtomicBool = AtomicBool::new(false);
static TASKBAR_LAYOUT: LazyRwLock<Option<Vec<Widget>>> = LazyRwLock::new();
thread_local! {
    static SHOW: RefCell<[bool; Widget::COUNT]> = const { RefCell::new([false; Widget::COUNT]) };
}
static TASKBAR_TITLES: AtomicBool = AtomicBool::new(true);
static TASKBAR_DOUBLE: AtomicBool = AtomicBool::new(false);
static TASKBAR_WHEEL: AtomicBool = AtomicBool::new(true);
static PAGER_NUMBERS: AtomicBool = AtomicBool::new(false);
static MENU_ON_SUPER_TAP: AtomicBool = AtomicBool::new(true);
static PAGER_PREVIEW: AtomicBool = AtomicBool::new(true);
static MINIMIZE_ANIMATION: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Widget {
    Menu,
    Workspaces,
    Windows,
    Tray,
    Cpu,
    Mem,
    Net,
    PowerAudio,
    Keyboard,
    Clock,
}

impl Widget {
    pub const COUNT: usize = 10;
    pub const ALL: [Self; Self::COUNT] = [
        Widget::Menu,
        Widget::Workspaces,
        Widget::Windows,
        Widget::Tray,
        Widget::Cpu,
        Widget::Mem,
        Widget::Net,
        Widget::PowerAudio,
        Widget::Keyboard,
        Widget::Clock,
    ];
    const fn index(self) -> usize {
        self as usize
    }
}

fn widget_token(name: &str) -> Option<Widget> {
    match name.trim().to_ascii_lowercase().as_str() {
        "menu" | "start" => Some(Widget::Menu),
        "workspaces" | "pager" => Some(Widget::Workspaces),
        "windows" | "tasks" | "taskbar" | "windowlist" => Some(Widget::Windows),
        "tray" | "systray" | "systemtray" => Some(Widget::Tray),
        "cpu" => Some(Widget::Cpu),
        "mem" | "memory" | "ram" => Some(Widget::Mem),
        "net" | "network" => Some(Widget::Net),
        "battery" | "power" | "batt" | "audio" | "volume" | "sound" => Some(Widget::PowerAudio),
        "keyboard" | "kbd" | "layout" => Some(Widget::Keyboard),
        "clock" | "time" => Some(Widget::Clock),
        _ => None,
    }
}

fn parse_layout(spec: &str) -> Option<Vec<Widget>> {
    let mut widgets: Vec<Widget> = Vec::new();
    for w in spec
        .split(&[' ', ',', '\t', '|'][..])
        .filter_map(widget_token)
    {
        if !widgets.contains(&w) {
            widgets.push(w);
        }
    }
    if !widgets.is_empty() {
        Some(widgets)
    } else {
        None
    }
}

pub fn set_taskbar_layout(spec: &str) {
    if let Ok(mut g) = TASKBAR_LAYOUT.write() {
        *g = parse_layout(spec);
    }
}

pub const DEFAULT_TITLE_LEFT: &str = "sp";
pub const DEFAULT_TITLE_RIGHT: &str = "xmir";

pub fn apply() {
    set_title_layout("sp", "xmir", -1);
    set_title_height(-1);
    antibox_ui::theme::set_title_side_override("");
    set_taskbar_align("");
    TASKBAR_GROUPING.store(false, Ordering::Relaxed);
    MENU_ON_SUPER_TAP.store(true, Ordering::Relaxed);
    if let Ok(mut g) = TASKBAR_LAYOUT.write() {
        *g = parse_layout("");
    }
    set_taskbar_show_titles(true);
    set_taskbar_double_height(false);
    set_taskbar_wheel(true);
    set_pager_style(false, true);
    set_minimize_animation(false);
    let present = |w: Widget| match w {
        Widget::Menu => false,
        Widget::Workspaces => true,
        Widget::Windows => true,
        Widget::Tray => true,
        Widget::Cpu => true,
        Widget::Mem => true,
        Widget::Net => true,
        Widget::PowerAudio => true,
        Widget::Keyboard => true,
        Widget::Clock => true,
    };
    for w in Widget::ALL.iter().cloned() {
        SHOW.with(|s| s.borrow_mut()[w.index()] = present(w));
    }
}

pub fn taskbar_layout() -> Option<Vec<Widget>> {
    TASKBAR_LAYOUT.read().ok().and_then(|g| g.clone())
}

pub fn taskbar_wants(w: Widget) -> bool {
    match taskbar_layout() {
        Some(list) => list.contains(&w),
        None => SHOW.with(|s| s.borrow()[w.index()]),
    }
}

pub fn taskbar_grouping() -> bool {
    TASKBAR_GROUPING.load(Ordering::Relaxed)
}

pub fn set_taskbar_show_titles(titles: bool) {
    TASKBAR_TITLES.store(titles, Ordering::Relaxed);
}

pub fn taskbar_show_titles() -> bool {
    TASKBAR_TITLES.load(Ordering::Relaxed)
}

pub fn set_taskbar_double_height(v: bool) {
    TASKBAR_DOUBLE.store(v, Ordering::Relaxed);
}

pub fn taskbar_double_height() -> bool {
    TASKBAR_DOUBLE.load(Ordering::Relaxed)
}

pub fn set_taskbar_wheel(v: bool) {
    TASKBAR_WHEEL.store(v, Ordering::Relaxed);
}

pub fn taskbar_wheel_enabled() -> bool {
    TASKBAR_WHEEL.load(Ordering::Relaxed)
}

pub fn set_pager_style(numbers: bool, preview: bool) {
    PAGER_NUMBERS.store(numbers, Ordering::Relaxed);
    PAGER_PREVIEW.store(preview, Ordering::Relaxed);
}

pub fn set_menu_on_super_tap(v: bool) {
    MENU_ON_SUPER_TAP.store(v, Ordering::Relaxed);
}

pub fn menu_on_super_tap() -> bool {
    MENU_ON_SUPER_TAP.load(Ordering::Relaxed)
}

pub fn pager_numbers() -> bool {
    PAGER_NUMBERS.load(Ordering::Relaxed)
}

pub fn pager_preview() -> bool {
    PAGER_PREVIEW.load(Ordering::Relaxed)
}

pub fn set_minimize_animation(v: bool) {
    MINIMIZE_ANIMATION.store(v, Ordering::Relaxed);
}

pub fn minimize_animation() -> bool {
    MINIMIZE_ANIMATION.load(Ordering::Relaxed)
}

pub fn set_title_height(base: i64) {
    TITLE_HEIGHT.store(base.clamp(-1, 128) as i16, Ordering::Relaxed);
}

pub fn title_height_override() -> Option<u16> {
    let v = TITLE_HEIGHT.load(Ordering::Relaxed);
    if v > 0 { Some(v as u16) } else { None }
}

pub fn set_title_layout(left: &str, right: &str, justify: i64) {
    if let Ok(mut g) = TITLE_LEFT.write() {
        *g = Some(left.to_string());
    }
    if let Ok(mut g) = TITLE_RIGHT.write() {
        *g = Some(right.to_string());
    }
    TITLE_JUSTIFY.store(justify.clamp(-1, 100) as i16, Ordering::Relaxed);
}

const TASKBAR_ALIGN_THEME: u8 = 255;

fn taskbar_code(name: &str) -> Option<u8> {
    match name.trim().to_ascii_lowercase().as_str() {
        "left" => Some(0),
        "center" | "centre" => Some(1),
        "right" => Some(2),
        "fill" | "fit" | "stretch" => Some(3),
        _ => None,
    }
}

pub fn set_taskbar_align(name: &str) {
    let v = taskbar_code(name).unwrap_or(TASKBAR_ALIGN_THEME);
    TASKBAR_ALIGN.store(v, Ordering::Relaxed);
}

fn taskbar_code_resolved() -> u8 {
    let v = TASKBAR_ALIGN.load(Ordering::Relaxed);
    if v == TASKBAR_ALIGN_THEME {
        taskbar_code(&antibox_ui::theme::taskbar_justify_default()).unwrap_or(0)
    } else {
        v
    }
}

pub fn taskbar_fill() -> bool {
    taskbar_code_resolved() == 3
}

pub fn title_buttons_left() -> String {
    let pref = TITLE_LEFT
        .read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| DEFAULT_TITLE_LEFT.to_string());

    let (theme_left, _) = antibox_ui::theme::title_layout();
    if pref == DEFAULT_TITLE_LEFT && !theme_left.is_empty() {
        return theme_left.to_string();
    }
    pref
}

pub fn title_buttons_right() -> String {
    let pref = TITLE_RIGHT
        .read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| DEFAULT_TITLE_RIGHT.to_string());
    let (_, theme_right) = antibox_ui::theme::title_layout();
    if pref == DEFAULT_TITLE_RIGHT && !theme_right.is_empty() {
        return theme_right.to_string();
    }
    pref
}

pub fn title_justify() -> u8 {
    let v = TITLE_JUSTIFY.load(Ordering::Relaxed);
    if v < 0 {
        antibox_ui::theme::title_justify_default()
    } else {
        v as u8
    }
}

pub fn taskbar_align() -> antibox_ui::widget::LabelAlign {
    match taskbar_code_resolved() {
        1 | 3 => antibox_ui::widget::LabelAlign::Center,
        2 => antibox_ui::widget::LabelAlign::Right,
        _ => antibox_ui::widget::LabelAlign::Left,
    }
}

#[cfg(test)]
#[path = "layout_preferences_tests.rs"]
mod tests;
