pub mod backend;
pub mod canvas;

pub mod logevent;
pub mod metrics;
pub mod mock;
pub mod libc;
pub mod paths;

pub use antibox_gfx::{colour, keysyms, point, rect, scale, sync, xpm};
pub mod cursor;
pub mod time;

pub use self::backend::*;

#[macro_export]
macro_rules! does_match {
    ($expression:expr, $($pattern:pat)|+ if $guard:expr) => {
        match $expression {
            $($pattern)|+ if $guard => true,
            _ => false,
        }
    };
    ($expression:expr, $($pattern:pat)|+) => {
        match $expression {
            $($pattern)|+ => true,
            _ => false,
        }
    };
}
