 use antibox_core::error::Result;
use crate::status_graph::impl_status_applet;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

const MEM_USER: usize = 0;
const MEM_BUFFERS: usize = 1;
const MEM_CACHED: usize = 2;
const MEM_FREE: usize = 3;
const MEM_STATES: usize = 4;

fn mem_colour(state: usize) -> u32 {
    use antibox_ui::theme;
    match state {
        0 => theme::graph_series(0),
        1 => theme::graph_series(2),
        2 => theme::graph_series(3),
        _ => theme::graph_bg(),
    }
}

#[derive(Clone, Copy, Default)]
struct MemSample {
    vals: [u64; MEM_STATES],
}

pub struct MemStatusApplet {
    conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    tooltip: Option<crate::tooltip::ToolTip>,
    samples: crate::status_graph::Samples<MemSample>,
    pub(crate) used_pct: f32,
    w: u16,
    h: u16,
    pref_w: u16,
}

impl MemStatusApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        width: u16,
    ) -> Result<Self> {
        let w = antibox_core::scale::scaled(width as i32) as u16;
        let h = antibox_core::scale::scaled(20) as u16;
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::POINTER_MOTION
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE,
        )?;
        let mut s = Self {
            conn: Arc::clone(conn),
            window,
            tooltip: None,
            samples: crate::status_graph::Samples::new(),
            used_pct: 0.0,
            w,
            h,
            pref_w: w,
        };
        s.update();
        Ok(s)
    }

    pub fn update(&mut self) {
        let Some(sample) = read_mem_sample() else { return };
        self.samples.push(sample);
        let total: u64 = sample.vals.iter().sum();
        self.used_pct = if total > 0 {
            sample.vals[MEM_USER] as f32 / total as f32
        } else {
            0.0
        };
    }

    fn paint_graph(&self, g: &dyn GraphicsContext) {
        let Some(plot) = crate::status_graph::plot(self.w, self.h, self.samples.len()) else { return };
        let n = plot.count();
        let used = |col: usize| -> u64 {
            let v = &self.samples.at(col, n).vals;
            v[MEM_USER] + v[MEM_BUFFERS] + v[MEM_CACHED]
        };

        let (mut umin, mut umax) = (u64::MAX, 0u64);
        for col in 0..n {
            let u = used(col);
            umin = umin.min(u);
            umax = umax.max(u);
        }
        let range = umax.saturating_sub(umin);
        let gh = plot.gh as f64;
        let bottom = plot.bottom();
        for col in 0..n {
            let v = &self.samples.at(col, n).vals;
            let u = v[MEM_USER] + v[MEM_BUFFERS] + v[MEM_CACHED];
            if u == 0 {
                continue;
            }
            let frac = if range == 0 {
                0.5
            } else {
                0.15 + 0.85 * (u - umin) as f64 / range as f64
            };
            let barh = (frac * gh).round() as i16;
            let x = plot.col_x(col);
            let mut y = bottom - 1;
            for state in &[MEM_USER, MEM_BUFFERS, MEM_CACHED] {
                let seg = ((v[*state] as f64 / u as f64) * barh as f64).round() as i16;
                y = plot.bar_up_heat(g, x, y, seg, mem_colour(*state));
            }
        }
    }

    fn tooltip(&self) -> String {
        let Some(sample) = self.samples.latest() else { return String::from("Memory") };
        let v = &sample.vals;
        let total: u64 = v.iter().sum();
        if total == 0 {
            return String::from("Memory");
        }
        format!(
            "RAM: {} / {}\nBuffers: {}\nCached: {}\nFree: {}",
            fmt_bytes(v[MEM_USER]),
            fmt_bytes(total),
            fmt_bytes(v[MEM_BUFFERS]),
            fmt_bytes(v[MEM_CACHED]),
            fmt_bytes(v[MEM_FREE]),
        )
    }
}

fn fmt_bytes(b: u64) -> String {
    let mb = b as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 {
        format!("{:.1}G", mb / 1024.0)
    } else {
        format!("{mb:.0}M")
    }
}

#[cfg(not(target_os = "linux"))]
fn read_mem_sample() -> Option<MemSample> {
    let m = crate::proc_reader::read_uvmexp()?;
    let user = m.total.saturating_sub(m.free + m.inactive + m.wired);
    let mut vals = [0u64; MEM_STATES];
    vals[MEM_USER] = user;
    vals[MEM_BUFFERS] = m.wired;
    vals[MEM_CACHED] = m.inactive;
    vals[MEM_FREE] = m.free;
    Some(MemSample { vals })
}

#[cfg(target_os = "linux")]
fn read_mem_sample() -> Option<MemSample> {
    let data = crate::proc_reader::read_proc("/proc/meminfo")?;
    let (mut total, mut free, mut buffers, mut cached) = (0u64, 0u64, 0u64, 0u64);
    for line in data.lines() {
        if let Some(val) = parse_meminfo_line(line, "MemTotal:") {
            total = val;
        } else if let Some(val) = parse_meminfo_line(line, "MemFree:") {
            free = val;
        } else if let Some(val) = parse_meminfo_line(line, "Buffers:") {
            buffers = val;
        } else if let Some(val) = parse_meminfo_line(line, "Cached:") {
            cached = val;
        }
    }
    if total == 0 {
        return None;
    }
    let user = total.saturating_sub(free + buffers + cached);
    let mut vals = [0u64; MEM_STATES];
    vals[MEM_USER] = user;
    vals[MEM_BUFFERS] = buffers;
    vals[MEM_CACHED] = cached;
    vals[MEM_FREE] = free;
    Some(MemSample { vals })
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_meminfo_line(line: &str, needle: &str) -> Option<u64> {
    if let Some(rest) = line.strip_prefix(needle) {
        let val: u64 = rest.split_whitespace().next()?.parse().ok()?;
        Some(val * 1024)
    } else {
        None
    }
}

impl_status_applet!(MemStatusApplet);

#[cfg(test)]
#[path = "mem_status_applet_tests.rs"]
mod tests;
