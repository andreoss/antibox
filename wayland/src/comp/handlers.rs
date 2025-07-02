use smithay::backend::renderer::utils::on_commit_buffer_handler;
use smithay::desktop::{
    find_popup_root_surface, get_popup_toplevel_coords, PopupKind, PopupManager, Space, Window,
};
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::protocol::wl_buffer;
use smithay::reexports::wayland_server::protocol::wl_seat;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, Resource};
use smithay::utils::Serial;
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{
    get_parent, is_sync_subsurface, with_states, CompositorClientState, CompositorHandler,
    CompositorState,
};
use smithay::wayland::output::OutputHandler;
use smithay::wayland::selection::data_device::{
    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
    ServerDndGrabHandler,
};
use smithay::wayland::selection::SelectionHandler;
use smithay::wayland::shell::xdg::{
    PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    XdgToplevelSurfaceData,
};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::{
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat, delegate_shm,
    delegate_xdg_shell,
};

use super::state::{window_wl_surface, ClientState, Compositor, KeyboardFocusTarget};

impl SeatHandler for Compositor {
    type KeyboardFocus = KeyboardFocusTarget;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&KeyboardFocusTarget>) {
        
        let dh = &self.display_handle;
        let client = focused
            .and_then(smithay::wayland::seat::WaylandFocus::wl_surface)
            .and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, client);
    }
}
delegate_seat!(Compositor);

impl SelectionHandler for Compositor {
    type SelectionUserData = ();
}
impl DataDeviceHandler for Compositor {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}
impl ClientDndGrabHandler for Compositor {}
impl ServerDndGrabHandler for Compositor {}
delegate_data_device!(Compositor);

impl OutputHandler for Compositor {}
delegate_output!(Compositor);

impl CompositorHandler for Compositor {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        if let Some(state) = client.get_data::<ClientState>() {
            return &state.compositor_state;
        }
        if let Some(state) = client.get_data::<smithay::xwayland::XWaylandClientData>() {
            return &state.compositor_state;
        }
        panic!("client has neither ricewm ClientState nor XWaylandClientData");
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .space
                .elements()
                .find(|w| window_wl_surface(w).as_ref() == Some(&root))
            {
                window.on_commit();
            }
        }
        handle_commit(&mut self.popups, &self.space, surface);
        self.shared.request_redraw();
    }
}
impl BufferHandler for Compositor {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}
impl ShmHandler for Compositor {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}
delegate_compositor!(Compositor);
delegate_shm!(Compositor);

impl XdgShellHandler for Compositor {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.decoration_mode =
                Some(smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide);
        });
        let window = Window::new_wayland_window(surface);
        self.register_client(window);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        if let Some(id) = self
            .clients
            .iter()
            .find_map(|(id, w)| (w.toplevel() == Some(&surface)).then_some(*id))
        {
            self.unregister_client(id);
        }
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        self.unconstrain_popup(&surface);
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });
        self.unconstrain_popup(&surface);
        surface.send_repositioned(token);
    }

    fn move_request(&mut self, _surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
    }

    fn resize_request(
        &mut self,
        _surface: ToplevelSurface,
        _seat: wl_seat::WlSeat,
        _serial: Serial,
        _edges: xdg_toplevel::ResizeEdge,
    ) {
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
}
delegate_xdg_shell!(Compositor);

impl Compositor {
    fn unconstrain_popup(&self, popup: &PopupSurface) {
        let Ok(root) = find_popup_root_surface(&PopupKind::Xdg(popup.clone())) else {
            return;
        };
        let Some(window) = self
            .space
            .elements()
            .find(|w| window_wl_surface(w).as_ref() == Some(&root))
        else {
            return;
        };
        let Some(output) = self.space.outputs().next() else {
            return;
        };
        let Some(output_geo) = self.space.output_geometry(output) else {
            return;
        };
        let window_geo = self.space.element_geometry(window).unwrap();

        let mut target = output_geo;
        target.loc -= get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));
        target.loc -= window_geo.loc;

        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }
}

pub(crate) fn handle_commit(popups: &mut PopupManager, space: &Space<Window>, surface: &WlSurface) {
    if let Some(window) = space
        .elements()
        .find(|w| window_wl_surface(w).as_ref() == Some(surface))
        .cloned()
    {
        if let Some(toplevel) = window.toplevel() {
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .map(|d| {
                        d.lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .initial_configure_sent
                    })
                    .unwrap_or(true)
            });
            if !initial_configure_sent {
                toplevel.send_configure();
            }
        }
    }

    popups.commit(surface);
    if let Some(popup) = popups.find_popup(surface) {
        match popup {
            PopupKind::Xdg(ref xdg) => {
                if !xdg.is_initial_configure_sent() {
                    if let Err(e) = xdg.send_configure() {
                        eprintln!("[antibox] xdg configure failed: {e}");
                    }
                }
            }
            PopupKind::InputMethod(ref _im) => {}
        }
    }
}
