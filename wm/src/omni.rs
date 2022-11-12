use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::*;
use antibox_core::keysyms::{
    KEY_Down, KEY_End, KEY_Home, KEY_Next, KEY_Prior, KEY_Right, KEY_Tab, KEY_Up,
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
    pub icon_normal: PixmapData,
    pub icon_selected: PixmapData,
    pub run: bool,
    pub command: Vec<String>,
}

pub enum OmniOutcome {
    Consumed,
    Activate(u32),
    Run(String),
    Launch(Vec<String>),
    Close,
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
    path_cmds: Option<Vec<String>>,
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
    (antibox_ui::metrics::field_height() + antibox_ui::metrics::gap()) as u16
}

fn row_h() -> u16 {
    (antibox_ui::metrics::menu_item_height() + antibox_ui::metrics::gap()) as u16
}

fn pad() -> i16 {
    antibox_ui::metrics::pad() as i16
}

fn icon_size() -> u16 {
    (row_h() as i32 - antibox_ui::metrics::gap()).max(8) as u16
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
            path_cmds: None,
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

    pub fn owns_window(&self, id: u32) -> bool {
        self.window.as_ref().map(|w| w.id()) == Some(id)
            || self.bar.as_ref().map_or(false, |b| b.owns_window(id))
    }

    fn visible_rows(&self) -> usize {
        self.filtered.len().clamp(1, self.max_rows)
    }

    fn height(&self) -> u16 {
        (pad() as u16) * 2 + bar_h() + row_h() * self.visible_rows() as u16
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
        let colours = crate::menu::MenuColors::default();
        let isz = icon_size();
        let mut order: Vec<ClientId> = Vec::new();
        for id in &wm.insertion_order {
            let fw = match wm.frames.get(id) {
                Some(fw) => fw,
                None => continue,
            };
            if fw.state().skip_taskbar {
                continue;
            }
            order.push(*id);
        }
        self.items = order
            .iter()
            .filter_map(|id| wm.frames.get(id).map(|fw| (*id, fw)))
            .map(|(id, fw)| {
                let class = fw.client().class_instance().unwrap_or("").to_string();
                let icon_normal =
                    crate::icon_render::resolve_client_icon(fw.client().icons(), isz, colours.bg);
                let icon_selected = crate::icon_render::resolve_client_icon(
                    fw.client().icons(),
                    isz,
                    colours.sel_bg,
                );
                OmniItem {
                    title: fw.client().title().to_string(),
                    class,
                    client_id: wm.xid_index.xid_of(id),
                    icon_normal,
                    icon_selected,
                    run: false,
                    command: Vec::new(),
                }
            })
            .collect();
        self.items.push(OmniItem {
            title: "Run\u{2026}".to_string(),
            class: "run".to_string(),
            client_id: 0,
            icon_normal: crate::icon_render::resolve_client_icon(&[], isz, colours.bg),
            icon_selected: crate::icon_render::resolve_client_icon(&[], isz, colours.sel_bg),
            run: true,
            command: Vec::new(),
        });
        self.run_items = self
            .run_history
            .iter()
            .map(|line| OmniItem {
                title: line.clone(),
                class: line.clone(),
                client_id: 0,
                icon_normal: crate::icon_render::resolve_client_icon(&[], isz, colours.bg),
                icon_selected: crate::icon_render::resolve_client_icon(&[], isz, colours.sel_bg),
                run: false,
                command: vec!["sh".to_string(), "-c".to_string(), line.clone()],
            })
            .collect();
        self.run_mode = false;
        self.selected = 0;
        self.offset = 0;
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
                        self.selected = 0;
                        self.refilter(conn);
                        self.paint(conn);
                    }
                }
                OmniOutcome::Consumed
            }
            BackendEvent::MotionNotify { window, point, .. } => {
                if self.window.as_ref().map(|w| w.id()) == Some(*window) {
                    if let Some(row) = self.row_at(point.y) {
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
        let depth = conn.screen_depth();
        if let Ok(pm) = conn.create_pixmap(w, h, depth) {
            if let Ok(g) = conn.create_graphics(pm) {
                self.render(&*g, w, h);
            }
            if let Ok(wg) = conn.create_graphics(win.id()) {
                let _ = wg.copy_from(pm, Rect::px(0, 0, w, h), Point::ZERO);
            }
            let _ = conn.free_pixmap(pm);
        } else if let Ok(g) = conn.create_graphics(win.id()) {
            self.render(&*g, w, h);
        }
        let _ = conn.flush();
    }

    fn render(&self, g: &dyn GraphicsContext, w: u16, _h: u16) {
        let c = crate::menu::MenuColors::default();
        let _ = crate::render::draw_menu_frame(g, w, self.height(), c.bg);
        let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
        let top = pad() * 2 + bar_h() as i16;
        let vis = self.visible_rows();
        let inner_w = (w as i16 - pad() * 2) as u16;
        for (row, &i) in self.filtered.iter().skip(self.offset).take(vis).enumerate() {
            let y = top + row as i16 * row_h() as i16;
            let sel = self.offset + row == self.selected;
            if sel {
                crate::render::fill_menu_selection(g, pad(), y, inner_w, row_h(), c.sel_bg);
            }
            let row_bg = if sel { c.sel_bg } else { c.bg };
            let item = &self.cur_items()[i];
            let icon = if sel {
                &item.icon_selected
            } else {
                &item.icon_normal
            };
            let iy = y + ((row_h() as i16 - icon.height as i16) / 2).max(0);
            let _ = g.draw_pixmap(pad() * 2, iy, icon);
            let text_x = pad() * 2 + icon_size() as i16 + scaled(6) as i16;
            let fg = if sel { c.sel_fg } else { c.fg };
            let _ = g.set_foreground(fg);
            let _ = g.set_background(row_bg);
            let avail = (w as i16 - text_x - pad() * 2) as u16;
            let label = crate::applet::fit_label(g, &item.title, avail);
            let baseline = antibox_ui::metrics::baseline(y as i32, row_h() as i32) as i16;
            let _ = g.draw_text(text_x, baseline, &label);
        }
        if self.filtered.len() > self.max_rows {
            let track_h = (row_h() * vis as u16) as i32;
            let sb_w = scaled(3).max(2) as u16;
            let sb_x = w as i16 - pad() + (pad() - sb_w as i16) / 2;
            let thumb = antibox_ui::thumb::Thumb::new(
                top as i32,
                vis as i32,
                self.offset as i32,
                self.filtered.len() as i32,
            );
            let (ty, th) = thumb.rect(track_h);
            let _ = g.set_foreground(antibox_ui::theme::shadow());
            let _ = g.fill_rect(sb_x, ty as i16, sb_w, th as u16);
        }
        if self.filtered.is_empty() && !self.run_mode {
            let _ = g.set_foreground(antibox_ui::theme::disabled());
            let _ = g.set_background(c.bg);
            let baseline = antibox_ui::metrics::baseline(top as i32, row_h() as i32) as i16;
            let _ = g.draw_text(pad() * 2, baseline, "No matches");
        }
    }
}

#[cfg(test)]
#[path = "omni_tests.rs"]
mod tests;
