 use antibox_core::error::Result;
fn main() -> Result<()> {
    antibox::run_cli(env!("CARGO_PKG_NAME"), |display| {
        antibox_x11::xcb::build_backend(display)
    })
}
