 use antibox_core::error::Result;
use crate::status_graph::impl_status_applet;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use std::sync::Arc;

const IWM_STATES: usize = 8;
const IWM_USER: usize = 0;
const IWM_NICE: usize = 1;
const IWM_SYS: usize = 2;
const IWM_IDLE: usize = 3;
const IWM_IOWAIT: usize = 4;
const IWM_INTR: usize = 5;
const IWM_SOFTIRQ: usize = 6;
const IWM_STEAL: usize = 7;

fn cpu_colour(state: usize) -> u32 {
    use antibox_ui::theme;
    match state {
        IWM_USER => theme::graph_series(0),
        IWM_SYS | IWM_STEAL => theme::graph_series(1),
        IWM_NICE | IWM_INTR => theme::graph_series(2),
        IWM_IOWAIT | IWM_SOFTIRQ => theme::graph_series(3),
        _ => theme::graph_bg(),
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct CpuDelta {
    vals: [u64; IWM_STATES],
}

#[cfg(not(target_os = "linux"))]
fn read_cpu_times() -> Option<Vec<[u64; IWM_STATES]>> {
    let cp = crate::proc_reader::read_cptime()?;
    let mut vals = [0u64; IWM_STATES];
    vals[IWM_USER] = cp[0];
    vals[IWM_NICE] = cp[1];
    vals[IWM_SYS] = cp[2].saturating_add(cp[3]);
    vals[IWM_INTR] = cp[4];
    vals[IWM_IDLE] = cp[5];
    Some(vec![vals])
}

#[cfg(target_os = "linux")]
fn read_cpu_times() -> Option<Vec<[u64; IWM_STATES]>> {
    let data = crate::proc_reader::read_proc("/proc/stat")?;
    let mut result = Vec::new();
    for line in data.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        if parts[0] == "cpu"
            || (parts[0].starts_with("cpu") && parts[0][3..].chars().all(|c| c.is_ascii_digit()))
        {
            if parts.len() < 5 {
                continue;
            }
            let vals = parse_cpu_line(&parts);
            if vals.iter().any(|v| *v != 0) {
                result.push(vals);
            }
        } else {
            break;
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_cpu_line(parts: &[&str]) -> [u64; IWM_STATES] {
    [
        parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0),
        parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0),
    ]
}

fn compute_deltas(
    prev: &[[u64; IWM_STATES]],
    cur: &[[u64; IWM_STATES]],
) -> Option<[u64; IWM_STATES]> {
    if prev.len() != cur.len() {
        return None;
    }
    let n = prev.len();
    if n == 0 {
        return None;
    }
    let mut merged = [0u64; IWM_STATES];
    if n == 1 {
        for i in 0..IWM_STATES {
            merged[i] = cur[0][i].saturating_sub(prev[0][i]);
        }
    } else {
        for c in 0..n {
            for i in 0..IWM_STATES {
                merged[i] = merged[i].saturating_add(cur[c][i].saturating_sub(prev[c][i]));
            }
        }
    }
    Some(merged)
}

#[cfg(test)]
fn total_delta(d: &[u64; IWM_STATES]) -> u64 {
    d.iter().sum()
}

fn read_loadavg() -> String {
    match crate::proc_reader::load_average() {
        Some(avgs) => format!("{:.2}/{:.2}", avgs[0], avgs[1]),
        None => String::new(),
    }
}

fn read_cpu_freq() -> Vec<(u32, f64)> {
    let mut freqs = Vec::new();
    for cpu in 0..8 {
        let mut found = None;
        for f in &["scaling_cur_freq", "cpuinfo_cur_freq"] {
            let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/{}", cpu, f);
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(v) = data.trim().parse::<f64>() {
                    found = Some(v);
                    break;
                }
            }
        }
        if let Some(freq) = found {
            freqs.push((cpu, freq));
        }
    }
    freqs
}

fn read_acpi_temp() -> Vec<(String, i32)> {
    let mut temps = Vec::new();
    let dir = std::path::Path::new("/sys/class/thermal");
    if !dir.is_dir() {
        return temps;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return temps,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("thermal_zone") {
            continue;
        }
        let temp_path = entry.path().join("temp");
        let type_path = entry.path().join("type");
        let ttype = std::fs::read_to_string(&type_path)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if let Ok(data) = std::fs::read_to_string(&temp_path) {
            if let Ok(temp) = data.trim().parse::<i32>() {
                temps.push((ttype, temp / 1000));
            }
        }
    }
    temps
}

fn fmt_freq(freq: f64) -> String {
    if freq > 1_000_000.0 {
        format!("{:.2}GHz", freq / 1_000_000.0)
    } else if freq > 1000.0 {
        format!("{:.0}MHz", freq / 1000.0)
    } else {
        format!("{:.0}KHz", freq)
    }
}

fn fmt_mem(kb: u64) -> String {
    let bytes = kb * 1024;
    if bytes >= 1073741824 {
        format!("{:.2}G", bytes as f64 / 1073741824.0)
    } else if bytes >= 1048576 {
        format!("{:.1}M", bytes as f64 / 1048576.0)
    } else {
        format!("{}K", kb)
    }
}

pub struct CpuStatusApplet {
    conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    tooltip: Option<crate::tooltip::ToolTip>,
    samples: crate::status_graph::Samples<CpuDelta>,
    prev_times: Option<Vec<[u64; IWM_STATES]>>,
    cpu_count: usize,
    w: u16,
    h: u16,
    pref_w: u16,
}

impl CpuStatusApplet {
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
        let cpu_count = read_cpu_times().map_or(1, |v| v.len());
        Ok(CpuStatusApplet {
            conn: Arc::clone(conn),
            window,
            tooltip: None,
            samples: crate::status_graph::Samples::new(),
            prev_times: None,
            cpu_count,
            w,
            h,
            pref_w: w,
        })
    }

    pub fn update(&mut self) -> bool {
        let cur = match read_cpu_times() {
            Some(cur) => cur,
            None => return false,
        };
        let delta = if let Some(prev) = &self.prev_times {
            compute_deltas(prev, &cur)
        } else {
            self.prev_times = Some(cur);
            return false;
        };
        self.prev_times = Some(cur);
        if let Some(d) = delta {
            self.samples.push(CpuDelta { vals: d });
        }
        true
    }

    #[cfg(test)]
    fn last_percent(&self) -> f32 {
        let d = match self.samples.latest() {
            Some(d) => d,
            None => return 0.0,
        };
        let total = total_delta(&d.vals);
        if total == 0 {
            return 0.0;
        }
        let busy = total.saturating_sub(d.vals[IWM_IDLE]);
        busy as f32 / total as f32
    }

    fn tooltip(&self) -> String {
        let mut s = String::new();
        let load = read_loadavg();
        if !load.is_empty() {
            s.push_str(&format!("CPU Load: {}\n", load));
        }
        if let Some((total, free)) = crate::proc_reader::read_proc_meminfo() {
            let used = total.saturating_sub(free);
            s.push_str(&format!("RAM: {} / {}\n", fmt_mem(used), fmt_mem(total)));
        }
        let freqs = read_cpu_freq();
        if !freqs.is_empty() {
            let avg: f64 = freqs.iter().map(|(_, f)| f).sum::<f64>() / freqs.len() as f64;
            s.push_str(&format!("CPU Freq: {}\n", fmt_freq(avg)));
        }
        let temps = read_acpi_temp();
        for (ttype, temp) in &temps {
            if *temp > 0 {
                s.push_str(&format!("{}: {}\u{B0}C\n", ttype, temp));
            }
        }
        if self.cpu_count > 1 {
            s.push_str(&format!("{} CPUs", self.cpu_count));
        }
        let s = s.trim_end().to_string();
        if s.is_empty() {
            "CPU usage".to_string()
        } else {
            s
        }
    }
}

impl_status_applet!(CpuStatusApplet);

impl CpuStatusApplet {
    fn paint_graph(&self, g: &dyn GraphicsContext) {
        let plot = match crate::status_graph::plot(self.w, self.h, self.samples.len()) {
            Some(plot) => plot,
            None => return,
        };
        let h64 = plot.gh as u64;
        let n = plot.count();
        for col in 0..n {
            let v = &self.samples.at(col, n).vals;
            let total: u64 = v.iter().sum();
            if total <= 1 {
                continue;
            }
            let x = plot.col_x(col);
            let round = total / h64 / 2;
            let bar = |state: usize| -> i16 { ((h64 * (v[state] + round)) / total) as i16 };

            let stealbar = bar(IWM_STEAL);
            let intrbar = bar(IWM_INTR);
            let softirqbar = bar(IWM_SOFTIRQ);
            let iowaitbar = bar(IWM_IOWAIT);
            let sysbar = bar(IWM_SYS);
            let nicebar = bar(IWM_NICE);
            let prior = stealbar + intrbar + softirqbar + iowaitbar + sysbar + nicebar;
            let userbar = ((h64 * ((total - v[IWM_IDLE]) + round)) / total) as i16 - prior;

            let mut y = plot.bottom() - 1;
            y = plot.bar_up_heat(g, x, y, stealbar, cpu_colour(IWM_STEAL));
            y = plot.bar_up_heat(g, x, y, intrbar, cpu_colour(IWM_INTR));
            y = plot.bar_up_heat(g, x, y, softirqbar, cpu_colour(IWM_SOFTIRQ));
            y = plot.bar_up_heat(g, x, y, sysbar, cpu_colour(IWM_SYS));
            y = plot.bar_up_heat(g, x, y, userbar, cpu_colour(IWM_USER));
            y = plot.bar_up_heat(g, x, y, nicebar, cpu_colour(IWM_NICE));
            let _ = plot.bar_up_heat(g, x, y, iowaitbar, cpu_colour(IWM_IOWAIT));
        }
    }
}

#[cfg(test)]
#[path = "cpu_status_applet_tests.rs"]
mod tests;
