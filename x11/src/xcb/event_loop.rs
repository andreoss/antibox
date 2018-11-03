
use antibox_core::libc;
use antibox_core::backend::{BackendEvent, DisplayBackend, EventLoopTrait, TimerCallback};
use std::os::unix::io::RawFd;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Timer {
    due: Instant,
    interval: Option<Duration>,
    callback: TimerCallback,
}

pub struct XcbEventLoop {
    backend: Arc<dyn DisplayBackend>,
    extra_fds: Vec<(RawFd, Box<dyn FnMut() + Send>)>,
    timers: Vec<Timer>,
    next_timer_id: u64,
    _signal: Option<crate::signal::SignalHandler>,
}

impl XcbEventLoop {
    pub fn new(backend: Arc<dyn DisplayBackend>) -> Self {
        let mut el = Self {
            backend,
            extra_fds: Vec::new(),
            timers: Vec::new(),
            next_timer_id: 1,
            _signal: None,
        };
        if let Ok(sig) = crate::signal::SignalHandler::new() {
            let fd = sig.read_fd();
            el.add_fd(
                fd,
                Box::new(move || {
                    let mut buf = [0u8; 64];
                    loop {
                        let r = unsafe {
                            antibox_core::libc::read(
                                fd,
                                buf.as_mut_ptr() as *mut antibox_core::libc::c_void,
                                buf.len(),
                            )
                        };
                        if r <= 0 {
                            break;
                        }
                    }
                    crate::signal::SignalHandler::reap_children();
                }),
            );
            el._signal = Some(sig);
        }
        el
    }
}

impl EventLoopTrait for XcbEventLoop {
    fn backend(&self) -> &Arc<dyn DisplayBackend> {
        &self.backend
    }

    fn add_fd(&mut self, fd: RawFd, callback: Box<dyn FnMut() + Send>) {
        self.extra_fds.push((fd, callback));
    }

    fn remove_fd(&mut self, fd: RawFd) {
        self.extra_fds.retain(|(f, _)| *f != fd);
    }

    fn add_timer(&mut self, delay: Duration, callback: TimerCallback) -> u64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        self.timers.push(Timer {
            due: Instant::now() + delay,
            interval: None,
            callback,
        });
        id
    }

    fn add_periodic_timer(&mut self, interval: Duration, callback: TimerCallback) -> u64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        self.timers.push(Timer {
            due: Instant::now() + interval,
            interval: Some(interval),
            callback,
        });
        id
    }

    fn time_to_next_timer(&self) -> Option<Duration> {
        self.timers
            .iter()
            .map(|t| {
                let now = Instant::now();
                if t.due > now {
                    t.due.duration_since(now)
                } else {
                    Duration::from_secs(0)
                }
            })
            .min()
    }

    fn wait_for_one_event(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<BackendEvent>, Box<dyn std::error::Error>> {
        let fd = self.backend.fd();
        let mut pollfds = vec![libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        }];
        for (efd, _) in &self.extra_fds {
            pollfds.push(libc::pollfd {
                fd: *efd,
                events: libc::POLLIN,
                revents: 0,
            });
        }
        let timeout_ms = (timeout.as_secs().saturating_mul(1000)
            + timeout.subsec_millis() as u64)
            .min(std::i32::MAX as u64) as i32;
        let rc = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as u64, timeout_ms) };
        if rc < 0 {
            let e = std::io::Error::last_os_error();
            if e.kind() == std::io::ErrorKind::Interrupted {
                return Ok(None);
            }
            return Err(format!("poll failed: {}", e).into());
        }
        for (i, pf) in pollfds.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if pf.revents & (libc::POLLIN | libc::POLLHUP) != 0 {
                let cb = &mut self.extra_fds[i - 1].1;
                cb();
            }
        }        if pollfds[0].revents & libc::POLLIN != 0 {
            self.backend.poll_for_event()
        } else {
            Ok(None)
        }
    }

    fn fire_timers(&mut self) {
        let now = Instant::now();
        let mut fired: Vec<Timer> = Vec::new();
        let mut i = 0;
        while i < self.timers.len() {
            if self.timers[i].due <= now {
                fired.push(self.timers.swap_remove(i));
            } else {
                i += 1;
            }
        }
        for mut timer in fired {
            (timer.callback)();
            if let Some(interval) = timer.interval {
                timer.due = now + interval;
                self.timers.push(timer);
            }
        }
    }

    fn process_pending(&mut self) -> Result<Vec<BackendEvent>, Box<dyn std::error::Error>> {
        let mut out = Vec::new();
        while let Some(e) = self.backend.poll_for_event()? {
            out.push(e);
        }
        Ok(out)
    }
}
