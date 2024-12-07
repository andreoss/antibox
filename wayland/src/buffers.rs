use antibox_core::backend::PixmapData;
use antibox_core::canvas::SoftCanvas;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const FIRST_DYNAMIC_ID: u32 = 0x0100_0000;

#[derive(Default)]
struct Inner {
    next_id: u32,
    canvases: HashMap<u32, SoftCanvas>,
}

pub struct BufferStore {
    inner: Mutex<Inner>,
}

impl BufferStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                next_id: FIRST_DYNAMIC_ID,
                canvases: HashMap::new(),
            }),
        })
    }

    pub fn alloc_id(&self) -> u32 {
        let mut inner = self.lock();
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1).max(FIRST_DYNAMIC_ID);
        id
    }

    pub fn create(&self, width: u16, height: u16) -> u32 {
        let id = self.alloc_id();
        self.lock()
            .canvases
            .insert(id, SoftCanvas::new(width, height));
        id
    }

    pub fn insert(&self, id: u32, canvas: SoftCanvas) {
        self.lock().canvases.insert(id, canvas);
    }

    pub fn remove(&self, id: u32) {
        self.lock().canvases.remove(&id);
    }

    pub fn contains(&self, id: u32) -> bool {
        self.lock().canvases.contains_key(&id)
    }

    pub fn with<R>(&self, id: u32, f: impl FnOnce(&mut SoftCanvas) -> R) -> Option<R> {
        let mut inner = self.lock();
        inner.canvases.get_mut(&id).map(f)
    }

    pub fn copy_region(&self, src: u32, src_area: Rect, dst: u32, dst_pos: Point) {
        let (sx, sy, w, h) = src_area.as_px();
        let (dx, dy) = (dst_pos.x as i16, dst_pos.y as i16);
        let mut inner = self.lock();
        let patch = match inner.canvases.get(&src) {
            Some(c) => crop(c, sx, sy, w, h),
            None => return,
        };
        if let Some(d) = inner.canvases.get_mut(&dst) {
            d.draw_pixmap(dx, dy, &patch);
        }
    }

    pub fn snapshot(&self, id: u32) -> Option<PixmapData> {
        self.lock()
            .canvases
            .get(&id)
            .map(SoftCanvas::to_pixmap_data)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn crop(c: &SoftCanvas, sx: i16, sy: i16, w: u16, h: u16) -> PixmapData {
    let mut out = vec![0u8; w as usize * h as usize * 4];
    let src = c.as_rgba();
    let cw = c.width() as i32;
    let ch = c.height() as i32;
    for row in 0..h as i32 {
        let syy = sy as i32 + row;
        if syy < 0 || syy >= ch {
            continue;
        }
        for col in 0..w as i32 {
            let sxx = sx as i32 + col;
            if sxx < 0 || sxx >= cw {
                continue;
            }
            let so = (syy * cw + sxx) as usize * 4;
            let doo = (row * w as i32 + col) as usize * 4;
            out[doo..doo + 4].copy_from_slice(&src[so..so + 4]);
        }
    }
    PixmapData::new(w, h, out)
}

#[cfg(test)]
#[path = "buffers_tests.rs"]
mod tests;
