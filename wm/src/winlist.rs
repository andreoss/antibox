use crate::compat::{ClampExt};
use crate::manager::WindowManager;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use antibox_core::scale::scaled;
use std::sync::Arc;

use antibox_ui::searchbar::{SearchBar, SearchEvent};
use antibox_ui::theme;

fn row_h() -> i16 {
    antibox_ui::metrics::menu_item_height() as i16
}
const TOP_PAD: i16 = 3;

fn bar_h() -> i16 {
    (antibox_ui::metrics::field_height() + antibox_ui::metrics::gap()) as i16
}

fn pad() -> i16 {
    antibox_ui::metrics::pad() as i16
}

fn sb_w() -> i16 {
    scaled(16) as i16
}

enum Row {
    Header(u32),
    Win(usize),
}

impl crate::menu::MenuItem for Row {
    fn is_separator(&self) -> bool {
        does_match!(self, Row::Header(_))
    }
}

pub struct WinListMenu {
    pub window: Option<Box<dyn WindowHandle>>,
    client_id: u32,
    pub items: Vec<WinListItem>,
    rows: Vec<Row>,
    pub visible: bool,
    x: i32,
    y: i32,
    w: u16,
    h: u16,
    offset: i16,
    selected: Option<usize>,
    scroll_drag: bool,
    bar: Option<SearchBar>,
    filter: String,
}

pub struct WinListItem {
    pub title: String,
    pub client_id: u32,
    pub workspace: u32,
}

impl Default for WinListMenu {
    fn default() -> WinListMenu {
        Self::new()
    }
}

impl WinListMenu {
    pub fn new() -> WinListMenu {
        WinListMenu {
            window: None,
            client_id: 0,
            items: Vec::new(),
            rows: Vec::new(),
            visible: false,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            offset: 0,
            selected: None,
            scroll_drag: false,
            bar: None,
            filter: String::new(),
        }
    }

    fn list_top(&self) -> i16 {
        if self.bar.is_some() {
            pad() * 2 + bar_h()
        } else {
            TOP_PAD
        }
    }

    fn total_rows(&self) -> i16 {
        self.rows.len() as i16
    }
    fn vis_rows(&self) -> i16 {
        ((self.h as i16 - self.list_top() - TOP_PAD) / row_h()).max(1)
    }
    fn needs_sb(&self) -> bool {
        self.total_rows() > self.vis_rows()
    }
    fn list_w(&self) -> i16 {
        (self.w as i16 - if self.needs_sb() { sb_w() } else { 0 }).max(1)
    }
    fn max_offset(&self) -> i16 {
        (self.total_rows() - self.vis_rows()).max(0)
    }
    fn clamp_offset(&mut self) {
        self.offset = self.offset.clamped(0, self.max_offset());
    }
    fn sb_x(&self) -> i16 {
        self.list_w()
    }
    fn trough_h(&self) -> i16 {
        (self.h as i16 - 2 * sb_w()).max(1)
    }
    fn thumb(&self) -> (i16, i16) {
        let total = self.total_rows().max(1);
        let th = ((self.trough_h() as i32 * self.vis_rows() as i32 / total as i32) as i16).max(12);
        let mo = self.max_offset();
        let ty = sb_w()
            + if mo > 0 {
                (self.trough_h() - th) * self.offset / mo
            } else {
                0
            };
        (ty, th)
    }

    pub fn owns_window(&self, window: u32) -> bool {
        self.visible
            && ((self.client_id != 0 && self.client_id == window)
                || self.bar.as_ref().map_or(false, |b| b.owns_window(window)))
    }

    pub const fn client_id(&self) -> u32 {
        self.client_id
    }

    fn placement(&self, conn: &Arc<dyn DisplayBackend>) -> (i32, i32) {
        let sw = conn.screen_width() as i32;
        let sh = conn.screen_height() as i32;
        let (w, h) = (self.w as i32, self.h as i32);
        let prefs = crate::wmconfig::Config::load_prefs();
        if prefs.winlist.position == "pointer" {
            if let Ok(p) = conn.query_pointer(conn.root().read_id()) {
                let x = (p.root_x as i32 - w / 2).clamped(0, (sw - w).max(0));
                let y = (p.root_y as i32 + scaled(8)).clamped(0, (sh - h).max(0));
                return (x, y);
            }
        }
        (((sw - w) / 2).max(0), ((sh - h) / 2).max(0))
    }

    fn rebuild(&mut self, wm: &WindowManager<dyn DisplayBackend>) {
        let needle = self.filter.to_lowercase();
        let mut entries: Vec<WinListItem> = wm
            .frames
            .iter()
            .filter(|(id, fw)| {
                !fw.state().skip_taskbar && wm.xid_index.xid_of(**id) != self.client_id
            })
            .map(|(id, fw)| WinListItem {
                title: fw.client().title().to_string(),
                client_id: wm.xid_index.xid_of(*id),
                workspace: fw.workspace(),
            })
            .filter(|e| needle.is_empty() || e.title.to_lowercase().contains(&needle))
            .collect();
        entries.sort_by(|a, b| a.workspace.cmp(&b.workspace).then(a.title.cmp(&b.title)));
        let mut rows = Vec::new();
        let mut cur = std::u32::MAX;
        for (i, e) in entries.iter().enumerate() {
            if e.workspace != cur {
                cur = e.workspace;
                rows.push(Row::Header(cur));
            }
            rows.push(Row::Win(i));
        }
        let keep = self
            .selected
            .and_then(|s| self.rows.get(s))
            .and_then(|r| match r {
                Row::Win(i) => self.items.get(*i).map(|it| it.client_id),
                _ => None,
            });
        self.items = entries;
        self.rows = rows;
        self.selected = keep
            .and_then(|id| {
                self.rows
                    .iter()
                    .position(|r| does_match!(r, Row::Win(i) if self.items[*i].client_id == id))
            })
            .or_else(|| self.next_win_row(None, 1));
        self.clamp_offset();
    }

    pub fn refresh(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &WindowManager<dyn DisplayBackend>,
    ) {
        if !self.visible {
            return;
        }
        self.rebuild(wm);
        self.paint(conn);
    }

    pub fn show(&mut self, conn: &Arc<dyn DisplayBackend>, wm: &WindowManager<dyn DisplayBackend>) {
        self.client_id = 0;
        self.filter.clear();
        self.rebuild(wm);
        if self.items.is_empty() {
            return;
        }
        if self.w == 0 {
            self.w = scaled(280) as u16;
            self.h = scaled(340) as u16;
        }
        let (x, y) = self.placement(conn);
        self.x = x;
        self.y = y;
        self.offset = 0;
        self.selected = self.next_win_row(None, 1);

        let win = match conn.create_window(
            conn.root().as_parent(),
            Rect::new(self.x, self.y, self.w as i32, self.h as i32),
            WmWindowClass::InputOutput,
            false,
            EventMask::EXPOSURE
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::KEY_PRESS
                | EventMask::STRUCTURE_NOTIFY,
        ) {
            Ok(win) => win,
            Err(_) => return,
        };
        let id = win.id();
        let _ = conn.change_property8(PropMode::Replace, id, 39, 31, b"Window List");
        let _ = conn.change_property8(PropMode::Replace, id, 67, 31, b"antibox-winlist\0Antibox\0");
        if let (Some(state), Some(skip)) = (
            wm.atoms.get("_NET_WM_STATE"),
            wm.atoms.get("_NET_WM_STATE_SKIP_TASKBAR"),
        ) {
            let _ = conn.change_property32(PropMode::Replace, id, state, 4, &[skip]);
        }
        let rconn: Arc<dyn RenderBackend> = wm.render_backend.clone().expect("render backend");
        let bw = (self.w as i16 - pad() * 2).max(1) as u16;
        if let Ok(mut bar) = SearchBar::new(&rconn, id, pad(), pad(), bw, bar_h() as u16) {
            bar.set_placeholder("Search windows");
            bar.set_focus(true);
            bar.show();
            self.bar = Some(bar);
        }
        self.window = Some(win);
        self.client_id = id;
        self.visible = true;
    }

    pub fn hide(&mut self, conn: &Arc<dyn DisplayBackend>) {
        if let Some(ref win) = self.window {
            let _ = win.destroy();
            let _ = conn.flush();
        }
        self.reset();
    }

    pub fn on_destroyed(&mut self, window: u32) {
        if window == self.client_id {
            self.reset();
        }
    }

    fn reset(&mut self) {
        self.visible = false;
        self.window = None;
        self.client_id = 0;
        self.items.clear();
        self.rows.clear();
        self.selected = None;
        self.scroll_drag = false;
        self.bar = None;
        self.filter.clear();
    }

    pub fn on_configure(&mut self, conn: &Arc<dyn DisplayBackend>, w: u16, h: u16) {
        if w == 0 || h == 0 || (w == self.w && h == self.h) {
            return;
        }
        self.w = w;
        self.h = h;
        if let Some(bar) = self.bar.as_mut() {
            let bw = (w as i16 - pad() * 2).max(1) as u16;
            bar.set_rect(pad(), pad(), bw, bar_h() as u16);
        }
        self.clamp_offset();
        self.paint(conn);
    }

    fn next_win_row(&self, from: Option<usize>, dir: i32) -> Option<usize> {
        crate::menu::next_selectable(&self.rows, from, dir)
    }

    fn scroll_to_selected(&mut self) {
        if let Some(s) = self.selected {
            let s = s as i16;
            if s < self.offset {
                self.offset = s;
            } else if s >= self.offset + self.vis_rows() {
                self.offset = s - self.vis_rows() + 1;
            }
            self.clamp_offset();
        }
    }

    fn activate(
        &self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        row: usize,
    ) {
        match self.rows.get(row) {
            Some(Row::Win(i)) => {
                if let Some(item) = self.items.get(*i) {
                    let id = item.client_id;
                    if let Some(cid) = wm.cid_for_xid(id) {
                        crate::focus::activate_window(wm, cid);
                    }
                    let _ = conn.flush();
                }
            }
            Some(Row::Header(ws)) => {
                let ws = *ws;
                if ws != !0 {
                    wm.activate_workspace(ws);
                    let _ = conn.flush();
                }
            }
            None => {}
        }
    }

    fn row_at(&self, py: i16) -> Option<usize> {
        let rel = py - self.list_top();
        if rel < 0 {
            return None;
        }
        let vr = rel / row_h();
        if vr >= self.vis_rows() {
            return None;
        }
        let row = (self.offset + vr) as usize;
        if row < self.rows.len() { Some(row) } else { None }
    }

    pub fn window_at(&mut self, p: Point) -> Option<u32> {
        let row = self.row_at(p.y as i16)?;
        if let Some(Row::Win(i)) = self.rows.get(row) {
            let id = self.items.get(*i)?.client_id;
            self.selected = Some(row);
            Some(id)
        } else {
            None
        }
    }

    pub fn handle_click(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        button: u8,
        p: Point,
    ) -> bool {
        let (px, py) = (p.x as i16, p.y as i16);

        if button == 4 || button == 5 {
            self.offset += if button == 4 { -3 } else { 3 };
            self.clamp_offset();
            self.paint(conn);
            return false;
        }

        if self.needs_sb() && px >= self.sb_x() && px < self.sb_x() + sb_w() {
            let bwid = sb_w();
            if py < bwid {
                self.offset -= 1;
            } else if py >= self.h as i16 - bwid {
                self.offset += 1;
            } else {
                let (ty, tht) = self.thumb();
                if py < ty {
                    self.offset -= self.vis_rows();
                } else if py >= ty + tht {
                    self.offset += self.vis_rows();
                } else {
                    self.scroll_drag = true;
                }
            }
            self.clamp_offset();
            self.paint(conn);
            return false;
        }

        if let Some(row) = self.row_at(py) {
            if does_match!(self.rows.get(row), Some(Row::Win(_))) {
                self.selected = Some(row);
            }
            self.activate(conn, wm, row);
            self.paint(conn);
            return does_match!(self.rows.get(row), Some(Row::Win(_)) | Some(Row::Header(_)));
        }
        false
    }

    pub fn handle_motion(&mut self, conn: &Arc<dyn DisplayBackend>, p: Point) {
        if !self.scroll_drag {
            return;
        }
        let bwid = sb_w();
        let (_, tht) = self.thumb();
        let span = (self.trough_h() - tht).max(1);
        let rel = (p.y as i16 - bwid - tht / 2).clamped(0, span);
        let mo = self.max_offset();
        self.offset = if span > 0 { rel * mo / span } else { 0 };
        self.clamp_offset();
        self.paint(conn);
    }

    pub fn end_drag(&mut self) {
        self.scroll_drag = false;
    }

    fn sync_filter(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
    ) {
        let t = self
            .bar
            .as_ref()
            .map(|b| b.text().to_string())
            .unwrap_or_default();
        if t != self.filter {
            self.filter = t;
            self.offset = 0;
            self.rebuild(wm);
        }
        self.paint(conn);
    }

    pub fn handle_bar_button(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        window: u32,
        p: Point,
        button: u8,
    ) -> bool {
        let bar = match self.bar.as_mut() {
            Some(bar) => bar,
            None => return false,
        };
        if !bar.owns_window(window) {
            return false;
        }
        let ev = bar.handle_button(window, p.x, p.y, button);
        bar.repaint();
        if ev == SearchEvent::Changed {
            self.sync_filter(conn, wm);
        }
        true
    }

    pub fn handle_key_input(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        keycode: u32,
        state: u16,
        mapping: &KeyboardMapping,
        ks: u32,
    ) -> bool {
        let ev = match self.bar.as_mut() {
            Some(bar) => {
                let ev = bar.handle_key(keycode, state, mapping);
                if ev != SearchEvent::None {
                    bar.repaint();
                }
                ev
            }
            None => SearchEvent::None,
        };
        match ev {
            SearchEvent::Changed => {
                self.sync_filter(conn, wm);
                false
            }
            SearchEvent::Submitted => {
                if let Some(s) = self.selected {
                    self.activate(conn, wm, s);
                    self.paint(conn);
                }
                false
            }
            SearchEvent::Cancelled => true,
            SearchEvent::None => self.handle_key(conn, wm, ks),
        }
    }

    pub fn handle_key(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        ks: u32,
    ) -> bool {
        match ks {
            0xFF52 | 0xFF54 => {
                let dir = if ks == 0xFF52 { -1 } else { 1 };
                self.selected = self.next_win_row(self.selected, dir);
                self.scroll_to_selected();
                self.paint(conn);
            }
            0xFF50 | 0xFF57 => {
                let dir = if ks == 0xFF50 { 1 } else { -1 };
                self.selected = self.next_win_row(None, dir);
                self.scroll_to_selected();
                self.paint(conn);
            }
            0xFF55 | 0xFF56 => {
                self.offset += if ks == 0xFF55 {
                    -self.vis_rows()
                } else {
                    self.vis_rows()
                };
                self.clamp_offset();
                self.paint(conn);
            }
            0xFF0D | 0xFF8D | 0x20 => {
                if let Some(s) = self.selected {
                    self.activate(conn, wm, s);
                    self.paint(conn);
                }
            }
            0xFF1B => return true,
            _ => {}
        }
        false
    }

    pub fn paint(&self, conn: &Arc<dyn DisplayBackend>) {
        let win = match &self.window { Some(v) => v, None => return };
        crate::paintbuf::buffered(&**conn, win.id(), self.w, self.h, |g| self.paint_to(g));
        if let Some(ref bar) = self.bar {
            bar.repaint();
        }
        let _ = conn.flush();
    }

    fn paint_to(&self, g: &dyn GraphicsContext) {
        let c = crate::menu::MenuColors::default();
        let field = theme::field();
        let lw = self.list_w() as u16;
        let _ = g.set_foreground(field);
        let _ = g.fill_rect(0, 0, self.w, self.h);
        let _ = g.set_font(&FontSpec::role(
            FontRole::Switch,
            antibox_ui::metrics::font_pt(),
        ));
        for vr in 0..self.vis_rows() {
            let row_idx = (self.offset + vr) as usize;
            let row = match self.rows.get(row_idx) {
                Some(row) => row,
                None => break,
            };
            let y = self.list_top() + vr * row_h();
            match row {
                Row::Header(ws) => {
                    let _ = g.set_foreground(c.sel_bg);
                    let _ = g.fill_rect(0, y, lw, row_h() as u16);
                    let _ = g.set_font(&FontSpec::role_styled(
                        FontRole::Switch,
                        antibox_ui::metrics::font_pt(),
                        true,
                        false,
                    ));
                    let _ = g.set_foreground(c.sel_fg);
                    let _ = g.set_background(c.sel_bg);
                    let _ = g.draw_text(
                        6,
                        antibox_ui::metrics::baseline(y as i32, row_h() as i32) as i16,
                        &format!("Workspace {}", ws + 1),
                    );
                    let _ = g.set_font(&FontSpec::role(
                        FontRole::Switch,
                        antibox_ui::metrics::font_pt(),
                    ));
                }
                Row::Win(i) => {
                    if let Some(item) = self.items.get(*i) {
                        let sel = self.selected == Some(row_idx);
                        if sel {
                            crate::render::fill_menu_selection(
                                g,
                                0,
                                y,
                                lw,
                                row_h() as u16,
                                c.sel_bg,
                            );
                        }
                        let _ = g.set_foreground(if sel { c.sel_fg } else { theme::text() });
                        let _ = g.set_background(if sel { c.sel_bg } else { field });
                        let _ = g.draw_text(
                            20,
                            antibox_ui::metrics::baseline(y as i32, row_h() as i32) as i16,
                            &item.title,
                        );
                    }
                }
            }
        }
        if self.needs_sb() {
            self.paint_scrollbar(g);
        }
    }

    fn paint_scrollbar(&self, g: &dyn GraphicsContext) {
        let face = theme::face();
        let x = self.sb_x();
        let bwid = sb_w();
        let w = bwid as u16;
        let h = self.h as i16;

        let _ = g.set_foreground(crate::render::bevel_light(face));
        let _ = g.fill_rect(x, bwid, w, (h - 2 * bwid).max(1) as u16);

        let _ = g.set_foreground(face);
        let _ = g.fill_rect(x, 0, w, bwid as u16);
        let _ = crate::render::draw_button_bevel(g, x, 0, w, bwid as u16, face, false);
        Self::arrow(g, x, 0, bwid, true);

        let _ = g.set_foreground(face);
        let _ = g.fill_rect(x, h - bwid, w, bwid as u16);
        let _ = crate::render::draw_button_bevel(g, x, h - bwid, w, bwid as u16, face, false);
        Self::arrow(g, x, h - bwid, bwid, false);

        let (ty, tht) = self.thumb();
        let _ = g.set_foreground(face);
        let _ = g.fill_rect(x, ty, w, tht as u16);
        let _ = crate::render::draw_button_bevel(g, x, ty, w, tht as u16, face, false);
    }

    fn arrow(g: &dyn GraphicsContext, x: i16, y: i16, bwid: i16, up: bool) {
        let cx = x + bwid / 2;
        let cy = y + bwid / 2;
        let r = (bwid / 4).max(2);
        let _ = g.set_foreground(theme::text());
        if up {
            let _ = g.fill_polygon(&[(cx, cy - r), (cx - r, cy + r), (cx + r, cy + r)]);
        } else {
            let _ = g.fill_polygon(&[(cx, cy + r), (cx - r, cy - r), (cx + r, cy - r)]);
        }
    }
}

#[cfg(test)]
#[path = "winlist_tests.rs"]
mod tests;
