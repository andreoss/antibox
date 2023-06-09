#![allow(clippy::incompatible_msrv)]
#![deny(unsafe_op_in_unsafe_fn)]
pub mod signal;
pub mod tray_backend;

pub mod xcb;

pub use self::xcb::{XcbConnection, XcbEventLoop, XcbGraphics, XcbWindow};

pub use self::signal::SignalHandler;
