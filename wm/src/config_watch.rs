use antibox_core::libc;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};

static CHANGED: AtomicBool = AtomicBool::new(false);

const IN_MODIFY: u32 = 0x2;
const IN_CLOSE_WRITE: u32 = 0x8;
const IN_MOVED_FROM: u32 = 0x40;
const IN_MOVED_TO: u32 = 0x80;
const IN_CREATE: u32 = 0x100;
const IN_DELETE: u32 = 0x200;

const WATCH_MASK: u32 =
    IN_MODIFY | IN_CLOSE_WRITE | IN_MOVED_FROM | IN_MOVED_TO | IN_CREATE | IN_DELETE;

pub const CONFIG_FILE: &str = "config.ini";

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

pub fn take_changed() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}

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

#[cfg(test)]
#[path = "config_watch_tests.rs"]
mod tests;
