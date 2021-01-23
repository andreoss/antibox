
use antibox_core::libc;
use super::bindings::*;
use crate::xcb::graphics::XcbGraphics;
use crate::xcb::window::XcbWindow;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::os::raw::{c_char, c_int};
use std::os::unix::io::RawFd;
use std::sync::Arc;

pub struct XcbConnection {
    conn: *mut xcb_connection_t,
    screen: xcb_screen_t,
    root: RootWindow,
    depth: u8,
    keycode_min: u8,
    keycode_max: u8,
    cursor_font: u32,
    xkb_event_base: u8,
    render_a8: u32,
    render_rgb24: u32,
    render_argb32: u32,
    atom_cache: std::cell::RefCell<std::collections::HashMap<String, u32>>,
    last_event_time: antibox_core::sync::atomic::AtomicU32,
}

unsafe impl Send for XcbConnection {}
unsafe impl Sync for XcbConnection {}

#[derive(Debug)]
pub struct XcbError(pub String);

impl std::fmt::Display for XcbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for XcbError {}

fn err(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(XcbError(msg.into()))
}

fn event_time(ev: &xcb_generic_event_t) -> Option<u32> {
    let off = match ev.response_type & 0x7f {
        2..=8 | 29 | 30 | 31 => 4,
        28 => 12,
        _ => return None,
    };
    let base = ev as *const xcb_generic_event_t as *const u8;
    Some(unsafe { std::ptr::read_unaligned(base.add(off) as *const u32) })
}

fn render_init(conn: *mut xcb_connection_t) -> (u32, u32, u32) {
    let cookie = unsafe { xcb_render_query_version(conn, 0, 11) };
    let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
    let vr = unsafe { xcb_render_query_version_reply(conn, cookie, &mut e) };
    if vr.is_null() {
        return (0, 0, 0);
    }
    unsafe { libc::free(vr as *mut libc::c_void) };
    let cookie = unsafe { xcb_render_query_pict_formats(conn) };
    let r = unsafe { xcb_render_query_pict_formats_reply(conn, cookie, &mut e) };
    if r.is_null() {
        return (0, 0, 0);
    }
    let count = unsafe { (*r).num_formats } as usize;
    let base = unsafe {
        (r as *const u8).add(std::mem::size_of::<xcb_render_query_pict_formats_reply_t>())
            as *const xcb_render_pictforminfo_t
    };
    let (mut a8, mut rgb24, mut argb32) = (0u32, 0u32, 0u32);
    for i in 0..count {
        let f = unsafe { &*base.add(i) };
        if f.type_ != XCB_RENDER_PICT_TYPE_DIRECT {
            continue;
        }
        if a8 == 0 && f.depth == 8 && f.direct.alpha_mask == 0xff && f.direct.red_mask == 0 {
            a8 = f.id;
        }
        if rgb24 == 0 && f.depth == 24 && f.direct.red_mask == 0xff && f.direct.alpha_mask == 0 {
            rgb24 = f.id;
        }
        if argb32 == 0 && f.depth == 32 && f.direct.red_mask == 0xff && f.direct.alpha_mask == 0xff
        {
            argb32 = f.id;
        }
    }
    unsafe { libc::free(r as *mut libc::c_void) };
    (a8, rgb24, argb32)
}

fn map_state(raw: u8) -> MapState {
    match raw {
        0 => MapState::Unmapped,
        1 => MapState::Unviewable,
        _ => MapState::Viewable,
    }
}

impl XcbConnection {
    pub fn open_arc(display: Option<&str>) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let resolved = display
            .map(|s| s.to_string())
            .or_else(|| std::env::var("DISPLAY").ok());
        let cstr = resolved
            .as_ref()
            .and_then(|s| std::ffi::CString::new(s.as_str()).ok());
        let name = cstr.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());
        let mut screen_num: c_int = 0;
        let conn = unsafe { xcb_connect(name, &mut screen_num) };
        if conn.is_null() {
            return Err(err("xcb_connect returned null"));
        }
        let err_code = unsafe { xcb_connection_has_error(conn) };
        if err_code != 0 {
            unsafe { xcb_disconnect(conn) };
            let proc_display = std::env::var("DISPLAY").unwrap_or_else(|_| "<unset>".to_string());
            let unix_sock = std::path::Path::new("/tmp/.X11-unix").exists();
            return Err(err(format!(
                "xcb connection error code {} (requested={:?} DISPLAY={} /tmp/.X11-unix={})",
                err_code, resolved, proc_display, unix_sock,
            )));
        }
        let setup = unsafe { xcb_get_setup(conn) };
        if setup.is_null() {
            unsafe { xcb_disconnect(conn) };
            return Err(err("xcb_get_setup returned null"));
        }
        let mut it = unsafe { xcb_setup_roots_iterator(setup) };
        let mut screen = unsafe { std::ptr::read(it.data) };
        for _ in 0..screen_num {
            unsafe { xcb_screen_next(&mut it) };
            screen = unsafe { std::ptr::read(it.data) };
        }
        let depth = screen.root_depth;
        let root = RootWindow::new(screen.root);
        let keycode_min = unsafe { (*setup).min_keycode };
        let keycode_max = unsafe { (*setup).max_keycode };
        let cursor_font = unsafe { xcb_generate_id(conn) };
        let font_name = b"cursor\0";
        unsafe {
            xcb_open_font(conn, cursor_font, font_name.len() as u16, font_name.as_ptr() as *const _)
        };
        let xkb_event_base = super::xkb::init(conn);
        let (render_a8, render_rgb24, render_argb32) = render_init(conn);
        Ok(Arc::new(XcbConnection {
            conn,
            screen,
            root,
            depth,
            keycode_min,
            keycode_max,
            cursor_font,
            xkb_event_base,
            render_a8,
            render_rgb24,
            render_argb32,
            atom_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            last_event_time: antibox_core::sync::atomic::AtomicU32::new(0),
        }))
    }

    pub fn raw(&self) -> *mut xcb_connection_t {
        self.conn
    }

    pub(crate) fn xkb_event_base(&self) -> u8 {
        self.xkb_event_base
    }

    pub(crate) fn render_a8_format(&self) -> u32 {
        self.render_a8
    }

    pub(crate) fn render_format_for(&self, depth: u8) -> u32 {
        match depth {
            24 => self.render_rgb24,
            32 => self.render_argb32,
            _ => 0,
        }
    }

    fn screen(&self) -> &xcb_screen_t {
        &self.screen
    }

    fn check(&self) -> Option<Box<dyn std::error::Error>> {
        let e = unsafe { xcb_connection_has_error(self.conn) };
        if e == 0 {
            None
        } else {
            Some(err(format!("xcb connection error code {}", e)))
        }
    }

    fn intern_atom_cached(&self, name: &str) -> u32 {
        if let Some(&a) = self.atom_cache.borrow().get(name) {
            return a;
        }
        let cname = std::ffi::CString::new(name).unwrap_or_default();
        let cookie = unsafe { xcb_intern_atom(self.conn, 0, name.len() as u16, cname.as_ptr()) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_intern_atom_reply(self.conn, cookie, &mut e) };
        let atom = if !r.is_null() {
            let a = unsafe { (*r).atom };
            unsafe { libc::free(r as *mut libc::c_void) };
            a
        } else {
            0
        };
        self.atom_cache.borrow_mut().insert(name.to_string(), atom);
        atom
    }

    fn alloc_id(&self) -> u32 {
        unsafe { xcb_generate_id(self.conn) }
    }
}

impl Drop for XcbConnection {
    fn drop(&mut self) {
        unsafe { xcb_disconnect(self.conn) };
    }
}

impl RenderBackend for XcbConnection {
    fn root(&self) -> RootWindow {
        self.root
    }
    fn screen_width(&self) -> u16 {
        self.screen().width_in_pixels
    }
    fn screen_height(&self) -> u16 {
        self.screen().height_in_pixels
    }
    fn screen_depth(&self) -> u8 {
        self.depth
    }

    fn select_root_input_checked(&self, mask: EventMask) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [mask.bits() as u32];
        unsafe {
            xcb_change_window_attributes(self.conn, self.root.read_id(), XCB_CW_EVENT_MASK, vals.as_ptr())
        };
        self.flush()?;
        if let Some(e) = self.check() {
            Err(e)
        } else {
            Ok(())
        }
    }

    fn create_window(
        &self,
        parent: u32,
        rect: Rect,
        class: WmWindowClass,
        override_redirect: bool,
        event_mask: EventMask,
    ) -> Result<Box<dyn WindowHandle>, Box<dyn std::error::Error>> {
        let wid = self.alloc_id();
        let class = match class {
            WmWindowClass::InputOutput => XCB_WINDOW_CLASS_INPUT_OUTPUT,
            WmWindowClass::InputOnly => XCB_WINDOW_CLASS_INPUT_ONLY,
            WmWindowClass::CopyFromParent => XCB_WINDOW_CLASS_COPY_FROM_PARENT,
        };
        let mut mask = 0u32;
        let mut vals: Vec<u32> = Vec::with_capacity(2);
        if override_redirect {
            mask |= XCB_CW_OVERRIDE_REDIRECT;
            vals.push(1);
        }
        mask |= XCB_CW_EVENT_MASK;
        vals.push(event_mask.bits() as u32);
        unsafe {
            xcb_create_window(
                self.conn,
                self.depth,
                wid,
                parent,
                rect.x as i16,
                rect.y as i16,
                rect.w as u16,
                rect.h as u16,
                0,
                class,
                self.screen().root_visual,
                mask,
                vals.as_ptr(),
            )
        };
        let conn = super::arcs::get_arc(self.conn);
        Ok(Box::new(XcbWindow::new(conn, wid)))
    }

    fn wrap_window(&self, xid: u32) -> Result<Box<dyn WindowHandle>, Box<dyn std::error::Error>> {
        let conn = super::arcs::get_arc(self.conn);
        Ok(Box::new(XcbWindow::new(conn, xid)))
    }

    fn create_graphics(&self, drawable: u32) -> Result<Box<dyn GraphicsContext>, Box<dyn std::error::Error>> {
        let gc = self.alloc_id();
        let vals = [self.screen().black_pixel, self.screen().white_pixel];
        unsafe {
            xcb_create_gc(self.conn, gc, drawable, XCB_GC_FOREGROUND | XCB_GC_BACKGROUND, vals.as_ptr())
        };
        let conn = super::arcs::get_arc(self.conn);
        Ok(Box::new(XcbGraphics::new(conn, drawable, gc, self.depth)))
    }

    fn intern_atom(&self, name: &str) -> Result<u32, Box<dyn std::error::Error>> {
        Ok(self.intern_atom_cached(name))
    }

    fn change_property8(
        &self,
        mode: PropMode,
        window: u32,
        atom: u32,
        type_atom: u32,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mode = match mode {
            PropMode::Replace => XCB_PROP_MODE_REPLACE,
            PropMode::Prepend => XCB_PROP_MODE_PREPEND,
            PropMode::Append => XCB_PROP_MODE_APPEND,
        };
        unsafe {
            xcb_change_property(self.conn, mode, window, atom, type_atom, 8, data.len() as u32, data.as_ptr() as *const _)
        };
        self.flush()
    }

    fn change_property32(
        &self,
        mode: PropMode,
        window: u32,
        atom: u32,
        type_atom: u32,
        data: &[u32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mode = match mode {
            PropMode::Replace => XCB_PROP_MODE_REPLACE,
            PropMode::Prepend => XCB_PROP_MODE_PREPEND,
            PropMode::Append => XCB_PROP_MODE_APPEND,
        };
        unsafe {
            xcb_change_property(self.conn, mode, window, atom, type_atom, 32, data.len() as u32, data.as_ptr() as *const _)
        };
        self.flush()
    }

    fn get_property(
        &self,
        window: u32,
        atom: u32,
        type_atom: u32,
        offset: u32,
        length: u32,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_get_property(self.conn, 0, window, atom, type_atom, offset, length) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_property_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Ok(None);
        }
        let format = unsafe { (*r).format };
        let num_items = unsafe { (*r).num_items };
        let type_ = unsafe { (*r).type_ };
        let data_ptr = unsafe { (r as *const u8).add(std::mem::size_of::<xcb_get_property_reply_t>()) };
        let bytes = match format {
            8 => num_items as usize,
            16 => num_items as usize * 2,
            32 => num_items as usize * 4,
            _ => 0,
        };
        let data = if bytes > 0 {
            unsafe { std::slice::from_raw_parts(data_ptr, bytes).to_vec() }
        } else {
            Vec::new()
        };
        unsafe { libc::free(r as *mut libc::c_void) };
        if type_ == 0 {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }

    fn delete_property(&self, window: u32, atom: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_delete_property(self.conn, window, atom) };
        self.flush()
    }

    fn grab_pointer(&self, grab: PointerGrab) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            xcb_grab_pointer(
                self.conn,
                grab.owner_events as u8,
                grab.window,
                grab.event_mask.bits() as u32 as u16,
                grab.pointer_mode as u8,
                grab.keyboard_mode as u8,
                grab.confine_to,
                grab.cursor,
                grab.time,
            )
        };
        self.flush()
    }

    fn ungrab_pointer(&self, time: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_ungrab_pointer(self.conn, time) };
        self.flush()
    }

    fn send_event(
        &self,
        propagate: bool,
        destination: u32,
        event_mask: u32,
        message_type: u32,
        data: &[u32; 5],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut ev: xcb_client_message_event_t = unsafe { std::mem::zeroed() };
        ev.response_type = 33;
        ev.format = 32;
        ev.window = destination;
        ev.type_ = message_type;
        ev.data = *data;
        unsafe {
            xcb_send_event(
                self.conn,
                propagate as u8,
                destination,
                event_mask,
                &ev as *const xcb_client_message_event_t as *const c_char,
            )
        };
        self.flush()
    }

    fn create_pixmap(&self, w: u16, h: u16, depth: u8) -> Result<u32, Box<dyn std::error::Error>> {
        if w == 0 || h == 0 {
            return Err(err("create_pixmap: zero dimension"));
        }
        let pid = self.alloc_id();
        unsafe { xcb_create_pixmap(self.conn, depth, pid, self.root.read_id(), w, h) };
        Ok(pid)
    }

    fn free_pixmap(&self, pixmap: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_free_pixmap(self.conn, pixmap) };
        self.flush()
    }

    fn convert_selection(&self, _r: u32, _s: u32, _t: u32, _p: u32, _time: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl DisplayBackend for XcbConnection {
    fn open(name: Option<&str>) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let arc = Self::open_arc(name)?;
        Arc::try_unwrap(arc).map_err(|_| err("unable to unwrap connection"))
    }


    fn fd(&self) -> RawFd {
        unsafe { xcb_get_file_descriptor(self.conn) }
    }

    fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_flush(self.conn) };
        Ok(())
    }

    fn poll_for_event(&self) -> Result<Option<BackendEvent>, Box<dyn std::error::Error>> {
        let ev = unsafe { xcb_poll_for_event(self.conn) };
        if ev.is_null() {
            return Ok(None);
        }
        let event = unsafe { std::ptr::read(ev) };
        unsafe { libc::free(ev as *mut libc::c_void) };
        if let Some(t) = event_time(&event) {
            self.last_event_time
                .store(t, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(crate::xcb::event::convert(&event, self))
    }

    fn last_event_time(&self) -> u32 {
        self.last_event_time
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn default_screen(&self) -> usize {
        0
    }

    fn screen_size_mm(&self) -> (u32, u32) {
        (
            self.screen().width_in_millimeters as u32,
            self.screen().height_in_millimeters as u32,
        )
    }

    fn keyboard_layout(&self) -> Option<String> {
        let (group, value) = super::xkb::group_and_rules(self)?;
        super::xkb::parse_xkb_layout(&value, group)
    }

    fn keyboard_info(&self) -> Option<KeyboardInfo> {
        let (group, value) = super::xkb::group_and_rules(self)?;
        super::xkb::parse_xkb_rules(&value, group)
    }

    fn set_keyboard_group(&self, group: usize) -> bool {
        self.xkb_event_base != 0 && super::xkb::set_group(self, group)
    }

    fn root_visual(&self) -> u32 {
        self.screen().root_visual
    }

    fn setup_min_keycode(&self) -> u8 {
        self.keycode_min
    }
    fn setup_max_keycode(&self) -> u8 {
        self.keycode_max
    }

    fn get_atom_name(&self, atom: u32) -> Result<String, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_get_atom_name(self.conn, atom) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_atom_name_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("get_atom_name failed"));
        }
        let len = unsafe { (*r).name_len } as usize;
        let data_ptr = unsafe { (r as *const u8).add(std::mem::size_of::<xcb_get_atom_name_reply_t>()) };
        let bytes = unsafe { std::slice::from_raw_parts(data_ptr, len) };
        let name = String::from_utf8_lossy(bytes).into_owned();
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(name)
    }

    fn query_extension(&self, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let cname = std::ffi::CString::new(name).unwrap_or_default();
        let cookie = unsafe { xcb_query_extension(self.conn, name.len() as u16, cname.as_ptr()) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_query_extension_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("query_extension failed"));
        }
        let present = unsafe { (*r).present != 0 };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(present)
    }

    fn get_keyboard_mapping(&self, first_keycode: u8, count: u8) -> Result<KeyboardMapping, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_get_keyboard_mapping(self.conn, first_keycode, count) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_keyboard_mapping_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("get_keyboard_mapping failed"));
        }
        let kpc = unsafe { (*r).keysyms_per_keycode } as usize;
        let total = count as usize * kpc;
        let data_ptr = unsafe { (r as *const u8).add(std::mem::size_of::<xcb_get_keyboard_mapping_reply_t>()) };
        let keysyms = unsafe { std::slice::from_raw_parts(data_ptr as *const u32, total).to_vec() };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(KeyboardMapping { keysyms_per_keycode: kpc as u8, keysyms })
    }

    fn get_modifier_mapping(&self) -> Result<ModifierMapping, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_get_modifier_mapping(self.conn) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_modifier_mapping_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("get_modifier_mapping failed"));
        }
        let kpm = unsafe { (*r).keycodes_per_modifier } as usize;
        let data_ptr = unsafe { (r as *const u8).add(std::mem::size_of::<xcb_get_modifier_mapping_reply_t>()) };
        let mut keycodes_per_modifier: [Vec<u8>; 8] = [
            Vec::new(), Vec::new(), Vec::new(), Vec::new(),
            Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        ];
        for m in 0..8 {
            let slice = unsafe { std::slice::from_raw_parts(data_ptr.add(m * kpm), kpm) };
            keycodes_per_modifier[m] = slice.to_vec();
        }
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(ModifierMapping { keycodes_per_modifier })
    }

    fn grab_key(
        &self,
        owner_events: bool,
        window: u32,
        modifiers: u16,
        keycode: u8,
        pointer_mode: GrabMode,
        keyboard_mode: GrabMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            xcb_grab_key(self.conn, owner_events as u8, window, modifiers, keycode, pointer_mode as u8, keyboard_mode as u8)
        };
        self.flush()
    }

    fn ungrab_key(&self, keycode: u8, modifiers: u16, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_ungrab_key(self.conn, keycode, window, modifiers) };
        self.flush()
    }

    fn grab_keyboard(&self, owner_events: bool, window: u32, time: u32, pointer_mode: GrabMode, keyboard_mode: GrabMode) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_grab_keyboard(self.conn, owner_events as u8, window, time, pointer_mode as u8, keyboard_mode as u8) };
        self.flush()
    }

    fn ungrab_keyboard(&self, time: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_ungrab_keyboard(self.conn, time) };
        self.flush()
    }

    fn grab_button(&self, grab: ButtonGrabSpec) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            xcb_grab_button(
                self.conn,
                grab.owner_events as u8,
                grab.window,
                grab.event_mask.bits() as u16,
                grab.pointer_mode as u8,
                grab.keyboard_mode as u8,
                grab.confine_to,
                grab.cursor,
                grab.button,
                grab.modifiers,
            )
        };
        self.flush()
    }

    fn ungrab_button(&self, button: u8, modifiers: u16, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_ungrab_button(self.conn, button, window, modifiers) };
        self.flush()
    }

    fn allow_events(&self, mode: u8, time: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_allow_events(self.conn, mode, time) };
        self.flush()
    }

    fn send_configure_notify(
        &self,
        window: u32,
        rect: Rect,
        border: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (x, y, width, height) = (rect.x, rect.y, rect.w.max(1) as u32, rect.h.max(1) as u32);
        let ev = xcb_configure_notify_event_t {
            response_type: 22,
            pad0: 0,
            sequence: 0,
            event: window,
            window,
            above_sibling: 0,
            x: x.max(std::i16::MIN as i32).min(std::i16::MAX as i32) as i16,
            y: y.max(std::i16::MIN as i32).min(std::i16::MAX as i32) as i16,
            width: width.min(std::u16::MAX as u32) as u16,
            height: height.min(std::u16::MAX as u32) as u16,
            border_width: border.min(std::u16::MAX as u32) as u16,
            override_redirect: 0,
            pad1: 0,
        };
        let mut buf = [0u8; 32];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &ev as *const xcb_configure_notify_event_t as *const u8,
                buf.as_mut_ptr(),
                std::mem::size_of::<xcb_configure_notify_event_t>(),
            );
            xcb_send_event(
                self.conn,
                0,
                window,
                XCB_EVENT_MASK_STRUCTURE_NOTIFY,
                buf.as_ptr() as *const c_char,
            )
        };
        self.flush()
    }

    fn check_for_error(&self) -> Option<Box<dyn std::error::Error>> {
        self.check()
    }

    fn query_pointer(&self, window: u32) -> Result<PointerState, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_query_pointer(self.conn, window) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_query_pointer_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("query_pointer failed"));
        }
        let root_x = unsafe { (*r).root_x };
        let root_y = unsafe { (*r).root_y };
        let mask = unsafe { (*r).mask };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(PointerState { root_x, root_y, mask: KeyButMask(mask) })
    }

    fn warp_pointer(&self, src_window: u32, dst_window: u32, src_area: Rect, dst: Point) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            xcb_warp_pointer(
                self.conn, src_window, dst_window,
                src_area.x as i16, src_area.y as i16, src_area.w as u16, src_area.h as u16,
                dst.x as i16, dst.y as i16,
            )
        };
        self.flush()
    }

    fn set_input_focus(&self, revert_to: u8, window: u32, time: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_set_input_focus(self.conn, revert_to, window, time) };
        self.flush()
    }

    fn map_window(&self, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_map_window(self.conn, window) };
        self.flush()
    }
    fn unmap_window(&self, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_unmap_window(self.conn, window) };
        self.flush()
    }
    fn destroy_window(&self, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_destroy_window(self.conn, window) };
        self.flush()
    }
    fn kill_client(&self, resource: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_kill_client(self.conn, resource) };
        self.flush()
    }
    fn configure_window(&self, window: u32, value_list: &[u32]) -> Result<(), Box<dyn std::error::Error>> {
        let mask = match value_list.len() {
            4 => XCB_CONFIG_WINDOW_X | XCB_CONFIG_WINDOW_Y | XCB_CONFIG_WINDOW_WIDTH | XCB_CONFIG_WINDOW_HEIGHT,
            2 => XCB_CONFIG_WINDOW_X | XCB_CONFIG_WINDOW_Y,
            1 => XCB_CONFIG_WINDOW_X,
            _ => return Err(err(format!("unsupported configure value list length: {}", value_list.len()))),
        };
        unsafe { xcb_configure_window(self.conn, window, mask as u16, value_list.as_ptr()) };
        self.flush()
    }
    fn change_window_attributes(&self, window: u32, value_list: &[u32]) -> Result<(), Box<dyn std::error::Error>> {
        if value_list.is_empty() {
            return self.flush();
        }
        let mask = value_list[0];
        let values = &value_list[1..];
        unsafe { xcb_change_window_attributes(self.conn, window, mask, values.as_ptr()) };
        self.flush()
    }
    fn reparent_window(&self, child: u32, parent: u32, pos: Point) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_reparent_window(self.conn, child, parent, pos.x as i16, pos.y as i16) };
        self.flush()
    }
    fn change_save_set(&self, window: u32, insert: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mode = if insert { XCB_SET_MODE_INSERT } else { XCB_SET_MODE_DELETE };
        unsafe { xcb_change_save_set(self.conn, mode, window) };
        self.flush()
    }
    fn clear_area(&self, exposures: bool, window: u32, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            xcb_clear_area(self.conn, exposures as u8, window, area.x as i16, area.y as i16, area.w as u16, area.h as u16)
        };
        self.flush()
    }
    fn set_selection_owner(&self, owner: u32, selection: u32, time: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_set_selection_owner(self.conn, owner, selection, time) };
        self.flush()
    }
    fn get_selection_owner(&self, selection: u32) -> Result<u32, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_get_selection_owner(self.conn, selection) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_selection_owner_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            let msg = if !e.is_null() {
                let code = unsafe { (*e).response_type };
                unsafe { libc::free(e as *mut libc::c_void) };
                format!("get_selection_owner failed (X error response_type={})", code)
            } else {
                "get_selection_owner reply null".to_string()
            };
            let conn_err = unsafe { xcb_connection_has_error(self.conn) };
            eprintln!("[xcb] get_selection_owner({}): {} conn_err={}", selection, msg, conn_err);
            return Err(err(msg));
        }
        let owner = unsafe { (*r).owner };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(owner)
    }
    fn generate_id(&self) -> Result<u32, Box<dyn std::error::Error>> {
        Ok(self.alloc_id())
    }
    fn query_tree(&self, window: u32) -> Result<QueryTreeResult, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_query_tree(self.conn, window) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_query_tree_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("query_tree failed"));
        }
        let root = unsafe { (*r).root };
        let parent = unsafe { (*r).parent };
        let children_len = unsafe { (*r).children_len } as usize;
        let data_ptr = unsafe { (r as *const u8).add(std::mem::size_of::<xcb_query_tree_reply_t>()) };
        let children = unsafe { std::slice::from_raw_parts(data_ptr as *const u32, children_len).to_vec() };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(QueryTreeResult { root, parent, children })
    }
    fn get_window_attributes(&self, window: u32) -> Result<WindowAttributes, Box<dyn std::error::Error>> {
        let cookie = unsafe { xcb_get_window_attributes(self.conn, window) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_window_attributes_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("get_window_attributes failed"));
        }
        let map_state = map_state(unsafe { (*r).map_state });
        let override_redirect = unsafe { (*r).override_redirect != 0 };
        let depth = self.depth;
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok(WindowAttributes { override_redirect, map_state, depth })
    }
    fn query_monitors(&self) -> Result<Vec<MonitorInfo>, Box<dyn std::error::Error>> {
        let s = self.screen();
        Ok(vec![MonitorInfo {
            x: 0,
            y: 0,
            width: s.width_in_pixels,
            height: s.height_in_pixels,
        }])
    }
    fn restack_windows(&self, windows: &[u32]) -> Result<(), Box<dyn std::error::Error>> {
        for i in 1..windows.len() {
            let vals = [windows[i - 1], XCB_STACK_MODE_ABOVE];
            unsafe {
                xcb_configure_window(self.conn, windows[i], XCB_CONFIG_WINDOW_SIBLING | XCB_CONFIG_WINDOW_STACK_MODE, vals.as_ptr())
            };
        }
        self.flush()
    }
    fn create_render_picture(&self, _pixmap: u32, _depth: u8) -> Result<u32, Box<dyn std::error::Error>> {
        Ok(self.alloc_id())
    }
    fn free_render_picture(&self, _picture: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn render_format_for_depth(&self, _depth: u8) -> Option<u32> {
        None
    }
    fn set_window_opacity(&self, window: u32, opacity: f32, opacity_atom: u32) -> Result<(), Box<dyn std::error::Error>> {
        let a = (opacity.max(0.0).min(1.0) * std::u32::MAX as f32) as u32;
        self.change_property32(PropMode::Replace, window, opacity_atom, self.intern_atom("CARDINAL")?, &[a])
    }
    fn grab_root_window(&self) -> Result<(u16, u16, Vec<u8>), Box<dyn std::error::Error>> {
        let w = self.screen().width_in_pixels;
        let h = self.screen().height_in_pixels;
        let cookie = unsafe { xcb_get_image(self.conn, XCB_IMAGE_FORMAT_Z_PIXMAP, self.root.read_id(), 0, 0, w, h, !0) };
        let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
        let r = unsafe { xcb_get_image_reply(self.conn, cookie, &mut e) };
        if r.is_null() {
            return Err(err("grab_root_window failed"));
        }
        let bytes = unsafe { (*r).length as usize * 4 };
        let data_ptr = unsafe { (r as *const u8).add(std::mem::size_of::<xcb_get_image_reply_t>()) };
        let data = unsafe { std::slice::from_raw_parts(data_ptr, bytes).to_vec() };
        unsafe { libc::free(r as *mut libc::c_void) };
        Ok((w, h, data))
    }
    fn send_selection_notify(&self, _r: u32, _s: u32, _t: u32, _p: u32, _time: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn set_win_gravity(&self, window: u32, gravity: u32) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [gravity];
        unsafe { xcb_change_window_attributes(self.conn, window, XCB_CW_WIN_GRAVITY, vals.as_ptr()) };
        self.flush()
    }
    fn define_cursor(&self, window: u32, cursor: u32) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [cursor];
        unsafe { xcb_change_window_attributes(self.conn, window, XCB_CW_CURSOR, vals.as_ptr()) };
        self.flush()
    }
    fn undefine_cursor(&self, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        let vals = [XCB_CURSOR_NONE];
        unsafe { xcb_change_window_attributes(self.conn, window, XCB_CW_CURSOR, vals.as_ptr()) };
        self.flush()
    }
    fn free_cursor(&self, cursor: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { xcb_free_cursor(self.conn, cursor) };
        self.flush()
    }
    fn create_cursor_from_pixmap(&self, source: u32, mask: u32, fore: [u16; 3], back: [u16; 3], hotspot: Point) -> Result<u32, Box<dyn std::error::Error>> {
        let cid = self.alloc_id();
        unsafe {
            xcb_create_cursor(self.conn, cid, source, mask, fore[0], fore[1], fore[2], back[0], back[1], back[2], hotspot.x as u16, hotspot.y as u16)
        };
        Ok(cid)
    }
    fn create_font_cursor(&self, glyph: u32) -> Result<u32, Box<dyn std::error::Error>> {
        let cid = self.alloc_id();
        unsafe {
            xcb_create_glyph_cursor(
                self.conn,
                cid,
                self.cursor_font,
                self.cursor_font,
                glyph as u16,
                glyph as u16,
                0,
                0,
                0,
                0xFFFF,
                0xFFFF,
                0xFFFF,
            )
        };
        Ok(cid)
    }
    fn create_cursor_from_rgba(&self, _pixels: &[u8], _size: antibox_core::point::Dimension, _hotspot: Point, _foreground: [u8; 3], _background: [u8; 3]) -> Result<u32, Box<dyn std::error::Error>> {
        Err(err("create_cursor_from_rgba not implemented in xcb backend"))
    }
    fn create_named_cursor(&self, _name: &str) -> Result<u32, Box<dyn std::error::Error>> {
        Err(err("create_named_cursor not implemented in xcb backend"))
    }
}
