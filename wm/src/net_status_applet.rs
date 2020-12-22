use crate::status_graph::impl_status_applet;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_core::sync::atomic::LazyRwLock;
use std::sync::Arc;

struct NetCounters {
    rx_bytes: u64,
    tx_bytes: u64,
}

static NET_DEVICE: LazyRwLock<String> = LazyRwLock::new();

pub fn set_net_device(pattern: &str) {
    if let Ok(mut g) = NET_DEVICE.write() {
        *g = pattern.trim().to_string();
    }
}

fn device_selected(filter: &str, name: &str) -> bool {
    filter
        .split(|c| c == ' ' || c == ',' || c == '\t')
        .filter(|t| !t.is_empty())
        .any(|t| {
            if t.ends_with('*') {
                name.starts_with(&t[..t.len() - 1])
            } else {
                name == t
            }
        })
}

fn device_wanted(name: &str) -> bool {
    NET_DEVICE
        .read()
        .map_or(true, |g| device_selected(&g, name))
}

fn read_net_counters() -> Option<Vec<(String, NetCounters)>> {
    let data = crate::proc_reader::read_proc("/proc/net/dev")?;
    let mut result = Vec::new();
    for line in data.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        let iface = parts[0].trim_end_matches(':');
        if iface == "lo" || !device_wanted(iface) {
            continue;
        }
        let rx_bytes: u64 = parts[1].parse().ok()?;
        let tx_bytes: u64 = parts[9].parse().ok()?;
        result.push((iface.to_string(), NetCounters { rx_bytes, tx_bytes }));
    }
    Some(result)
}

fn net_recv_colour() -> u32 {
    antibox_ui::theme::graph_series(0)
}
fn net_send_colour() -> u32 {
    antibox_ui::theme::graph_series(1)
}

#[derive(Clone, Copy, Default)]
struct NetSample {
    inb: u64,
    outb: u64,
}

pub struct NetStatusApplet {
    conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    tooltip: Option<crate::tooltip::ToolTip>,
    prev: Option<Vec<(String, NetCounters)>>,
    rates: Vec<(String, f64, f64)>,
    rx_total: f64,
    tx_total: f64,
    samples: crate::status_graph::Samples<NetSample>,
    w: u16,
    h: u16,
    pref_w: u16,
}

impl NetStatusApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        width: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
        Ok(NetStatusApplet {
            conn: Arc::clone(conn),
            window,
            tooltip: None,
            prev: None,
            rates: Vec::new(),
            rx_total: 0.0,
            tx_total: 0.0,
            samples: crate::status_graph::Samples::new(),
            w,
            h,
            pref_w: w,
        })
    }

    pub fn update(&mut self) {
        let cur = match read_net_counters() {
            Some(cur) => cur,
            None => return,
        };
        if let Some(ref prev) = self.prev {
            let mut total_rx = 0u64;
            let mut total_tx = 0u64;
            let mut new_rates = Vec::new();
            for (name, cur_cnt) in &cur {
                if let Some(p) = prev.iter().find(|(n, _)| n == name) {
                    let drx = cur_cnt.rx_bytes.saturating_sub(p.1.rx_bytes);
                    let dtx = cur_cnt.tx_bytes.saturating_sub(p.1.tx_bytes);
                    total_rx += drx;
                    total_tx += dtx;
                    if drx > 0 || dtx > 0 {
                        new_rates.push((name.clone(), drx as f64, dtx as f64));
                    }
                }
            }
            self.rates = new_rates;
            self.rx_total = total_rx as f64;
            self.tx_total = total_tx as f64;
            self.samples.push(NetSample {
                inb: total_rx,
                outb: total_tx,
            });
        }
        self.prev = Some(cur);
    }

    fn paint_graph(&self, g: &dyn GraphicsContext) {
        let plot = match crate::status_graph::plot(self.w, self.h, self.samples.len()) {
            Some(plot) => plot,
            None => return,
        };
        let n = plot.count();
        let (mut in_max, mut out_max) = (0u64, 0u64);
        for col in 0..n {
            let s = self.samples.at(col, n);
            in_max = in_max.max(s.inb);
            out_max = out_max.max(s.outb);
        }
        let max_bytes = (in_max + out_max).max(1024);
        let h64 = plot.gh as u64;
        let round = max_bytes / h64 / 2;

        for col in 0..n {
            let s = self.samples.at(col, n);
            let x = plot.col_x(col);
            let inbar = ((h64 * (s.inb + round)) / max_bytes).min(h64) as i16;
            let outbar = ((h64 * (s.outb + round)) / max_bytes).min(h64) as i16;
            if inbar > 0 {
                let _ = g.set_foreground(net_recv_colour());
                let _ = g.fill_rect(x, plot.bottom() - inbar, plot.cw, inbar as u16);
            }
            if outbar > 0 {
                let _ = g.set_foreground(net_send_colour());
                let _ = g.fill_rect(x, plot.top(), plot.cw, outbar as u16);
            }
        }
    }

    fn tooltip(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "\u{2193} {}  \u{2191} {}\n",
            fmt_rate(self.rx_total),
            fmt_rate(self.tx_total)
        ));
        for (name, rx, tx) in &self.rates {
            s.push_str(&format!(
                "{}: \u{2193}{} \u{2191}{} ",
                name,
                fmt_rate(*rx),
                fmt_rate(*tx)
            ));
        }
        s
    }
}

fn fmt_rate(bytes: f64) -> String {
    if bytes >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1}G", bytes / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024.0 * 1024.0 {
        format!("{:.1}M", bytes / (1024.0 * 1024.0))
    } else if bytes >= 1024.0 {
        format!("{:.0}K", bytes / 1024.0)
    } else {
        format!("{:.0}B", bytes)
    }
}

impl_status_applet!(NetStatusApplet);

#[cfg(test)]
#[path = "net_status_applet_tests.rs"]
mod tests;
