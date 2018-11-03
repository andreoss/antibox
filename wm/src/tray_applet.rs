use crate::compat::{ClampExt};
use crate::applet::Applet;
use crate::tooltip::ToolTip;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

const SYSTEM_TRAY_REQUEST_DOCK: u32 = 0;
const ATOM_ATOM: u32 = 4;
const CURRENT_TIME: u32 = 0;

const XEMBED_EMBEDDED_NOTIFY: u32 = 0;
const XEMBED_REQUEST_FOCUS: u32 = 3;

const ATOM_STRING: u32 = 31;

pub struct TrayApplet {
    pub(crate) window: Box<dyn WindowHandle>,
    conn: Arc<dyn DisplayBackend>,
    tray_atom: u32,
    opcode_atom: u32,
    xembed_atom: u32,
    xembed_info_atom: u32,
    net_wm_name_atom: u32,
    pub(crate) embedded: Vec<EmbeddedClient>,
    tray: Option<Arc<dyn TrayBackend>>,
    damage_ids: Vec<(u32, u32)>,
    composite_available: bool,
    hovered: Option<usize>,
    tooltip: Option<ToolTip>,
    draw_bevel: bool,
    height: u16,
    bg: antibox_core::colour::Colour,
}

pub struct EmbeddedClient {
    pub window: u32,
    pub width: u16,
    pub height: u16,
    pub xembed_version: u32,
    pub xembed_mapped: bool,
    pub title: String,
}

impl TrayApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        atom_manager: &AtomManager,
        tray: Option<Arc<dyn TrayBackend>>,
        composite_available: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, 24, 24),
            WmWindowClass::InputOutput,
            true,
            EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::POINTER_MOTION,
        )?;
        let tray_atom = atom_manager.get("_NET_SYSTEM_TRAY_S0").unwrap_or(0);
        let opcode_atom = atom_manager.get("_NET_SYSTEM_TRAY_OPCODE").unwrap_or(0);
        let xembed_atom = atom_manager.get("_XEMBED").unwrap_or(0);
        let xembed_info_atom = atom_manager.get("_XEMBED_INFO").unwrap_or(0);
        let net_wm_name_atom = atom_manager.get("_NET_WM_NAME").unwrap_or(0);

        let tray = TrayApplet {
            window,
            conn: Arc::clone(conn),
            tray_atom,
            opcode_atom,
            xembed_atom,
            xembed_info_atom,
            net_wm_name_atom,
            embedded: Vec::new(),
            tray,
            damage_ids: Vec::new(),
            composite_available,
            hovered: None,
            tooltip: None,
            draw_bevel: true,
            bg: antibox_ui::theme::tray_face(),
            height: 28,
        };

        let root = tray.conn.root().read_id();
        let _ = tray
            .conn
            .set_selection_owner(tray.window.id(), tray.tray_atom, CURRENT_TIME);
        let _ = tray.conn.change_property32(
            PropMode::Replace,
            root,
            tray.tray_atom,
            ATOM_ATOM,
            &[tray.window.id()],
        );
        if let Some(manager_atom) = atom_manager.get("MANAGER") {
            const STRUCTURE_NOTIFY: u32 = 1 << 17;
            let _ = tray.conn.send_event(
                false,
                root,
                STRUCTURE_NOTIFY,
                manager_atom,
                &[CURRENT_TIME, tray.tray_atom, tray.window.id(), 0, 0],
            );
        }

        if let Some(orient_atom) = atom_manager.get("_NET_SYSTEM_TRAY_ORIENTATION") {
            let _ = tray.conn.change_property32(
                PropMode::Replace,
                tray.window.id(),
                orient_atom,
                6,
                &[0u32],
            );
        }
        if let Some(visual_atom) = atom_manager.get("_NET_SYSTEM_TRAY_VISUAL") {
            let visual = if composite_available {
                tray.conn
                    .argb_visual()
                    .unwrap_or_else(|| tray.conn.root_visual())
            } else {
                tray.conn.root_visual()
            };
            let _ = tray.conn.change_property32(
                PropMode::Replace,
                tray.window.id(),
                visual_atom,
                32,
                &[visual],
            );
        }

        Ok(tray)
    }

    pub fn set_colours(&mut self) {
        self.bg = antibox_ui::theme::tray_face();
    }

    pub fn handle_client_message(&mut self, msg_type: u32, data: &[u32; 5]) {
        if msg_type != self.opcode_atom {
            return;
        }
        if data[1] == SYSTEM_TRAY_REQUEST_DOCK {
            let client = data[2];
            self.embed_client(client);
        }
    }

    fn fetch_client_title(&self, window: u32) -> String {
        let c = match self.tray {
            Some(ref c) => c,
            None => return String::new(),
        };
        if self.net_wm_name_atom != 0 {
            if let Some(s) = c.get_text_property(window, self.net_wm_name_atom) {
                return s;
            }
        }
        c.get_text_property(window, ATOM_STRING).unwrap_or_default()
    }

    fn embed_client(&mut self, client: u32) {
        if self.composite_available {
            if let Some(ref c) = self.tray {
                c.composite_redirect(client);
                if let Some(damage_id) = c.create_damage(client) {
                    self.damage_ids.push((client, damage_id));
                }
            }
        }
        let _ = self.conn.reparent_window(client, self.window.id(), antibox_core::point::Point::ZERO);

        let _ = self.conn.change_save_set(client, true);
        if !self.composite_available {
            let opaque_bg = 0xFF00_0000 | (self.bg & 0x00FF_FFFF);
            let _ = self
                .conn
                .change_window_attributes(client, &[1 << 1, opaque_bg]);
        }
        let mut xembed_version = 0u32;
        let mut xembed_mapped = true;
        if let Some(ref c) = self.tray {
            if let Some((version, mapped)) = c.get_xembed_info(client, self.xembed_info_atom) {
                xembed_version = version;
                xembed_mapped = mapped;
            }
        }
        let (cw, ch) = self
            .conn
            .wrap_window(client)
            .ok()
            .and_then(|w| w.get_geometry().ok())
            .map_or((24, 24), |(w, h)| (w.max(1), h.max(1)));
        let title = self.fetch_client_title(client);
        let _ = self.conn.map_window(client);
        self.send_xembed_message(client, XEMBED_EMBEDDED_NOTIFY, 0, self.window.id(), 0);
        self.embedded.push(EmbeddedClient {
            window: client,
            width: cw,
            height: ch,
            xembed_version,
            xembed_mapped,
            title,
        });
        self.relayout();
    }

    fn send_xembed_message(&self, target: u32, message: u32, detail: u32, data1: u32, data2: u32) {
        if let Some(ref c) = self.tray {
            let data = [target, message, detail, data1, data2];
            c.send_xembed(target, self.xembed_atom, data);
        }
    }

    pub fn handle_xembed_message(&mut self, msg_window: u32, data: &[u32; 5]) {
        let idx = match self.embedded.iter().position(|e| e.window == msg_window) {
            Some(idx) => idx,
            None => return,
        };
        let msg = data[1];
        match msg {
            XEMBED_REQUEST_FOCUS => {
                let _ = self.conn.set_input_focus(0, msg_window, CURRENT_TIME);
            }
            _ => {
                eprintln!("unhandled XEMBED message: {} window={}", msg, msg_window);
            }
        }
        self.embedded[idx].title = self.fetch_client_title(msg_window);
    }

    fn slot_count(&self) -> usize {
        self.embedded.len()
    }

    fn tpad(&self) -> i16 {
        antibox_ui::metrics::pad() as i16
    }

    fn tgap(&self) -> i16 {
        antibox_ui::metrics::pad() as i16
    }

    fn icon_size(&self) -> u16 {
        let margin = antibox_ui::metrics::gap() as i16;
        (self.height as i16 - 2 * margin).clamped(8, 64) as u16
    }

    fn slot(&self) -> u16 {
        (self.icon_size() as i16 + self.tgap()) as u16
    }

    fn relayout(&mut self) {
        let icon = self.icon_size();
        let pad = self.tpad();
        let gap = self.tgap();
        let y = ((self.height as i16 - icon as i16) / 2).max(0);
        let mut x = pad;
        for client in &self.embedded {
            let _ = self.conn.configure_window(
                client.window,
                &[x as u32, y as u32, icon as u32, icon as u32],
            );
            let _ = if client.xembed_mapped {
                self.conn.map_window(client.window)
            } else {
                self.conn.unmap_window(client.window)
            };
            x += icon as i16 + gap;
        }
    }

    fn destroy_damage(&self, damage_id: u32) {
        if let Some(ref c) = self.tray {
            c.destroy_damage(damage_id);
        }
    }

    fn unredirect_client(&self, client: u32) {
        if let Some(ref c) = self.tray {
            c.composite_unredirect(client);
        }
    }

    pub fn forget_window(&mut self, window: u32) -> bool {
        if self.embedded_idx(window).is_some() {
            self.remove_client(window);
            true
        } else {
            false
        }
    }

    fn remove_client(&mut self, window: u32) {
        if let Some(idx) = self.embedded.iter().position(|e| e.window == window) {
            self.embedded.remove(idx);
            let mut to_remove = Vec::new();
            let mut i = 0;
            while i < self.damage_ids.len() {
                if self.damage_ids[i].0 == window {
                    let (c, d) = self.damage_ids.remove(i);
                    to_remove.push((c, d));
                } else {
                    i += 1;
                }
            }
            for (c, d) in to_remove {
                self.destroy_damage(d);
                self.unredirect_client(c);
            }
            if self.hovered == Some(idx) {
                self.hovered = None;
                self.hide_tooltip();
            } else if let Some(h) = self.hovered {
                if h > idx {
                    self.hovered = Some(h - 1);
                }
            }
            self.relayout();
        }
    }

    fn embedded_idx(&self, window: u32) -> Option<usize> {
        self.embedded.iter().position(|e| e.window == window)
    }

    fn show_tooltip_at(&mut self, idx: usize) {
        if idx >= self.embedded.len() {
            return;
        }
        let title = self.embedded[idx].title.clone();
        if title.is_empty() {
            return;
        }
        let (w, h) = (self.embedded[idx].width, self.embedded[idx].height);
        self.tooltip.get_or_insert_with(ToolTip::new).request(
            self.conn.as_ref(),
            &title,
            Rect::new(0, 0, w as i32, h as i32),
        );
    }

    fn hide_tooltip(&mut self) {
        if let Some(ref mut tt) = self.tooltip {
            tt.hide(self.conn.as_ref());
        }
    }
}

impl Drop for TrayApplet {
    fn drop(&mut self) {
        for (client, damage_id) in &self.damage_ids {
            self.destroy_damage(*damage_id);
            self.unredirect_client(*client);
        }
    }
}

impl Applet for TrayApplet {
    fn set_theme_colours(&mut self, _tc: &crate::render::ThemeColors) {
        self.set_colours();
    }
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }

    fn paint(&self, g: &dyn GraphicsContext) {
        let w = self.preferred_width() as u16;
        let h = self.height;
        let _ = g.set_foreground(self.bg);
        let _ = g.fill_rect(0, 0, w, h);
        if self.draw_bevel {
            antibox_ui::theme::well(g, 0, 0, w, h);
        }
        let icon = self.icon_size();
        let slot = self.slot();
        let pad = self.tpad();
        let iy = ((h as i16 - icon as i16) / 2).max(0);
        for (i, _) in self.embedded.iter().enumerate() {
            let x = pad + i as i16 * slot as i16;
            if self.hovered == Some(i) {
                let _ = g.set_foreground(crate::render::darken_colour(self.bg, 0.85));
                let _ = g.fill_rect(x, iy, icon, icon);
            }
        }
    }

    fn preferred_width(&self) -> u32 {
        let slots = self.slot_count() as u32;
        if slots == 0 {
            return 0;
        }
        let pad = self.tpad() as u32;
        let icon = self.icon_size() as u32;
        let gap = self.tgap() as u32;
        let content = self.embedded.len() as u32 * icon;
        2 * pad + content + slots.saturating_sub(1) * gap
    }

    fn preferred_height(&self) -> u32 {
        antibox_ui::metrics::panel_height() as u32
    }

    fn handle_click(&mut self, x: i32, _y: i32, _button: u8) -> Option<u32> {
        if !self.embedded.is_empty() {
            let idx = ((x - self.tpad() as i32).max(0) / self.slot() as i32) as usize;
            if idx < self.embedded.len() {
                return Some(self.embedded[idx].window);
            }
        }
        None
    }

    fn handle_motion(&mut self, _x: i32, _y: i32) {
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn handle_leave(&mut self) {
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn tick_tooltip(&mut self) {
        crate::tooltip::pump_tip(&mut self.tooltip, self.conn.as_ref());
    }
    fn tooltip_pending(&self) -> bool {
        crate::tooltip::tip_pending(&self.tooltip)
    }

    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.height = h;
        let _ = self
            .window
            .configure(Some(x as i32), Some(y as i32), Some(w), Some(h));
        self.relayout();
    }

    fn owns_window(&self, id: u32) -> bool {
        if self.window.id() == id {
            return true;
        }
        self.embedded.iter().any(|e| e.window == id)
    }

    fn handle_other_event(&mut self, event: &BackendEvent, _conn: &Arc<dyn DisplayBackend>) {
        match event {
            BackendEvent::DestroyNotify { window } => {
                self.remove_client(*window);
            }
            BackendEvent::EnterNotify { window, .. } => {
                if let Some(idx) = self.embedded_idx(*window) {
                    self.hovered = Some(idx);
                    self.show_tooltip_at(idx);
                }
            }
            BackendEvent::LeaveNotify { window, .. } if self.embedded_idx(*window).is_some() => {
                self.hovered = None;
                self.hide_tooltip();
            }
            BackendEvent::ConfigureRequest { window, rect, .. } => {
                if let Some(idx) = self.embedded_idx(*window) {
                    let w = rect.w.max(1) as u16;
                    let h = rect.h.max(1) as u16;
                    self.embedded[idx].width = w;
                    self.embedded[idx].height = h;
                    self.relayout();
                }
            }
            BackendEvent::PropertyNotify { window, atom, .. } => {
                if let Some(idx) = self.embedded_idx(*window) {
                    if *atom == self.net_wm_name_atom {
                        self.embedded[idx].title = self.fetch_client_title(*window);
                    }
                }
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "tray_applet_tests.rs"]
mod tests;
