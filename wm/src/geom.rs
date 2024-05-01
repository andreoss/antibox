use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
use antibox_core::point::Point;
use antibox_core::rect::Rect;

pub fn screen_dims<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) -> (i32, i32) {
    wm.backend().map_or((0, 0), |b| {
        (b.screen_width() as i32, b.screen_height() as i32)
    })
}

pub fn screen_rect<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) -> Rect {
    let (w, h) = screen_dims(wm);
    Rect::new(0, 0, w, h)
}

pub fn workarea<H: DisplayBackend + 'static + ?Sized>(wm: &WindowManager<H>) -> Rect {
    wm.workareas
        .first().copied()
        .unwrap_or_else(|| screen_rect(wm))
}

pub const fn center_of(r: Rect) -> Point {
    Point::new(r.x + r.w / 2, r.y + r.h / 2)
}

pub const fn center_in(inner: Rect, area: Rect) -> Point {
    Point::new(
        area.x + (area.w - inner.w) / 2,
        area.y + (area.h - inner.h) / 2,
    )
}

#[cfg(test)]
#[path = "geom_tests.rs"]
mod tests;
