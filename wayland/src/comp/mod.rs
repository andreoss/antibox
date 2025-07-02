mod bridge;
mod eventloop;
mod handlers;
mod input;
mod render;
mod state;
mod winit;
mod xprops;
mod xwayland;

pub use eventloop::WaylandEventLoop;
pub use state::Compositor;

use crate::backend::WaylandCompositor;
use crate::buffers::BufferStore;
use crate::shared::Shared;
use antibox_core::backend::{DisplayBackend, EventLoopTrait};
use antibox_core::error::Result;
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use std::sync::Arc;

pub fn build_nested() -> Result<(Arc<WaylandCompositor>, Box<dyn EventLoopTrait>)> {
    let mut event_loop: EventLoop<'static, Compositor> =
        EventLoop::try_new().map_err(|e| antibox_core::error::Error::message(e.to_string()))?;
    let display: Display<Compositor> =
        Display::new().map_err(|e| antibox_core::error::Error::message(e.to_string()))?;

    let shared = Shared::new(1280, 720);
    let buffers = BufferStore::new();

    let mut state = Compositor::new(
        &mut event_loop,
        display,
        shared.clone(),
        Arc::clone(&buffers),
    );

    winit::init_winit(&mut event_loop, &mut state)?;
    state.build_keymap();
    xwayland::spawn_xwayland(&event_loop, &mut state);

    let backend = Arc::new(WaylandCompositor::new(shared, buffers));
    let dyn_backend: Arc<dyn DisplayBackend> = Arc::clone(&backend) as Arc<dyn DisplayBackend>;
    let event_loop = Box::new(WaylandEventLoop::new(event_loop, state, dyn_backend));
    Ok((backend, event_loop))
}

pub(crate) fn wrap<E: std::fmt::Display>(e: E) -> antibox_core::error::Error {
    antibox_core::error::Error::message(e.to_string())
}
