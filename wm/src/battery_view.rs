use crate::icon_dsl::{COLOR_CRITICAL, COLOR_GOOD as COLOR_FULL, COLOR_LOW, COLOR_MEDIUM};
use crate::power::{read_status, BatteryInfo as BatInfo};
use antibox_core::backend::GraphicsContext;
use antibox_ui::theme;

fn line_thickness() -> i16 {
    antibox_core::scale::scaled(2).max(2) as i16
}

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

pub struct BatteryView {
    pub(crate) batteries: Vec<BatInfo>,
    pub(crate) ac_online: bool,
    pub vertical: bool,
}

impl BatteryView {
    pub fn new(vertical: bool) -> Self {
        Self::from_status(read_status(), vertical)
    }

    pub fn from_status(st: crate::power::PowerStatus, vertical: bool) -> Self {
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
        let t = line_thickness();
        let tu = t as u16;
        let _ = g.set_foreground(colour);
        let _ = g.fill_rect(x, y, w, tu);
        let _ = g.fill_rect(x, y + h as i16 - t, w, tu);
        let _ = g.fill_rect(x, y, tu, h);
        let _ = g.fill_rect(x + w as i16 - t, y, tu, h);
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

    fn draw_exclamation(
        &self,
        g: &dyn GraphicsContext,
        ix: i16,
        iy: i16,
        iw: u16,
        ih: u16,
        colour: antibox_core::colour::Colour,
    ) {
        let bw = (iw / 3).max(1);
        let cx = ix + (iw as i16 - bw as i16) / 2;
        let dot_h = bw;
        let bar_h = (ih.saturating_sub(dot_h + 2) * 6 / 10).max(2);
        let top = iy + (ih as i16 - (bar_h as i16 + dot_h as i16 + 2)) / 2;
        let _ = g.set_foreground(colour);
        let _ = g.fill_rect(cx, top, bw, bar_h);
        let _ = g.fill_rect(cx, top + bar_h as i16 + 2, bw, dot_h);
    }

    fn draw_overlays(&self, g: &dyn GraphicsContext, ix: i16, iy: i16, iw: u16, ih: u16) {
        if self.is_critical() {
            self.draw_exclamation(g, ix, iy, iw, ih, COLOR_CRITICAL);
            return;
        }
        if self.combined_charging() {
            let bh = ih.saturating_sub(3).max(6);
            let bw = (((bh as i16) * 3 / 5).clamp(4, (iw as i16 - 2).max(4))) as u16;
            let gx = ix + (iw as i16 - bw as i16) / 2;
            let gy = iy + (ih as i16 - bh as i16) / 2;
            crate::icon_dsl::POWER_ICON.draw_outlined(
                g,
                gx,
                gy,
                bw,
                bh,
                theme::light(),
                theme::text(),
            );
        }
    }

    pub fn draw(&self, g: &dyn GraphicsContext, x0: i16, y0: i16, w: u16, h: u16) {
        if self.vertical {
            self.draw_vertical(g, x0, y0, w, h);
        } else {
            self.draw_horizontal(g, x0, y0, w, h);
        }
    }

    fn draw_vertical(&self, g: &dyn GraphicsContext, x0: i16, y0: i16, w: u16, h: u16) {
        use crate::icon_dsl::{self, BATTERY_ICON, POWER_ICON};
        let gh = icon_dsl::glyph_zone_h(h);
        let gw = (gh * 3 / 5).clamp(7, (w as i16 - 2).max(7));
        let gx = x0 + (w as i16 - gw) / 2;
        let gy = y0 + icon_dsl::glyph_top();
        let body = if self.is_critical() {
            COLOR_CRITICAL
        } else {
            theme::text()
        };
        BATTERY_ICON.draw_outlined(g, gx, gy, gw as u16, gh as u16, body, theme::shadow());

        let side = ((gw as f32 * 0.14).round() as i16 + 1).max(2);
        let top_in = ((gh as f32 * 0.12).round() as i16 + 1).max(2);
        let ix = gx + side;
        let iw = (gw - 2 * side).max(2) as u16;
        let iy = gy + top_in;
        let ih = (gh - top_in - 1).max(2);
        let _ = g.set_foreground(theme::graph_bg());
        let _ = g.fill_rect(ix, iy, iw, ih as u16);

        let pct = self.combined_percent();
        let level = if pct >= 0 {
            pct.clamp(0, 100) as i16
        } else if self.ac_online {
            100
        } else {
            0
        };
        let fillh = (ih * level / 100).max(0);
        if fillh > 0 {
            let _ = g.set_foreground(self.fill_colour().unwrap_or(body));
            let _ = g.fill_rect(ix, iy + ih - fillh, iw, fillh as u16);
        }
        if self.is_critical() {
            self.draw_exclamation(g, ix, iy, iw, ih as u16, COLOR_CRITICAL);
        } else if self.combined_charging() {
            let bw2 = (iw as i16 * 4 / 5).max(4);
            let bh2 = (ih * 4 / 5).max(6);
            let bx2 = ix + (iw as i16 - bw2) / 2;
            let by2 = iy + (ih - bh2) / 2;
            if bw2 >= 8 {
                POWER_ICON.draw_outlined(
                    g,
                    bx2,
                    by2,
                    bw2 as u16,
                    bh2 as u16,
                    theme::light(),
                    theme::text(),
                );
            } else {
                POWER_ICON.draw(g, bx2, by2, bw2 as u16, bh2 as u16, theme::light());
            }
        }
    }

    fn draw_horizontal(&self, g: &dyn GraphicsContext, x0: i16, y0: i16, w: u16, h: u16) {
        let t = line_thickness();
        let inset = crate::status_graph::INSET;
        let iw = (w as i16 - inset * 2).max(4);
        let ih = (h as i16 - inset * 2).max(4);
        let ox = x0 + inset;
        let oy = y0 + inset;
        let nub_w = t as u16;
        let body_h = (ih * 4 / 5).clamp((4 * t).max(7), ih) as u16;
        let body_w = (iw - nub_w as i16 - 1).max(4 * t) as u16;
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

        let inner_x = bx + t;
        let inner_y = by + t;
        let inner_w = body_w.saturating_sub(2 * t as u16);
        let inner_h = body_h.saturating_sub(2 * t as u16);
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

#[cfg(test)]
#[path = "battery_view_tests.rs"]
mod tests;
