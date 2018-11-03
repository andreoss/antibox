use crate::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static ROUND_TRIPS: AtomicU64 = AtomicU64::new(0);


pub fn round_trips() -> u64 {
    ROUND_TRIPS.load(Ordering::Relaxed)
}
