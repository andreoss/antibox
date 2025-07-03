use crate::buffers::BufferStore;
use crate::graphics::WaylandGraphics;
use crate::shared::{ButtonGrab, Intent, KeyGrab, Shared, WinKind, WinRec, ROOT_WINDOW};
#[allow(unused_imports)]
use antibox_core::backend::{
    BackendEvent, DisplayBackend, EventMask, GrabMode, GraphicsContext, KeyButMask,
    KeyboardMapping, ModifierMapping, MonitorInfo, PointerState, PropMode, QueryTreeResult,
    RenderBackend, WindowAttributes, WindowHandle, WmWindowClass,
};
use antibox_core::canvas::SoftCanvas;
use antibox_core::rect::Rect;
use std::os::unix::io::RawFd;
use std::sync::Arc;

use antibox_core::error::Result;

type R<T = ()> = Result<T>;

const MIN_KEYCODE: u8 = 8;
const MAX_KEYCODE: u8 = 255;

pub struct WaylandCompositor {
    shared: Shared,
    buffers: Arc<BufferStore>,
}

impl WaylandCompositor {
    pub const fn new(shared: Shared, buffers: Arc<BufferStore>) -> Self {
        Self { shared, buffers }
    }

    pub const fn shared(&self) -> &Shared {
        &self.shared
    }

    pub const fn buffers(&self) -> &Arc<BufferStore> {
        &self.buffers
    }

    fn ensure_buffer(&self, id: u32, w: u16, h: u16) {
        if !self.buffers.contains(id) {
            self.buffers.insert(id, SoftCanvas::new(w.max(1), h.max(1)));
        }
    }

    fn push_intent(&self, intent: Intent) {
        self.shared.lock().intents.push(intent);
    }
}

mod display;
mod render;

pub struct WaylandWindow {
    shared: Shared,
    buffers: Arc<BufferStore>,
    id: u32,
}

impl WaylandWindow {
    pub const fn new(shared: Shared, buffers: Arc<BufferStore>, id: u32) -> Self {
        Self {
            shared,
            buffers,
            id,
        }
    }

    fn set_prop_str(&self, atom_name: &str, value: &str) {
        let mut s = self.shared.lock();
        let atom = s.atoms.intern(atom_name);
        s.props.insert((self.id, atom), value.as_bytes().to_vec());
    }
}

impl WindowHandle for WaylandWindow {
    fn id(&self) -> u32 {
        self.id
    }
    fn map(&self) -> R {
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&self.id) {
                rec.mapped = true;
                if rec.kind == WinKind::Server {
                    let rect = rec.rect;
                    s.events.push(BackendEvent::Expose {
                        window: self.id,
                        rect: Rect::new(0, 0, rect.w, rect.h),
                    });
                }
            }
        }
        self.shared.lock().intents.push(Intent::Map(self.id));
        Ok(())
    }
    fn unmap(&self) -> R {
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&self.id) {
                let was = rec.mapped;
                rec.mapped = false;
                if was && rec.kind == WinKind::Client {
                    s.events.push(BackendEvent::UnmapNotify { window: self.id });
                }
            }
        }
        self.shared.lock().intents.push(Intent::Unmap(self.id));
        Ok(())
    }
    fn destroy(&self) -> R {
        let ids = {
            let mut s = self.shared.lock();
            let ids = s.subtree(self.id);
            for id in &ids {
                s.windows.remove(id);
            }
            ids
        };
        for id in ids {
            self.buffers.remove(id);
            self.shared.lock().intents.push(Intent::Destroy(id));
        }
        Ok(())
    }
    fn configure(&self, x: Option<i32>, y: Option<i32>, w: Option<u16>, h: Option<u16>) -> R {
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&self.id) {
                if let Some(x) = x {
                    rec.rect.x = x;
                }
                if let Some(y) = y {
                    rec.rect.y = y;
                }
                if let Some(w) = w {
                    rec.rect.w = w as i32;
                }
                if let Some(h) = h {
                    rec.rect.h = h as i32;
                }
            }
        }
        if let (Some(w), Some(h)) = (w, h) {
            let same = self
                .buffers
                .with(self.id, |c| c.width() == w.max(1) && c.height() == h.max(1))
                .unwrap_or(false);
            if !same {
                let mut canvas = SoftCanvas::new(w.max(1), h.max(1));
                if let Some(pd) = self.buffers.snapshot(self.id) {
                    canvas.draw_pixmap(0, 0, &pd);
                }
                self.buffers.insert(self.id, canvas);
                let mut s = self.shared.lock();
                if s.windows.get(&self.id).map(|r| (r.kind, r.mapped))
                    == Some((WinKind::Server, true))
                {
                    s.events.push(BackendEvent::Expose {
                        window: self.id,
                        rect: Rect::new(0, 0, w as i32, h as i32),
                    });
                }
            }
        }
        self.shared.lock().intents.push(Intent::Configure {
            win: self.id,
            x,
            y,
            w,
            h,
        });
        Ok(())
    }
    fn raise(&self) -> R {
        self.shared.lock().intents.push(Intent::Raise(self.id));
        Ok(())
    }
    fn lower(&self) -> R {
        self.shared.lock().intents.push(Intent::Lower(self.id));
        Ok(())
    }
    fn reparent(&self, parent: u32, point: antibox_core::point::Point) -> R {
        let mut s = self.shared.lock();
        if let Some(rec) = s.windows.get_mut(&self.id) {
            rec.parent = parent;
            rec.rect.x = point.x;
            rec.rect.y = point.y;
            if rec.mapped && rec.kind == WinKind::Client {
                s.events.push(BackendEvent::UnmapNotify { window: self.id });
            }
        }
        Ok(())
    }
    fn set_title(&self, title: &str) -> R {
        self.set_prop_str("_NET_WM_NAME", title);
        Ok(())
    }
    fn set_class(&self, instance: &str, class: &str) -> R {
        let mut buf = instance.as_bytes().to_vec();
        buf.push(0);
        buf.extend_from_slice(class.as_bytes());
        buf.push(0);
        let mut s = self.shared.lock();
        let atom = s.atoms.intern("WM_CLASS");
        s.props.insert((self.id, atom), buf);
        Ok(())
    }
    fn select_input(&self, event_mask: EventMask) -> R {
        if let Some(rec) = self.shared.lock().windows.get_mut(&self.id) {
            rec.event_mask = event_mask.bits();
        }
        Ok(())
    }
    fn get_property(&self, atom: u32, _offset: u32, _length: u32) -> R<Option<Vec<u8>>> {
        Ok(self.shared.lock().props.get(&(self.id, atom)).cloned())
    }
    fn get_geometry(&self) -> R<(u16, u16)> {
        let s = self.shared.lock();
        let rec = s.windows.get(&self.id);
        Ok(rec.map_or((0, 0), |r| (r.rect.w as u16, r.rect.h as u16)))
    }
    fn move_window(&self, point: antibox_core::point::Point) -> R {
        self.configure(Some(point.x), Some(point.y), None, None)
    }
    fn resize(&self, w: u16, h: u16) -> R {
        self.configure(None, None, Some(w), Some(h))
    }
    fn restack(&self, _sibling: Option<u32>, mode: antibox_core::backend::StackMode) -> R {
        use antibox_core::backend::StackMode;
        let intent = match mode {
            StackMode::Below | StackMode::BottomIf => Intent::Lower(self.id),
            _ => Intent::Raise(self.id),
        };
        self.shared.lock().intents.push(intent);
        Ok(())
    }
    fn translate_coords(&self, point: antibox_core::point::Point) -> R<antibox_core::point::Point> {
        let (ox, oy) = self.shared.lock().absolute_origin(self.id);
        Ok(antibox_core::point::Point::new(point.x + ox, point.y + oy))
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
