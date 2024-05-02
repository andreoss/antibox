#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
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
#[macro_use]
pub mod tooltip;

pub mod action {
    pub use antibox_ui::action::*;
}
#[macro_use]
pub mod applet;
pub mod bindings;
pub mod config_watch;
pub mod client;
pub mod clientmsg;
pub mod clock_applet;
pub mod container;
pub mod cursors;
pub mod desktop_apps;
pub mod dock;
pub mod dockmenu {
    pub use antibox_ui::dockmenu::*;
}
pub mod drag;
pub mod drag_outline;
pub mod ewmh;
pub mod focus;
pub mod fonts;
pub mod frame;
pub mod frame_store;
pub mod geom;
pub mod handler;
pub mod icon_dsl;
pub mod icon_render;
pub mod id;
pub mod audio;
pub mod audio_events;
pub mod audio_view;
pub mod battery_view;
pub mod cpu_status_applet;
pub mod keyboard_applet;
pub mod listview {
    pub use antibox_ui::listview::*;
}
pub mod menu_applet;
pub mod keys_parser;
pub mod layout;
pub mod layout_preferences;
pub mod manager;
pub mod mem_status_applet;
pub mod net_status_applet;
pub mod menu {
    pub use antibox_ui::menu::*;
}
pub mod menu_tree {
    pub use antibox_ui::menu_tree::*;
}
pub mod omni;
pub mod mouse_parser;
pub mod option;
pub mod paintbuf {
    pub use antibox_ui::paintbuf::*;
}
pub mod panic_guard;
pub mod placement;
pub mod power;
pub mod power_audio_applet;
pub mod preview;
pub mod proc_reader;
pub mod render;
pub mod settings_io;
pub mod resize_popup;
pub mod run;
pub mod snap;
pub mod status_graph;
pub mod taskbar;
pub mod taskpane;
#[cfg(feature = "tray")]
pub mod tray_applet;
pub mod winlist;
pub mod winmenu {
    pub use antibox_ui::winmenu::*;
}
pub mod wmaction;
pub mod wmapp;
pub mod wmconfig;
pub mod wmstate;
pub mod workspace_pane;
