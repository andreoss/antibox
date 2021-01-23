#![allow(non_camel_case_types)]

use super::bindings::*;
use super::connection::XcbConnection;
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong, c_ushort, c_void};
use std::sync::{Mutex, Once};

const FT_LOAD_RENDER: i32 = 4;

#[repr(C)]
struct FT_Vector {
    x: c_long,
    y: c_long,
}

#[repr(C)]
struct FT_Bitmap {
    rows: c_uint,
    width: c_uint,
    pitch: c_int,
    buffer: *mut u8,
    num_grays: c_ushort,
    pixel_mode: u8,
    palette_mode: u8,
    palette: *mut c_void,
}

#[repr(C)]
struct FT_Generic {
    data: *mut c_void,
    finalizer: *mut c_void,
}

#[repr(C)]
struct FT_Glyph_Metrics {
    width: c_long,
    height: c_long,
    hori_bearing_x: c_long,
    hori_bearing_y: c_long,
    hori_advance: c_long,
    vert_bearing_x: c_long,
    vert_bearing_y: c_long,
    vert_advance: c_long,
}

#[repr(C)]
struct FT_GlyphSlotRec {
    library: *mut c_void,
    face: *mut c_void,
    next: *mut c_void,
    glyph_index: c_uint,
    generic: FT_Generic,
    metrics: FT_Glyph_Metrics,
    linear_hori_advance: c_long,
    linear_vert_advance: c_long,
    advance: FT_Vector,
    format: c_uint,
    bitmap: FT_Bitmap,
    bitmap_left: c_int,
    bitmap_top: c_int,
}

#[repr(C)]
struct FT_Size_Metrics {
    x_ppem: c_ushort,
    y_ppem: c_ushort,
    x_scale: c_long,
    y_scale: c_long,
    ascender: c_long,
    descender: c_long,
    height: c_long,
    max_advance: c_long,
}

#[repr(C)]
struct FT_SizeRec {
    face: *mut c_void,
    generic: FT_Generic,
    metrics: FT_Size_Metrics,
    internal: *mut c_void,
}

#[repr(C)]
struct FT_FaceRec {
    num_faces: c_long,
    face_index: c_long,
    face_flags: c_long,
    style_flags: c_long,
    num_glyphs: c_long,
    family_name: *mut c_char,
    style_name: *mut c_char,
    num_fixed_sizes: c_int,
    available_sizes: *mut c_void,
    num_charmaps: c_int,
    charmaps: *mut c_void,
    generic: FT_Generic,
    bbox: [c_long; 4],
    units_per_em: c_ushort,
    ascender: i16,
    descender: i16,
    height: i16,
    max_advance_width: i16,
    max_advance_height: i16,
    underline_position: i16,
    underline_thickness: i16,
    glyph: *mut FT_GlyphSlotRec,
    size: *mut FT_SizeRec,
    charmap: *mut c_void,
}

#[link(name = "freetype")]
extern "C" {
    fn FT_Init_FreeType(lib: *mut *mut c_void) -> c_int;
    fn FT_New_Face(
        lib: *mut c_void,
        path: *const c_char,
        index: c_long,
        face: *mut *mut FT_FaceRec,
    ) -> c_int;
    fn FT_Set_Pixel_Sizes(face: *mut FT_FaceRec, width: c_uint, height: c_uint) -> c_int;
    fn FT_Load_Char(face: *mut FT_FaceRec, code: c_ulong, flags: i32) -> c_int;
}

const FC_MATCH_PATTERN: c_int = 0;
const FC_RESULT_MATCH: c_int = 0;

#[link(name = "fontconfig")]
extern "C" {
    fn FcInit() -> c_int;
    fn FcNameParse(name: *const u8) -> *mut c_void;
    fn FcConfigSubstitute(config: *mut c_void, pattern: *mut c_void, kind: c_int) -> c_int;
    fn FcDefaultSubstitute(pattern: *mut c_void);
    fn FcFontMatch(config: *mut c_void, pattern: *mut c_void, result: *mut c_int) -> *mut c_void;
    fn FcPatternGetString(
        pattern: *mut c_void,
        object: *const c_char,
        n: c_int,
        s: *mut *mut u8,
    ) -> c_int;
    fn FcPatternGetDouble(
        pattern: *mut c_void,
        object: *const c_char,
        n: c_int,
        d: *mut f64,
    ) -> c_int;
    fn FcPatternGetInteger(
        pattern: *mut c_void,
        object: *const c_char,
        n: c_int,
        i: *mut c_int,
    ) -> c_int;
    fn FcPatternDestroy(pattern: *mut c_void);
}

struct FtLib {
    lib: *mut c_void,
    fc_ok: bool,
}

unsafe impl Send for FtLib {}

fn ft_lib() -> &'static Mutex<Option<FtLib>> {
    static INIT: Once = Once::new();
    static mut LIB: *const Mutex<Option<FtLib>> = std::ptr::null();
    INIT.call_once(|| {
        let mut lib: *mut c_void = std::ptr::null_mut();
        let ok = unsafe { FT_Init_FreeType(&mut lib) } == 0;
        let fc_ok = unsafe { FcInit() } != 0;
        let state = if ok { Some(FtLib { lib, fc_ok }) } else { None };
        unsafe { LIB = Box::into_raw(Box::new(Mutex::new(state))) };
    });
    unsafe { &*LIB }
}

pub fn match_pattern(pattern: &str) -> Option<(String, i32, u16)> {
    let guard = ft_lib().lock().ok()?;
    let lib = guard.as_ref()?;
    if !lib.fc_ok {
        return None;
    }
    let cpat = std::ffi::CString::new(pattern).ok()?;
    let pat = unsafe { FcNameParse(cpat.as_ptr() as *const u8) };
    if pat.is_null() {
        return None;
    }
    unsafe {
        FcConfigSubstitute(std::ptr::null_mut(), pat, FC_MATCH_PATTERN);
        FcDefaultSubstitute(pat);
    }
    let mut result: c_int = 0;
    let matched = unsafe { FcFontMatch(std::ptr::null_mut(), pat, &mut result) };
    unsafe { FcPatternDestroy(pat) };
    if matched.is_null() {
        return None;
    }
    let file_obj = b"file\0";
    let mut s: *mut u8 = std::ptr::null_mut();
    let got_file =
        unsafe { FcPatternGetString(matched, file_obj.as_ptr() as *const c_char, 0, &mut s) }
            == FC_RESULT_MATCH
            && !s.is_null();
    let file = if got_file {
        unsafe { std::ffi::CStr::from_ptr(s as *const c_char) }
            .to_string_lossy()
            .into_owned()
    } else {
        String::new()
    };
    let mut index: c_int = 0;
    let index_obj = b"index\0";
    unsafe { FcPatternGetInteger(matched, index_obj.as_ptr() as *const c_char, 0, &mut index) };
    let mut px: f64 = 0.0;
    let px_obj = b"pixelsize\0";
    unsafe { FcPatternGetDouble(matched, px_obj.as_ptr() as *const c_char, 0, &mut px) };
    unsafe { FcPatternDestroy(matched) };
    if file.is_empty() {
        return None;
    }
    let px = px.round().max(6.0).min(96.0) as u16;
    Some((file, index, px))
}

struct GlyphState {
    advance: u16,
    uploaded: bool,
}

pub struct FtFont {
    face: *mut FT_FaceRec,
    pub px: u16,
    ascent: i16,
    descent: i16,
    glyphset: antibox_core::sync::atomic::AtomicU32,
    glyphs: Mutex<std::collections::HashMap<u32, GlyphState>>,
}

unsafe impl Send for FtFont {}
unsafe impl Sync for FtFont {}

pub fn open(pattern: &str) -> Option<std::sync::Arc<FtFont>> {
    let (file, index, px) = match_pattern(pattern)?;
    let guard = ft_lib().lock().ok()?;
    let lib = guard.as_ref()?;
    let cpath = std::ffi::CString::new(file).ok()?;
    let mut face: *mut FT_FaceRec = std::ptr::null_mut();
    if unsafe { FT_New_Face(lib.lib, cpath.as_ptr(), index as c_long, &mut face) } != 0
        || face.is_null()
    {
        return None;
    }
    if unsafe { FT_Set_Pixel_Sizes(face, 0, px as c_uint) } != 0 {
        return None;
    }
    let size = unsafe { (*face).size };
    if size.is_null() {
        return None;
    }
    let m = unsafe { &(*size).metrics };
    let ascent = (m.ascender >> 6) as i16;
    let descent = (-(m.descender >> 6)) as i16;
    Some(std::sync::Arc::new(FtFont {
        face,
        px,
        ascent,
        descent,
        glyphset: antibox_core::sync::atomic::AtomicU32::new(0),
        glyphs: Mutex::new(std::collections::HashMap::new()),
    }))
}

impl FtFont {
    pub fn metrics(&self) -> (u16, u16, u16) {
        let a = self.ascent.max(0) as u16;
        let d = self.descent.max(0) as u16;
        (a, d, a.saturating_add(d))
    }

    pub fn text_width(&self, text: &str) -> u32 {
        let mut total = 0u32;
        let mut cache = match self.glyphs.lock() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        for ch in text.chars() {
            let code = ch as u32;
            if let Some(st) = cache.get(&code) {
                total += st.advance as u32;
                continue;
            }
            let advance = self.load_advance(code);
            cache.insert(
                code,
                GlyphState {
                    advance,
                    uploaded: false,
                },
            );
            total += advance as u32;
        }
        total
    }

    fn load_advance(&self, code: u32) -> u16 {
        let guard = match ft_lib().lock() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        if guard.is_none() {
            return 0;
        }
        if unsafe { FT_Load_Char(self.face, code as c_ulong, 0) } != 0 {
            return 0;
        }
        let slot = unsafe { (*self.face).glyph };
        if slot.is_null() {
            return 0;
        }
        (unsafe { (*slot).advance.x } >> 6).max(0) as u16
    }

    pub fn glyphset(&self) -> u32 {
        self.glyphset.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn ensure_uploaded(&self, conn: &XcbConnection, text: &str) {
        let a8 = conn.render_a8_format();
        if a8 == 0 {
            return;
        }
        let mut gs = self.glyphset();
        if gs == 0 {
            gs = unsafe { xcb_generate_id(conn.raw()) };
            unsafe { xcb_render_create_glyph_set(conn.raw(), gs, a8) };
            self.glyphset
                .store(gs, std::sync::atomic::Ordering::Relaxed);
        }
        let mut cache = match self.glyphs.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        for ch in text.chars() {
            let code = ch as u32;
            let done = cache.get(&code).map_or(false, |st| st.uploaded);
            if done {
                continue;
            }
            let advance = self.upload_glyph(conn, gs, code);
            cache.insert(
                code,
                GlyphState {
                    advance,
                    uploaded: true,
                },
            );
        }
    }

    fn upload_glyph(&self, conn: &XcbConnection, gs: u32, code: u32) -> u16 {
        let guard = match ft_lib().lock() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        if guard.is_none() {
            return 0;
        }
        if unsafe { FT_Load_Char(self.face, code as c_ulong, FT_LOAD_RENDER) } != 0 {
            return 0;
        }
        let slot = unsafe { (*self.face).glyph };
        if slot.is_null() {
            return 0;
        }
        let advance = (unsafe { (*slot).advance.x } >> 6).max(0) as u16;
        let bm = unsafe { &(*slot).bitmap };
        let w = bm.width as usize;
        let h = bm.rows as usize;
        let stride = (w + 3) & !3;
        let mut data = vec![0u8; stride * h];
        if !bm.buffer.is_null() && w > 0 {
            for row in 0..h {
                let src = unsafe { bm.buffer.offset(row as isize * bm.pitch as isize) };
                let src = unsafe { std::slice::from_raw_parts(src, w) };
                data[row * stride..row * stride + w].copy_from_slice(src);
            }
        }
        let info = xcb_render_glyphinfo_t {
            width: w as u16,
            height: h as u16,
            x: -(unsafe { (*slot).bitmap_left } as i16),
            y: unsafe { (*slot).bitmap_top } as i16,
            x_off: advance as i16,
            y_off: 0,
        };
        unsafe {
            xcb_render_add_glyphs(
                conn.raw(),
                gs,
                1,
                &code,
                &info,
                data.len() as u32,
                data.as_ptr(),
            )
        };
        advance
    }
}
