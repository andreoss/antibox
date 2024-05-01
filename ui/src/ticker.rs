use antibox_gfx::backend::GraphicsContext;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
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

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Scrolled {
    pub panel: bool,
    pub title: bool,
    pub panel_targets: Vec<u64>,
    pub title_targets: Vec<u64>,
}

impl Scrolled {
    pub const fn any(&self) -> bool {
        self.panel || self.title
    }
}

fn push_unique(v: &mut Vec<u64>, target: u64) {
    if !v.contains(&target) {
        v.push(target);
    }
}

fn note_target(surface: Surface, target: u64) {
    match surface {
        Surface::Panel => PANEL_TARGETS.with(|c| push_unique(&mut c.borrow_mut(), target)),
        Surface::Title => TITLE_TARGETS.with(|c| push_unique(&mut c.borrow_mut(), target)),
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
        panel_targets: PANEL_TARGETS.with(|c| c.borrow_mut().drain(..).collect()),
        title_targets: TITLE_TARGETS.with(|c| c.borrow_mut().drain(..).collect()),
    }
}

pub fn rearm(s: Scrolled) {
    if s.panel {
        SCROLLED_PANEL.store(true, Ordering::Relaxed);
    }
    if s.title {
        SCROLLED_TITLE.store(true, Ordering::Relaxed);
    }
    PANEL_TARGETS.with(|c| {
        let mut v = c.borrow_mut();
        for t in s.panel_targets {
            push_unique(&mut v, t);
        }
    });
    TITLE_TARGETS.with(|c| {
        let mut v = c.borrow_mut();
        for t in s.title_targets {
            push_unique(&mut v, t);
        }
    });
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

struct Metrics {
    chars: Vec<char>,
    widths: Vec<u32>,
    total: u32,
}

const CACHE_LIMIT: usize = 512;

thread_local! {
    static METRICS: RefCell<HashMap<u64, HashMap<String, Metrics>>> = RefCell::new(HashMap::new());
    static PANEL_TARGETS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    static TITLE_TARGETS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

fn font_stamp(g: &dyn GraphicsContext) -> u64 {
    let (ascent, descent, height) = g.font_metrics();
    let dpi = antibox_gfx::scale::dpi().max(0) as u64;
    (u64::from(ascent) << 48)
        | (u64::from(descent) << 32)
        | (u64::from(height) << 16)
        | (dpi & 0xffff)
}

fn measure(g: &dyn GraphicsContext, label: &str) -> Metrics {
    let mut chars = Vec::with_capacity(label.len() + GAP.len());
    chars.extend(label.chars());
    chars.extend(GAP.chars());
    let mut buf = String::with_capacity(chars.len());
    let mut prev = 0u32;
    let mut widths = Vec::with_capacity(chars.len());
    for &c in &chars {
        buf.push(c);
        let cum = g.text_width(&buf).unwrap_or(0);
        widths.push(cum.saturating_sub(prev));
        prev = cum;
    }
    Metrics {
        chars,
        widths,
        total: prev,
    }
}

fn window_at(m: &Metrics, phase: usize, avail: u16) -> (String, u16) {
    if m.total == 0 {
        return (String::new(), 0);
    }
    let mut rem = phase % m.total as usize;
    let mut first = 0usize;
    for (i, &w) in m.widths.iter().enumerate() {
        if rem < w as usize {
            first = i;
            break;
        }
        rem -= w as usize;
    }
    let shift = rem as u16;
    let want = avail as usize + shift as usize;
    let n = m.chars.len();
    let mut out = String::with_capacity(n + 1);
    let mut covered = 0usize;
    let mut i = first;
    while covered < want {
        out.push(m.chars[i]);
        covered += m.widths[i] as usize;
        i += 1;
        if i == n {
            i = 0;
        }
    }
    (out, shift)
}

fn scroll_at(g: &dyn GraphicsContext, label: &str, phase: usize, avail: u16) -> (String, u16) {
    let stamp = font_stamp(g);
    METRICS.with(|cell| {
        let mut cache = cell.borrow_mut();
        if cache.values().map(HashMap::len).sum::<usize>() > CACHE_LIMIT {
            cache.clear();
        }
        let by_label = cache.entry(stamp).or_default();
        if !by_label.contains_key(label) {
            by_label.insert(label.to_string(), measure(g, label));
        }
        window_at(&by_label[label], phase, avail)
    })
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
    fit_on_at(surface, g, label, avail, u64::from(g.drawable()))
}

pub fn fit_on_at<'a>(
    surface: Surface,
    g: &dyn GraphicsContext,
    label: &'a str,
    avail: u16,
    target: u64,
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
    note_target(surface, target);
    let (text, shift) = scroll_at(g, label, phase(), avail);
    Fit::Scroll { text, shift }
}

#[cfg(test)]
#[path = "ticker_tests.rs"]
mod tests;
