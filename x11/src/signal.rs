use antibox_core::libc;
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicIsize, Ordering};

static SIGNAL_WRITE_FD: AtomicIsize = AtomicIsize::new(-1);

extern "C" fn sigchld_handler(_sig: libc::c_int) {
    let fd = SIGNAL_WRITE_FD.load(Ordering::Relaxed);
    if fd >= 0 {
        let buf: [u8; 1] = [1];
        let _ = unsafe { libc::write(fd as RawFd, buf.as_ptr().cast(), 1) };
    }
}

pub struct SignalHandler {
    read_fd: RawFd,
    write_fd: RawFd,
}

impl SignalHandler {
    pub fn new() -> std::io::Result<Self> {
        let mut fds = [-1i32; 2];
        let ret = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) };
        if ret != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let read_fd = fds[0];
        let write_fd = fds[1];
        SIGNAL_WRITE_FD.store(write_fd as isize, Ordering::Relaxed);
        let mut sa: libc::sigaction = unsafe { std::mem::zeroed() };
        sa.sa_sigaction = sigchld_handler as *const () as usize;
        sa.sa_flags = libc::SA_RESTART | libc::SA_NOCLDSTOP;
        let ret = unsafe { libc::sigaction(libc::SIGCHLD, std::ptr::addr_of!(sa), std::ptr::null_mut()) };
        if ret != 0 {
            unsafe {
                let _ = libc::close(read_fd);
                let _ = libc::close(write_fd);
            }
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self { read_fd, write_fd })
    }

    pub const fn read_fd(&self) -> RawFd {
        self.read_fd
    }

    pub fn was_signalled(&self) -> bool {
        let mut pfd = libc::pollfd {
            fd: self.read_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ret = unsafe { libc::poll(std::ptr::addr_of_mut!(pfd), 1, 0) };
        ret > 0 && (pfd.revents & libc::POLLIN) != 0
    }

    pub fn clear(&self) {
        let mut buf = [0u8; 64];
        loop {
            let ret = unsafe {
                libc::read(
                    self.read_fd,
                    buf.as_mut_ptr().cast::<libc::c_void>(),
                    buf.len(),
                )
            };
            if ret <= 0 {
                break;
            }
        }
    }

    pub fn reap_children() {
        loop {
            let mut status: i32 = 0;
            let ret = unsafe { libc::waitpid(-1, std::ptr::addr_of_mut!(status), libc::WNOHANG) };
            if ret <= 0 {
                break;
            }
        }
    }
}

impl Drop for SignalHandler {
    fn drop(&mut self) {
        let _ = SIGNAL_WRITE_FD.compare_exchange(
            self.write_fd as isize,
            -1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
        unsafe {
            let _ = libc::close(self.read_fd);
            let _ = libc::close(self.write_fd);
        }
    }
}

#[cfg(test)]
#[path = "signal_tests.rs"]
mod tests;
