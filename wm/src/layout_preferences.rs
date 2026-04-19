use antibox_core::sync::atomic::LazyRwLock;

#[derive(Clone, PartialEq, Eq)]
pub struct LayoutPrefs {
    title_left: Option<String>,
    title_right: Option<String>,
    title_justify: i16,
    title_height: i16,
    taskbar_align: u8,
    taskbar_grouping: bool,
    taskbar_layout: Option<Vec<Widget>>,
    show: [bool; Widget::COUNT],
    taskbar_titles: bool,
    taskbar_double: bool,
    taskbar_wheel: bool,
    pager_numbers: bool,
    pager_preview: bool,
    taskbar_preview: bool,
    omni_preview: bool,
    taskbar_position: u8,
    taskbar_width_px: u16,
    taskbar_rows: u8,
    taskbar_locked: bool,
    pager_hover: bool,
    pager_grid: bool,
    winlist_preview: bool,
    minimize_animation: bool,
}

impl Default for LayoutPrefs {
    fn default() -> Self {
        Self {
            title_left: None,
            title_right: None,
            title_justify: -1,
            title_height: -1,
            taskbar_align: TASKBAR_ALIGN_THEME,
            taskbar_grouping: false,
            taskbar_layout: None,
            show: [false; Widget::COUNT],
            taskbar_titles: true,
            taskbar_double: false,
            taskbar_wheel: true,
            pager_numbers: false,
            pager_preview: true,
            taskbar_preview: true,
            omni_preview: true,
            taskbar_position: 1,
            taskbar_width_px: 0,
            taskbar_rows: 1,
            taskbar_locked: true,
            pager_hover: true,
            pager_grid: false,
            winlist_preview: true,
            minimize_animation: false,
        }
    }
}

static STATE: LazyRwLock<LayoutPrefs> = LazyRwLock::new();

fn with<R>(f: impl FnOnce(&LayoutPrefs) -> R) -> R {
    let guard = STATE.read().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&guard)
}

fn with_mut(f: impl FnOnce(&mut LayoutPrefs)) {
    let mut guard = STATE.write().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard);
}

pub fn snapshot() -> LayoutPrefs {
    with(Clone::clone)
}

pub fn install(prefs: LayoutPrefs) {
    with_mut(|s| *s = prefs);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Widget {
    Menu,
    Launch,
    Workspaces,
    Windows,
    Tray,
    Cpu,
    Mem,
    Net,
    Mpris,
    PowerAudio,
    Keyboard,
    Clock,
    ShowDesktop,
}

impl Widget {
    pub const COUNT: usize = 13;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Menu,
        Self::Launch,
        Self::Workspaces,
        Self::Windows,
        Self::Tray,
        Self::Cpu,
        Self::Mem,
        Self::Net,
        Self::Mpris,
        Self::PowerAudio,
        Self::Keyboard,
        Self::Clock,
        Self::ShowDesktop,
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
        "mpris" => Some(Widget::Mpris),
        "battery" | "power" | "batt" | "audio" | "volume" | "sound" => Some(Widget::PowerAudio),
        "keyboard" | "kbd" | "layout" => Some(Widget::Keyboard),
        "clock" | "time" => Some(Widget::Clock),
        "launch" | "quicklaunch" | "launchers" => Some(Widget::Launch),
        "showdesktop" | "desktop" => Some(Widget::ShowDesktop),
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
    let parsed = parse_layout(spec);
    with_mut(|s| s.taskbar_layout = parsed);
}

pub const DEFAULT_TITLE_LEFT: &str = "sp";
pub const DEFAULT_TITLE_RIGHT: &str = "xmir";

pub fn apply() {
    set_title_layout("sp", "xmir", -1);
    set_title_height(-1);
    antibox_ui::theme::set_title_side_override("");
    set_taskbar_align("");
    set_pager_style(false, true);
    set_pager_grid(false);
    set_minimize_animation(false);
    with_mut(|s| {
        s.taskbar_grouping = false;
        s.taskbar_layout = parse_layout("");
        s.taskbar_titles = true;
        s.taskbar_double = false;
        s.taskbar_wheel = true;
        s.show = [true; Widget::COUNT];
    });
}

pub fn apply_widget_prefs(prefs: &crate::wmconfig::Prefs) {
    set_taskbar_layout(&prefs.taskbar.layout);
    set_taskbar_preview(prefs.taskbar.preview);
    set_omni_preview(prefs.omni.preview);
    set_winlist_preview(prefs.winlist.preview);
    set_pager_hover_preview(prefs.pager.preview);
    set_pager_grid(prefs.pager.style == "grid");
}

pub fn taskbar_layout() -> Option<Vec<Widget>> {
    with(|s| s.taskbar_layout.clone())
}

pub fn taskbar_wants(w: Widget) -> bool {
    with(|s| match &s.taskbar_layout {
        Some(list) => list.contains(&w),
        None => s.show[w.index()],
    })
}

pub fn taskbar_grouping() -> bool {
    with(|s| s.taskbar_grouping)
}

pub fn set_taskbar_show_titles(titles: bool) {
    with_mut(|s| s.taskbar_titles = titles);
}

pub fn taskbar_show_titles() -> bool {
    with(|s| s.taskbar_titles)
}

pub fn set_taskbar_double_height(v: bool) {
    with_mut(|s| s.taskbar_double = v);
}

pub fn taskbar_double_height() -> bool {
    with(|s| s.taskbar_double)
}

pub fn set_taskbar_wheel(v: bool) {
    with_mut(|s| s.taskbar_wheel = v);
}

pub fn taskbar_wheel_enabled() -> bool {
    with(|s| s.taskbar_wheel)
}

pub fn set_pager_style(numbers: bool, preview: bool) {
    with_mut(|s| {
        s.pager_numbers = numbers;
        s.pager_preview = preview;
    });
}

pub fn pager_numbers() -> bool {
    with(|s| s.pager_numbers)
}

pub fn pager_preview() -> bool {
    with(|s| s.pager_preview)
}

pub fn set_pager_hover_preview(v: bool) {
    with_mut(|s| s.pager_hover = v);
}

pub fn pager_hover_preview() -> bool {
    with(|s| s.pager_hover)
}

pub fn set_pager_grid(v: bool) {
    with_mut(|s| s.pager_grid = v);
}

pub fn pager_grid() -> bool {
    with(|s| s.pager_grid)
}

pub fn set_winlist_preview(v: bool) {
    with_mut(|s| s.winlist_preview = v);
}

pub fn winlist_preview() -> bool {
    with(|s| s.winlist_preview)
}

pub fn set_taskbar_position(position: crate::taskbar::TaskBarPosition) {
    with_mut(|s| s.taskbar_position = position.code());
}

pub fn taskbar_position() -> crate::taskbar::TaskBarPosition {
    crate::taskbar::TaskBarPosition::from_code(with(|s| s.taskbar_position))
}

pub fn taskbar_upright() -> bool {
    taskbar_position().is_upright()
}

pub fn set_taskbar_width_px(px: i64) {
    let px = px.clamp(0, 1000) as u16;
    with_mut(|s| s.taskbar_width_px = px);
}

pub fn taskbar_width_px() -> u16 {
    with(|s| s.taskbar_width_px)
}

pub fn set_taskbar_rows(rows: i64) {
    let rows = rows.clamp(1, 4) as u8;
    with_mut(|s| s.taskbar_rows = rows);
}

pub fn taskbar_rows() -> u16 {
    u16::from(with(|s| s.taskbar_rows).max(1))
}

pub fn set_taskbar_locked(v: bool) {
    with_mut(|s| s.taskbar_locked = v);
}

pub fn taskbar_locked() -> bool {
    with(|s| s.taskbar_locked)
}

pub fn set_taskbar_preview(v: bool) {
    with_mut(|s| s.taskbar_preview = v);
}

pub fn taskbar_preview() -> bool {
    with(|s| s.taskbar_preview)
}

pub fn set_omni_preview(v: bool) {
    with_mut(|s| s.omni_preview = v);
}

pub fn omni_preview() -> bool {
    with(|s| s.omni_preview)
}

pub fn set_minimize_animation(v: bool) {
    with_mut(|s| s.minimize_animation = v);
}

pub fn set_title_height(base: i64) {
    let base = base.clamp(-1, 128) as i16;
    with_mut(|s| s.title_height = base);
}

pub fn title_height_override() -> Option<u16> {
    let v = with(|s| s.title_height);
    if v > 0 {
        Some(v as u16)
    } else {
        None
    }
}

pub fn set_title_layout(left: &str, right: &str, justify: i64) {
    let justify = justify.clamp(-1, 100) as i16;
    let (left, right) = (left.to_string(), right.to_string());
    with_mut(|s| {
        s.title_left = Some(left);
        s.title_right = Some(right);
        s.title_justify = justify;
    });
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
    with_mut(|s| s.taskbar_align = v);
}

fn taskbar_code_resolved() -> u8 {
    let v = with(|s| s.taskbar_align);
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
    let pref = with(|s| s.title_left.clone()).unwrap_or_else(|| DEFAULT_TITLE_LEFT.to_string());
    let (theme_left, _) = antibox_ui::theme::title_layout();
    if pref == DEFAULT_TITLE_LEFT && !theme_left.is_empty() {
        return theme_left.to_string();
    }
    pref
}

pub fn title_buttons_right() -> String {
    let pref = with(|s| s.title_right.clone()).unwrap_or_else(|| DEFAULT_TITLE_RIGHT.to_string());
    let (_, theme_right) = antibox_ui::theme::title_layout();
    if pref == DEFAULT_TITLE_RIGHT && !theme_right.is_empty() {
        return theme_right.to_string();
    }
    pref
}

pub fn title_justify() -> u8 {
    let v = with(|s| s.title_justify);
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
