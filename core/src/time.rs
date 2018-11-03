use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Monotime(Instant);

impl Monotime {
    pub fn now() -> Self {
        Monotime(Instant::now())
    }

    pub fn checked_add(&self, d: Duration) -> Option<Self> {
        Some(Monotime(self.0 + d))
    }
}

impl std::ops::Sub for Monotime {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        self.0 - other.0
    }
}

#[cfg(test)]
#[path = "time_tests.rs"]
mod tests;
