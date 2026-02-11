use super::wl::{wl_list, wl_signal};
use std::os::raw::{c_int, c_void};

pub const DRM_FORMAT_ARGB8888: u32 = 0x3432_5241;

pub const WLR_BUFFER_DATA_PTR_ACCESS_READ: u32 = 1;

#[repr(C)]
pub struct wlr_buffer_impl {
    pub destroy: Option<unsafe extern "C" fn(*mut wlr_buffer)>,
    pub get_dmabuf: Option<unsafe extern "C" fn(*mut wlr_buffer, *mut c_void) -> bool>,
    pub get_shm: Option<unsafe extern "C" fn(*mut wlr_buffer, *mut c_void) -> bool>,
    pub begin_data_ptr_access: Option<
        unsafe extern "C" fn(*mut wlr_buffer, u32, *mut *mut c_void, *mut u32, *mut usize) -> bool,
    >,
    pub end_data_ptr_access: Option<unsafe extern "C" fn(*mut wlr_buffer)>,
}

#[repr(C)]
pub struct wlr_buffer {
    pub impl_: *const wlr_buffer_impl,
    pub width: c_int,
    pub height: c_int,
    pub dropped: bool,
    pub n_locks: usize,
    pub accessing_data_ptr: bool,
    pub events: wlr_buffer_events,
    pub addons: wl_list,
}

#[repr(C)]
pub struct wlr_buffer_events {
    pub destroy: wl_signal,
    pub release: wl_signal,
}

#[link(name = "wlroots-0.19")]
extern "C" {
    pub fn wlr_buffer_init(
        buffer: *mut wlr_buffer,
        impl_: *const wlr_buffer_impl,
        width: c_int,
        height: c_int,
    );
    pub fn wlr_buffer_finish(buffer: *mut wlr_buffer);
    pub fn wlr_buffer_drop(buffer: *mut wlr_buffer);
}

#[repr(C)]
pub struct PixBuffer {
    pub base: wlr_buffer,
    pub pixels: Vec<u32>,
    pub stride: usize,
}

unsafe extern "C" fn pix_destroy(buffer: *mut wlr_buffer) {
    let this = buffer.cast::<PixBuffer>();
    wlr_buffer_finish(buffer);
    drop(Box::from_raw(this));
}

unsafe extern "C" fn pix_begin(
    buffer: *mut wlr_buffer,
    _flags: u32,
    data: *mut *mut c_void,
    format: *mut u32,
    stride: *mut usize,
) -> bool {
    let this = &mut *buffer.cast::<PixBuffer>();
    *data = this.pixels.as_mut_ptr().cast::<c_void>();
    *format = DRM_FORMAT_ARGB8888;
    *stride = this.stride;
    true
}

unsafe extern "C" fn pix_end(_buffer: *mut wlr_buffer) {}

static PIX_IMPL: wlr_buffer_impl = wlr_buffer_impl {
    destroy: Some(pix_destroy),
    get_dmabuf: None,
    get_shm: None,
    begin_data_ptr_access: Some(pix_begin),
    end_data_ptr_access: Some(pix_end),
};

impl PixBuffer {
    pub fn create(width: u16, height: u16, pixels: Vec<u32>) -> *mut wlr_buffer {
        let stride = width as usize * 4;
        let boxed = Box::new(Self {
            base: unsafe { std::mem::zeroed() },
            pixels,
            stride,
        });
        let raw = Box::into_raw(boxed);
        unsafe {
            wlr_buffer_init(
                std::ptr::addr_of_mut!((*raw).base),
                &PIX_IMPL,
                c_int::from(width),
                c_int::from(height),
            );
            std::ptr::addr_of_mut!((*raw).base)
        }
    }
}
