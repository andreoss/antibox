#![allow(non_camel_case_types)]
pub use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

pub type time_t = i64;
pub type sighandler_t = usize;

#[cfg(target_os = "linux")]
pub type nfds_t = u64;
#[cfg(not(target_os = "linux"))]
pub type nfds_t = std::os::raw::c_uint;

pub const POLLIN: i16 = 0x001;
pub const POLLHUP: i16 = 0x010;
pub const WNOHANG: c_int = 1;
pub const O_RDONLY: c_int = 0;

#[cfg(target_os = "linux")]
pub const SIGCHLD: c_int = 17;
#[cfg(target_os = "linux")]
pub const SA_NOCLDSTOP: c_int = 0x0000_0001;
#[cfg(target_os = "linux")]
pub const SA_RESTART: c_int = 0x1000_0000;
#[cfg(target_os = "linux")]
pub const O_NONBLOCK: c_int = 0o4000;
#[cfg(target_os = "linux")]
pub const O_CLOEXEC: c_int = 0o2_000_000;

#[cfg(not(target_os = "linux"))]
pub const SIGCHLD: c_int = 20;
#[cfg(not(target_os = "linux"))]
pub const SA_NOCLDSTOP: c_int = 0x0008;
#[cfg(not(target_os = "linux"))]
pub const SA_RESTART: c_int = 0x0002;
#[cfg(not(target_os = "linux"))]
pub const O_NONBLOCK: c_int = 0x4;
#[cfg(not(target_os = "linux"))]
pub const O_CLOEXEC: c_int = 0x10000;

#[cfg(not(target_os = "linux"))]
pub const EVFILT_VNODE: i16 = -4;
#[cfg(not(target_os = "linux"))]
pub const EV_ADD: u16 = 0x0001;
#[cfg(not(target_os = "linux"))]
pub const EV_CLEAR: u16 = 0x0020;
#[cfg(not(target_os = "linux"))]
pub const NOTE_DELETE: u32 = 0x0001;
#[cfg(not(target_os = "linux"))]
pub const NOTE_WRITE: u32 = 0x0002;
#[cfg(not(target_os = "linux"))]
pub const NOTE_EXTEND: u32 = 0x0004;
#[cfg(not(target_os = "linux"))]
pub const NOTE_RENAME: u32 = 0x0020;

#[repr(C)]
pub struct pollfd {
    pub fd: c_int,
    pub events: i16,
    pub revents: i16,
}

#[cfg(target_os = "linux")]
#[repr(C)]
pub struct sigaction {
    pub sa_sigaction: sighandler_t,
    pub sa_mask: [u64; 16],
    pub sa_flags: c_int,
    pub sa_restorer: usize,
}

#[cfg(not(target_os = "linux"))]
#[repr(C)]
pub struct sigaction {
    pub sa_sigaction: sighandler_t,
    pub sa_mask: u32,
    pub sa_flags: c_int,
}

#[cfg(not(target_os = "linux"))]
#[repr(C)]
pub struct kevent {
    pub ident: usize,
    pub filter: i16,
    pub flags: u16,
    pub fflags: u32,
    pub data: i64,
    pub udata: *mut c_void,
}

#[cfg(not(target_os = "linux"))]
#[repr(C)]
pub struct ifaddrs {
    pub ifa_next: *mut ifaddrs,
    pub ifa_name: *mut c_char,
    pub ifa_flags: c_uint,
    pub ifa_addr: *mut c_void,
    pub ifa_netmask: *mut c_void,
    pub ifa_dstaddr: *mut c_void,
    pub ifa_data: *mut c_void,
}

#[repr(C)]
pub struct timespec {
    pub tv_sec: time_t,
    pub tv_nsec: c_long,
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
    pub fn open(path: *const c_char, flags: c_int) -> c_int;
}

#[cfg(target_os = "linux")]
extern "C" {
    pub fn inotify_init1(flags: c_int) -> c_int;
    pub fn inotify_add_watch(fd: c_int, pathname: *const c_char, mask: u32) -> c_int;
}

#[cfg(not(target_os = "linux"))]
extern "C" {
    pub fn kqueue() -> c_int;
    pub fn kevent(
        kq: c_int,
        changelist: *const kevent,
        nchanges: c_int,
        eventlist: *mut kevent,
        nevents: c_int,
        timeout: *const timespec,
    ) -> c_int;
    pub fn sysctl(
        name: *const c_int,
        namelen: c_uint,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *const c_void,
        newlen: usize,
    ) -> c_int;
    pub fn getifaddrs(ifap: *mut *mut ifaddrs) -> c_int;
    pub fn freeifaddrs(ifa: *mut ifaddrs);
    pub fn ioctl(fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int;
}
