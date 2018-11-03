use crate::manager::WindowManager;
use antibox_core::backend::{DisplayBackend, EventMask, ShapeOp, WmWindowClass};
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
    if wm.drag_outline.is_none() {
        let win = match b.create_window(
            b.root().as_parent(),
            rect,
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE,
        ) {
            Ok(win) => win,
            Err(_) => return,
        };
        let _ = win.map();
        wm.drag_outline = Some(win);
    }
    if let Some(win) = &wm.drag_outline {
        let _ = win.configure(
            Some(rect.x),
            Some(rect.y),
            Some(rect.w.max(1) as u16),
            Some(rect.h.max(1) as u16),
        );
        let t = ring();
        let (w, h) = (rect.w.max(1) as u16, rect.h.max(1) as u16);
        let _ = win.set_shape_rectangles(&ring_rectangles(w, h, t), ShapeOp::Set);
        let _ = win.raise();
        if let Ok(g) = b.create_graphics(win.id()) {
            let _ = g.set_foreground(antibox_ui::theme::sel_line());
            let _ = g.fill_rect(0, 0, w, h);
        }
        let _ = b.flush();
    }
}

pub fn hide<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>) {
    if let Some(win) = wm.drag_outline.take() {
        let _ = win.unmap();
        let _ = win.destroy();
        if let Some(b) = wm.backend() {
            let _ = b.flush();
        }
    }
}
