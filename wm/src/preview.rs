use crate::manager::WindowManager;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::sync::Arc;

const WS_W: u16 = 160;
const WS_H: u16 = 100;
const DESK_BG: u32 = 0x336699;
const DESK_ACTIVE_BG: u32 = 0x4A7DB0;

pub struct PreviewWindow {
    pub window: Option<Box<dyn WindowHandle>>,
    pub visible: bool,
    pub ws_count: u32,
}

impl Default for PreviewWindow {
    fn default() -> PreviewWindow {
        Self::new()
    }
}

impl PreviewWindow {
    pub fn new() -> PreviewWindow {
        PreviewWindow {
            window: None,
            visible: false,
            ws_count: 4,
        }
    }

    pub fn show(&mut self, conn: &Arc<dyn DisplayBackend>, wm: &WindowManager<dyn DisplayBackend>) {
        self.ws_count = wm.config.workspace_count;
        let cols = 2;
        let rows = (self.ws_count as u16 + cols - 1) / cols;
        let pw = cols * (WS_W + 8) + 4;
        let ph = rows * (WS_H + 8) + 4;
        if let Ok(win) = conn.create_window(
            conn.root().as_parent(),
            Rect::new(200, 100, pw as i32, ph as i32),
            WmWindowClass::InputOutput,
            true,
            EventMask::NO_EVENT,
        ) {
            let _ = win.map();
            self.window = Some(win);
            self.visible = true;
            self.paint(conn, wm);
        }
    }

    pub fn hide(&mut self) {
        if let Some(ref win) = self.window {
            let _ = win.unmap();
            let _ = win.destroy();
        }
        self.visible = false;
        self.window = None;
    }

    pub fn paint(&self, conn: &Arc<dyn DisplayBackend>, wm: &WindowManager<dyn DisplayBackend>) {
        let win = match &self.window { Some(v) => v, None => return };
        let g = match conn.create_graphics(win.id()) {
            Ok(g) => g,
            Err(_) => return,
        };
        let cols = 2;
        let pw = 2 * (WS_W + 8) + 4;
        let rows = (self.ws_count as u16 + 1) / 2;
        let ph = rows * (WS_H + 8) + 4;
        let face = antibox_ui::theme::face();
        let _ = g.set_foreground(face);
        let _ = g.fill_rect(0, 0, pw, ph);
        let _ = crate::render::draw_button_bevel(&*g, 0, 0, pw, ph, face, false);
        for wi in 0..self.ws_count {
            let cx = (wi % cols) as i16 * (WS_W as i16 + 8) + 4;
            let cy = (wi / cols) as i16 * (WS_H as i16 + 8) + 4;
            let active = wi == wm.active_workspace();
            let _ = g.set_foreground(if active { DESK_ACTIVE_BG } else { DESK_BG });
            let _ = g.fill_rect(cx, cy, WS_W, WS_H);
            let _ = crate::render::draw_button_bevel(&*g, cx, cy, WS_W, WS_H, face, true);
            for fw in wm
                .frames
                .values()
                .filter(|f| f.workspace() == wi && !f.state().minimized)
            {
                let fr = fw.frame_rect();
                let sx = cx + 4 + (fr.x as i16 / 8).clamp(0, WS_W as i16 - 20);
                let sy = cy + 4 + (fr.y as i16 / 8).clamp(0, WS_H as i16 - 14);
                let sw = (fr.w as u16 / 8).clamp(8, WS_W - sx as u16 + cx as u16 - 8);
                let sh = (fr.h as u16 / 8).clamp(8, WS_H - sy as u16 + cy as u16 - 8);
                let sw = sw.min(WS_W - 8);
                let sh = sh.min(WS_H - 8);
                let focused = wm.focused_window == Some(fw.client_id());
                let _ = g.set_foreground(antibox_ui::theme::field());
                let _ = g.fill_rect(sx, sy, sw, sh);
                let _ = g.set_foreground(if focused {
                    antibox_ui::theme::title_active()
                } else {
                    antibox_ui::theme::title_inactive()
                });
                let _ = g.fill_rect(sx, sy, sw, 3.min(sh));
                let _ = g.set_foreground(antibox_ui::theme::shadow());
                let _ = g.draw_rect(sx, sy, sw, sh);
            }
        }
    }

    pub fn handle_click(&self, wm: &mut WindowManager<dyn DisplayBackend>, p: Point) {
        let active = self.hit_test(p);
        if active < self.ws_count {
            wm.activate_workspace(active);
        }
    }

    fn hit_test(&self, p: Point) -> u32 {
        let cols = 2;
        let col = (p.x - 4) / (WS_W as i32 + 8);
        let row = (p.y - 4) / (WS_H as i32 + 8);
        let idx = row as u32 * cols + col as u32;
        idx.min(self.ws_count - 1)
    }
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
