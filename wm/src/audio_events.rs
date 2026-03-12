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
    if !crate::audio::pulse_available() {
        return None;
    }
    let child = Command::new("pactl")
        .arg("subscribe")
        .env("LC_ALL", "C")
        .env("LANGUAGE", "C")
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

fn line_matters(line: &str) -> bool {
    match line.split(" on ").nth(1) {
        Some(rest) => {
            let facility = rest.split_whitespace().next().unwrap_or("");
            matches!(facility, "sink" | "source" | "source-output" | "server")
        }
        None => false,
    }
}

pub fn drain(fd: RawFd) {
    let mut buf = [0u8; 4096];
    let n = unsafe { libc::read(fd, buf.as_mut_ptr().cast::<libc::c_void>(), buf.len()) };
    match n.cmp(&0) {
        std::cmp::Ordering::Greater => {
            let text = String::from_utf8_lossy(&buf[..n as usize]);
            if text.lines().any(line_matters) {
                CHANGED.store(true, Ordering::Relaxed);
            }
        }
        std::cmp::Ordering::Equal => ALIVE.store(false, Ordering::Relaxed),
        std::cmp::Ordering::Less => {}
    }
}

pub fn active() -> bool {
    ALIVE.load(Ordering::Relaxed)
}

pub fn take_changed() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}

#[cfg(test)]
#[path = "audio_events_tests.rs"]
mod tests;
