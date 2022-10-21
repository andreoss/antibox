use crate::applet::Applet;
use crate::power::{read_status, BatteryInfo as BatInfo};
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_ui::theme;
use std::sync::Arc;

fn fmt_time(minutes: u64) -> String {
    let h = minutes / 60;
    let m = minutes % 60;
    format!("{}:{:02}", h, m)
}

fn fmt_power(microwatts: u64) -> String {
    if microwatts >= 1_000_000 {
        format!("{:.1}W", microwatts as f64 / 1_000_000.0)
    } else if microwatts >= 1000 {
        format!("{}mW", microwatts / 1000)
    } else {
        format!("{}uW", microwatts)
    }
}

const COLOR_CRITICAL: u32 = 0xCC0000;
const COLOR_LOW: u32 = 0xCC8800;
const COLOR_MEDIUM: u32 = 0xBBBB00;
const COLOR_FULL: u32 = 0x00AA00;

pub struct BatteryView {
    batteries: Vec<BatInfo>,
    ac_online: bool,
    pub vertical: bool,
}

impl BatteryView {
    pub fn new(vertical: bool) -> Self {
        let st = read_status();
        Self {
            batteries: st.batteries,
            ac_online: st.ac_online,
            vertical,
        }
    }

    pub fn update(&mut self) -> bool {
        let st = read_status();
        if st.batteries != self.batteries || st.ac_online != self.ac_online {
            self.batteries = st.batteries;
            self.ac_online = st.ac_online;
            true
        } else {
            false
        }
    }

    pub fn present(&self) -> bool {
        if self.batteries.is_empty() {
            return self.ac_online;
        }
        if self.ac_online && self.combined_percent() >= 98 {
            return false;
        }
        true
    }

    fn combined_percent(&self) -> i32 {
        if self.batteries.is_empty() {
            return -1;
        }
        let mut total_now = 0u64;
        let mut total_full = 0u64;
        for b in &self.batteries {
            total_now += b.energy_now.unwrap_or(b.percent as u64 * 100);
            total_full += b.energy_full.unwrap_or(10000);
        }
        if total_full == 0 {
            return self.batteries.iter().map(|b| b.percent).sum::<i32>()
                / self.batteries.len() as i32;
        }
        (total_now * 100 / total_full) as i32
    }

    fn combined_charging(&self) -> bool {
        self.batteries.iter().any(|b| b.charging)
    }

    fn combined_discharging(&self) -> bool {
        self.batteries.iter().any(|b| b.discharging)
    }

    fn time_remaining_secs(&self) -> Option<u64> {
        let mut energy = 0u64;
        let mut power = 0u64;
        for b in &self.batteries {
            if b.discharging {
                if let (Some(e), Some(p)) = (b.energy_now, b.power_now) {
                    energy += e;
                    power += p;
                }
            }
        }
        if power > 0 && energy > 0 {
            return Some(energy * 3600 / power);
        }
        let mut minutes = 0u64;
        let mut have_minutes = false;
        for b in &self.batteries {
            if let Some(m) = b.minutes_left {
                minutes += m as u64;
                have_minutes = true;
            }
        }
        if have_minutes {
            Some(minutes * 60)
        } else {
            None
        }
    }

    fn level_colour(&self) -> u32 {
        let pct = self.combined_percent();
        if pct < 0 {
            COLOR_FULL
        } else if pct < 10 {
            COLOR_CRITICAL
        } else if pct < 25 {
            COLOR_LOW
        } else if pct < 50 {
            COLOR_MEDIUM
        } else {
            COLOR_FULL
        }
    }

    fn is_critical(&self) -> bool {
        let pct = self.combined_percent();
        (0..10).contains(&pct) && !self.combined_charging()
    }

    pub fn tooltip(&self) -> String {
        let mut s = String::new();
        let pct = self.combined_percent();
        if pct >= 0 {
            let state = if self.combined_charging() {
                "Charging"
            } else if self.combined_discharging() {
                "Discharging"
            } else {
                "Full"
            };
            s.push_str(&format!("Battery: {}% ({})", pct, state));
        }
        for (i, b) in self.batteries.iter().enumerate() {
            s.push_str(&format!("\nBAT{}: {}%", i, b.percent));
            if b.charging {
                s.push_str(" charging");
            } else if b.discharging {
                s.push_str(" discharging");
            }
            if let Some(p) = b.power_now {
                if p > 0 {
                    s.push_str(&format!(" {}", fmt_power(p)));
                }
            }
        }
        if let Some(secs) = self.time_remaining_secs() {
            let verb = if self.combined_charging() {
                "Full in"
            } else {
                "Remaining"
            };
            s.push_str(&format!("\n{}: {}", verb, fmt_time(secs / 60)));
        }
        if self.batteries.is_empty() {
            s.push_str(if self.ac_online {
                "On AC power"
            } else {
                "No battery"
            });
        } else {
            s.push_str(if self.ac_online {
                "\nAC: connected"
            } else {
                "\nAC: disconnected"
            });
        }
        if self.is_critical() {
            s.push_str("\nBattery critically low");
        }
        s
    }

    fn outline(
        &self,
        g: &dyn GraphicsContext,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        colour: antibox_core::colour::Colour,
    ) {
        let _ = g.set_foreground(colour);
        let _ = g.fill_rect(x, y, w, 1);
        let _ = g.fill_rect(x, y + h as i16 - 1, w, 1);
        let _ = g.fill_rect(x, y, 1, h);
        let _ = g.fill_rect(x + w as i16 - 1, y, 1, h);
    }

    fn fill_colour(&self) -> Option<u32> {
        let pct = self.combined_percent();
        if pct >= 0 {
            Some(self.level_colour())
        } else if self.ac_online {
            Some(COLOR_FULL)
        } else {
            None
        }
    }

    fn draw_bolt(
        &self,
        g: &dyn GraphicsContext,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        colour: antibox_core::colour::Colour,
    ) {
        let wf = w as f32;
        let hf = h as f32;
        let pts: Vec<(i16, i16)> = [
            (0.60, 0.0),
            (0.05, 0.58),
            (0.45, 0.58),
            (0.30, 1.0),
            (0.95, 0.40),
            (0.55, 0.40),
        ]
        .iter()
        .map(|(nx, ny)| (x + (nx * wf) as i16, y + (ny * hf) as i16))
        .collect();
        let _ = g.set_foreground(colour);
        let _ = g.fill_polygon(&pts);
    }

    fn draw_exclamation(&self, g: &dyn GraphicsContext, ix: i16, iy: i16, iw: u16, ih: u16) {
        let bw = (iw / 3).max(1);
        let cx = ix + (iw as i16 - bw as i16) / 2;
        let dot_h = bw;
        let bar_h = (ih.saturating_sub(dot_h + 2) * 6 / 10).max(2);
        let top = iy + (ih as i16 - (bar_h as i16 + dot_h as i16 + 2)) / 2;
        let _ = g.set_foreground(COLOR_CRITICAL);
        let _ = g.fill_rect(cx, top, bw, bar_h);
        let _ = g.fill_rect(cx, top + bar_h as i16 + 2, bw, dot_h);
    }

    fn draw_overlays(&self, g: &dyn GraphicsContext, ix: i16, iy: i16, iw: u16, ih: u16) {
        if self.is_critical() {
            self.draw_exclamation(g, ix, iy, iw, ih);
            return;
        }
        if self.combined_charging() {
            let bh = ih.saturating_sub(3).max(6);
            let bw = (((bh as i16) * 3 / 5).clamp(4, (iw as i16 - 2).max(4))) as u16;
            let gx = ix + (iw as i16 - bw as i16) / 2;
            let gy = iy + (ih as i16 - bh as i16) / 2;
            let halo = theme::tray_face();
            for (dx, dy) in [(-1i16, 0i16), (1, 0), (0, -1), (0, 1)] {
                self.draw_bolt(g, gx + dx, gy + dy, bw, bh, halo);
            }
            self.draw_bolt(g, gx, gy, bw, bh, theme::text());
        }
    }

    pub fn draw(&self, g: &dyn GraphicsContext, x0: i16, y0: i16, w: u16, h: u16) {
        if self.vertical {
            self.draw_vertical(g, x0, y0, w, h);
        } else {
            self.draw_horizontal(g, x0, y0, w, h);
        }
    }

    fn rounded_outline(
        &self,
        g: &dyn GraphicsContext,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        colour: antibox_core::colour::Colour,
    ) {
        let wi = w as i16;
        let hi = h as i16;
        let t = antibox_core::scale::scaled(1).max(1) as i16;
        let _ = g.set_foreground(colour);
        let _ = g.fill_rect(x + 1, y, (wi - 2).max(1) as u16, t as u16);
        let _ = g.fill_rect(x + 1, y + hi - t, (wi - 2).max(1) as u16, t as u16);
        let _ = g.fill_rect(x, y + 1, t as u16, (hi - 2).max(1) as u16);
        let _ = g.fill_rect(x + wi - t, y + 1, t as u16, (hi - 2).max(1) as u16);
    }

    fn draw_vertical(&self, g: &dyn GraphicsContext, x0: i16, y0: i16, w: u16, h: u16) {
        let glyph_h = crate::status_graph::glyph_h(h as i16);
        let nub_h: i16 = antibox_core::scale::scaled(2).max(2) as i16;
        let body_h = (glyph_h - nub_h).max(6) as u16;
        let body_w = ((body_h as i16) * 3 / 5).clamp(6, (w as i16 - 2).max(6)) as u16;
        let top = y0 + ((h as i16 - glyph_h) / 2).max(0);
        let bx = x0 + (w as i16 - body_w as i16) / 2;
        let by = top + nub_h;

        let colour = if self.is_critical() {
            COLOR_CRITICAL
        } else {
            theme::text()
        };
        let nub_w = ((body_w as i16) * 2 / 5).max(3);
        let nx = bx + (body_w as i16 - nub_w) / 2;
        let _ = g.set_foreground(colour);
        let _ = g.fill_rect(nx + 1, top, (nub_w - 2).max(1) as u16, 1);
        let _ = g.fill_rect(nx, top + 1, nub_w as u16, 1);
        self.rounded_outline(g, bx, by, body_w, body_h, colour);

        let _ = g.set_foreground(theme::graph_bg());
        let _ = g.fill_rect(
            bx + 1,
            by + 1,
            body_w.saturating_sub(2),
            body_h.saturating_sub(2),
        );

        let fx = bx + 2;
        let fw = body_w.saturating_sub(4);
        let area_y = by + 2;
        let area_h = body_h.saturating_sub(4) as i16;
        let pct = self.combined_percent();
        let level = if pct >= 0 {
            pct.clamp(0, 100)
        } else if self.ac_online {
            100
        } else {
            0
        } as i16;
        let fillh = (area_h * level / 100).max(0);
        if fillh > 0 && fw > 0 {
            let _ = g.set_foreground(self.fill_colour().unwrap_or(colour));
            let _ = g.fill_rect(fx, area_y + area_h - fillh, fw, fillh as u16);
        }
        self.draw_overlays(g, bx, by, body_w, body_h);
    }

    fn draw_horizontal(&self, g: &dyn GraphicsContext, x0: i16, y0: i16, w: u16, h: u16) {
        let inset = crate::status_graph::INSET;
        let iw = (w as i16 - inset * 2).max(4);
        let ih = (h as i16 - inset * 2).max(4);
        let ox = x0 + inset;
        let oy = y0 + inset;
        let nub_w: u16 = 2;
        let body_h = (ih * 4 / 5).clamp(7, ih) as u16;
        let body_w = (iw - nub_w as i16 - 1).max(4) as u16;
        let bx = ox + 1;
        let by = oy + (ih - body_h as i16) / 2;

        let edge = if self.is_critical() {
            COLOR_CRITICAL
        } else {
            theme::text()
        };
        self.outline(g, bx, by, body_w, body_h, edge);
        let nub_h = (body_h / 2).max(2);
        let _ = g.set_foreground(edge);
        let _ = g.fill_rect(
            bx + body_w as i16,
            by + (body_h as i16 - nub_h as i16) / 2,
            nub_w,
            nub_h,
        );

        let inner_x = bx + 1;
        let inner_y = by + 1;
        let inner_w = body_w.saturating_sub(2);
        let inner_h = body_h.saturating_sub(2);
        let _ = g.set_foreground(theme::graph_bg());
        let _ = g.fill_rect(inner_x, inner_y, inner_w, inner_h);

        let pct = self.combined_percent();
        if let Some(colour) = self.fill_colour() {
            let fillw = if pct >= 0 {
                (inner_w as i32 * pct.clamp(0, 100) / 100) as u16
            } else {
                inner_w
            };
            if fillw > 0 {
                let _ = g.set_foreground(colour);
                let _ = g.fill_rect(inner_x, inner_y, fillw, inner_h);
            }
        }
        self.draw_overlays(g, inner_x, inner_y, inner_w, inner_h);
    }
}

pub struct BatteryStatusApplet {
    pub(crate) window: Box<dyn WindowHandle>,
    tooltip: Option<crate::tooltip::ToolTip>,
    conn: Arc<dyn DisplayBackend>,
    view: BatteryView,
    bg: antibox_core::colour::Colour,
    w: u16,
    h: u16,
}

impl BatteryStatusApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let h = crate::status_graph::pref_h() as u16;
        let w = ((h as u32 / 2).max(antibox_core::scale::scaled(12) as u32)) as u16;
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            EventMask::ENTER_WINDOW | EventMask::LEAVE_WINDOW | EventMask::POINTER_MOTION,
        )?;
        Ok(Self {
            conn: Arc::clone(conn),
            window,
            tooltip: None,
            view: BatteryView::new(true),
            bg: theme::tray_face(),
            w,
            h,
        })
    }

    pub fn set_colours(&mut self, bg: antibox_core::colour::Colour) {
        self.bg = bg;
    }

    pub fn update(&mut self) -> bool {
        self.view.update()
    }
}

impl Applet for BatteryStatusApplet {
    fn set_theme_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.set_colours(tc.task_bar_colour);
    }
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }

    fn paint(&self, g: &dyn GraphicsContext) {
        let _ = g.set_foreground(self.bg);
        let _ = g.fill_rect(0, 0, self.w, self.h);
        theme::well(g, 0, 0, self.w, self.h);
        self.view.draw(g, 0, 0, self.w, self.h);
    }

    fn preferred_width(&self) -> u32 {
        if self.view.present() {
            self.w as u32
        } else {
            0
        }
    }

    fn preferred_height(&self) -> u32 {
        crate::status_graph::pref_h()
    }

    fn handle_click(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
        None
    }

    impl_applet_tooltip!(view.tooltip);

    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.w = w;
        self.h = h;
        let _ = self
            .window
            .configure(Some(x as i32), Some(y as i32), Some(w), Some(h));
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "battery_status_applet_tests.rs"]
mod tests;
