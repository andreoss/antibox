#![allow(non_camel_case_types)]
pub use std::os::raw::{c_char, c_int, c_long, c_void};

pub type time_t = i64;
pub type nfds_t = u64;
pub type sighandler_t = usize;

pub const POLLIN: i16 = 0x001;
pub const POLLHUP: i16 = 0x010;
pub const WNOHANG: c_int = 1;
pub const SIGCHLD: c_int = 17;
pub const SA_NOCLDSTOP: c_int = 0x0000_0001;
pub const SA_RESTART: c_int = 0x1000_0000;
pub const O_NONBLOCK: c_int = 0o4000;
pub const O_CLOEXEC: c_int = 0o2_000_000;

#[repr(C)]
pub struct pollfd {
    pub fd: c_int,
    pub events: i16,
    pub revents: i16,
}

#[repr(C)]
pub struct sigaction {
    pub sa_sigaction: sighandler_t,
    pub sa_mask: [u64; 16],
    pub sa_flags: c_int,
    pub sa_restorer: usize,
}

#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

extern "C" {
    pub fn free(p: *mut c_void);
    pub fn close(fd: c_int) -> c_int;
    pub fn poll(fds: *mut pollfd, nfds: nfds_t, timeout: c_int) -> c_int;
    pub fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    pub fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    pub fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int;
    pub fn pipe2(fds: *mut c_int, flags: c_int) -> c_int;
    pub fn sigaction(sig: c_int, act: *const sigaction, oldact: *mut sigaction) -> c_int;
    pub fn strftime(s: *mut c_char, max: usize, format: *const c_char, tm: *const tm) -> usize;
    pub fn localtime_r(time: *const time_t, result: *mut tm) -> *mut tm;
    pub fn getloadavg(loadavg: *mut f64, nelem: c_int) -> c_int;
}
