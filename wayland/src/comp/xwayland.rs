use std::ffi::OsString;

use smithay::delegate_xwayland_shell;
use smithay::desktop::Window;
use smithay::reexports::calloop::EventLoop;
use smithay::utils::{Logical, Point, Rectangle, Size};
use smithay::wayland::xwayland_shell::{XWaylandShellHandler, XWaylandShellState};
use smithay::xwayland::xwm::{Reorder, ResizeEdge, X11Window, XwmId};
use smithay::xwayland::{X11Surface, X11Wm, XWayland, XWaylandEvent, XwmHandler};

use antibox_core::backend::BackendEvent;
use antibox_core::rect::Rect;

use super::state::Compositor;
use crate::shared::{WinKind, WinRec, ROOT_WINDOW};

impl Compositor {
    fn x11_id_for(&self, surface: &X11Surface) -> Option<u32> {
        let target = surface.window_id();
        self.clients.iter().find_map(|(id, w)| {
            w.x11_surface()
                .map(|x| x.window_id() == target)
                .unwrap_or(false)
                .then_some(*id)
        })
    }
}

impl XwmHandler for Compositor {
    fn xwm_state(&mut self, _xwm: XwmId) -> &mut X11Wm {
        self.xwm.as_mut().expect("xwm not started")
    }

    fn new_window(&mut self, _xwm: XwmId, _window: X11Surface) {}
    fn new_override_redirect_window(&mut self, _xwm: XwmId, _window: X11Surface) {}

    fn map_window_request(&mut self, _xwm: XwmId, window: X11Surface) {
        let _ = window.set_mapped(true);
        let surface = window.clone();
        let w = Window::new_x11_window(window);
        let id = self.register_client(w);
        self.sync_x11_props(id, &surface);
    }

    fn mapped_override_redirect_window(&mut self, _xwm: XwmId, window: X11Surface) {
        let geo = window.geometry();
        let w = Window::new_x11_window(window);
        let id = {
            let mut s = self.shared.lock();
            let id = s.alloc_id();
            s.windows.insert(
                id,
                WinRec {
                    kind: WinKind::Client,
                    rect: Rect::new(geo.loc.x, geo.loc.y, geo.size.w, geo.size.h),
                    mapped: true,
                    override_redirect: true,
                    depth: 32,
                    parent: ROOT_WINDOW,
                    event_mask: 0,
                },
            );
            id
        };
        self.clients.insert(id, w.clone());
        self.space.map_element(w, (geo.loc.x, geo.loc.y), true);
    }

    fn unmapped_window(&mut self, _xwm: XwmId, window: X11Surface) {
        if let Some(id) = self.x11_id_for(&window) {
            if let Some(w) = self.clients.get(&id).cloned() {
                self.space.unmap_elem(&w);
            }
            self.synth(BackendEvent::UnmapNotify { window: id });
        }
    }

    fn destroyed_window(&mut self, _xwm: XwmId, window: X11Surface) {
        if let Some(id) = self.x11_id_for(&window) {
            self.unregister_client(id);
        }
    }

    fn configure_request(
        &mut self,
        _xwm: XwmId,
        window: X11Surface,
        x: Option<i32>,
        y: Option<i32>,
        w: Option<u32>,
        h: Option<u32>,
        _reorder: Option<Reorder>,
    ) {
        let geo = window.geometry();
        let nx = x.unwrap_or(geo.loc.x);
        let ny = y.unwrap_or(geo.loc.y);
        let nw = w.map(|v| v as i32).unwrap_or(geo.size.w);
        let nh = h.map(|v| v as i32).unwrap_or(geo.size.h);
        let _ = window.configure(Some(Rectangle::<i32, Logical>::new(
            Point::from((nx, ny)),
            Size::from((nw, nh)),
        )));
        if let Some(id) = self.x11_id_for(&window) {
            self.synth(BackendEvent::ConfigureRequest {
                window: id,
                parent: ROOT_WINDOW,
                rect: Rect::new(nx, ny, nw, nh),
                border_width: 0,
                value_mask: 0,
            });
        }
    }

    fn configure_notify(
        &mut self,
        _xwm: XwmId,
        window: X11Surface,
        geometry: Rectangle<i32, Logical>,
        _above: Option<X11Window>,
    ) {
        if let Some(id) = self.x11_id_for(&window) {
            let is_or = self
                .shared
                .lock()
                .windows
                .get(&id)
                .map(|r| r.override_redirect)
                .unwrap_or(false);
            if is_or {
                if let Some(w) = self.clients.get(&id).cloned() {
                    self.space
                        .map_element(w, (geometry.loc.x, geometry.loc.y), true);
                }
            }
        }
    }

    fn resize_request(
        &mut self,
        _xwm: XwmId,
        _window: X11Surface,
        _button: u32,
        _edge: ResizeEdge,
    ) {
    }

    fn move_request(&mut self, _xwm: XwmId, _window: X11Surface, _button: u32) {}

    fn property_notify(
        &mut self,
        _xwm: XwmId,
        window: X11Surface,
        property: smithay::xwayland::xwm::WmWindowProperty,
    ) {
        if let Some(id) = self.x11_id_for(&window) {
            self.sync_x11_prop(id, &window, property);
        }
    }
}

impl XWaylandShellHandler for Compositor {
    fn xwayland_shell_state(&mut self) -> &mut XWaylandShellState {
        &mut self.xwayland_shell_state
    }
}
delegate_xwayland_shell!(Compositor);

pub(crate) fn spawn_xwayland(event_loop: &EventLoop<'static, Compositor>, state: &mut Compositor) {
    let handle = event_loop.handle();
    let dh = state.display_handle.clone();

    let (xwayland, client) = match XWayland::spawn(
        &dh,
        None::<u32>,
        std::iter::empty::<(OsString, OsString)>(),
        true,
        std::process::Stdio::null(),
        std::process::Stdio::null(),
        |_| {},
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[antibox] xwayland setup failed: {e}");
            return;
        }
    };

    let loop_handle = handle.clone();
    let res = handle.insert_source(
        xwayland,
        move |event, _, state: &mut Compositor| match event {
            XWaylandEvent::Ready {
                x11_socket,
                display_number,
            } => match X11Wm::start_wm(loop_handle.clone(), x11_socket, client.clone()) {
                Ok(wm) => {
                    state.xwm = Some(wm);
                    std::env::set_var("DISPLAY", format!(":{display_number}"));
                }
                Err(e) => eprintln!("[antibox] failed to start X11 WM: {e}"),
            },
            XWaylandEvent::Error => {
            }
        },
    );
    if let Err(e) = res {
        eprintln!("[antibox] xwayland source failed: {e}");
    }
}
