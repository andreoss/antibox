use antibox_core::backend::{DisplayBackend, GraphicsContext};

pub fn buffered(
    conn: &dyn DisplayBackend,
    dest: u32,
    w: u16,
    h: u16,
    paint: impl FnOnce(&dyn GraphicsContext),
) {
    let depth = conn.screen_depth();
    if let Ok(pm) = conn.create_pixmap(w, h, depth) {
        if let Ok(g) = conn.create_graphics(pm) {
            paint(&*g);
        }
        if let Ok(wg) = conn.create_graphics(dest) {
            let _ = wg.copy_from(
                pm,
                antibox_core::rect::Rect::px(0, 0, w, h),
                antibox_core::point::Point::ZERO,
            );
        }
        let _ = conn.free_pixmap(pm);
    } else if let Ok(g) = conn.create_graphics(dest) {
        paint(&*g);
    }
}

#[cfg(test)]
#[path = "paintbuf_tests.rs"]
mod tests;
