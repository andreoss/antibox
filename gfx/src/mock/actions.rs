use super::{
    display::MockDisplay,
    graphics::MockGraphics,
    window::{Lifecycle, MockError, MockWindow},
};
use crate::backend::{
    EventMask, GraphicsContext, PointerGrab, PropMode, RenderBackend, RootWindow, WindowHandle,
    WmWindowClass,
};
use crate::rect::Rect;
use std::error::Error;

impl RenderBackend for MockDisplay {
    fn root(&self) -> RootWindow {
        RootWindow::new(self.root_window)
    }

    fn screen_width(&self) -> u16 {
        self.width.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn screen_height(&self) -> u16 {
        self.height.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn screen_depth(&self) -> u8 {
        self.depth
    }

    fn create_window(
        &self,
        _parent: u32,
        _rect: Rect,
        _class: WmWindowClass,
        _override_redirect: bool,
        _event_mask: EventMask,
    ) -> Result<Box<dyn WindowHandle>, Box<dyn Error>> {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        let window = MockWindow::with_lifecycle(id, self.lifecycle.clone());
        self.lifecycle.lock().unwrap().push((id, Lifecycle::Create));
        self.windows.lock().unwrap().insert(id, window.clone());
        Ok(Box::new(window))
    }

    fn wrap_window(&self, xid: u32) -> Result<Box<dyn WindowHandle>, Box<dyn Error>> {
        if self.broken.lock().unwrap().contains(&xid) {
            return Err(Box::new(MockError::from(format!("broken window {}", xid))));
        }
        let mut windows = self.windows.lock().unwrap();
        let entry = windows
            .entry(xid)
            .or_insert_with(|| MockWindow::with_lifecycle(xid, self.lifecycle.clone()));
        Ok(Box::new(entry.clone()))
    }

    fn create_graphics(&self, drawable: u32) -> Result<Box<dyn GraphicsContext>, Box<dyn Error>> {
        Ok(Box::new(MockGraphics::new(drawable)))
    }

    fn intern_atom(&self, name: &str) -> Result<u32, Box<dyn Error>> {
        let mut atoms = self.atoms.lock().unwrap();
        if let Some(&id) = atoms.get(name) {
            return Ok(id);
        }
        let id = atoms.len() as u32 + 1;
        atoms.insert(name.to_string(), id);
        Ok(id)
    }

    fn change_property8(
        &self,
        _mode: PropMode,
        window: u32,
        atom: u32,
        _type_atom: u32,
        data: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        if let Some(w) = self.windows.lock().unwrap().get(&window) {
            w.change_property(atom, data.to_vec());
        }
        Ok(())
    }

    fn change_property32(
        &self,
        _mode: PropMode,
        window: u32,
        atom: u32,
        _type_atom: u32,
        data: &[u32],
    ) -> Result<(), Box<dyn Error>> {
        if let Some(w) = self.windows.lock().unwrap().get(&window) {
            let mut bytes: Vec<u8> = Vec::with_capacity(data.len() * 4);
            for &v in data {
                bytes.push(v as u8);
                bytes.push((v >> 8) as u8);
                bytes.push((v >> 16) as u8);
                bytes.push((v >> 24) as u8);
            }
            w.change_property(atom, bytes);
        }
        Ok(())
    }

    fn get_property(
        &self,
        window: u32,
        atom: u32,
        _type_atom: u32,
        _offset: u32,
        _length: u32,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        let windows = self.windows.lock().unwrap();
        Ok(windows.get(&window).and_then(|w| w.get_property_data(atom)))
    }

    fn delete_property(&self, window: u32, atom: u32) -> Result<(), Box<dyn Error>> {
        if let Some(w) = self.windows.lock().unwrap().get(&window) {
            w.delete_property(atom);
        }
        Ok(())
    }

    fn grab_pointer(&self, _grab: PointerGrab) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn ungrab_pointer(&self, _time: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn send_event(
        &self,
        _propagate: bool,
        _destination: u32,
        _event_mask: u32,
        _message_type: u32,
        _data: &[u32; 5],
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn create_pixmap(&self, w: u16, h: u16, _depth: u8) -> Result<u32, Box<dyn Error>> {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        let mw = MockWindow::new(id);
        mw.configure(None, None, Some(w), Some(h))?;
        self.windows.lock().unwrap().insert(id, mw);
        Ok(id)
    }

    fn free_pixmap(&self, pixmap: u32) -> Result<(), Box<dyn Error>> {
        self.windows.lock().unwrap().remove(&pixmap);
        Ok(())
    }

    fn convert_selection(
        &self,
        _requestor: u32,
        _selection: u32,
        _target: u32,
        _property: u32,
        _time: u32,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
