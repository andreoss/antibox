
use antibox_core::libc;
use super::bindings::*;
use super::connection::XcbConnection;
use antibox_core::backend::{DisplayBackend, EventMask, PropMode, RenderBackend, StackMode, WindowHandle};
use antibox_core::point::Point;
use std::sync::Arc;

pub struct XcbWindow {
    conn: Arc<XcbConnection>,
    id: u32,
}

impl XcbWindow {
    pub fn new(conn: Arc<XcbConnection>, id: u32) -> Self {
        Self { conn, id }
    }
}

impl WindowHandle for XcbWindow {
    fn id(&self) -> u32 {
        self.id
    }

    fn map(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.map_window(self.id)
    }

    fn unmap(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.unmap_window(self.id)
    }

    fn destroy(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.destroy_window(self.id)
    }

    fn configure(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        w: Option<u16>,
        h: Option<u16>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut vals: Vec<u32> = Vec::with_capacity(4);
        let mut mask = 0u16;
        if let Some(x) = x {
            mask |= XCB_CONFIG_WINDOW_X;
            vals.push(x as u32);
        }
        if let Some(y) = y {
            mask |= XCB_CONFIG_WINDOW_Y;
            vals.push(y as u32);
        }
        if let Some(w) = w {
            mask |= XCB_CONFIG_WINDOW_WIDTH;
            vals.push(w as u32);
        }
        if let Some(h) = h {
            mask |= XCB_CONFIG_WINDOW_HEIGHT;
            vals.push(h as u32);
        }
        unsafe { xcb_configure_window(self.conn.raw(), self.id, mask, vals.as_ptr()) };
        self.conn.flush()
    }

    fn raise(&self) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [XCB_STACK_MODE_ABOVE];
        unsafe {
            xcb_configure_window(
                self.conn.raw(),
                self.id,
                XCB_CONFIG_WINDOW_STACK_MODE,
                vals.as_ptr(),
            )
        };
        self.conn.flush()
    }

    fn lower(&self) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [XCB_STACK_MODE_BELOW];
        unsafe {
            xcb_configure_window(
                self.conn.raw(),
                self.id,
                XCB_CONFIG_WINDOW_STACK_MODE,
                vals.as_ptr(),
            )
        };
        self.conn.flush()
    }

    fn reparent(&self, parent: u32, point: Point) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.reparent_window(self.id, parent, point)
    }

    fn set_title(&self, title: &str) -> Result<(), Box<dyn std::error::Error>> {
        let wm_name = self.conn.intern_atom("_NET_WM_NAME")?;
        let utf8 = self.conn.intern_atom("UTF8_STRING")?;
        self.conn.change_property8(PropMode::Replace, self.id, wm_name, utf8, title.as_bytes())
    }

    fn set_class(&self, instance: &str, class: &str) -> Result<(), Box<dyn std::error::Error>> {
        let wm_class = self.conn.intern_atom("WM_CLASS")?;
        let string = self.conn.intern_atom("STRING")?;
        let mut data = Vec::with_capacity(instance.len() + class.len() + 1);
        data.extend_from_slice(instance.as_bytes());
        data.push(0);
        data.extend_from_slice(class.as_bytes());
        data.push(0);
        self.conn.change_property8(PropMode::Replace, self.id, wm_class, string, &data)
    }

    fn select_input(&self, event_mask: EventMask) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [event_mask.bits() as u32];
        unsafe {
            xcb_change_window_attributes(self.conn.raw(), self.id, XCB_CW_EVENT_MASK, vals.as_ptr())
        };
        self.conn.flush()
    }

    fn get_property(
        &self,
        atom: u32,
        offset: u32,
        length: u32,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        self.conn.get_property(self.id, atom, 0, offset, length)
    }

    fn get_geometry(&self) -> Result<(u16, u16), Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_get_geometry(self.conn.raw(), self.id) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_geometry_reply(self.conn.raw(), cookie, &mut e) };
        if r.is_null() {
            return Err("get_geometry failed".into());
        }
        let w = unsafe { (*r).width };
        let h = unsafe { (*r).height };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok((w, h))
    }

    fn move_window(&self, point: Point) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [point.x as u32, point.y as u32];
        unsafe { xcb_configure_window(self.conn.raw(), self.id, XCB_CONFIG_WINDOW_X | XCB_CONFIG_WINDOW_Y, vals.as_ptr()) };
        self.conn.flush()
    }

    fn resize(&self, w: u16, h: u16) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [w as u32, h as u32];
        unsafe { xcb_configure_window(self.conn.raw(), self.id, XCB_CONFIG_WINDOW_WIDTH | XCB_CONFIG_WINDOW_HEIGHT, vals.as_ptr()) };
        self.conn.flush()
    }

    fn restack(&self, sibling: Option<u32>, mode: StackMode) -> Result<(), Box<dyn std::error::Error>> {
        let mode = match mode {
            StackMode::Above => XCB_STACK_MODE_ABOVE,
            StackMode::Below => XCB_STACK_MODE_BELOW,
            StackMode::TopIf => XCB_STACK_MODE_TOP_IF,
            StackMode::BottomIf => XCB_STACK_MODE_BOTTOM_IF,
            StackMode::Opposite => XCB_STACK_MODE_OPPOSITE,
        };
        let mut mask = 0u16;
        let mut vals: Vec<u32> = Vec::with_capacity(2);
        if let Some(s) = sibling {
            mask |= XCB_CONFIG_WINDOW_SIBLING;
            vals.push(s);
        }
        mask |= XCB_CONFIG_WINDOW_STACK_MODE;
        vals.push(mode);
        unsafe { xcb_configure_window(self.conn.raw(), self.id, mask, vals.as_ptr()) };
        self.conn.flush()
    }

    fn translate_coords(&self, point: Point) -> Result<Point, Box<dyn std::error::Error>> {
        let root = self.conn.root().read_id();
        let cookie = unsafe { xcb_translate_coordinates(self.conn.raw(), self.id, root, point.x as i16, point.y as i16) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_translate_coordinates_reply(self.conn.raw(), cookie, &mut e) };
        if r.is_null() {
            return Err("translate_coords failed".into());
        }
        let x = unsafe { (*r).dst_x };
        let y = unsafe { (*r).dst_y };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(Point::new(x as i32, y as i32))
    }
}
