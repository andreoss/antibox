 use antibox_core::error::Result;

use antibox_core::libc;
use super::bindings::*;
use super::connection::XcbConnection;
use antibox_core::backend::{
    DisplayBackend, EventMask, PropMode, RenderBackend, ShapeOp, StackMode, WindowHandle,
};
use antibox_core::point::Point;
use antibox_core::rect::Rect;
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

    fn map(&self) -> Result<()> {
        self.conn.map_window(self.id)
    }

    fn unmap(&self) -> Result<()> {
        self.conn.unmap_window(self.id)
    }

    fn destroy(&self) -> Result<()> {
        self.conn.destroy_window(self.id)
    }

    fn configure(
        &self,
        x: Option<i32>,
        y: Option<i32>,
        w: Option<u16>,
        h: Option<u16>,
    ) -> Result<()> {
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
        Ok(())
    }

    fn raise(&self) -> Result<()> {
        let vals = [XCB_STACK_MODE_ABOVE];
        unsafe {
            xcb_configure_window(
                self.conn.raw(),
                self.id,
                XCB_CONFIG_WINDOW_STACK_MODE,
                vals.as_ptr(),
            )
        };
        Ok(())
    }

    fn lower(&self) -> Result<()> {
        let vals = [XCB_STACK_MODE_BELOW];
        unsafe {
            xcb_configure_window(
                self.conn.raw(),
                self.id,
                XCB_CONFIG_WINDOW_STACK_MODE,
                vals.as_ptr(),
            )
        };
        Ok(())
    }

    fn reparent(&self, parent: u32, point: Point) -> Result<()> {
        self.conn.reparent_window(self.id, parent, point)
    }

    fn set_title(&self, title: &str) -> Result<()> {
        let wm_name = self.conn.intern_atom("_NET_WM_NAME")?;
        let utf8 = self.conn.intern_atom("UTF8_STRING")?;
        self.conn.change_property8(PropMode::Replace, self.id, wm_name, utf8, title.as_bytes())
    }

    fn set_class(&self, instance: &str, class: &str) -> Result<()> {
        let wm_class = self.conn.intern_atom("WM_CLASS")?;
        let string = self.conn.intern_atom("STRING")?;
        let mut data = Vec::with_capacity(instance.len() + class.len() + 1);
        data.extend_from_slice(instance.as_bytes());
        data.push(0);
        data.extend_from_slice(class.as_bytes());
        data.push(0);
        self.conn.change_property8(PropMode::Replace, self.id, wm_class, string, &data)
    }

    fn select_input(&self, event_mask: EventMask) -> Result<()> {
        let vals = [event_mask.bits() as u32];
        unsafe {
            xcb_change_window_attributes(self.conn.raw(), self.id, XCB_CW_EVENT_MASK, vals.as_ptr())
        };
        Ok(())
    }

    fn get_property(
        &self,
        atom: u32,
        offset: u32,
        length: u32,
    ) -> Result<Option<Vec<u8>>> {
        self.conn.get_property(self.id, atom, 0, offset, length)
    }

    fn get_geometry(&self) -> Result<(u16, u16)> {
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

    fn get_geometry_rect(&self) -> Result<Rect> {
        let cookie = unsafe { xcb_get_geometry(self.conn.raw(), self.id) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_geometry_reply(self.conn.raw(), cookie, &mut e) };
        if r.is_null() {
            return Err("get_geometry failed".into());
        }
        let rect = unsafe {
            Rect::new(
                (*r).x as i32,
                (*r).y as i32,
                (*r).width as i32,
                (*r).height as i32,
            )
        };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(rect)
    }

    fn move_window(&self, point: Point) -> Result<()> {
        let vals = [point.x as u32, point.y as u32];
        unsafe { xcb_configure_window(self.conn.raw(), self.id, XCB_CONFIG_WINDOW_X | XCB_CONFIG_WINDOW_Y, vals.as_ptr()) };
        Ok(())
    }

    fn resize(&self, w: u16, h: u16) -> Result<()> {
        let vals = [w as u32, h as u32];
        unsafe { xcb_configure_window(self.conn.raw(), self.id, XCB_CONFIG_WINDOW_WIDTH | XCB_CONFIG_WINDOW_HEIGHT, vals.as_ptr()) };
        Ok(())
    }

    fn restack(&self, sibling: Option<u32>, mode: StackMode) -> Result<()> {
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
        Ok(())
    }

    fn translate_coords(&self, point: Point) -> Result<Point> {
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

    fn select_shape_input(&self) -> Result<()> {
        if self.conn.shape_event_base() == 0 {
            return Ok(());
        }
        unsafe { xcb_shape_select_input(self.conn.raw(), self.id, 1) };
        Ok(())
    }

    fn set_shape_rectangles(&self, rects: &[(i16, i16, u16, u16)], op: ShapeOp) -> Result<()> {
        if self.conn.shape_event_base() == 0 {
            return Ok(());
        }
        let xrects: Vec<xcb_rectangle_t> = rects
            .iter()
            .map(|&(x, y, width, height)| xcb_rectangle_t {
                x,
                y,
                width,
                height,
            })
            .collect();
        unsafe {
            xcb_shape_rectangles(
                self.conn.raw(),
                shape_so(op),
                XCB_SHAPE_SK_BOUNDING,
                0,
                self.id,
                0,
                0,
                xrects.len() as u32,
                xrects.as_ptr(),
            )
        };
        Ok(())
    }

    fn combine_shape(&self, src: u32, offset: (i16, i16), op: ShapeOp) -> Result<()> {
        if self.conn.shape_event_base() == 0 {
            return Ok(());
        }
        unsafe {
            xcb_shape_combine(
                self.conn.raw(),
                shape_so(op),
                XCB_SHAPE_SK_BOUNDING,
                XCB_SHAPE_SK_BOUNDING,
                self.id,
                offset.0,
                offset.1,
                src,
            )
        };
        Ok(())
    }

    fn query_shaped(&self) -> Result<bool> {
        if self.conn.shape_event_base() == 0 {
            return Ok(false);
        }
        let cookie = unsafe { xcb_shape_query_extents(self.conn.raw(), self.id) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_shape_query_extents_reply(self.conn.raw(), cookie, &mut e) };
        if r.is_null() {
            return Ok(false);
        }
        let shaped = unsafe { (*r).bounding_shaped != 0 };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(shaped)
    }
}

const fn shape_so(op: ShapeOp) -> u8 {
    match op {
        ShapeOp::Set => XCB_SHAPE_SO_SET,
        ShapeOp::Union => XCB_SHAPE_SO_UNION,
        ShapeOp::Intersect => XCB_SHAPE_SO_INTERSECT,
        ShapeOp::Subtract => XCB_SHAPE_SO_SUBTRACT,
    }
}
