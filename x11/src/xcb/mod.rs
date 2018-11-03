
pub mod arcs;
pub mod bindings;
pub mod connection;
pub mod event;
pub mod event_loop;
pub mod font;
pub mod graphics;
pub mod window;

pub use self::connection::{XcbConnection, XcbError};
pub use self::event_loop::XcbEventLoop;
pub use self::graphics::XcbGraphics;
pub use self::window::XcbWindow;

use antibox_core::backend::{DisplayBackend, EventLoopTrait, RenderBackend, TrayBackend};
use std::sync::Arc;

pub fn build_backend(
    display: Option<&str>,
) -> Result<
    (
        Arc<dyn DisplayBackend>,
        Arc<dyn RenderBackend>,
        Box<dyn EventLoopTrait>,
        Option<Arc<dyn TrayBackend>>,
    ),
    Box<dyn std::error::Error>,
> {
    let conn = XcbConnection::open_arc(display)?;
    arcs::register(&conn);
    super::xcb::font::register_global_width_provider(&conn);
    let render: Arc<dyn RenderBackend> = conn.clone();
    let backend: Arc<dyn DisplayBackend> = conn;
    let event_loop: Box<dyn EventLoopTrait> = Box::new(XcbEventLoop::new(Arc::clone(&backend)));
    let tray: Option<Arc<dyn TrayBackend>> = None;
    Ok((backend, render, event_loop, tray))
}
