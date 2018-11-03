use crate::id::ClientId;
use crate::manager::WindowManager;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_ui::searchbar::{SearchBar, SearchEvent};
use std::sync::Arc;

pub enum SwitcherKey {
    Filtered,
    Submitted,
    Cancelled,
    Unhandled,
}

pub struct SwitcherWindow {
    pub window: Option<Box<dyn WindowHandle>>,
    all: Vec<SwitcherItem>,
    pub items: Vec<SwitcherItem>,
    pub active_idx: usize,
    pub visible: bool,
    bar: Option<SearchBar>,
}

#[derive(Clone)]
pub struct SwitcherItem {
    pub title: String,
    pub client_id: u32,
    pub members: Vec<u32>,
    pub workspace: u32,
    pub class_instance: Option<String>,
}

impl Default for SwitcherWindow {
    fn default() -> SwitcherWindow {
        Self::new()
    }
}

fn bar_h() -> i32 {
    antibox_ui::metrics::field_height() + antibox_ui::metrics::gap()
}

fn pad() -> i32 {
    antibox_ui::metrics::pad()
}

impl SwitcherWindow {
    pub fn new() -> SwitcherWindow {
        SwitcherWindow {
            window: None,
            all: Vec::new(),
            items: Vec::new(),
            active_idx: 0,
            visible: false,
            bar: None,
        }
    }

    fn rows_top() -> i32 {
        pad() + bar_h()
    }

    fn panel_h(count: usize) -> i32 {
        use antibox_core::scale::scaled;
        Self::rows_top() + (count.max(1) as i32) * scaled(48) + scaled(16)
    }

    pub fn show(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &WindowManager<dyn DisplayBackend>,
        forward: bool,
    ) {
        let cur = wm.active_workspace();
        let mut ids: Vec<ClientId> = wm
            .frames
            .iter()
            .filter(|(_, fw)| {
                let ws = fw.workspace();
                (ws == cur || ws == !0) && !fw.state().skip_taskbar
            })
            .map(|(id, _)| *id)
            .collect();
        ids.sort();
        self.all = ids
            .iter()
            .filter_map(|id| Some((*id, wm.frames.get(id)?)))
            .map(|(id, fw)| SwitcherItem {
                title: fw.client().title().to_string(),
                client_id: wm.xid_index.xid_of(id),
                members: vec![wm.xid_index.xid_of(id)],
                workspace: fw.workspace(),
                class_instance: fw.client().class_instance().map(ToString::to_string),
            })
            .collect();
        self.items = self.all.clone();
        if self.items.is_empty() {
            return;
        }
        self.active_idx = wm
            .focused_window()
            .and_then(|f| {
                self.items
                    .iter()
                    .position(|it| it.client_id == wm.xid_index.xid_of(f))
            })
            .unwrap_or(0);
        if forward {
            self.active_idx = (self.active_idx + 1) % self.items.len();
        } else {
            self.active_idx = (self.active_idx + self.items.len() - 1) % self.items.len();
        }
        use antibox_core::scale::scaled;
        let margin = scaled(40);
        let sw = conn.screen_width() as i32;
        let sh = conn.screen_height() as i32;
        let pw = scaled(420).min(sw - margin);
        let ph = Self::panel_h(self.items.len()).min(sh - margin);
        let px = (sw - pw) / 2;
        let py = (sh - ph) / 3;

        match conn.create_window(
            conn.root().as_parent(),
            Rect::new(px, py, pw, ph),
            WmWindowClass::InputOutput,
            true,
            EventMask::KEY_PRESS | EventMask::KEY_RELEASE | EventMask::EXPOSURE,
        ) {
            Ok(win) => {
                let _ = win.map();
                let _ = win.raise();
                let _ = conn.grab_keyboard(
                    false,
                    conn.root().read_id(),
                    0,
                    GrabMode::Async,
                    GrabMode::Async,
                );
                let rconn: Arc<dyn RenderBackend> = wm.render_backend.clone().expect("render backend");
                let bw = (pw - pad() * 2).max(1) as u16;
                if let Ok(mut bar) = SearchBar::new(
                    &rconn,
                    win.id(),
                    pad() as i16,
                    pad() as i16,
                    bw,
                    bar_h() as u16,
                ) {
                    bar.set_placeholder("Search windows");
                    bar.set_focus(true);
                    bar.show();
                    self.bar = Some(bar);
                }
                self.window = Some(win);
                self.visible = true;
                self.paint(conn);
                let _ = conn.flush();
            }
            Err(e) => eprintln!("switcher create_window failed: {:?}", e),
        }
    }

    fn refilter(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let needle = self
            .bar
            .as_ref()
            .map(|b| b.text().to_lowercase())
            .unwrap_or_default();
        let keep = self.items.get(self.active_idx).map(|it| it.client_id);
        self.items = self
            .all
            .iter()
            .filter(|it| {
                needle.is_empty()
                    || it.title.to_lowercase().contains(&needle)
                    || it
                        .class_instance
                        .as_ref()
                        .map_or(false, |c| c.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect();
        self.active_idx = keep
            .and_then(|id| self.items.iter().position(|it| it.client_id == id))
            .unwrap_or(0);
        if let Some(ref win) = self.window {
            use antibox_core::scale::scaled;
            let sh = conn.screen_height() as i32;
            let ph = Self::panel_h(self.items.len()).min(sh - scaled(40));
            let _ = win.configure(None, None, None, Some(ph.max(1) as u16));
        }
        self.paint(conn);
        let _ = conn.flush();
    }

    pub fn handle_search_key(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        keycode: u32,
        state: u16,
        mapping: &KeyboardMapping,
    ) -> SwitcherKey {
        let bar = match self.bar.as_mut() {
            Some(bar) => bar,
            None => return SwitcherKey::Unhandled,
        };
        match bar.handle_key(keycode, state, mapping) {
            SearchEvent::Changed => {
                bar.repaint();
                self.refilter(conn);
                SwitcherKey::Filtered
            }
            SearchEvent::Submitted => SwitcherKey::Submitted,
            SearchEvent::Cancelled => SwitcherKey::Cancelled,
            SearchEvent::None => SwitcherKey::Unhandled,
        }
    }

    pub fn hide(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
    ) {
        let _ = conn.ungrab_keyboard(0);
        if let Some(ref win) = self.window {
            let _ = win.unmap();
            let _ = win.destroy();
        }
        self.visible = false;
        self.window = None;
        self.bar = None;
        if self.active_idx < self.items.len() {
            if let Some(id) = wm.cid_for_xid(self.items[self.active_idx].client_id) {
                let prev = wm.focused_window.replace(id);
                if let Some(fw) = wm.frame(id) {
                    crate::focus::give_input_focus(
                        conn.as_ref(),
                        &wm.atoms,
                        fw,
                        wm.xid_index.xid_of(id),
                    );
                }
                wm.raise_to_top(id);
                crate::placement::restack_windows(wm);
                if let Some(p) = prev.filter(|p| *p != id) {
                    crate::handler::redraw_frame_decor(wm, p);
                }
                crate::handler::redraw_frame_decor(wm, id);
                if wm.config.warp_pointer {
                    if let Some(fw) = wm.frame(id) {
                        let r = fw.frame_rect();
                        let _ = conn.warp_pointer(
                            0,
                            fw.frame().id(),
                            Rect::ZERO,
                            antibox_core::point::Point::new(r.w / 2, r.h / 2),
                        );
                    }
                }
                let _ = conn.flush();
            }
        }
    }

    pub fn cycle(&mut self, forward: bool) {
        if self.items.is_empty() {
            return;
        }
        if forward {
            self.active_idx = (self.active_idx + 1) % self.items.len();
        } else {
            self.active_idx = if self.active_idx == 0 {
                self.items.len() - 1
            } else {
                self.active_idx - 1
            };
        }
    }

    pub fn paint(&self, conn: &Arc<dyn DisplayBackend>) {
        let win = match &self.window { Some(v) => v, None => return };
        let g = match conn.create_graphics(win.id()) {
            Ok(g) => g,
            Err(_) => return,
        };
        use antibox_core::scale::scaled;
        let c = crate::menu::MenuColors::default();
        let (face, sel_bg, sel_fg, text) = (c.bg, c.sel_bg, c.sel_fg, c.fg);
        let w: u16 = scaled(420) as u16;
        let item_h = scaled(48);
        let h: u16 = Self::panel_h(self.items.len()) as u16;
        let _ = g.set_foreground(face);
        let _ = g.fill_rect(0, 0, w, h);
        let _ = crate::render::draw_button_bevel(&*g, 0, 0, w, h, face, false);
        let _ = g.set_font(&FontSpec::role(
            FontRole::Switch,
            antibox_ui::metrics::font_pt(),
        ));
        for (i, item) in self.items.iter().enumerate() {
            let y = Self::rows_top() as i16 + i as i16 * item_h as i16;
            let active = i == self.active_idx;
            if active {
                let _ = g.set_foreground(sel_bg);
                let _ = g.fill_rect(
                    scaled(4) as i16,
                    y,
                    w - scaled(8) as u16,
                    item_h as u16 - scaled(2) as u16,
                );
            }
            let row_bg = if active { sel_bg } else { face };
            let iy: i16 = y + scaled(4) as i16;
            let _ = g.set_font(&FontSpec::role(
                FontRole::Switch,
                antibox_ui::metrics::font_pt(),
            ));
            let _ = g.set_foreground(if active { sel_fg } else { text });
            let _ = g.set_background(row_bg);
            let _ = g.draw_text(
                scaled(10) as i16,
                iy + scaled(16) as i16,
                &item.title,
            );
            let ws_text = format!("WS {}", item.workspace + 1);
            let _ = g.set_foreground(if active {
                sel_fg
            } else {
                antibox_ui::theme::disabled()
            });
            let _ = g.set_background(row_bg);
            let _ = g.draw_text(
                w as i16 - scaled(60) as i16,
                iy + scaled(16) as i16,
                &ws_text,
            );
        }
        if let Some(ref bar) = self.bar {
            bar.repaint();
        }
    }
}

#[cfg(test)]
#[path = "switcher_tests.rs"]
mod tests;
