#![allow(unsafe_code)]
use crate::libc;
use std::sync::Once;
use std::time::{Duration, Instant};

static LOCALE: Once = Once::new();

pub fn use_locale() {
    LOCALE.call_once(|| unsafe {
        libc::setlocale(libc::LC_TIME, "\0".as_ptr().cast::<libc::c_char>());
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Monotime(Instant);

impl Monotime {
    pub fn now() -> Self {
        Self(Instant::now())
    }

    pub fn checked_add(&self, d: Duration) -> Option<Self> {
        Some(Self(self.0 + d))
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
