use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::Arc;

use crate::buffers::BufferStore;
use crate::shared::Shared;
use smithay::desktop::{PopupManager, Space, Window, WindowSurfaceType};
use smithay::input::{Seat, SeatState};
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{EventLoop, Interest, LoopSignal, Mode, PostAction};
use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Display, DisplayHandle};
use smithay::utils::{Logical, Point};
use smithay::wayland::compositor::{CompositorClientState, CompositorState};
use smithay::wayland::output::OutputManagerState;
use smithay::wayland::selection::data_device::DataDeviceState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shm::ShmState;
use smithay::wayland::socket::ListeningSocketSource;
use smithay::wayland::xwayland_shell::XWaylandShellState;
use smithay::xwayland::X11Wm;

pub struct Compositor {
    pub start_time: std::time::Instant,
    pub socket_name: OsString,
    pub display_handle: DisplayHandle,

    pub space: Space<Window>,
    pub loop_signal: LoopSignal,

    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub xdg_decoration_state: smithay::wayland::shell::xdg::decoration::XdgDecorationState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<Compositor>,
    pub data_device_state: DataDeviceState,
    pub popups: PopupManager,

    pub seat: Seat<Self>,

    pub shared: Shared,
    pub buffers: Arc<BufferStore>,
    pub clients: HashMap<u32, Window>,
    pub pending_map: std::collections::HashSet<u32>,

    pub xwayland_shell_state: XWaylandShellState,
    pub xwm: Option<X11Wm>,

    pub winit: Option<super::winit::WinitPresenter>,
}

impl Compositor {
    pub fn new(
        event_loop: &mut EventLoop<'static, Self>,
        display: Display<Self>,
        shared: Shared,
        buffers: Arc<BufferStore>,
    ) -> Self {
        let start_time = std::time::Instant::now();
        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let xdg_decoration_state =
            smithay::wayland::shell::xdg::decoration::XdgDecorationState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        let xwayland_shell_state = XWaylandShellState::new::<Self>(&dh);
        let popups = PopupManager::default();

        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, "winit");
        seat.add_keyboard(Default::default(), 200, 25)
            .expect("keyboard");
        seat.add_pointer();

        let space = Space::default();
        let socket_name = Self::init_wayland_listener(display, event_loop);
        let loop_signal = event_loop.get_signal();

        Self {
            start_time,
            display_handle: dh,
            space,
            loop_signal,
            socket_name,
            compositor_state,
            xdg_shell_state,
            xdg_decoration_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            popups,
            seat,
            shared,
            buffers,
            clients: HashMap::new(),
            pending_map: std::collections::HashSet::new(),
            xwayland_shell_state,
            xwm: None,
            winit: None,
        }
    }

    pub fn client_id_for_surface(&self, surface: &WlSurface) -> Option<u32> {
        self.clients
            .iter()
            .find_map(|(id, w)| (window_wl_surface(w).as_ref() == Some(surface)).then_some(*id))
    }

    fn init_wayland_listener(
        display: Display<Self>,
        event_loop: &mut EventLoop<'static, Self>,
    ) -> OsString {
        let listening_socket = ListeningSocketSource::new_auto().expect("wayland socket");
        let socket_name = listening_socket.socket_name().to_os_string();
        let handle = event_loop.handle();

        handle
            .insert_source(
                listening_socket,
                move |client_stream, _, data: &mut Self| {
                    data.display_handle
                        .insert_client(client_stream, Arc::new(ClientState::default()))
                        .expect("insert client");
                },
            )
            .expect("init wayland event source");

        handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, data: &mut Self| {
                    unsafe {
                        if let Err(e) = display.get_mut().dispatch_clients(data) {
                            eprintln!("[antibox] wayland dispatch failed: {e}");
                        }
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("init display source");

        socket_name
    }

    pub fn surface_under(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(s, p)| (s, (p + location).to_f64()))
            })
    }
}

pub(crate) fn window_wl_surface(window: &Window) -> Option<WlSurface> {
    if let Some(t) = window.toplevel() {
        return Some(t.wl_surface().clone());
    }
    window.x11_surface().and_then(smithay::xwayland::X11Surface::wl_surface)
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardFocusTarget {
    Wayland(WlSurface),
    X11(smithay::xwayland::X11Surface),
}

impl KeyboardFocusTarget {
    pub fn for_window(window: &Window) -> Option<Self> {
        if let Some(x11) = window.x11_surface() {
            return Some(Self::X11(x11.clone()));
        }
        window_wl_surface(window).map(KeyboardFocusTarget::Wayland)
    }
}

impl smithay::wayland::seat::WaylandFocus for KeyboardFocusTarget {
    fn wl_surface(&self) -> Option<std::borrow::Cow<'_, WlSurface>> {
        match self {
            Self::Wayland(s) => Some(std::borrow::Cow::Borrowed(s)),
            Self::X11(x) => x.wl_surface().map(std::borrow::Cow::Owned),
        }
    }
}

impl smithay::utils::IsAlive for KeyboardFocusTarget {
    fn alive(&self) -> bool {
        match self {
            Self::Wayland(s) => s.alive(),
            Self::X11(x) => x.alive(),
        }
    }
}

impl smithay::input::keyboard::KeyboardTarget<Compositor> for KeyboardFocusTarget {
    fn enter(
        &self,
        seat: &Seat<Compositor>,
        data: &mut Compositor,
        keys: Vec<smithay::input::keyboard::KeysymHandle<'_>>,
        serial: smithay::utils::Serial,
    ) {
        use smithay::input::keyboard::KeyboardTarget;
        match self {
            Self::Wayland(s) => KeyboardTarget::enter(s, seat, data, keys, serial),
            Self::X11(x) => KeyboardTarget::enter(x, seat, data, keys, serial),
        }
    }

    fn leave(
        &self,
        seat: &Seat<Compositor>,
        data: &mut Compositor,
        serial: smithay::utils::Serial,
    ) {
        use smithay::input::keyboard::KeyboardTarget;
        match self {
            Self::Wayland(s) => KeyboardTarget::leave(s, seat, data, serial),
            Self::X11(x) => KeyboardTarget::leave(x, seat, data, serial),
        }
    }

    fn key(
        &self,
        seat: &Seat<Compositor>,
        data: &mut Compositor,
        key: smithay::input::keyboard::KeysymHandle<'_>,
        state: smithay::backend::input::KeyState,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        use smithay::input::keyboard::KeyboardTarget;
        match self {
            Self::Wayland(s) => {
                KeyboardTarget::key(s, seat, data, key, state, serial, time);
            }
            Self::X11(x) => {
                KeyboardTarget::key(x, seat, data, key, state, serial, time);
            }
        }
    }

    fn modifiers(
        &self,
        seat: &Seat<Compositor>,
        data: &mut Compositor,
        modifiers: smithay::input::keyboard::ModifiersState,
        serial: smithay::utils::Serial,
    ) {
        use smithay::input::keyboard::KeyboardTarget;
        match self {
            Self::Wayland(s) => {
                KeyboardTarget::modifiers(s, seat, data, modifiers, serial);
            }
            Self::X11(x) => {
                KeyboardTarget::modifiers(x, seat, data, modifiers, serial);
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
