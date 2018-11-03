fn main() -> Result<(), Box<dyn std::error::Error>> {
    antibox::run_cli(env!("CARGO_PKG_NAME"), |display| {
        antibox_x11::xcb::build_backend(display)
    })
}
