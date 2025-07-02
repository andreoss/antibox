use antibox_core::backend::{DisplayBackend, RenderBackend};
use std::sync::Arc;

fn main() -> antibox_core::error::Result<()> {
    antibox::run_cli(env!("CARGO_PKG_NAME"), |_display| {
        antibox::wayland_glyphs::install();
        let (compositor, event_loop) = antibox_wayland::comp::build_nested()?;
        let display: Arc<dyn DisplayBackend> = Arc::clone(&compositor) as Arc<dyn DisplayBackend>;
        let render: Arc<dyn RenderBackend> = compositor as Arc<dyn RenderBackend>;
        Ok((display, render, event_loop, None))
    })
}
