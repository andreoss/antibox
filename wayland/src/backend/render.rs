use super::*;
use antibox_core::backend::{PointerGrab, RootWindow};

impl RenderBackend for WaylandCompositor {
    fn root(&self) -> RootWindow {
        RootWindow::new(ROOT_WINDOW)
    }

    fn screen_width(&self) -> u16 {
        self.shared.lock().screen_w
    }

    fn screen_height(&self) -> u16 {
        self.shared.lock().screen_h
    }

    fn screen_depth(&self) -> u8 {
        32
    }

    fn create_window(
        &self,
        parent: u32,
        rect: Rect,
        class: WmWindowClass,
        override_redirect: bool,
        event_mask: EventMask,
    ) -> R<Box<dyn WindowHandle>> {
        let kind = match class {
            WmWindowClass::InputOnly => WinKind::InputOnly,
            _ => WinKind::Server,
        };
        let id = {
            let mut s = self.shared.lock();
            let id = s.alloc_id();
            s.windows.insert(
                id,
                WinRec {
                    kind,
                    rect,
                    mapped: false,
                    override_redirect,
                    depth: 32,
                    parent,
                    event_mask: event_mask.bits(),
                },
            );
            id
        };
        if kind == WinKind::Server {
            self.ensure_buffer(id, rect.w as u16, rect.h as u16);
        }
        Ok(Box::new(WaylandWindow::new(
            self.shared.clone(),
            Arc::clone(&self.buffers),
            id,
        )))
    }

    fn wrap_window(&self, xid: u32) -> R<Box<dyn WindowHandle>> {
        Ok(Box::new(WaylandWindow::new(
            self.shared.clone(),
            Arc::clone(&self.buffers),
            xid,
        )))
    }

    fn create_graphics(&self, drawable: u32) -> R<Box<dyn GraphicsContext>> {
        if !self.buffers.contains(drawable) {
            let (w, h) = {
                let s = self.shared.lock();
                s.windows
                    .get(&drawable)
                    .map_or((s.screen_w, s.screen_h), |r| {
                        (r.rect.w as u16, r.rect.h as u16)
                    })
            };
            self.ensure_buffer(drawable, w, h);
        }
        Ok(Box::new(WaylandGraphics::new(
            Arc::clone(&self.buffers),
            drawable,
        )))
    }

    fn intern_atom(&self, name: &str) -> R<u32> {
        Ok(self.shared.lock().atoms.intern(name))
    }

    fn change_property8(
        &self,
        _mode: PropMode,
        window: u32,
        atom: u32,
        _type_atom: u32,
        data: &[u8],
    ) -> R {
        self.shared
            .lock()
            .props
            .insert((window, atom), data.to_vec());
        Ok(())
    }

    fn change_property32(
        &self,
        _mode: PropMode,
        window: u32,
        atom: u32,
        _type_atom: u32,
        data: &[u32],
    ) -> R {
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_ne_bytes()).collect();
        self.shared.lock().props.insert((window, atom), bytes);
        Ok(())
    }

    fn get_property(
        &self,
        window: u32,
        atom: u32,
        _type_atom: u32,
        _offset: u32,
        _length: u32,
    ) -> R<Option<Vec<u8>>> {
        Ok(self.shared.lock().props.get(&(window, atom)).cloned())
    }

    fn delete_property(&self, window: u32, atom: u32) -> R {
        self.shared.lock().props.remove(&(window, atom));
        Ok(())
    }

    fn grab_pointer(&self, grab: PointerGrab) -> R {
        self.shared.lock().pointer_grab = Some(grab.window);
        Ok(())
    }

    fn ungrab_pointer(&self, _time: u32) -> R {
        self.shared.lock().pointer_grab = None;
        Ok(())
    }

    fn send_event(
        &self,
        _propagate: bool,
        _destination: u32,
        _event_mask: u32,
        _message_type: u32,
        _data: &[u32; 5],
    ) -> R {
        Ok(())
    }

    fn create_pixmap(&self, w: u16, h: u16, _depth: u8) -> R<u32> {
        Ok(self.buffers.create(w, h))
    }

    fn free_pixmap(&self, pixmap: u32) -> R {
        self.buffers.remove(pixmap);
        Ok(())
    }
}
