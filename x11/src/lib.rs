#![allow(clippy::incompatible_msrv)]
pub mod signal;

pub mod xcb;

pub use self::xcb::{XcbConnection, XcbEventLoop, XcbGraphics, XcbWindow};

pub use self::signal::SignalHandler;
