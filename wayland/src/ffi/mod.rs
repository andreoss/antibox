#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]
pub mod buffer;
pub mod wl;
pub mod wlr;
pub mod xkb;
pub mod xwayland;

use std::ffi::CStr;
use std::os::raw::c_char;

pub unsafe fn cstr_to_string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p)
        .to_str()
        .ok()
        .map(str::to_string)
}
