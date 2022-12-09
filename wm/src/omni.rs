use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::*;
use antibox_core::keysyms::{
    KEY_Down, KEY_End, KEY_Escape, KEY_Home, KEY_Left, KEY_Next, KEY_Prior, KEY_Return, KEY_Right,
    KEY_Tab, KEY_Up,
};
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use antibox_core::scale::scaled;
use antibox_ui::searchbar::{SearchBar, SearchEvent};
use std::sync::Arc;

pub struct OmniItem {
    pub title: String,
    pub class: String,
    pub client_id: u32,
    pub workspace: u32,
    pub icon: PixmapData,
    pub run: bool,
    pub marked: bool,
    pub command: Vec<String>,
}

pub enum OmniOutcome {
    Consumed,
    Activate(u32),
    Run(String),
    Launch(Vec<String>),
    WindowOp { target: u32, op: OmniWinOp },
    Close,
    CloseMarked,
    KillMarked,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OmniWinOp {
    Close,
    Kill,
    SendTo(u32),
    Join(u32),
    ToggleMark,
    Separator,
    CloseMarked,
    KillMarked,
    UnmarkAll,
}

#[derive(Clone, Copy)]
enum OpAction {
    Do(OmniWinOp),
    OpenSend,
    OpenJoin,
    CycleSort,
    ToggleGrouping,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OmniSort {
    Natural,
    Title,
    Class,
    Workspace,
    Window,
}

impl Default for OmniSort {
    fn default() -> OmniSort {
        OmniSort::Natural
    }
}

impl OmniSort {
    pub const fn next(self) -> Self {
        match self {
            Self::Natural => Self::Title,
            Self::Title => Self::Class,
            Self::Class => Self::Workspace,
            Self::Workspace => Self::Window,
            Self::Window => Self::Natural,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Natural => "natural",
            Self::Title => "title",
            Self::Class => "class",
            Self::Workspace => "workspace",
            Self::Window => "window",
        }
    }
}

fn cmp_ci(a: &str, b: &str) -> std::cmp::Ordering {
    a.chars()
        .map(|c| c.to_ascii_lowercase())
        .cmp(b.chars().map(|c| c.to_ascii_lowercase()))
}

fn compare_windows(
    mode: OmniSort,
    a: (&str, &str, u32, u32),
    b: (&str, &str, u32, u32),
) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (at, ac, aw, ai) = a;
    let (bt, bc, bw, bi) = b;
    match mode {
        OmniSort::Natural => Ordering::Equal,
        OmniSort::Title => cmp_ci(at, bt).then(ai.cmp(&bi)),
        OmniSort::Class => cmp_ci(ac, bc).then(cmp_ci(at, bt)).then(ai.cmp(&bi)),
        OmniSort::Workspace => aw.cmp(&bw).then(cmp_ci(at, bt)).then(ai.cmp(&bi)),
        OmniSort::Window => ai.cmp(&bi),
    }
}

fn compare_items(mode: OmniSort, a: &OmniItem, b: &OmniItem) -> std::cmp::Ordering {
    match (a.run, b.run) {
        (false, true) => std::cmp::Ordering::Less,
        (true, false) => std::cmp::Ordering::Greater,
        (true, true) => std::cmp::Ordering::Equal,
        (false, false) => compare_windows(
            mode,
            (&a.title, &a.class, a.workspace, a.client_id),
            (&b.title, &b.class, b.workspace, b.client_id),
        ),
    }
}

struct OpLevel {
    all: Vec<(String, OpAction)>,
    rows: Vec<(String, OpAction)>,
    selected: usize,
}

impl OpLevel {
    fn new(all: Vec<(String, OpAction)>) -> Self {
        Self {
            rows: all.clone(),
            all,
            selected: 0,
        }
    }

    fn apply_filter(&mut self, needle: &str) {
        let needle = needle.to_lowercase();
        self.rows = if needle.is_empty() {
            self.all.clone()
        } else {
            self.all
                .iter()
                .filter(|(l, _)| !l.is_empty() && l.to_lowercase().contains(&needle))
                .cloned()
                .collect()
        };
        self.selected = crate::menu::next_selectable(&self.rows, None, 1).unwrap_or(0);
    }
}

impl crate::menu::MenuItem for (String, OpAction) {
    fn is_separator(&self) -> bool {
        self.0.is_empty()
    }
}

pub struct Omni {
    pub window: Option<Box<dyn WindowHandle>>,
    pub bar: Option<SearchBar>,
    pub items: Vec<OmniItem>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub offset: usize,
    pub pos: Point,
    pub visible: bool,
    pub run_mode: bool,
    pub run_items: Vec<OmniItem>,
    run_history: Vec<String>,
    mapping: Option<KeyboardMapping>,
    min_keycode: u8,
    menu_target: u32,
    menu_levels: Vec<OpLevel>,
    saved_query: String,
    sort: OmniSort,
    group_override: Option<bool>,
    ws_count: u32,
    ws_names: Vec<String>,
    win_list: Vec<(u32, String)>,
    path_cmds: Option<Vec<String>>,
    apps: Option<Vec<crate::desktop_apps::DesktopApp>>,
    panel_w: u16,
    max_rows: usize,
}

impl Default for Omni {
    fn default() -> Self {
        Self::new()
    }
}

const DEFAULT_ROWS: usize = 14;
const RUN_HISTORY_MAX: usize = 50;

fn run_history_file() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".cache")))?;
    Some(base.join("antibox").join("run_history"))
}

fn load_run_history() -> Vec<String> {
    let text = match run_history_file().and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(t) => t,
        None => return Vec::new(),
    };
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .take(RUN_HISTORY_MAX)
        .collect()
}

fn save_run_history(history: &[String]) {
    let path = match run_history_file() {
        Some(p) => p,
        None => return,
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&path, history.join("\n"));
}

fn longest_common_prefix<'a>(items: &[&'a str]) -> &'a str {
    let first = match items.first() {
        Some(f) => f,
        None => return "",
    };
    let mut len = first.len();
    for s in &items[1..] {
        len = len.min(s.len());
        while len > 0
            && (!first.is_char_boundary(len)
                || !s.is_char_boundary(len)
                || first[..len] != s[..len])
        {
            len -= 1;
        }
    }
    &first[..len]
}

fn panel_min_w() -> i32 {
    scaled(340)
}

fn panel_w_for(mon_w: i32) -> u16 {
    (mon_w / 3).max(panel_min_w()).min(mon_w.max(1)) as u16
}

fn rows_for(mon_h: i32) -> usize {
    let list_h = mon_h / 3 - i32::from(bar_h()) - pad() as i32 * 2;
    (list_h / i32::from(row_h())).max(DEFAULT_ROWS as i32) as usize
}

fn bar_h() -> u16 {
    crate::winlist::bar_h() as u16
}

fn row_h() -> u16 {
    crate::winlist::row_h() as u16
}

fn pad() -> i16 {
    crate::winlist::pad()
}

fn icon_size() -> u16 {
    crate::winlist::row_icon_px()
}

impl Omni {
    pub fn new() -> Self {
        Self {
            window: None,
            bar: None,
            items: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            offset: 0,
            pos: Point::new(0, 0),
            visible: false,
            run_mode: false,
            run_items: Vec::new(),
            run_history: load_run_history(),
            mapping: None,
            min_keycode: 8,
            menu_target: 0,
            menu_levels: Vec::new(),
            saved_query: String::new(),
            sort: OmniSort::default(),
            group_override: None,
            ws_count: 1,
            ws_names: Vec::new(),
            win_list: Vec::new(),
            path_cmds: None,
            apps: None,
            panel_w: 0,
            max_rows: DEFAULT_ROWS,
        }
    }

    fn width(&self) -> u16 {
        if self.panel_w != 0 {
            self.panel_w
        } else {
            panel_min_w() as u16
        }
    }

    pub fn record_run(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        self.run_history.retain(|h| h != line);
        self.run_history.insert(0, line.to_string());
        self.run_history.truncate(RUN_HISTORY_MAX);
        save_run_history(&self.run_history);
    }

    const fn cur_items(&self) -> &Vec<OmniItem> {
        if self.run_mode {
            &self.run_items
        } else {
            &self.items
        }
    }

    pub fn is_marked(&self, client_id: u32) -> bool {
        self.items
            .iter()
            .any(|it| it.client_id == client_id && it.marked)
    }

    pub fn owns_window(&self, id: u32) -> bool {
        self.window.as_ref().map(|w| w.id()) == Some(id)
            || self.bar.as_ref().map_or(false, |b| b.owns_window(id))
    }

    fn visible_rows(&self) -> usize {
        if let Some(level) = self.menu_levels.last() {
            return level.rows.len().clamp(1, self.max_rows);
        }
        self.filtered.len().clamp(1, self.max_rows)
    }

    fn height(&self) -> u16 {
        (pad() as u16) * 2 + bar_h() + row_h() * self.visible_rows() as u16
    }

    fn in_submenu(&self) -> bool {
        !self.menu_levels.is_empty()
    }

    fn relayout(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let w = self.width();
        let h = self.height();
        if let Some(win) = self.window.as_ref() {
            let _ = win.configure(None, None, Some(w), Some(h));
        }
        self.paint(conn);
        let _ = conn.flush();
    }

    fn grouping_enabled(&self) -> bool {
        self.group_override
            .unwrap_or_else(crate::layout_preferences::taskbar_grouping)
    }

    fn sort_row_label(&self) -> String {
        format!("Sort: {}", self.sort.label())
    }

    fn group_row_label(&self) -> String {
        let state = if self.grouping_enabled() { "on" } else { "off" };
        format!("Group by class: {}", state)
    }

    fn apply_sort(&mut self) {
        let mode = self.sort;
        self.items.sort_by(|a, b| compare_items(mode, a, b));
    }

    fn refresh_ops_labels(&mut self) {
        let sort_label = self.sort_row_label();
        let group_label = self.group_row_label();
        if let Some(level) = self.menu_levels.last_mut() {
            for (label, action) in level.all.iter_mut().chain(level.rows.iter_mut()) {
                match action {
                    OpAction::CycleSort => *label = sort_label.clone(),
                    OpAction::ToggleGrouping => *label = group_label.clone(),
                    _ => {}
                }
            }
        }
    }

    fn ops_bar_reset(&mut self) {
        if let Some(bar) = self.bar.as_mut() {
            bar.set_text("");
        }
    }

    fn pop_ops_level(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.menu_levels.pop();
        if self.menu_levels.is_empty() {
            let saved = std::mem::take(&mut self.saved_query);
            if let Some(bar) = self.bar.as_mut() {
                bar.set_text(&saved);
            }
            self.refilter(conn);
        } else {
            self.ops_bar_reset();
        }
        self.relayout(conn);
    }

    fn refilter_ops(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let needle = self
            .bar
            .as_ref()
            .map(|b| b.text().to_string())
            .unwrap_or_default();
        if let Some(level) = self.menu_levels.last_mut() {
            level.apply_filter(&needle);
        }
        self.relayout(conn);
    }

    fn resort(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let keep = self.menu_target;
        self.apply_sort();
        self.win_list = self
            .items
            .iter()
            .filter(|it| !it.run)
            .map(|it| (it.client_id, it.title.clone()))
            .collect();
        self.refilter(conn);
        if let Some(pos) = self
            .filtered
            .iter()
            .position(|&i| !self.items[i].run && self.items[i].client_id == keep)
        {
            self.selected = pos;
        }
        self.ensure_visible();
        self.refresh_ops_labels();
        self.relayout(conn);
    }

    fn ws_label(&self, i: u32) -> String {
        self.ws_names
            .get(i as usize)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Workspace {}", i + 1))
    }

    fn open_ops_menu(&mut self, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        if self.run_mode || self.in_submenu() {
            return OmniOutcome::Consumed;
        }
        let i = match self.filtered.get(self.selected) {
            Some(&i) => i,
            None => return OmniOutcome::Consumed,
        };
        let item = &self.items[i];
        if item.run {
            return OmniOutcome::Consumed;
        }
        self.menu_target = item.client_id;
        let marked_count = self.items.iter().filter(|it| it.marked).count();
        let toggle = if item.marked { "Unmark" } else { "Mark" };
        let mut rows: Vec<(String, OpAction)> = vec![
            (toggle.into(), OpAction::Do(OmniWinOp::ToggleMark)),
            (String::new(), OpAction::Do(OmniWinOp::Separator)),
            ("Close".into(), OpAction::Do(OmniWinOp::Close)),
            ("Kill".into(), OpAction::Do(OmniWinOp::Kill)),
        ];
        if self.ws_count > 1 {
            rows.push(("Send to".into(), OpAction::OpenSend));
        }
        if self.win_list.iter().any(|(id, _)| *id != self.menu_target) {
            rows.push(("Join".into(), OpAction::OpenJoin));
        }
        if marked_count > 0 {
            rows.push((String::new(), OpAction::Do(OmniWinOp::Separator)));
            rows.push(("Close marked".into(), OpAction::Do(OmniWinOp::CloseMarked)));
            rows.push(("Kill marked".into(), OpAction::Do(OmniWinOp::KillMarked)));
            if marked_count > 1 {
                rows.push(("Unmark all".into(), OpAction::Do(OmniWinOp::UnmarkAll)));
            }
        }
        rows.push((String::new(), OpAction::Do(OmniWinOp::Separator)));
        rows.push((self.sort_row_label(), OpAction::CycleSort));
        rows.push((self.group_row_label(), OpAction::ToggleGrouping));
        self.menu_levels.push(OpLevel::new(rows));
        self.saved_query = self
            .bar
            .as_ref()
            .map(|b| b.text().to_string())
            .unwrap_or_default();
        self.ops_bar_reset();
        self.relayout(conn);
        OmniOutcome::Consumed
    }

    fn push_send_level(&mut self) {
        let rows: Vec<(String, OpAction)> = (0..self.ws_count)
            .map(|i| (self.ws_label(i), OpAction::Do(OmniWinOp::SendTo(i))))
            .collect();
        self.menu_levels.push(OpLevel::new(rows));
    }

    fn push_join_level(&mut self) {
        let rows: Vec<(String, OpAction)> = self
            .win_list
            .iter()
            .filter(|(id, _)| *id != self.menu_target)
            .map(|(id, title)| (title.clone(), OpAction::Do(OmniWinOp::Join(*id))))
            .collect();
        self.menu_levels.push(OpLevel::new(rows));
    }

    fn submenu_key(&mut self, ks: u32, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        match ks {
            k if k == KEY_Up || k == KEY_Down => {
                let dir = if k == KEY_Up { -1 } else { 1 };
                if let Some(level) = self.menu_levels.last_mut() {
                    if let Some(next) =
                        crate::menu::next_selectable(&level.rows, Some(level.selected), dir)
                    {
                        level.selected = next;
                    }
                }
                self.paint(conn);
            }
            k if k == KEY_Left || k == KEY_Escape => {
                self.pop_ops_level(conn);
            }
            k if k == KEY_Return || k == KEY_Right => {
                return self.activate_submenu(conn);
            }
            _ => {}
        }
        OmniOutcome::Consumed
    }

    fn activate_submenu(&mut self, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        let action = match self
            .menu_levels
            .last()
            .and_then(|level| level.rows.get(level.selected))
        {
            Some((_, action)) => *action,
            None => return OmniOutcome::Consumed,
        };
        match action {
            OpAction::Do(op) => match op {
                OmniWinOp::ToggleMark => {
                    if let Some(&i) = self.filtered.get(self.selected) {
                        self.items[i].marked = !self.items[i].marked;
                    }
                    OmniOutcome::Consumed
                }
                OmniWinOp::Separator => OmniOutcome::Consumed,
                OmniWinOp::UnmarkAll => {
                    for it in &mut self.items {
                        it.marked = false;
                    }
                    OmniOutcome::Consumed
                }
                OmniWinOp::CloseMarked => OmniOutcome::CloseMarked,
                OmniWinOp::KillMarked => OmniOutcome::KillMarked,
                _ => OmniOutcome::WindowOp {
                    target: self.menu_target,
                    op,
                },
            },
            OpAction::OpenSend => {
                self.push_send_level();
                self.ops_bar_reset();
                self.relayout(conn);
                OmniOutcome::Consumed
            }
            OpAction::OpenJoin => {
                self.push_join_level();
                self.ops_bar_reset();
                self.relayout(conn);
                OmniOutcome::Consumed
            }
            OpAction::CycleSort => {
                self.sort = self.sort.next();
                self.resort(conn);
                OmniOutcome::Consumed
            }
            OpAction::ToggleGrouping => {
                self.group_override = Some(!self.grouping_enabled());
                self.refresh_ops_labels();
                self.paint(conn);
                OmniOutcome::Consumed
            }
        }
    }

    fn enter_run_mode(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.run_mode = true;
        self.selected = 0;
        self.offset = 0;
        if let Some(bar) = self.bar.as_mut() {
            bar.set_text("");
        }
        self.refilter(conn);
        self.paint(conn);
    }

    fn exit_run_mode(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.run_mode = false;
        self.selected = 0;
        if let Some(bar) = self.bar.as_mut() {
            bar.set_text("");
        }
        self.refilter(conn);
        self.paint(conn);
    }

    fn activate_selected(&mut self, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        if self.run_mode {
            if let Some(&i) = self.filtered.get(self.selected) {
                return OmniOutcome::Launch(self.run_items[i].command.clone());
            }
            let line = self.bar.as_ref().map(|b| b.text().trim().to_string());
            return match line {
                Some(l) if !l.is_empty() => OmniOutcome::Run(l),
                _ => OmniOutcome::Consumed,
            };
        }
        match self.filtered.get(self.selected).copied() {
            Some(i) if self.items[i].run => {
                self.enter_run_mode(conn);
                OmniOutcome::Consumed
            }
            Some(i) => OmniOutcome::Activate(self.items[i].client_id),
            None => OmniOutcome::Consumed,
        }
    }

    fn monitor_for_pointer(
        &self,
        conn: &Arc<dyn DisplayBackend>,
        monitors: &[MonitorInfo],
    ) -> Rect {
        let full = Rect::new(
            0,
            0,
            conn.screen_width() as i32,
            conn.screen_height() as i32,
        );
        if monitors.is_empty() {
            return full;
        }
        let (px, py) = conn
            .query_pointer(conn.root().read_id())
            .map_or((full.w / 2, full.h / 2), |p| {
                (p.root_x as i32, p.root_y as i32)
            });
        monitors
            .iter()
            .find(|m| {
                px >= m.x as i32
                    && px < m.x as i32 + m.width as i32
                    && py >= m.y as i32
                    && py < m.y as i32 + m.height as i32
            })
            .or_else(|| monitors.first())
            .map_or(full, |m| {
                Rect::new(m.x as i32, m.y as i32, m.width as i32, m.height as i32)
            })
    }

    fn place(&self, mon: Rect, w: u16, h: u16) -> Point {
        Point::new(
            (mon.x + (mon.w - w as i32) / 2).clamp(mon.x, (mon.x + mon.w - w as i32).max(mon.x)),
            (mon.y + (mon.h - h as i32) / 3).clamp(mon.y, (mon.y + mon.h - h as i32).max(mon.y)),
        )
    }

    pub fn show(&mut self, conn: &Arc<dyn DisplayBackend>, wm: &WindowManager<dyn DisplayBackend>) {
        if self.visible {
            return;
        }
        let isz = icon_size();
        let grouping = self.grouping_enabled();

        let mut order: Vec<ClientId> = Vec::new();
        let mut counts: std::collections::HashMap<(String, u32), usize> =
            std::collections::HashMap::new();
        let mut first: std::collections::HashSet<(String, u32)> = std::collections::HashSet::new();
        for id in &wm.insertion_order {
            let fw = match wm.frames.get(id) {
                Some(fw) => fw,
                None => continue,
            };
            if fw.state().skip_taskbar {
                continue;
            }
            if grouping {
                let key = (
                    fw.client().class_instance().unwrap_or("").to_string(),
                    fw.workspace(),
                );
                *counts.entry(key.clone()).or_insert(0) += 1;
                if first.insert(key) {
                    order.push(*id);
                }
            } else {
                order.push(*id);
            }
        }
        self.items = order
            .iter()
            .filter_map(|id| wm.frames.get(id).map(|fw| (*id, fw)))
            .map(|(id, fw)| {
                let class = fw.client().class_instance().unwrap_or("").to_string();
                let icon = crate::icon_render::resolve_client_icon(
                    fw.client().icons(),
                    isz,
                    antibox_ui::theme::field(),
                );
                let count = if grouping {
                    *counts.get(&(class.clone(), fw.workspace())).unwrap_or(&1)
                } else {
                    1
                };
                let base = fw.client().title();
                let title = if count > 1 {
                    format!("{}  ({})", base, count)
                } else {
                    base.to_string()
                };
                OmniItem {
                    title,
                    class,
                    client_id: wm.xid_index.xid_of(id),
                    workspace: fw.workspace(),
                    icon,
                    marked: false,
                    run: false,
                    command: Vec::new(),
                }
            })
            .collect();

        self.items.push(OmniItem {
            title: "Run\u{2026}".to_string(),
            class: "run".to_string(),
            client_id: 0,
            workspace: !0,
            icon: crate::icon_render::resolve_client_icon(&[], isz, antibox_ui::theme::field()),
            marked: false,
            run: true,
            command: Vec::new(),
        });
        self.apply_sort();

        if self.apps.is_none() {
            self.apps = Some(crate::desktop_apps::scan());
        }
        self.run_items = self
            .apps
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|app| OmniItem {
                title: app.name.clone(),
                class: app.command.first().cloned().unwrap_or_default(),
                client_id: 0,
                workspace: !0,
                icon: crate::icon_render::resolve_client_icon(&[], isz, antibox_ui::theme::field()),
                marked: false,
                run: false,
                command: app.command.clone(),
            })
            .collect();
        let history: Vec<OmniItem> = self
            .run_history
            .iter()
            .map(|line| OmniItem {
                title: line.clone(),
                class: line.clone(),
                client_id: 0,
                workspace: !0,
                icon: crate::icon_render::resolve_client_icon(&[], isz, antibox_ui::theme::field()),
                marked: false,
                run: false,
                command: vec!["sh".to_string(), "-c".to_string(), line.clone()],
            })
            .collect();
        self.run_items.splice(0..0, history);
        self.run_mode = false;
        self.selected = 0;
        self.offset = 0;
        self.menu_target = 0;
        self.menu_levels.clear();
        self.ws_count = wm.config.workspace_count;
        self.ws_names = wm.workspace_names.clone();
        self.win_list = self
            .items
            .iter()
            .filter(|it| !it.run)
            .map(|it| (it.client_id, it.title.clone()))
            .collect();
        self.filtered = (0..self.items.len()).collect();
        self.min_keycode = conn.setup_min_keycode();
        let max = conn.setup_max_keycode();
        self.mapping = conn
            .get_keyboard_mapping(self.min_keycode, max - self.min_keycode + 1)
            .ok();
        let mon = self.monitor_for_pointer(conn, &wm.monitors);
        self.panel_w = panel_w_for(mon.w);
        self.max_rows = rows_for(mon.h);
        let w = self.width();
        let h = self.height();
        self.pos = self.place(mon, w, h);
        let mask = EventMask::EXPOSURE
            | EventMask::KEY_PRESS
            | EventMask::BUTTON_PRESS
            | EventMask::POINTER_MOTION;
        let win = match conn.create_window(
            conn.root().as_parent(),
            Rect::new(self.pos.x, self.pos.y, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            mask,
        ) {
            Ok(w) => w,
            Err(_) => return,
        };
        if conn
            .grab_keyboard(
                false,
                conn.root().read_id(),
                0,
                GrabMode::Async,
                GrabMode::Async,
            )
            .is_err()
        {
            let _ = win.destroy();
            return;
        }
        let _ = win.map();
        let _ = win.raise();
        let bw = (w as i16 - pad() * 2).max(1) as u16;
        if let Some(rconn) = wm.render_backend.clone() {
            if let Ok(mut bar) = SearchBar::new(&rconn, win.id(), pad(), pad(), bw, bar_h()) {
                bar.set_focus(true);
                bar.show();
                self.bar = Some(bar);
            }
        }
        self.window = Some(win);
        self.visible = true;
        self.refilter(conn);
        self.paint(conn);
        let _ = conn.flush();
    }

    pub fn hide(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let _ = conn.ungrab_keyboard(0);
        if let Some(bar) = self.bar.take() {
            bar.hide();
        }
        if let Some(win) = self.window.take() {
            let _ = win.unmap();
            let _ = win.destroy();
        }
        self.visible = false;
        self.menu_levels.clear();
        self.saved_query.clear();
        let _ = conn.flush();
    }

    fn path_commands(&mut self) -> &[String] {
        if self.path_cmds.is_none() {
            let mut set = std::collections::BTreeSet::new();
            if let Some(path) = std::env::var_os("PATH") {
                for dir in std::env::split_paths(&path) {
                    let rd = match std::fs::read_dir(&dir) {
                        Ok(rd) => rd,
                        Err(_) => continue,
                    };
                    for entry in rd.flatten() {
                        let is_exec = entry
                            .file_type()
                            .map_or(false, |t| t.is_file() || t.is_symlink());
                        if is_exec {
                            if let Ok(name) = entry.file_name().into_string() {
                                set.insert(name);
                            }
                        }
                    }
                }
            }
            self.path_cmds = Some(set.into_iter().collect());
        }
        self.path_cmds.as_deref().unwrap_or(&[])
    }

    fn complete_command(&mut self, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        let text = self
            .bar
            .as_ref()
            .map(|b| b.text().to_string())
            .unwrap_or_default();
        if text.is_empty() {
            return OmniOutcome::Consumed;
        }
        let single_word = !text.contains(char::is_whitespace);
        if single_word {
            let _ = self.path_commands();
        }
        let completed = {
            let mut matches: Vec<&str> = self
                .run_history
                .iter()
                .filter(|h| h.starts_with(&text))
                .map(String::as_str)
                .collect();
            if single_word {
                let cmds: &[String] = self.path_cmds.as_deref().unwrap_or(&[]);
                matches.extend(
                    cmds.iter()
                        .filter(|c| c.starts_with(&text))
                        .map(String::as_str),
                );
            }
            if matches.is_empty() {
                None
            } else {
                let lcp = longest_common_prefix(&matches);
                Some(if lcp.len() > text.len() {
                    lcp.to_string()
                } else {
                    matches[0].to_string()
                })
            }
        };
        if let Some(completed) = completed.filter(|c| *c != text) {
            if let Some(bar) = self.bar.as_mut() {
                bar.set_text(&completed);
            }
            self.selected = 0;
            self.refilter(conn);
            self.paint(conn);
        }
        OmniOutcome::Consumed
    }

    fn cursor_at_end(&self) -> bool {
        self.bar
            .as_ref()
            .map_or(false, |b| b.input.cursor_pos() == b.text().len())
    }

    fn command_line(item: &OmniItem) -> String {
        match item.command.as_slice() {
            [sh, flag, line] if sh == "sh" && flag == "-c" => line.clone(),
            cmd => cmd.join(" "),
        }
    }

    fn complete_selection(&mut self, conn: &Arc<dyn DisplayBackend>) -> bool {
        let i = match self.filtered.get(self.selected) {
            Some(&i) => i,
            None => return false,
        };
        let line = Self::command_line(&self.run_items[i]);
        if line.is_empty() || self.bar.as_ref().map_or(false, |b| b.text() == line) {
            return false;
        }
        if let Some(bar) = self.bar.as_mut() {
            bar.set_text(&line);
        }
        self.selected = 0;
        self.refilter(conn);
        self.paint(conn);
        true
    }

    fn refilter(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let needle = self
            .bar
            .as_ref()
            .map(|b| b.text().to_ascii_lowercase())
            .unwrap_or_default();
        let src = self.cur_items();
        let filtered: Vec<usize> = src
            .iter()
            .enumerate()
            .filter(|(_, it)| {
                it.run
                    || needle.is_empty()
                    || it.title.to_ascii_lowercase().contains(&needle)
                    || it.class.to_ascii_lowercase().contains(&needle)
            })
            .map(|(i, _)| i)
            .collect();
        self.filtered = filtered;
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
        self.offset = 0;
        self.ensure_visible();
        if let Some(win) = self.window.as_ref() {
            let w = self.width();
            let h = self.height();
            let _ = conn.configure_window(
                win.id(),
                &[
                    self.pos.x.max(0) as u32,
                    self.pos.y.max(0) as u32,
                    w as u32,
                    h as u32,
                ],
            );
        }
    }

    fn ensure_visible(&mut self) {
        if self.selected < self.offset {
            self.offset = self.selected;
        }
        if self.selected >= self.offset + self.max_rows {
            self.offset = self.selected + 1 - self.max_rows;
        }
        let max_off = self.filtered.len().saturating_sub(self.max_rows);
        self.offset = self.offset.min(max_off);
    }

    fn row_at(&self, y: i32) -> Option<usize> {
        let top = pad() as i32 * 2 + bar_h() as i32;
        if y < top {
            return None;
        }
        let idx = self.offset + ((y - top) / row_h() as i32) as usize;
        if idx < self.filtered.len() && idx < self.offset + self.visible_rows() {
            Some(idx)
        } else {
            None
        }
    }

    fn menu_row_at(&self, y: i32) -> Option<usize> {
        let top = pad() as i32 * 2 + bar_h() as i32;
        if y < top {
            return None;
        }
        let idx = ((y - top) / row_h() as i32) as usize;
        let level = self.menu_levels.last()?;
        level
            .rows
            .get(idx)
            .filter(|(label, _)| !label.is_empty())
            .map(|_| idx)
    }

    fn scroll(&mut self, delta: i32) {
        let max_off = self.filtered.len().saturating_sub(self.max_rows) as i32;
        self.offset = (self.offset as i32 + delta).clamp(0, max_off.max(0)) as usize;
        let last = (self.offset + self.visible_rows()).saturating_sub(1);
        self.selected = self.selected.clamp(self.offset, last);
    }

    fn select(&mut self, idx: usize, conn: &Arc<dyn DisplayBackend>) {
        self.selected = idx.min(self.filtered.len().saturating_sub(1));
        self.ensure_visible();
        self.paint(conn);
    }

    fn keysym_for(&self, keycode: u32) -> u32 {
        let m = match self.mapping.as_ref() {
            Some(m) => m,
            None => return 0,
        };
        let off = (keycode as usize).saturating_sub(self.min_keycode as usize)
            * m.keysyms_per_keycode as usize;
        m.keysyms.get(off).copied().unwrap_or(0)
    }

    pub fn handle_event(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        event: &BackendEvent,
    ) -> OmniOutcome {
        match event {
            BackendEvent::KeyPress { keycode, state, .. } => {
                let shift = state & 0x01 != 0;
                let ks = self.keysym_for(*keycode);
                if self.in_submenu() {
                    let has_query = self.bar.as_ref().map_or(false, |b| !b.text().is_empty());
                    if ks == KEY_Up
                        || ks == KEY_Down
                        || ks == KEY_Return
                        || ks == KEY_Right
                        || ((ks == KEY_Left || ks == KEY_Escape) && !has_query)
                    {
                        return self.submenu_key(ks, conn);
                    }
                    if ks == KEY_Escape {
                        self.ops_bar_reset();
                        self.refilter_ops(conn);
                        return OmniOutcome::Consumed;
                    }
                    if let (Some(mapping), Some(bar)) = (self.mapping.as_ref(), self.bar.as_mut()) {
                        if bar.handle_key(*keycode, *state, mapping) == SearchEvent::Changed {
                            self.refilter_ops(conn);
                        }
                    }
                    return OmniOutcome::Consumed;
                }
                if ks == KEY_Left && !self.run_mode {
                    return self.open_ops_menu(conn);
                }
                if ks == KEY_Right
                    && self.run_mode
                    && self.cursor_at_end()
                    && self.complete_selection(conn)
                {
                    return OmniOutcome::Consumed;
                }
                match ks {
                    k if k == KEY_Up => {
                        self.select(self.selected.saturating_sub(1), conn);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Down => {
                        self.select(self.selected + 1, conn);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Tab => {
                        if self.run_mode && !shift {
                            return self.complete_command(conn);
                        }
                        let next = if shift {
                            self.selected.saturating_sub(1)
                        } else {
                            self.selected + 1
                        };
                        self.select(next, conn);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Home => {
                        self.select(0, conn);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_End => {
                        self.select(self.filtered.len().saturating_sub(1), conn);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Prior => {
                        self.select(self.selected.saturating_sub(self.max_rows), conn);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Next => {
                        self.select(self.selected + self.max_rows, conn);
                        return OmniOutcome::Consumed;
                    }
                    _ => {}
                }
                let mapping = match self.mapping.as_ref() {
                    Some(m) => m,
                    None => return OmniOutcome::Close,
                };
                let bar = match self.bar.as_mut() {
                    Some(b) => b,
                    None => return OmniOutcome::Close,
                };
                match bar.handle_key(*keycode, *state, mapping) {
                    SearchEvent::Changed => {
                        self.selected = 0;
                        self.refilter(conn);
                        self.paint(conn);
                        OmniOutcome::Consumed
                    }
                    SearchEvent::Submitted => self.activate_selected(conn),
                    SearchEvent::Cancelled => {
                        if self.run_mode {
                            self.exit_run_mode(conn);
                            OmniOutcome::Consumed
                        } else {
                            OmniOutcome::Close
                        }
                    }
                    SearchEvent::None => OmniOutcome::Consumed,
                }
            }
            BackendEvent::ButtonPress {
                window,
                point,
                button,
                ..
            } => {
                let own = self.window.as_ref().map(|w| w.id()) == Some(*window);
                if !own && !self.owns_window(*window) {
                    return OmniOutcome::Close;
                }
                if own && self.in_submenu() {
                    if let Some(row) = self.menu_row_at(point.y) {
                        if let Some(level) = self.menu_levels.last_mut() {
                            level.selected = row;
                        }
                        return self.activate_submenu(conn);
                    }
                    return OmniOutcome::Consumed;
                }
                if own && (*button == 4 || *button == 5) {
                    self.scroll(if *button == 4 { -1 } else { 1 });
                    self.paint(conn);
                    return OmniOutcome::Consumed;
                }
                if let Some(row) = own.then(|| self.row_at(point.y)).flatten() {
                    self.selected = row;
                    return self.activate_selected(conn);
                }
                if let Some(bar) = self.bar.as_mut() {
                    if bar.handle_button(*window, point.x, point.y, *button) == SearchEvent::Changed
                    {
                        if self.in_submenu() {
                            self.refilter_ops(conn);
                        } else {
                            self.selected = 0;
                            self.refilter(conn);
                            self.paint(conn);
                        }
                    }
                }
                OmniOutcome::Consumed
            }
            BackendEvent::MotionNotify { window, point, .. } => {
                if self.window.as_ref().map(|w| w.id()) == Some(*window) {
                    if self.in_submenu() {
                        if let Some(row) = self.menu_row_at(point.y) {
                            if self.menu_levels.last().map(|l| l.selected) != Some(row) {
                                if let Some(level) = self.menu_levels.last_mut() {
                                    level.selected = row;
                                }
                                self.paint(conn);
                            }
                        }
                    } else if let Some(row) = self.row_at(point.y) {
                        if row != self.selected {
                            self.selected = row;
                            self.paint(conn);
                        }
                    }
                } else if let Some(bar) = self.bar.as_mut() {
                    bar.handle_motion(*window, point.x);
                }
                OmniOutcome::Consumed
            }
            BackendEvent::ButtonRelease { window, button, .. } => {
                if let Some(bar) = self.bar.as_mut() {
                    bar.handle_release(*window, *button);
                }
                OmniOutcome::Consumed
            }
            BackendEvent::Expose { .. } => {
                self.paint(conn);
                if let Some(bar) = self.bar.as_ref() {
                    bar.repaint();
                }
                OmniOutcome::Consumed
            }
            _ => OmniOutcome::Consumed,
        }
    }

    pub fn paint(&self, conn: &Arc<dyn DisplayBackend>) {
        let win = match self.window.as_ref() {
            Some(w) => w,
            None => return,
        };
        let w = self.width();
        let h = self.height();
        crate::paintbuf::buffered(&**conn, win.id(), w, h, |g| self.render(g, w, h));
        if let Some(bar) = self.bar.as_ref() {
            bar.repaint();
        }
        let _ = conn.flush();
    }

    fn needs_sb(&self) -> bool {
        self.menu_levels.is_empty() && self.filtered.len() > self.max_rows
    }

    fn list_w(&self, w: u16) -> u16 {
        if self.needs_sb() {
            (w as i16 - crate::winlist::sb_w()).max(1) as u16
        } else {
            w
        }
    }

    fn render(&self, g: &dyn GraphicsContext, w: u16, h: u16) {
        let c = crate::menu::MenuColors::default();
        let field = antibox_ui::theme::field();
        let lw = self.list_w(w);
        let _ = g.set_foreground(field);
        let _ = g.fill_rect(0, 0, w, h);
        let _ = g.set_font(&FontSpec::role(
            FontRole::Switch,
            antibox_ui::metrics::font_pt(),
        ));
        let top = pad() * 2 + bar_h() as i16;
        let vis = self.visible_rows();
        if let Some(level) = self.menu_levels.last() {
            for (row, (label, action)) in level.rows.iter().enumerate().take(vis) {
                let y = top + row as i16 * row_h() as i16;
                if label.is_empty() {
                    let _ = g.set_foreground(antibox_ui::theme::shadow());
                    let _ =
                        g.draw_line(0, y + row_h() as i16 / 2, lw as i16, y + row_h() as i16 / 2);
                    continue;
                }
                let sel = row == level.selected;
                if sel {
                    crate::render::fill_menu_selection(g, 0, y, lw, row_h(), c.sel_bg);
                }
                let fg = if sel {
                    c.sel_fg
                } else {
                    antibox_ui::theme::text()
                };
                let _ = g.set_foreground(fg);
                let _ = g.set_background(if sel { c.sel_bg } else { field });
                let baseline = antibox_ui::metrics::baseline(y as i32, row_h() as i32) as i16;
                let _ = g.draw_text(20, baseline, label);
                if matches!(action, OpAction::OpenSend | OpAction::OpenJoin) {
                    crate::render::draw_submenu_arrow(
                        g,
                        lw as i16 - scaled(12) as i16,
                        y + row_h() as i16 / 2,
                        scaled(7) as i16,
                        fg,
                    );
                }
            }
            antibox_ui::theme::well(g, 0, 0, w, h);
            return;
        }
        for (row, &i) in self.filtered.iter().skip(self.offset).take(vis).enumerate() {
            let y = top + row as i16 * row_h() as i16;
            let sel = self.offset + row == self.selected;
            if sel {
                crate::render::fill_menu_selection(g, 0, y, lw, row_h(), c.sel_bg);
            }
            let item = &self.cur_items()[i];
            let iy = y + ((row_h() as i16 - item.icon.height as i16) / 2).max(0);
            let _ = g.draw_pixmap(4, iy, &item.icon);
            let text_x = 4 + item.icon.width as i16 + antibox_ui::metrics::gap() as i16;
            let fg = if sel {
                c.sel_fg
            } else {
                antibox_ui::theme::text()
            };
            let _ = g.set_foreground(fg);
            let _ = g.set_background(if sel { c.sel_bg } else { field });
            let avail = (lw as i16 - text_x - pad() * 2) as u16;
            let label = crate::applet::fit_label(g, &item.title, avail);
            let baseline = antibox_ui::metrics::baseline(y as i32, row_h() as i32) as i16;
            let _ = g.draw_text(text_x, baseline, &label);
            if item.marked {
                let mx = lw as i16 - pad() - scaled(12) as i16;
                let my = y + (row_h() as i16 - scaled(10) as i16) / 2;
                let _ = g.set_foreground(if sel {
                    c.sel_fg
                } else {
                    antibox_ui::theme::sel_bg()
                });
                let _ = g.fill_rect(mx, my, scaled(8) as u16, scaled(8) as u16);
            }
        }
        if self.needs_sb() {
            let bwid = crate::winlist::sb_w();
            let track_top = top + bwid;
            let track_h = (h as i16 - track_top - bwid).max(1);
            let thumb = antibox_ui::thumb::Thumb::new(
                track_top as i32,
                vis as i32,
                self.offset as i32,
                self.filtered.len() as i32,
            );
            let (ty, th) = thumb.rect(track_h as i32);
            crate::winlist::draw_scrollbar(g, lw as i16, top, h as i16, (ty as i16, th as i16));
        }
        if self.filtered.is_empty() && !self.run_mode {
            let _ = g.set_foreground(antibox_ui::theme::disabled());
            let _ = g.set_background(field);
            let baseline = antibox_ui::metrics::baseline(top as i32, row_h() as i32) as i16;
            let _ = g.draw_text(20, baseline, "No matches");
        }
        antibox_ui::theme::well(g, 0, 0, w, h);
    }
}

#[cfg(test)]
#[path = "omni_tests.rs"]
mod tests;
