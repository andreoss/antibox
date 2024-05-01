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
fn main() -> Result<()> {
    antibox::run_cli(env!("CARGO_PKG_NAME"), |display| {
        antibox_x11::xcb::build_backend(display)
    })
}
