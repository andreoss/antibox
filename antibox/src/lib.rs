#![deny(unsafe_code)]
#![deny(
    elided_lifetimes_in_paths,
    meta_variable_misuse,
    unreachable_pub,
    unused_lifetimes,
    unused_qualifications
)]
#![deny(
    clippy::cloned_instead_of_copied,
    clippy::dbg_macro,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    clippy::todo,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unimplemented,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    clippy::use_self
)]
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
            eprintln!("{e}");
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

pub mod wayland_glyphs;
