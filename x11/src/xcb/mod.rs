
pub mod arcs;
pub mod bindings;
pub mod connection;
pub mod event;
pub mod event_loop;
pub mod font;
pub mod graphics;
pub mod ft;
pub mod window;
pub mod xkb;

pub use self::connection::{XcbConnection, XcbError};
pub use self::event_loop::XcbEventLoop;
pub use self::graphics::XcbGraphics;
pub use self::window::XcbWindow;

use crate::tray_backend::XcbTray;
use antibox_core::backend::{DisplayBackend, EventLoopTrait, RenderBackend, TrayBackend};
use antibox_core::error::Result;
use std::sync::Arc;

pub type Backend = (
    Arc<dyn DisplayBackend>,
    Arc<dyn RenderBackend>,
    Box<dyn EventLoopTrait>,
    Option<Arc<dyn TrayBackend>>,
);

pub fn build_backend(display: Option<&str>) -> Result<Backend> {
    let conn = XcbConnection::open_arc(display)?;
    arcs::register(&conn);
    font::register_global_width_provider(&conn);
    let render: Arc<dyn RenderBackend> = conn.clone();
    let tray: Arc<dyn TrayBackend> = Arc::new(XcbTray::new(conn.clone()));
    let backend: Arc<dyn DisplayBackend> = conn;
    let event_loop: Box<dyn EventLoopTrait> = Box::new(XcbEventLoop::new(Arc::clone(&backend)));
    Ok((backend, render, event_loop, Some(tray)))
}
