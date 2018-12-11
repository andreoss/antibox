use crate::manager::WindowManager;
use antibox_core::backend::{DisplayBackend, EventMask, WmWindowClass};
use antibox_core::rect::Rect;

fn ring() -> u16 {
    antibox_core::scale::scaled(2).max(1) as u16
}

pub(crate) fn ring_rectangles(w: u16, h: u16, t: u16) -> [(i16, i16, u16, u16); 4] {
    [
        (0, 0, w, t),
        (0, h.saturating_sub(t) as i16, w, t),
        (0, 0, t, h),
        (w.saturating_sub(t) as i16, 0, t, h),
    ]
}

pub fn show<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, rect: Rect) {
    let b = match wm.backend.clone() {
        Some(b) => b,
        None => return,
    };
    let t = ring();
    let (w, h) = (rect.w.max(1) as u16, rect.h.max(1) as u16);
    let strips = ring_rectangles(w, h, t);
    if wm.drag_outline.is_none() {
        let mut wins = Vec::with_capacity(4);
        for (sx, sy, sw, sh) in strips {
            let strip = Rect::new(
                rect.x + sx as i32,
                rect.y + sy as i32,
                (sw as i32).max(1),
                (sh as i32).max(1),
            );
            let win = match b.create_window(
                b.root().as_parent(),
                strip,
                WmWindowClass::InputOutput,
                true,
                EventMask::EXPOSURE,
            ) {
                Ok(win) => win,
                Err(_) => continue,
            };
            let _ = win.map();
            wins.push(win);
        }
        wm.drag_outline = Some(wins);
    }
    if let Some(wins) = &wm.drag_outline {
        for (win, (sx, sy, sw, sh)) in wins.iter().zip(strips.iter()) {
            let _ = win.configure(
                Some(rect.x + *sx as i32),
                Some(rect.y + *sy as i32),
                Some((*sw).max(1)),
                Some((*sh).max(1)),
            );
            let _ = win.raise();
            if let Ok(g) = b.create_graphics(win.id()) {
                let _ = g.set_foreground(antibox_ui::theme::sel_line());
                let _ = g.fill_rect(0, 0, (*sw).max(1), (*sh).max(1));
            }
        }
        let _ = b.flush();
    }
}

pub fn hide<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(wins) = wm.drag_outline.take() {
        for win in wins {
            let _ = win.unmap();
            let _ = win.destroy();
        }
        if let Some(b) = wm.backend() {
            let _ = b.flush();
        }
    }
}
