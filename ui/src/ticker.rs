use antibox_gfx::backend::GraphicsContext;
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(true);
static SCROLLED_PANEL: AtomicBool = AtomicBool::new(false);
static SCROLLED_TITLE: AtomicBool = AtomicBool::new(false);

const GAP: &str = "   ";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Panel,
    Title,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Scrolled {
    pub panel: bool,
    pub title: bool,
}

impl Scrolled {
    pub const fn any(self) -> bool {
        self.panel || self.title
    }
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub const fn interval() -> std::time::Duration {
    std::time::Duration::from_millis(80)
}

fn speed_px_per_interval() -> usize {
    antibox_gfx::scale::scaled(2).max(1) as usize
}

pub fn advance() -> Scrolled {
    Scrolled {
        panel: SCROLLED_PANEL.swap(false, Ordering::Relaxed),
        title: SCROLLED_TITLE.swap(false, Ordering::Relaxed),
    }
}

pub fn rearm(s: Scrolled) {
    if s.panel {
        SCROLLED_PANEL.store(true, Ordering::Relaxed);
    }
    if s.title {
        SCROLLED_TITLE.store(true, Ordering::Relaxed);
    }
}

pub fn active() -> bool {
    SCROLLED_PANEL.load(Ordering::Relaxed) || SCROLLED_TITLE.load(Ordering::Relaxed)
}

pub fn phase() -> usize {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis()) as usize;
    let per = interval().as_millis().max(1) as usize;
    ms.wrapping_mul(speed_px_per_interval()) / per
}

pub enum Fit<'a> {
    Plain(Cow<'a, str>),
    Scroll { text: String, shift: u16 },
}

fn scroll_at(g: &dyn GraphicsContext, label: &str, phase: usize, avail: u16) -> (String, u16) {
    let cycle: Vec<char> = label.chars().chain(GAP.chars()).collect();
    let widths: Vec<u32> = {
        let mut buf = String::new();
        let mut prev = 0u32;
        cycle
            .iter()
            .map(|&c| {
                buf.push(c);
                let cum = g.text_width(&buf).unwrap_or(0);
                let w = cum.saturating_sub(prev);
                prev = cum;
                w
            })
            .collect()
    };
    let total: usize = widths.iter().map(|&w| w as usize).sum();
    if total == 0 {
        return (String::new(), 0);
    }
    let mut rem = phase % total;
    let mut first = 0usize;
    for (i, &w) in widths.iter().enumerate() {
        if rem < w as usize {
            first = i;
            break;
        }
        rem -= w as usize;
    }
    let shift = rem as u16;
    let want = avail as usize + shift as usize;
    let mut out = String::new();
    let mut covered = 0usize;
    let mut i = first;
    while covered < want {
        out.push(cycle[i]);
        covered += widths[i] as usize;
        i = (i + 1) % cycle.len();
    }
    (out, shift)
}

pub fn fit<'a>(g: &dyn GraphicsContext, label: &'a str, avail: u16) -> Fit<'a> {
    fit_on(Surface::Panel, g, label, avail)
}

pub fn fit_on<'a>(
    surface: Surface,
    g: &dyn GraphicsContext,
    label: &'a str,
    avail: u16,
) -> Fit<'a> {
    if g.text_width(label).unwrap_or(0) <= avail as u32 {
        return Fit::Plain(Cow::Borrowed(label));
    }
    if !enabled() {
        return Fit::Plain(crate::widget::fit_label(g, label, avail));
    }
    if avail == 0 {
        return Fit::Plain(Cow::Borrowed(""));
    }
    match surface {
        Surface::Panel => SCROLLED_PANEL.store(true, Ordering::Relaxed),
        Surface::Title => SCROLLED_TITLE.store(true, Ordering::Relaxed),
    }
    let (text, shift) = scroll_at(g, label, phase(), avail);
    Fit::Scroll { text, shift }
}

#[cfg(test)]
#[path = "ticker_tests.rs"]
mod tests;
