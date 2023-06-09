#![deny(unsafe_code)]
 use antibox_core::error::Result;
use std::sync::Arc;

use antibox_core::backend::{DisplayBackend, EventLoopTrait, RenderBackend, TrayBackend};

pub type Backend = (
    Arc<dyn DisplayBackend>,
    Arc<dyn RenderBackend>,
    Box<dyn EventLoopTrait>,
    Option<Arc<dyn TrayBackend>>,
);

pub fn run_cli<F>(
    _prog_name: &'static str,
    build_backend: F,
) -> Result<()>
where
    F: FnOnce(Option<&str>) -> Result<Backend>,
{
    parse_config();
    let (backend, render, event_loop, tray) = build_backend(None)?;

    let mut app = match antibox_wm::wmapp::App::new(
        backend,
        render,
        event_loop,
        tray.as_ref(),
        antibox_wm::wmapp::LaunchOptions { display: None },
    ) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    app.run()
}

fn parse_config() {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                eprintln!("usage: antibox");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                eprintln!("antibox {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
