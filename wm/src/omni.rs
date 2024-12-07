use crate::id::ClientId;
use crate::listview::{ListNav, ListView, Place};
use crate::manager::WindowManager;
use crate::menu_tree::{FlatEntry, MenuNode};
use antibox_core::backend::*;
use antibox_core::keysyms::{
    KEY_Down, KEY_End, KEY_Escape, KEY_Home, KEY_Left, KEY_Next, KEY_Prior, KEY_Return, KEY_Right,
    KEY_Tab, KEY_Up,
};
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use antibox_core::scale::scaled;
use antibox_ui::searchbar::SearchEvent;
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

#[derive(Clone)]
pub enum OmniAct {
    Window(u32),
    Launch(Vec<String>),
    RunEntry,
    Op(OmniWinOp),
    CycleSort,
    ToggleGrouping,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OmniSort {
    #[default]
    Natural,
    Title,
    Class,
    Workspace,
    Window,
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

pub struct Omni {
    pub view: ListView<OmniAct>,
    pub items: Vec<OmniItem>,
    pub run_items: Vec<OmniItem>,
    run_history: Vec<String>,
    mapping: Option<KeyboardMapping>,
    min_keycode: u8,
    pub run_mode: bool,
    in_ops: bool,
    ops_target: u32,
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
    let Some(text) = run_history_file().and_then(|p| std::fs::read_to_string(p).ok()) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .take(RUN_HISTORY_MAX)
        .collect()
}

fn save_run_history(history: &[String]) {
    let Some(path) = run_history_file() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&path, history.join("\n"));
}

fn longest_common_prefix<'a>(items: &[&'a str]) -> &'a str {
    let Some(first) = items.first() else { return "" };
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
            view: ListView::new(),
            items: Vec::new(),
            run_items: Vec::new(),
            run_history: load_run_history(),
            mapping: None,
            min_keycode: 8,
            run_mode: false,
            in_ops: false,
            ops_target: 0,
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

    pub const fn visible(&self) -> bool {
        self.view.visible
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

    pub fn is_marked(&self, client_id: u32) -> bool {
        self.items
            .iter()
            .any(|it| it.client_id == client_id && it.marked)
    }

    pub fn owns_window(&self, id: u32) -> bool {
        self.view.owns_window(id)
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
        format!("Group by class: {state}")
    }

    fn apply_sort(&mut self) {
        let mode = self.sort;
        self.items.sort_by(|a, b| compare_items(mode, a, b));
    }

    fn ws_label(&self, i: u32) -> String {
        self.ws_names
            .get(i as usize)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Workspace {}", i + 1))
    }

    fn window_nodes(&self) -> Vec<MenuNode<OmniAct>> {
        let wins: Vec<MenuNode<OmniAct>> = self
            .items
            .iter()
            .filter(|it| !it.run)
            .map(|it| {
                let title = if it.marked {
                    format!("{} \u{25A0}", it.title)
                } else {
                    it.title.clone()
                };
                MenuNode::leaf(title, OmniAct::Window(it.client_id))
                    .with_icon(Some(it.icon.clone()))
            })
            .collect();
        let apps: Vec<MenuNode<OmniAct>> = self
            .apps
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|app| MenuNode::leaf(app.name.clone(), OmniAct::Launch(app.command.clone())))
            .collect();
        let mut nodes = Vec::new();
        if !wins.is_empty() {
            nodes.push(MenuNode::group_expanded("Windows", wins));
        }
        if !apps.is_empty() {
            nodes.push(MenuNode::group("Applications", apps));
        }
        nodes.push(MenuNode::group_expanded(
            "Actions",
            vec![MenuNode::leaf("Run\u{2026}", OmniAct::RunEntry)],
        ));
        nodes
    }

    fn run_nodes(&self) -> Vec<MenuNode<OmniAct>> {
        self.run_items
            .iter()
            .map(|it| {
                MenuNode::leaf(it.title.clone(), OmniAct::Launch(it.command.clone()))
                    .with_icon(Some(it.icon.clone()))
            })
            .collect()
    }

    fn ops_nodes(&self) -> Vec<MenuNode<OmniAct>> {
        let marked = self
            .items
            .iter()
            .find(|it| !it.run && it.client_id == self.ops_target)
            .is_some_and(|it| it.marked);
        let marked_count = self.items.iter().filter(|it| it.marked).count();
        let toggle = if marked { "Unmark" } else { "Mark" };
        let mut nodes: Vec<MenuNode<OmniAct>> = vec![
            MenuNode::leaf(toggle, OmniAct::Op(OmniWinOp::ToggleMark)),
            MenuNode::separator(),
            MenuNode::leaf("Close", OmniAct::Op(OmniWinOp::Close)),
            MenuNode::leaf("Kill", OmniAct::Op(OmniWinOp::Kill)),
        ];
        if self.ws_count > 1 {
            let ws = (0..self.ws_count)
                .map(|i| MenuNode::leaf(self.ws_label(i), OmniAct::Op(OmniWinOp::SendTo(i))))
                .collect();
            nodes.push(MenuNode::group("Send to", ws));
        }
        let join: Vec<MenuNode<OmniAct>> = self
            .win_list
            .iter()
            .filter(|(id, _)| *id != self.ops_target)
            .map(|(id, title)| MenuNode::leaf(title.clone(), OmniAct::Op(OmniWinOp::Join(*id))))
            .collect();
        if !join.is_empty() {
            nodes.push(MenuNode::group("Join", join));
        }
        if marked_count > 0 {
            nodes.push(MenuNode::separator());
            nodes.push(MenuNode::leaf(
                "Close marked",
                OmniAct::Op(OmniWinOp::CloseMarked),
            ));
            nodes.push(MenuNode::leaf(
                "Kill marked",
                OmniAct::Op(OmniWinOp::KillMarked),
            ));
            if marked_count > 1 {
                nodes.push(MenuNode::leaf(
                    "Unmark all",
                    OmniAct::Op(OmniWinOp::UnmarkAll),
                ));
            }
        }
        nodes.push(MenuNode::separator());
        nodes.push(MenuNode::leaf(self.sort_row_label(), OmniAct::CycleSort));
        nodes.push(MenuNode::leaf(
            self.group_row_label(),
            OmniAct::ToggleGrouping,
        ));
        nodes
    }

    fn current_nodes(&self) -> Vec<MenuNode<OmniAct>> {
        if self.in_ops {
            self.ops_nodes()
        } else if self.run_mode {
            self.run_nodes()
        } else {
            self.window_nodes()
        }
    }

    fn query(&self) -> String {
        self.view
            .bar()
            .map(|b| b.text().to_string())
            .unwrap_or_default()
    }

    fn set_query(&mut self, text: &str) {
        if let Some(bar) = self.view.bar_mut() {
            bar.set_text(text);
            bar.repaint();
        }
    }

    fn reselect(&mut self) {
        self.view.selected = self
            .view
            .next_leaf(None, 1)
            .or_else(|| self.view.next_selectable(None, 1));
    }

    fn reload(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let nodes = self.current_nodes();
        self.view.set_tree(nodes);
        self.view.offset = 0;
        self.reselect();
        self.view.sync_geometry(conn.as_ref());
        self.paint(conn);
    }

    fn reload_keep_row(&mut self, conn: &Arc<dyn DisplayBackend>, title_prefix: &str) {
        let nodes = self.current_nodes();
        self.view.set_tree(nodes);
        self.view.selected = self
            .view
            .items
            .iter()
            .position(|r| r.title.starts_with(title_prefix))
            .or_else(|| self.view.next_leaf(None, 1));
        self.view.sync_geometry(conn.as_ref());
        self.paint(conn);
    }

    fn refilter(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.view.refilter(conn.as_ref());
        self.reselect();
        self.view.paint(conn.as_ref());
    }

    fn open_ops_menu(&mut self, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        if self.run_mode || self.in_ops {
            return OmniOutcome::Consumed;
        }
        let target = match self
            .view
            .selected
            .and_then(|i| self.view.items.get(i))
            .and_then(antibox_ui::menu_tree::FlatRow::payload)
        {
            Some(OmniAct::Window(id)) => *id,
            _ => return OmniOutcome::Consumed,
        };
        self.ops_target = target;
        self.saved_query = self.query();
        self.in_ops = true;
        self.set_query("");
        self.reload(conn);
        OmniOutcome::Consumed
    }

    fn pop_ops(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.in_ops = false;
        let saved = std::mem::take(&mut self.saved_query);
        self.set_query(&saved);
        self.reload(conn);
    }

    fn enter_run_mode(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.run_mode = true;
        self.set_query("");
        self.reload(conn);
    }

    fn exit_run_mode(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.run_mode = false;
        self.set_query("");
        self.reload(conn);
    }

    fn resort(&mut self, conn: &Arc<dyn DisplayBackend>) {
        self.apply_sort();
        self.win_list = self
            .items
            .iter()
            .filter(|it| !it.run)
            .map(|it| (it.client_id, it.title.clone()))
            .collect();
        if self.in_ops {
            self.reload_keep_row(conn, "Sort:");
        } else {
            self.reload(conn);
        }
    }

    fn perform(&mut self, conn: &Arc<dyn DisplayBackend>, act: OmniAct) -> OmniOutcome {
        match act {
            OmniAct::Window(id) => OmniOutcome::Activate(id),
            OmniAct::RunEntry => {
                self.enter_run_mode(conn);
                OmniOutcome::Consumed
            }
            OmniAct::Launch(cmd) => OmniOutcome::Launch(cmd),
            OmniAct::CycleSort => {
                self.sort = self.sort.next();
                self.resort(conn);
                OmniOutcome::Consumed
            }
            OmniAct::ToggleGrouping => {
                self.group_override = Some(!self.grouping_enabled());
                if self.in_ops {
                    self.reload_keep_row(conn, "Group by class:");
                }
                OmniOutcome::Consumed
            }
            OmniAct::Op(op) => match op {
                OmniWinOp::ToggleMark => {
                    let target = self.ops_target;
                    for it in &mut self.items {
                        if !it.run && it.client_id == target {
                            it.marked = !it.marked;
                        }
                    }
                    self.reload_keep_row(conn, "");
                    OmniOutcome::Consumed
                }
                OmniWinOp::UnmarkAll => {
                    for it in &mut self.items {
                        it.marked = false;
                    }
                    self.reload_keep_row(conn, "");
                    OmniOutcome::Consumed
                }
                OmniWinOp::Separator => OmniOutcome::Consumed,
                OmniWinOp::CloseMarked => OmniOutcome::CloseMarked,
                OmniWinOp::KillMarked => OmniOutcome::KillMarked,
                _ => OmniOutcome::WindowOp {
                    target: self.ops_target,
                    op,
                },
            },
        }
    }

    fn activate_selected(&mut self, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        if let Some(idx) = self.view.selected {
            match self.view.items.get(idx).map(|r| r.entry.clone()) {
                Some(FlatEntry::Group { .. }) => {
                    self.view.toggle_row(conn.as_ref(), idx);
                    return OmniOutcome::Consumed;
                }
                Some(FlatEntry::Leaf(act)) => return self.perform(conn, act),
                _ => {}
            }
        }
        if self.run_mode {
            let line = self.query().trim().to_string();
            if !line.is_empty() {
                return OmniOutcome::Run(line);
            }
        }
        OmniOutcome::Consumed
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
        if self.visible() {
            return;
        }
        let isz = icon_size();
        let grouping = self.grouping_enabled();

        let mut order: Vec<ClientId> = Vec::new();
        let mut counts: std::collections::HashMap<(String, u32), usize> =
            std::collections::HashMap::new();
        let mut first: std::collections::HashSet<(String, u32)> = std::collections::HashSet::new();
        for id in &wm.insertion_order {
            let Some(fw) = wm.frames.get(id) else { continue };
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
                    format!("{base}  ({count})")
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
        self.in_ops = false;
        self.ops_target = 0;
        self.saved_query.clear();
        self.ws_count = wm.config.workspace_count;
        self.ws_names = wm.workspace_names.clone();
        self.win_list = self
            .items
            .iter()
            .filter(|it| !it.run)
            .map(|it| (it.client_id, it.title.clone()))
            .collect();
        self.min_keycode = conn.setup_min_keycode();
        let max = conn.setup_max_keycode();
        self.mapping = conn
            .get_keyboard_mapping(self.min_keycode, max - self.min_keycode + 1)
            .ok();
        let mon = self.monitor_for_pointer(conn, &wm.monitors);
        self.panel_w = panel_w_for(mon.w);
        self.max_rows = rows_for(mon.h);
        self.view.set_window_frame();
        let w = self.width();
        let extra = 2 * self.view.chrome_extra() as i32;
        let cap = pad() as i32 * 3 + bar_h() as i32 + self.max_rows as i32 * row_h() as i32 + extra;
        let nodes = self.current_nodes();
        let rows = crate::menu_tree::flatten_nodes(&nodes).len() as i32;
        let want = pad() as i32 * 3 + bar_h() as i32 + rows * row_h() as i32 + extra;
        let h = want.clamp(scaled(120), cap.max(scaled(120))) as u16;
        let pos = self.place(mon, w, h);
        self.view.show_with(conn.as_ref(), nodes, Place::At(pos), w, cap, true, |_, _| {});
        if !self.view.visible {
            return;
        }
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
            self.view.hide(conn.as_ref());
            return;
        }
        if let Some(win) = &self.view.window {
            let _ = win.raise();
        }
        if let Some(rb) = wm.render_backend.clone() {
            self.view.enable_filter(&rb);
        }
        self.reselect();
        self.paint(conn);
        let _ = conn.flush();
    }

    pub fn hide(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let _ = conn.ungrab_keyboard(0);
        self.view.hide(conn.as_ref());
        self.run_mode = false;
        self.in_ops = false;
        self.saved_query.clear();
        let _ = conn.flush();
    }

    fn path_commands(&mut self) -> &[String] {
        if self.path_cmds.is_none() {
            let mut set = std::collections::BTreeSet::new();
            if let Some(path) = std::env::var_os("PATH") {
                for dir in std::env::split_paths(&path) {
                    let Ok(rd) = std::fs::read_dir(&dir) else { continue };
                    for entry in rd.flatten() {
                        let is_exec = entry
                            .file_type()
                            .is_ok_and(|t| t.is_file() || t.is_symlink());
                        if is_exec {
                            if let Ok(name) = entry.file_name().into_string() {
                                set.insert(name);
                            }
                        }
                    }
                }
            }
            for it in &self.run_items {
                if let Some(prog) = it.command.first().filter(|p| *p != "sh") {
                    let base = prog.rsplit('/').next().unwrap_or(prog);
                    set.insert(base.to_string());
                }
            }
            self.path_cmds = Some(set.into_iter().collect());
        }
        self.path_cmds.as_deref().unwrap_or(&[])
    }

    fn complete_command(&mut self, conn: &Arc<dyn DisplayBackend>) -> OmniOutcome {
        let text = self.query();
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
            self.set_query(&completed);
            self.refilter(conn);
        }
        OmniOutcome::Consumed
    }

    fn cursor_at_end(&self) -> bool {
        self.view
            .bar()
            .is_some_and(|b| b.input.cursor_pos() == b.text().len())
    }

    fn command_line_argv(cmd: &[String]) -> String {
        match cmd {
            [sh, flag, line] if sh == "sh" && flag == "-c" => line.clone(),
            cmd => cmd.join(" "),
        }
    }

    fn complete_selection(&mut self, conn: &Arc<dyn DisplayBackend>) -> bool {
        let line = match self
            .view
            .selected
            .and_then(|i| self.view.items.get(i))
            .and_then(antibox_ui::menu_tree::FlatRow::payload)
        {
            Some(OmniAct::Launch(cmd)) => Self::command_line_argv(cmd),
            _ => return false,
        };
        if line.is_empty() || self.query() == line {
            return false;
        }
        self.set_query(&line);
        self.refilter(conn);
        true
    }

    fn keysym_for(&self, keycode: u32) -> u32 {
        let Some(m) = self.mapping.as_ref() else { return 0 };
        let off = (keycode as usize).saturating_sub(self.min_keycode as usize)
            * m.keysyms_per_keycode as usize;
        m.keysyms.get(off).copied().unwrap_or(0)
    }

    fn move_selection(&mut self, conn: &Arc<dyn DisplayBackend>, dir: i32) {
        self.view.selected = self.view.next_selectable(self.view.selected, dir);
        self.view.scroll_to_selected();
        self.view.paint(conn.as_ref());
    }

    fn select_edge(&mut self, conn: &Arc<dyn DisplayBackend>, dir: i32) {
        self.view.selected = self.view.next_selectable(None, dir);
        self.view.scroll_to_selected();
        self.view.paint(conn.as_ref());
    }

    fn page(&mut self, conn: &Arc<dyn DisplayBackend>, dir: i32) {
        let vr = self.view.vis_rows();
        self.view.scroll_by(if dir < 0 { -vr } else { vr });
        let first = self.view.offset as usize;
        let from = first.checked_sub(1);
        self.view.selected = self.view.next_selectable(from, 1);
        self.view.paint(conn.as_ref());
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
                if self.in_ops {
                    let has_query = !self.query().is_empty();
                    if (ks == KEY_Left || ks == KEY_Escape) && !has_query {
                        self.pop_ops(conn);
                        return OmniOutcome::Consumed;
                    }
                    if ks == KEY_Escape {
                        self.set_query("");
                        self.refilter(conn);
                        return OmniOutcome::Consumed;
                    }
                    if ks == KEY_Up || ks == KEY_Down {
                        self.move_selection(conn, if ks == KEY_Up { -1 } else { 1 });
                        return OmniOutcome::Consumed;
                    }
                    if ks == KEY_Return || ks == KEY_Right {
                        return self.activate_selected(conn);
                    }
                    if let (Some(mapping), Some(bar)) =
                        (self.mapping.as_ref(), self.view.bar_mut())
                    {
                        if bar.handle_key(*keycode, *state, mapping) == SearchEvent::Changed {
                            bar.repaint();
                            self.refilter(conn);
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
                        self.move_selection(conn, -1);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Down => {
                        self.move_selection(conn, 1);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Tab => {
                        if self.run_mode && !shift {
                            return self.complete_command(conn);
                        }
                        self.move_selection(conn, if shift { -1 } else { 1 });
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Home => {
                        self.select_edge(conn, 1);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_End => {
                        self.select_edge(conn, -1);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Prior => {
                        self.page(conn, -1);
                        return OmniOutcome::Consumed;
                    }
                    k if k == KEY_Next => {
                        self.page(conn, 1);
                        return OmniOutcome::Consumed;
                    }
                    _ => {}
                }
                let Some(mapping) = self.mapping.as_ref() else { return OmniOutcome::Close };
                let ev = match self.view.bar_mut() {
                    Some(bar) => {
                        let ev = bar.handle_key(*keycode, *state, mapping);
                        if ev != SearchEvent::None {
                            bar.repaint();
                        }
                        ev
                    }
                    None => return OmniOutcome::Close,
                };
                match ev {
                    SearchEvent::Changed => {
                        self.refilter(conn);
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
                root,
                button,
                ..
            } => {
                let own = self.view.window_id() == *window;
                if !own && !self.owns_window(*window) {
                    return OmniOutcome::Close;
                }
                if own && (*button == 4 || *button == 5) {
                    self.view.scroll_by(if *button == 4 { -3 } else { 3 });
                    self.view.paint(conn.as_ref());
                    return OmniOutcome::Consumed;
                }
                if own {
                    match self.view.handle_click(conn.as_ref(), *root) {
                        ListNav::Activate(act) => return self.perform(conn, act),
                        ListNav::Handled => return OmniOutcome::Consumed,
                        _ => {}
                    }
                }
                let changed = match self.view.bar_mut() {
                    Some(bar) => {
                        bar.handle_button(*window, point.x, point.y, *button)
                            == SearchEvent::Changed
                    }
                    None => false,
                };
                if changed {
                    self.refilter(conn);
                }
                OmniOutcome::Consumed
            }
            BackendEvent::MotionNotify {
                window,
                point,
                root,
                ..
            } => {
                if self.view.window_id() == *window {
                    if self.view.row_at(*root).is_some() {
                        self.view.handle_motion(conn.as_ref(), *root);
                    }
                } else if let Some(bar) = self.view.bar_mut() {
                    bar.handle_motion(*window, point.x);
                }
                OmniOutcome::Consumed
            }
            BackendEvent::ButtonRelease { window, button, .. } => {
                if let Some(bar) = self.view.bar_mut() {
                    bar.handle_release(*window, *button);
                }
                OmniOutcome::Consumed
            }
            BackendEvent::Expose { .. } => {
                self.paint(conn);
                OmniOutcome::Consumed
            }
            _ => OmniOutcome::Consumed,
        }
    }

    pub fn paint(&self, conn: &Arc<dyn DisplayBackend>) {
        self.view.paint(conn.as_ref());
        if let Some(bar) = self.view.bar() {
            bar.repaint();
        }
        let _ = conn.flush();
    }
}

#[cfg(test)]
#[path = "omni_tests.rs"]
mod tests;
