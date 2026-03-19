mod bridge;
mod dispatch;
mod eventloop;
mod input;
mod state;
mod xwayland;

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

    let compositor = wlr_compositor_create(display, 6, renderer);
    wlr_subcompositor_create(display);
    wlr_data_device_manager_create(display);

    let layout = wlr_output_layout_create(display);
    let scene = wlr_scene_create();
    wlr_scene_attach_output_layout(scene, layout);
    let client_tree = wlr_scene_tree_create(std::ptr::addr_of_mut!((*scene).tree));

    wlr_primary_selection_v1_device_manager_create(display);
    wlr_xdg_output_manager_v1_create(display, layout);
    wlr_screencopy_manager_v1_create(display);
    wlr_viewporter_create(display);
    wlr_presentation_create(display, backend, 2);
    wlr_fractional_scale_manager_v1_create(display, 1);
    wlr_single_pixel_buffer_manager_v1_create(display);
    wlr_gamma_control_manager_v1_create(display);
    wlr_idle_notifier_v1_create(display);

    let xdg_shell = wlr_xdg_shell_create(display, 3);
    let decoration_manager = wlr_xdg_decoration_manager_v1_create(display);
    let kde_decoration = wlr_server_decoration_manager_create(display);
    if !kde_decoration.is_null() {
        wlr_server_decoration_manager_set_default_mode(
            kde_decoration,
            WLR_SERVER_DECORATION_MANAGER_MODE_SERVER,
        );
    }
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
        renderer,
        allocator,
        scene,
        layout,
        client_tree,
        compositor,
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
        xwayland: std::ptr::null_mut(),
        kde_decorations: Vec::new(),
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
        std::ptr::addr_of_mut!((*decoration_manager).events.new_toplevel_decoration),
        Tag::NewDecoration,
        0,
    );
    if !kde_decoration.is_null() {
        server.hook(
            std::ptr::addr_of_mut!((*kde_decoration).events.new_decoration),
            Tag::NewKdeDecoration,
            0,
        );
    }
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

    server.set_default_cursor();
    server.start_xwayland();

    if !wlr_backend_start(backend) {
        return Err(Error::message("cannot start the wayland backend"));
    }
    server.build_keymap();

    let backend_arc = Arc::new(WaylandCompositor::new(shared, buffers));
    let dyn_backend: Arc<dyn DisplayBackend> = Arc::clone(&backend_arc) as Arc<dyn DisplayBackend>;
    let event_loop = Box::new(WaylandEventLoop::new(server, dyn_backend));
    Ok((backend_arc, event_loop))
}
