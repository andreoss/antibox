#![allow(unsafe_code)]
use antibox_core::libc;
use antibox_core::sync::atomic::LazyRwLock;
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

static CHANGED: AtomicBool = AtomicBool::new(false);
static ALIVE: AtomicBool = AtomicBool::new(false);
static CHILD: LazyRwLock<Option<Child>> = LazyRwLock::new();

pub fn init() -> Option<RawFd> {
    let child = Command::new("pactl")
        .arg("subscribe")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let fd = child.stdout.as_ref()?.as_raw_fd();
    if let Ok(mut g) = CHILD.write() {
        *g = Some(child);
    }
    ALIVE.store(true, Ordering::Relaxed);
    Some(fd)
}

pub fn drain(fd: RawFd) {
    let mut buf = [0u8; 1024];
    let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    if n > 0 {
        CHANGED.store(true, Ordering::Relaxed);
    } else if n == 0 {
        ALIVE.store(false, Ordering::Relaxed);
    }
}

pub fn active() -> bool {
    ALIVE.load(Ordering::Relaxed)
}

pub fn take_changed() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}
