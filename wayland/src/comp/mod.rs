mod bridge;
mod dispatch;
mod eventloop;
mod input;
mod state;

pub use eventloop::WaylandEventLoop;
pub use state::Server;

use crate::backend::WaylandCompositor;
use crate::buffers::BufferStore;
use crate::ffi::wl::*;
use crate::ffi::wlr::*;
use crate::shared::Shared;
use antibox_core::backend::{DisplayBackend, EventLoopTrait};
use antibox_core::error::{Error, Result};
use state::{Server as ServerState, Tag};
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Arc;

pub fn build_nested() -> Result<(Arc<WaylandCompositor>, Box<dyn EventLoopTrait>)> {
    unsafe { build() }
}

unsafe fn build() -> Result<(Arc<WaylandCompositor>, Box<dyn EventLoopTrait>)> {
    let display = wl_display_create();
    if display.is_null() {
        return Err(Error::message("cannot create a wayland display"));
    }
    let event_loop = wl_display_get_event_loop(display);
    let backend = wlr_backend_autocreate(event_loop, std::ptr::null_mut());
    if backend.is_null() {
        return Err(Error::message("cannot create a wayland backend"));
    }
    let renderer = wlr_renderer_autocreate(backend);
    if renderer.is_null() {
        return Err(Error::message("cannot create a renderer"));
    }
    wlr_renderer_init_wl_display(renderer, display);
    let allocator = wlr_allocator_autocreate(backend, renderer);
    if allocator.is_null() {
        return Err(Error::message("cannot create an allocator"));
    }

    wlr_compositor_create(display, 6, renderer);
    wlr_subcompositor_create(display);
    wlr_data_device_manager_create(display);

    let layout = wlr_output_layout_create(display);
    let scene = wlr_scene_create();
    wlr_scene_attach_output_layout(scene, layout);
    let client_tree = wlr_scene_tree_create(std::ptr::addr_of_mut!((*scene).tree));
    let decor_tree = wlr_scene_tree_create(std::ptr::addr_of_mut!((*scene).tree));

    let xdg_shell = wlr_xdg_shell_create(display, 3);
    let seat_name = CString::new("seat0").map_err(|e| Error::message(e.to_string()))?;
    let seat = wlr_seat_create(display, seat_name.as_ptr());
    let cursor = wlr_cursor_create();
    wlr_cursor_attach_output_layout(cursor, layout);
    let theme = CString::new("default").map_err(|e| Error::message(e.to_string()))?;
    let cursor_mgr = wlr_xcursor_manager_create(theme.as_ptr(), 24);
    wlr_xcursor_manager_load(cursor_mgr, 1.0);

    let shared = Shared::new(1280, 720);
    let buffers = BufferStore::new();

    let mut server = Box::new(ServerState {
        display,
        event_loop,
        backend,
        renderer,
        allocator,
        scene,
        layout,
        client_tree,
        decor_tree,
        xdg_shell,
        seat,
        cursor,
        cursor_mgr,
        keyboard: std::ptr::null_mut(),
        outputs: Vec::new(),
        scene_outputs: Vec::new(),
        clients: HashMap::new(),
        decorations: HashMap::new(),
        hooks: Vec::new(),
        shared: shared.clone(),
        buffers: Arc::clone(&buffers),
        socket: None,
    });

    server.hook(
        std::ptr::addr_of_mut!((*backend).events.new_output),
        Tag::NewOutput,
        0,
    );
    server.hook(
        std::ptr::addr_of_mut!((*backend).events.new_input),
        Tag::NewInput,
        0,
    );
    server.hook(
        std::ptr::addr_of_mut!((*xdg_shell).events.new_toplevel),
        Tag::NewToplevel,
        0,
    );
    server.hook(
        std::ptr::addr_of_mut!((*cursor).events.motion),
        Tag::CursorMotion,
        0,
    );
    server.hook(
        std::ptr::addr_of_mut!((*cursor).events.motion_absolute),
        Tag::CursorMotionAbsolute,
        0,
    );
    server.hook(
        std::ptr::addr_of_mut!((*cursor).events.button),
        Tag::CursorButton,
        0,
    );
    server.hook(
        std::ptr::addr_of_mut!((*cursor).events.axis),
        Tag::CursorAxis,
        0,
    );
    server.hook(
        std::ptr::addr_of_mut!((*cursor).events.frame),
        Tag::CursorFrame,
        0,
    );

    let socket = wl_display_add_socket_auto(display);
    let socket = crate::ffi::cstr_to_string(socket)
        .ok_or_else(|| Error::message("cannot open a wayland socket"))?;
    std::env::set_var("WAYLAND_DISPLAY", &socket);
    server.socket = Some(socket);

    if !wlr_backend_start(backend) {
        return Err(Error::message("cannot start the wayland backend"));
    }
    server.build_keymap();

    let compositor = Arc::new(WaylandCompositor::new(shared, buffers));
    let dyn_backend: Arc<dyn DisplayBackend> = Arc::clone(&compositor) as Arc<dyn DisplayBackend>;
    let event_loop = Box::new(WaylandEventLoop::new(server, dyn_backend));
    Ok((compositor, event_loop))
}
