use crate::expose::crop_scale;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_core::scale::scaled;

fn max_size() -> (u16, u16) {
    (scaled(240) as u16, scaled(160) as u16)
}

fn edge() -> i32 {
    scaled(1)
}

fn gap() -> i32 {
    scaled(4)
}

pub fn fit(src: (u16, u16), max: (u16, u16)) -> (u16, u16) {
    let (sw, sh) = (i32::from(src.0).max(1), i32::from(src.1).max(1));
    let (mw, mh) = (i32::from(max.0).max(1), i32::from(max.1).max(1));
    let w = mw.min(sw).min(mh * sw / sh).max(1);
    let h = (w * sh / sw).clamp(1, mh);
    (w as u16, h as u16)
}

pub fn place(near: Rect, size: (u16, u16), screen: (i32, i32), beside: bool) -> (i32, i32) {
    let (w, h) = (i32::from(size.0), i32::from(size.1));
    let (x, y) = if beside {
        (near.x + near.w + gap(), near.y)
    } else {
        (near.x + near.w / 2 - w / 2, near.y - h - gap())
    };
    let x = if beside && x + w > screen.0 {
        near.x - w - gap()
    } else {
        x
    };
    let y = if !beside && y < 0 {
        near.y + near.h + gap()
    } else {
        y
    };
    (
        x.clamp(0, (screen.0 - w).max(0)),
        y.clamp(0, (screen.1 - h).max(0)),
    )
}

const RETRIES: u8 = 10;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Want {
    Window(u32),
    Desk { key: u64, wins: Vec<DeskWin>, screen: Rect },
}

#[derive(Clone, Debug)]
pub struct DeskWin {
    pub client: u32,
    pub rect: Rect,
    pub icon: Option<PixmapData>,
}

impl PartialEq for DeskWin {
    fn eq(&self, other: &Self) -> bool {
        self.client == other.client && self.rect == other.rect
    }
}

impl Eq for DeskWin {}

impl Want {
    fn key(&self) -> u64 {
        match self {
            Self::Window(id) => u64::from(*id),
            Self::Desk { key, .. } => u64::MAX - *key,
        }
    }
}

struct Part {
    at: Rect,
    art: Option<Vec<u8>>,
    icon: Option<PixmapData>,
}

pub struct WindowPreview {
    window: Option<Box<dyn WindowHandle>>,
    frame: (i32, i32),
    shown: Option<u64>,
    parts: Vec<Part>,
    wanted: Option<(Want, Rect, bool)>,
    icon: Option<PixmapData>,
    tries: u8,
}

impl Default for WindowPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowPreview {
    pub const fn new() -> Self {
        Self {
            window: None,
            frame: (0, 0),
            shown: None,
            parts: Vec::new(),
            wanted: None,
            icon: None,
            tries: 0,
        }
    }

    pub fn shown_for(&self) -> Option<u32> {
        match self.wanted.as_ref() {
            Some((Want::Window(id), _, _)) if self.window.is_some() => Some(*id),
            _ => None,
        }
    }

    pub fn owns_window(&self, id: u32) -> bool {
        self.window.as_ref().is_some_and(|w| w.id() == id)
    }

    pub fn show(
        &mut self,
        conn: &dyn DisplayBackend,
        client: u32,
        icon: Option<PixmapData>,
        near: Rect,
        beside: bool,
    ) {
        if self.wanted.as_ref().is_some_and(|(w, _, _)| *w == Want::Window(client)) {
            return;
        }
        self.want(conn, Want::Window(client), icon, near, beside);
    }

    pub fn show_desk(
        &mut self,
        conn: &dyn DisplayBackend,
        key: u64,
        wins: Vec<DeskWin>,
        screen: Rect,
        near: Rect,
        beside: bool,
    ) {
        self.want(conn, Want::Desk { key, wins, screen }, None, near, beside);
    }

    fn want(
        &mut self,
        conn: &dyn DisplayBackend,
        want: Want,
        icon: Option<PixmapData>,
        near: Rect,
        beside: bool,
    ) {
        if self.wanted.as_ref().is_some_and(|(w, _, _)| *w == want) {
            return;
        }
        self.hide(conn);
        self.icon = icon;
        self.wanted = Some((want, near, beside));
        self.tries = RETRIES;
        self.open(conn);
    }

    pub fn pump(&mut self, conn: &dyn DisplayBackend) {
        if self.tries > 0 && !self.complete() {
            self.tries -= 1;
            self.open(conn);
        }
    }

    fn complete(&self) -> bool {
        self.window.is_some() && self.parts.iter().all(|p| p.art.is_some())
    }

    fn snap(conn: &dyn DisplayBackend, client: u32, size: (u16, u16)) -> Option<Vec<u8>> {
        let (cw, ch, data) = conn.capture_window(client, Some(size)).ok()?;
        let whole = Rect::new(0, 0, i32::from(cw), i32::from(ch));
        crop_scale(&data, cw, ch, whole, size.0, size.1)
    }

    fn compose(
        conn: &dyn DisplayBackend,
        want: &Want,
        icon: Option<&PixmapData>,
    ) -> Option<((u16, u16), Vec<Part>)> {
        match want {
            Want::Window(client) => {
                let taken = conn.capture_window(*client, Some(max_size())).ok().and_then(
                    |(cw, ch, data)| {
                        let size = fit((cw, ch), max_size());
                        let whole = Rect::new(0, 0, i32::from(cw), i32::from(ch));
                        crop_scale(&data, cw, ch, whole, size.0, size.1).map(|art| (size, art))
                    },
                );
                let (size, art) = match taken {
                    Some((size, art)) => (size, Some(art)),
                    None => (max_size(), None),
                };
                let at = Rect::new(0, 0, i32::from(size.0), i32::from(size.1));
                let icon = art.is_none().then(|| icon.cloned()).flatten();
                Some((size, vec![Part { at, art, icon }]))
            }
            Want::Desk { wins, screen, .. } => {
                let size = fit(
                    (screen.w.max(1) as u16, screen.h.max(1) as u16),
                    max_size(),
                );
                let parts = wins
                    .iter()
                    .map(|win| {
                        let at = scale_into(win.rect, *screen, size);
                        let art = Self::snap(conn, win.client, (at.w as u16, at.h as u16));
                        let icon = art.is_none().then(|| win.icon.clone()).flatten();
                        Part { at, art, icon }
                    })
                    .collect();
                Some((size, parts))
            }
        }
    }

    fn open(&mut self, conn: &dyn DisplayBackend) {
        let Some((want, near, beside)) = self.wanted.as_ref() else { return };
        let Some((size, parts)) = Self::compose(conn, want, self.icon.as_ref()) else { return };
        let (near, beside, key) = (*near, *beside, want.key());
        self.parts = parts;
        let screen = (conn.screen_width() as i32, conn.screen_height() as i32);
        let frame = (
            i32::from(size.0) + edge() * 2,
            i32::from(size.1) + edge() * 2,
        );
        let (x, y) = place(near, (frame.0 as u16, frame.1 as u16), screen, beside);
        if self.shown == Some(key) {
            if let Some(win) = self.window.as_ref() {
                if self.frame != frame {
                    let _ = win.configure(
                        Some(x),
                        Some(y),
                        Some(frame.0 as u16),
                        Some(frame.1 as u16),
                    );
                    self.frame = frame;
                }
                self.paint(conn);
                let _ = conn.flush();
                return;
            }
        }
        let Ok(window) = conn.create_window(
            conn.root().as_parent(),
            Rect::new(x, y, frame.0, frame.1),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE,
        ) else {
            return;
        };
        let _ = window.map();
        let _ = window.raise();
        self.window = Some(window);
        self.shown = Some(key);
        self.frame = frame;
        self.paint(conn);
        let _ = conn.flush();
    }

    pub fn hide(&mut self, conn: &dyn DisplayBackend) {
        if let Some(win) = self.window.take() {
            let _ = win.unmap();
            let _ = win.destroy();
            let _ = conn.flush();
        }
        self.shown = None;
        self.frame = (0, 0);
        self.parts.clear();
        self.wanted = None;
        self.icon = None;
        self.tries = 0;
    }

    pub fn paint(&self, conn: &dyn DisplayBackend) {
        let Some(win) = &self.window else { return };
        let Ok(g) = conn.create_graphics(win.id()) else { return };
        let (gw, gh) = win.get_geometry().unwrap_or((2, 2));
        let _ = g.set_foreground(antibox_ui::theme::shadow());
        let _ = g.fill_rect(0, 0, gw, gh);
        let _ = g.set_foreground(antibox_core::colour::scale(antibox_ui::theme::face(), 0.3));
        let _ = g.fill_rect(
            edge() as i16,
            edge() as i16,
            gw.saturating_sub(edge() as u16 * 2),
            gh.saturating_sub(edge() as u16 * 2),
        );
        for part in &self.parts {
            let (x, y) = (
                (part.at.x + edge()) as i16,
                (part.at.y + edge()) as i16,
            );
            let (w, h) = (part.at.w as u16, part.at.h as u16);
            match &part.art {
                Some(art) => {
                    let _ = g.draw_image(x, y, w, h, art);
                }
                None => {
                    let band = (h / 5).max(1);
                    let _ = g.set_foreground(antibox_ui::theme::face());
                    let _ = g.fill_rect(x, y, w, h);
                    let _ = g.set_foreground(antibox_ui::theme::title_inactive());
                    let _ = g.fill_rect(x, y, w, band);
                    if let Some(icon) = &part.icon {
                        let ix = x + (w as i16 - icon.width as i16) / 2;
                        let iy = y + band as i16 + (h.saturating_sub(band) as i16
                            - icon.height as i16)
                            / 2;
                        let _ = g.draw_pixmap(ix.max(x), iy.max(y), icon);
                    }
                }
            }
            if self.parts.len() > 1 {
                let _ = g.set_foreground(antibox_ui::theme::shadow());
                let _ = g.draw_rect(x, y, w, h);
            }
        }
    }
}

pub fn scale_into(rect: Rect, screen: Rect, size: (u16, u16)) -> Rect {
    let (sw, sh) = (screen.w.max(1), screen.h.max(1));
    let (bw, bh) = (i32::from(size.0), i32::from(size.1));
    let x = ((rect.x - screen.x).max(0) * bw / sw).min(bw - 1);
    let y = ((rect.y - screen.y).max(0) * bh / sh).min(bh - 1);
    let w = (rect.w * bw / sw).clamp(1, bw - x);
    let h = (rect.h * bh / sh).clamp(1, bh - y);
    Rect::new(x, y, w, h)
}

#[cfg(test)]
#[path = "winpreview_tests.rs"]
mod tests;
