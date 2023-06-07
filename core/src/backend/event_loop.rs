 use antibox_gfx::error::Result;
use crate::backend::BackendEvent;
use crate::backend::TimerCallback;
use std::os::unix::io::RawFd;
use std::sync::Arc;
use std::time::Duration;

pub trait EventLoopTrait {
    fn backend(&self) -> &Arc<dyn crate::backend::DisplayBackend>;
    fn add_fd(&mut self, fd: RawFd, callback: Box<dyn FnMut() + Send>);
    fn remove_fd(&mut self, fd: RawFd);
    fn add_timer(&mut self, delay: Duration, callback: TimerCallback) -> u64;
    fn add_periodic_timer(&mut self, interval: Duration, callback: TimerCallback) -> u64 {
        self.add_timer(interval, callback)
    }
    fn time_to_next_timer(&self) -> Option<Duration>;
    fn wait_for_one_event(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<BackendEvent>>;
    fn fire_timers(&mut self);
    fn process_pending(&mut self) -> Result<Vec<BackendEvent>>;
}
