pub(crate) use crate::listview::{bar_h, pad, row_h, sb_w};
use crate::listview::{ListNav, ListView};
use crate::manager::WindowManager;
use crate::menu_tree::MenuNode;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::scale::scaled;
use std::sync::Arc;

use antibox_ui::theme;

pub(crate) use crate::listview::row_icon_px;

pub struct WinListItem {
    pub title: String,
    pub client_id: u32,
    pub workspace: u32,
    pub icon: Option<PixmapData>,
}

impl Clone for WinListItem {
    fn clone(&self) -> Self {
        WinListItem {
            title: self.title.clone(),
            client_id: self.client_id,
            workspace: self.workspace,
            icon: self.icon.clone(),
        }
    }
}

pub struct WinListMenu {
    pub view: ListView<WinListItem>,
    client_id: u32,
    pub visible: bool,
    pub switcher: bool,
    active_ws: u32,
}

impl Default for WinListMenu {
    fn default() -> WinListMenu {
        Self::new()
    }
}

impl WinListMenu {
    pub fn new() -> WinListMenu {
        WinListMenu {
            view: ListView::new(),
            client_id: 0,
            visible: false,
            switcher: false,
            active_ws: 0,
        }
    }

    pub const fn client_id(&self) -> u32 {
        self.client_id
    }

    pub fn owns_window(&self, window: u32) -> bool {
        self.visible && (self.view.owns_window(window) || self.client_id == window)
    }

    fn placement(&self, conn: &dyn DisplayBackend) -> (i32, i32) {
        let sw = conn.screen_width() as i32;
        let sh = conn.screen_height() as i32;
        let (w, h) = (self.view.w as i32, self.view.h as i32);
        let prefs = crate::wmconfig::Config::load_prefs();
        if prefs.winlist.position == "pointer" {
            if let Ok(p) = conn.query_pointer(conn.root().read_id()) {
                let x = (p.root_x as i32 - w / 2).clamp(0, (sw - w).max(0));
                let y = (p.root_y as i32 + scaled(8)).clamp(0, (sh - h).max(0));
                return (x, y);
            }
        }
        (((sw - w) / 2).max(0), ((sh - h) / 2).max(0))
    }

    fn build_tree(&mut self, wm: &WindowManager<dyn DisplayBackend>) -> Vec<MenuNode<WinListItem>> {
        self.active_ws = wm.active_workspace();
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
                icon: Some(crate::icon_render::resolve_client_icon(
                    fw.client().icons(),
                    row_icon_px(),
                    theme::field(),
                )),
            })
            .collect();
        entries.sort_by(|a, b| a.workspace.cmp(&b.workspace).then(a.title.cmp(&b.title)));

        let mut nodes: Vec<MenuNode<WinListItem>> = Vec::new();
        let mut cur = u32::MAX;
        for e in entries {
            if e.workspace != cur {
                cur = e.workspace;
                let label = if cur == self.active_ws {
                    format!("Workspace {} *", cur + 1)
                } else {
                    format!("Workspace {}", cur + 1)
                };
                nodes.push(MenuNode::group_expanded(label, Vec::new()));
            }
            if let Some(MenuNode::Group { children, .. }) = nodes.last_mut() {
                children.push(MenuNode::leaf(e.title.clone(), e.clone()).with_icon(e.icon.clone()));
            }
        }
        nodes
    }

    fn rebuild(&mut self, wm: &WindowManager<dyn DisplayBackend>) {
        let keep = self.selected_client_id();
        let nodes = self.build_tree(wm);
        self.view.set_tree(nodes);
        self.view.selected = keep
            .and_then(|id| self.row_of_client(id))
            .or_else(|| self.view.next_leaf(None, 1));
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
        self.view.sync_geometry(conn.as_ref());
        self.paint(conn);
    }

    pub fn show(&mut self, conn: &Arc<dyn DisplayBackend>, wm: &WindowManager<dyn DisplayBackend>) {
        self.show_common(conn, wm, false, true);
    }

    pub fn show_switcher(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &WindowManager<dyn DisplayBackend>,
        forward: bool,
    ) {
        self.show_common(conn, wm, true, forward);
    }

    fn show_common(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &WindowManager<dyn DisplayBackend>,
        switcher: bool,
        forward: bool,
    ) {
        self.client_id = 0;
        let nodes = self.build_tree(wm);
        let rows = crate::menu_tree::flatten_nodes(&nodes).len();
        if rows == 0 {
            return;
        }
        let w = scaled(280) as u16;
        let want = pad() as i32 * 3 + bar_h() as i32 + rows as i32 * row_h() as i32;
        let h = if switcher {
            let max_h = (conn.screen_height() as i32 * 3 / 4).max(scaled(120));
            want.clamp(scaled(120), max_h) as u16
        } else {
            want.clamp(scaled(120), scaled(340)) as u16
        };
        self.view.set_size(w, h);
        let (x, y) = self.placement(conn.as_ref());
        let atoms_state = wm.atoms.get("_NET_WM_STATE");
        let atoms_skip = wm.atoms.get("_NET_WM_STATE_SKIP_TASKBAR");
        let atoms_mwm = wm.atoms.get("_MOTIF_WM_HINTS");
        self.view.show_with(
            conn.as_ref(),
            nodes,
            crate::listview::Place::Anchor(Point::new(x, y + h as i32)),
            w,
            h as i32,
            switcher,
            |c, id| {
                let _ = c.change_property8(PropMode::Replace, id, 39, 31, b"Window List");
                let _ = c.change_property8(
                    PropMode::Replace,
                    id,
                    67,
                    31,
                    b"antibox-winlist\0Antibox\0",
                );
                if let (Some(state), Some(skip)) = (atoms_state, atoms_skip) {
                    let _ = c.change_property32(PropMode::Replace, id, state, 4, &[skip]);
                }
                if let Some(mwm) = atoms_mwm {
                    use antibox_core::backend::hints::{mwm_func, mwm_hints_flags};
                    let _ = c.change_property32(
                        PropMode::Replace,
                        id,
                        mwm,
                        mwm,
                        &[
                            mwm_hints_flags::FUNCTIONS,
                            mwm_func::ALL | mwm_func::MAXIMIZE | mwm_func::MINIMIZE,
                            0,
                            0,
                            0,
                        ],
                    );
                }
                if switcher {
                    let _ = c.grab_keyboard(
                        false,
                        c.root().read_id(),
                        0,
                        GrabMode::Async,
                        GrabMode::Async,
                    );
                }
                let _ = id;
            },
        );
        if let Some(rb) = wm.render_backend.clone() {
            self.view.enable_filter(&rb);
        }
        self.view.pos_set(Point::new(x, y));
        self.switcher = switcher;
        self.client_id = self.view.window_id();
        self.visible = self.view.visible;
        if switcher {
            self.view.selected = self.selection_from_focus(wm, forward);
        } else {
            self.view.selected = self.view.next_leaf(None, 1);
        }
        self.paint(conn);
    }

    pub fn cycle(&mut self, conn: &Arc<dyn DisplayBackend>, forward: bool) {
        let dir = if forward { 1 } else { -1 };
        self.view.selected = self
            .view
            .next_leaf(self.view.selected, dir)
            .or_else(|| self.view.next_leaf(None, dir));
        self.view.scroll_to_selected();
        self.paint(conn);
    }

    pub fn activate_selected(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
    ) {
        if let Some(s) = self.view.selected {
            self.activate(conn, wm, s);
        }
    }

    pub fn selected_client_id(&self) -> Option<u32> {
        let s = self.view.selected?;
        let row = self.view.items.get(s)?;
        row.payload().map(|p| p.client_id)
    }

    pub fn hide(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let was_switcher = self.switcher;
        self.view.hide(conn.as_ref());
        self.reset();
        if was_switcher {
            let _ = conn.ungrab_keyboard(0);
            let _ = conn.flush();
        }
    }

    pub fn on_destroyed(&mut self, window: u32) {
        if window == self.client_id {
            self.reset();
        }
    }

    fn reset(&mut self) {
        self.visible = false;
        self.switcher = false;
        self.client_id = 0;
        self.view = ListView::new();
    }

    pub fn on_configure(&mut self, conn: &Arc<dyn DisplayBackend>, w: u16, h: u16) {
        if w == 0 || h == 0 || (w == self.view.w && h == self.view.h) {
            return;
        }
        self.view.set_size(w, h);
        self.paint(conn);
    }

    fn row_of_client(&self, xid: u32) -> Option<usize> {
        self.view
            .items
            .iter()
            .position(|r| r.payload().map_or(false, |p| p.client_id == xid))
    }

    fn selection_from_focus(
        &self,
        wm: &WindowManager<dyn DisplayBackend>,
        forward: bool,
    ) -> Option<usize> {
        let fid = wm.focused_window().map(|f| wm.xid_index.xid_of(f));
        if forward {
            let prev = wm
                .last_focused_window
                .map(|p| wm.xid_index.xid_of(p))
                .filter(|p| Some(*p) != fid)
                .and_then(|p| self.row_of_client(p));
            if prev.is_some() {
                return prev;
            }
        }
        let start = fid.and_then(|id| self.row_of_client(id));
        let dir = if forward { 1 } else { -1 };
        self.view
            .next_leaf(start, dir)
            .or_else(|| self.view.next_leaf(None, dir))
    }

    fn activate(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        row: usize,
    ) {
        let payload = match self.view.items.get(row) {
            Some(r) if r.is_group() => {
                self.view.toggle_row(conn.as_ref(), row);
                let _ = conn.flush();
                return;
            }
            Some(r) => r.payload().cloned(),
            None => return,
        };
        if let Some(p) = payload {
            if let Some(cid) = wm.cid_for_xid(p.client_id) {
                crate::focus::activate_window(wm, cid);
            }
        }
        let _ = conn.flush();
    }

    pub fn window_at(&mut self, p: Point) -> Option<u32> {
        let row = self.view.row_at(p)?;
        let r = self.view.items.get(row)?;
        let id = r.payload()?.client_id;
        self.view.selected = Some(row);
        Some(id)
    }

    pub fn handle_click(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        button: u8,
        p: Point,
    ) -> bool {
        if button == 4 || button == 5 {
            self.view.scroll_by(if button == 4 { -3 } else { 3 });
            self.paint(conn);
            return false;
        }
        if self.view.needs_sb()
            && p.x - self.view.pos().x >= self.view.sb_x() as i32
            && p.x - self.view.pos().x < self.view.sb_x() as i32 + sb_w() as i32
        {
            if self.view.sb_hit(p) {
                self.paint(conn);
            }
            return false;
        }
        let row = match self.view.row_at(p) {
            Some(r) => r,
            None => return false,
        };
        if let Some(r) = self.view.items.get(row) {
            if r.payload().is_some() {
                self.view.selected = Some(row);
            }
        }
        let activated = self
            .view
            .items
            .get(row)
            .map_or(false, |r| r.payload().is_some());
        self.activate(conn, wm, row);
        self.paint(conn);
        activated
    }

    pub fn handle_motion(&mut self, conn: &Arc<dyn DisplayBackend>, p: Point) {
        if self.view.scroll_drag {
            self.view.handle_motion(conn.as_ref(), p);
        }
    }

    pub fn end_drag(&mut self) {
        self.view.end_drag();
    }

    pub fn handle_bar_button(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        window: u32,
        p: Point,
        button: u8,
    ) -> bool {
        if !self.view.handle_bar_button(conn.as_ref(), window, p, button) {
            return false;
        }
        self.rebuild(wm);
        self.view.sync_geometry(conn.as_ref());
        self.paint(conn);
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
        match self.view.bar_key(keycode, state, mapping) {
            1 => {
                self.rebuild(wm);
                self.view.sync_geometry(conn.as_ref());
                self.paint(conn);
                false
            }
            2 => self.handle_key(conn, wm, 0xFF0D),
            3 => true,
            _ => self.handle_key(conn, wm, ks),
        }
    }

    pub fn handle_key(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        ks: u32,
    ) -> bool {
        match self.view.handle_key(conn.as_ref(), ks, false) {
            ListNav::Close => true,
            ListNav::Activate(item) => {
                if let Some(cid) = wm.cid_for_xid(item.client_id) {
                    crate::focus::activate_window(wm, cid);
                }
                self.paint(conn);
                let _ = conn.flush();
                false
            }
            _ => false,
        }
    }

    pub fn paint(&self, conn: &Arc<dyn DisplayBackend>) {
        self.view.paint(conn.as_ref());
    }
}

#[cfg(test)]
#[path = "winlist_tests.rs"]
mod tests;
