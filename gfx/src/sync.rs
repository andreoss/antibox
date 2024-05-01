use std::sync::{Mutex, MutexGuard};

#[allow(unsafe_code)]
pub mod atomic {
    use std::cell::UnsafeCell;
    use std::ops::Add;
    use std::sync::atomic::Ordering;
    use std::sync::{Mutex, Once};

    pub struct LazyMutex<T> {
        once: Once,
        initial: T,
        cell: UnsafeCell<Option<Mutex<T>>>,
    }

    unsafe impl<T: Send> Sync for LazyMutex<T> {}

    impl<T> LazyMutex<T> {
        pub const fn new(initial: T) -> Self {
            Self {
                once: Once::new(),
                initial,
                cell: UnsafeCell::new(None),
            }
        }
    }

    impl<T: Copy> LazyMutex<T> {
        fn cell(&self) -> &Mutex<T> {
            let initial = self.initial;
            let cell = &self.cell;
            self.once
                .call_once(move || unsafe { *cell.get() = Some(Mutex::new(initial)) });
            unsafe { (*self.cell.get()).as_ref().unwrap() }
        }
        pub fn load(&self, _o: Ordering) -> T {
            *self.cell().lock().unwrap()
        }
        pub fn store(&self, v: T, _o: Ordering) {
            *self.cell().lock().unwrap() = v;
        }
    }

    impl<T: Copy + Add<Output = T>> LazyMutex<T> {
        pub fn fetch_add(&self, v: T, _o: Ordering) -> T {
            let mut g = self.cell().lock().unwrap();
            let old = *g;
            *g = old + v;
            old
        }
    }

    impl<T: Copy + Default> Default for LazyMutex<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Copy + std::fmt::Debug> std::fmt::Debug for LazyMutex<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.load(Ordering::Relaxed).fmt(f)
        }
    }

    pub struct LazyLock<T> {
        once: Once,
        cell: UnsafeCell<Option<Mutex<T>>>,
    }

    unsafe impl<T: Send> Sync for LazyLock<T> {}

    impl<T> Default for LazyLock<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> LazyLock<T> {
        pub const fn new() -> Self {
            Self {
                once: Once::new(),
                cell: UnsafeCell::new(None),
            }
        }
    }

    impl<T: Default> LazyLock<T> {
        pub fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, T>> {
            let cell = &self.cell;
            self.once
                .call_once(move || unsafe { *cell.get() = Some(Mutex::new(T::default())) });
            unsafe { (*self.cell.get()).as_ref().unwrap().lock() }
        }
    }

    pub struct LazyRwLock<T> {
        once: Once,
        cell: UnsafeCell<Option<std::sync::RwLock<T>>>,
    }

    unsafe impl<T: Send + Sync> Sync for LazyRwLock<T> {}

    impl<T> Default for LazyRwLock<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> LazyRwLock<T> {
        pub const fn new() -> Self {
            Self {
                once: Once::new(),
                cell: UnsafeCell::new(None),
            }
        }
    }

    impl<T: Default> LazyRwLock<T> {
        fn get(&self) -> &std::sync::RwLock<T> {
            let cell = &self.cell;
            self.once
                .call_once(move || unsafe { *cell.get() = Some(std::sync::RwLock::new(T::default())) });
            unsafe { (*self.cell.get()).as_ref().unwrap() }
        }
        pub fn read(&self) -> std::sync::LockResult<std::sync::RwLockReadGuard<'_, T>> {
            self.get().read()
        }
        pub fn write(&self) -> std::sync::LockResult<std::sync::RwLockWriteGuard<'_, T>> {
            self.get().write()
        }
    }

    pub type AtomicU8 = LazyMutex<u8>;
    pub type AtomicU16 = LazyMutex<u16>;
    pub type AtomicU32 = LazyMutex<u32>;
    pub type AtomicU64 = LazyMutex<u64>;
    pub type AtomicI16 = LazyMutex<i16>;
    pub type AtomicI32 = LazyMutex<i32>;
}

pub trait LockRecover<T: ?Sized> {
    fn lock_recover(&self) -> MutexGuard<'_, T>;
}

impl<T: ?Sized> LockRecover<T> for Mutex<T> {
    fn lock_recover(&self) -> MutexGuard<'_, T> {
        self.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
#[path = "sync_tests.rs"]
mod tests;
