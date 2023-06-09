#![allow(unsafe_code)]
use antibox_core::libc;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};

static CHANGED: AtomicBool = AtomicBool::new(false);

pub const CONFIG_FILE: &str = "config.toml";

pub fn take_changed() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}

#[cfg(target_os = "linux")]
const IN_MODIFY: u32 = 0x2;
#[cfg(target_os = "linux")]
const IN_CLOSE_WRITE: u32 = 0x8;
#[cfg(target_os = "linux")]
const IN_MOVED_FROM: u32 = 0x40;
#[cfg(target_os = "linux")]
const IN_MOVED_TO: u32 = 0x80;
#[cfg(target_os = "linux")]
const IN_CREATE: u32 = 0x100;
#[cfg(target_os = "linux")]
const IN_DELETE: u32 = 0x200;

#[cfg(target_os = "linux")]
const WATCH_MASK: u32 =
    IN_MODIFY | IN_CLOSE_WRITE | IN_MOVED_FROM | IN_MOVED_TO | IN_CREATE | IN_DELETE;

#[cfg(target_os = "linux")]
pub fn init_watch() -> Option<RawFd> {
    let fd = unsafe { libc::inotify_init1(libc::O_NONBLOCK | libc::O_CLOEXEC) };
    if fd < 0 {
        return None;
    }
    let mut any = false;
    for d in crate::wmconfig::Config::search_dirs() {
        if !d.is_dir() {
            continue;
        }
        let path = match std::ffi::CString::new(d.to_string_lossy().into_owned()) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if unsafe { libc::inotify_add_watch(fd, path.as_ptr(), WATCH_MASK) } >= 0 {
            any = true;
        }
    }
    if any {
        Some(fd)
    } else {
        unsafe { libc::close(fd) };
        None
    }
}

#[cfg(target_os = "linux")]
pub fn drain(fd: RawFd) {
    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n <= 0 {
            break;
        }
        if event_names(&buf[..n as usize])
            .iter()
            .any(|name| name == CONFIG_FILE)
        {
            CHANGED.store(true, Ordering::Relaxed);
        }
    }
}

#[cfg(target_os = "linux")]
fn event_names(buf: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut pos = 0;
    while pos + 16 <= buf.len() {
        let len = u32::from_ne_bytes([buf[pos + 12], buf[pos + 13], buf[pos + 14], buf[pos + 15]])
            as usize;
        let start = pos + 16;
        if start + len > buf.len() {
            break;
        }
        let raw = &buf[start..start + len];
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        if end > 0 {
            names.push(String::from_utf8_lossy(&raw[..end]).into_owned());
        }
        pos = start + len;
    }
    names
}

#[cfg(not(target_os = "linux"))]
fn file_fds() -> &'static std::sync::Mutex<Vec<RawFd>> {
    use std::cell::UnsafeCell;
    use std::sync::{Mutex, Once};
    struct Cell(UnsafeCell<Option<Mutex<Vec<RawFd>>>>);
    unsafe impl Sync for Cell {}
    static ONCE: Once = Once::new();
    static CELL: Cell = Cell(UnsafeCell::new(None));
    ONCE.call_once(|| unsafe { *CELL.0.get() = Some(Mutex::new(Vec::new())) });
    unsafe { (*CELL.0.get()).as_ref().unwrap() }
}

#[cfg(not(target_os = "linux"))]
fn add_vnode_watch(kq: RawFd, path: &std::path::Path) -> Option<RawFd> {
    let cpath = std::ffi::CString::new(path.to_string_lossy().into_owned()).ok()?;
    let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
    if fd < 0 {
        return None;
    }
    let change = libc::kevent {
        ident: fd as usize,
        filter: libc::EVFILT_VNODE,
        flags: libc::EV_ADD | libc::EV_CLEAR,
        fflags: libc::NOTE_WRITE | libc::NOTE_EXTEND | libc::NOTE_DELETE | libc::NOTE_RENAME,
        data: 0,
        udata: std::ptr::null_mut(),
    };
    let rc = unsafe { libc::kevent(kq, &change, 1, std::ptr::null_mut(), 0, std::ptr::null()) };
    if rc < 0 {
        unsafe { libc::close(fd) };
        return None;
    }
    Some(fd)
}

#[cfg(not(target_os = "linux"))]
fn rewatch_files(kq: RawFd) {
    let mut fds = match file_fds().lock() {
        Ok(v) => v,
        Err(e) => e.into_inner(),
    };
    for fd in fds.drain(..) {
        unsafe { libc::close(fd) };
    }
    for d in crate::wmconfig::Config::search_dirs() {
        let file = d.join(CONFIG_FILE);
        if file.is_file() {
            if let Some(fd) = add_vnode_watch(kq, &file) {
                fds.push(fd);
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn init_watch() -> Option<RawFd> {
    let kq = unsafe { libc::kqueue() };
    if kq < 0 {
        return None;
    }
    let mut any = false;
    for d in crate::wmconfig::Config::search_dirs() {
        if d.is_dir() && add_vnode_watch(kq, &d).is_some() {
            any = true;
        }
    }
    rewatch_files(kq);
    if any {
        Some(kq)
    } else {
        unsafe { libc::close(kq) };
        None
    }
}

#[cfg(not(target_os = "linux"))]
pub fn drain(fd: RawFd) {
    let zero = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let mut events: [libc::kevent; 8] = unsafe { std::mem::zeroed() };
    let mut any = false;
    loop {
        let n = unsafe { libc::kevent(fd, std::ptr::null(), 0, events.as_mut_ptr(), 8, &zero) };
        if n <= 0 {
            break;
        }
        any = true;
        if n < 8 {
            break;
        }
    }
    if any {
        CHANGED.store(true, Ordering::Relaxed);
        rewatch_files(fd);
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "config_watch_tests.rs"]
mod tests;
