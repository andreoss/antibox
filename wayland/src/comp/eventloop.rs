use antibox_core::backend::{BackendEvent, DisplayBackend, EventLoopTrait, TimerCallback};
use antibox_core::error::Result;
use antibox_core::time::Monotime;
use std::collections::BinaryHeap;
use std::os::unix::io::RawFd;
use std::sync::Arc;
use std::time::Duration;

use super::state::Server;
use crate::ffi::wl::{wl_display_flush_clients, wl_event_loop_dispatch};

#[derive(PartialEq, Eq)]
struct TimerEntry {
    fire_at: Monotime,
    id: u64,
    interval: Option<Duration>,
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.fire_at.cmp(&self.fire_at)
    }
}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct WaylandEventLoop {
    server: Box<Server>,
    backend: Arc<dyn DisplayBackend>,
    timers: BinaryHeap<TimerEntry>,
    next_timer_id: u64,
    timer_callbacks: Vec<(u64, TimerCallback)>,
}

impl WaylandEventLoop {
    pub(crate) fn new(mut server: Box<Server>, backend: Arc<dyn DisplayBackend>) -> Self {
        server.retarget_hooks();
        Self {
            server,
            backend,
            timers: BinaryHeap::new(),
            next_timer_id: 0,
            timer_callbacks: Vec::new(),
        }
    }

    fn pump(&mut self, timeout: Duration) -> Result<()> {
        unsafe {
            self.server.apply_intents();
            if self.server.shared.take_needs_redraw() {
                self.server.sync_all_decorations();
            }
            let ms = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
            wl_event_loop_dispatch(self.server.event_loop, ms);
            wl_display_flush_clients(self.server.display);
        }
        Ok(())
    }
}

impl EventLoopTrait for WaylandEventLoop {
    fn backend(&self) -> &Arc<dyn DisplayBackend> {
        &self.backend
    }

    fn add_fd(&mut self, _fd: RawFd, _callback: Box<dyn FnMut() + Send>) {}

    fn remove_fd(&mut self, _fd: RawFd) {}

    fn add_timer(&mut self, delay: Duration, callback: TimerCallback) -> u64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        self.timers.push(TimerEntry {
            fire_at: Monotime::now().checked_add(delay).unwrap(),
            id,
            interval: None,
        });
        self.timer_callbacks.push((id, callback));
        id
    }

    fn add_periodic_timer(&mut self, interval: Duration, callback: TimerCallback) -> u64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        self.timers.push(TimerEntry {
            fire_at: Monotime::now().checked_add(interval).unwrap(),
            id,
            interval: Some(interval),
        });
        self.timer_callbacks.push((id, callback));
        id
    }

    fn time_to_next_timer(&self) -> Option<Duration> {
        self.timers.peek().map(|e| {
            let now = Monotime::now();
            if e.fire_at > now {
                e.fire_at - now
            } else {
                Duration::ZERO
            }
        })
    }

    fn wait_for_one_event(&mut self, timeout: Duration) -> Result<Option<BackendEvent>> {
        self.pump(timeout)?;
        Ok(self.server.shared.lock().events.pop())
    }

    fn fire_timers(&mut self) {
        let now = Monotime::now();
        while let Some(entry) = self.timers.peek() {
            if entry.fire_at > now {
                break;
            }
            let entry = self.timers.pop().unwrap();
            if let Some(pos) = self
                .timer_callbacks
                .iter()
                .position(|(id, _)| *id == entry.id)
            {
                let (_, mut cb) = self.timer_callbacks.remove(pos);
                cb();
                if let Some(interval) = entry.interval {
                    self.timer_callbacks.push((entry.id, cb));
                    self.timers.push(TimerEntry {
                        fire_at: now.checked_add(interval).unwrap(),
                        id: entry.id,
                        interval: Some(interval),
                    });
                }
            }
        }
    }

    fn process_pending(&mut self) -> Result<Vec<BackendEvent>> {
        self.pump(Duration::ZERO)?;
        let mut out = Vec::new();
        let mut s = self.server.shared.lock();
        while let Some(e) = s.events.pop() {
            out.push(e);
        }
        Ok(out)
    }
}
