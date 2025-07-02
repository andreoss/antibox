use std::time::Duration;

use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::winit::{self, WinitEvent, WinitGraphicsBackend};
use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::calloop::EventLoop;
use smithay::utils::{Rectangle, Transform};

use super::state::Compositor;

pub struct WinitPresenter {
    backend: WinitGraphicsBackend<GlesRenderer>,
    output: Output,
    damage_tracker: OutputDamageTracker,
}

impl Compositor {
    pub fn request_redraw(&self) {
        if let Some(p) = &self.winit {
            p.backend.window().request_redraw();
        }
    }

    pub fn present_winit(&mut self) {
        let Some(mut p) = self.winit.take() else {
            return;
        };
        self.render_to_winit(&mut p);
        self.winit = Some(p);
    }

    fn render_to_winit(&mut self, p: &mut WinitPresenter) {
        let size = p.backend.window_size();
        let damage = Rectangle::from_size(size);
        let scale = p.output.current_scale().fractional_scale();
        {
            let clear = self.shared.lock().clear_colour();
            let (renderer, mut framebuffer) = match p.backend.bind() {
                Ok(rf) => rf,
                Err(e) => {
                    eprintln!("[antibox] winit bind failed: {e}");
                    return;
                }
            };
            let elements = super::render::output_elements(self, renderer, scale);
            if let Err(e) =
                p.damage_tracker
                    .render_output(renderer, &mut framebuffer, 0, &elements, clear)
            {
                eprintln!("[antibox] render failed: {e}");
            }
        }
        if let Err(e) = p.backend.submit(Some(&[damage])) {
            eprintln!("[antibox] frame submit failed: {e}");
        }
        let elapsed = self.start_time.elapsed();
        self.space.elements().for_each(|window| {
            window.send_frame(&p.output, elapsed, Some(Duration::ZERO), |_, _| {
                Some(p.output.clone())
            });
        });
        self.space.refresh();
        self.popups.cleanup();
        let _ = self.display_handle.flush_clients();
    }
}

pub(crate) fn init_winit(
    event_loop: &mut EventLoop<'static, Compositor>,
    state: &mut Compositor,
) -> antibox_core::error::Result<()> {
    let display_handle = state.display_handle.clone();

    let (backend, winit) = winit::init::<GlesRenderer>().map_err(super::wrap)?;

    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

    let output = Output::new(
        "winit".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "ricewm".into(),
            model: "Winit".into(),
        },
    );
    let _global = output.create_global::<Compositor>(&display_handle);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);
    state.space.map_output(&output, (0, 0));

    {
        let sz = backend.window_size();
        let host_scale = backend.scale_factor();
        let mut s = state.shared.lock();
        s.screen_w = sz.w.max(1) as u16;
        s.screen_h = sz.h.max(1) as u16;
        s.screen_scale = host_scale.max(1.0);
    }

    let damage_tracker = OutputDamageTracker::from_output(&output);
    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);

    let present_output = output.clone();
    state.winit = Some(WinitPresenter {
        backend,
        output,
        damage_tracker,
    });

    event_loop.handle().insert_source(
        winit,
        move |event, _, state: &mut Compositor| match event {
            WinitEvent::Resized { size, .. } => {
                present_output.change_current_state(
                    Some(Mode {
                        size,
                        refresh: 60_000,
                    }),
                    None,
                    None,
                    None,
                );
                {
                    let mut s = state.shared.lock();
                    s.screen_w = size.w.max(1) as u16;
                    s.screen_h = size.h.max(1) as u16;
                    if let Some(ref p) = state.winit {
                        s.screen_scale = p.backend.scale_factor().max(1.0);
                    }
                    s.events
                        .push(antibox_core::backend::BackendEvent::ScreenSizeChanged {
                            width: size.w.max(1) as u16,
                            height: size.h.max(1) as u16,
                        });
                }
                state.request_redraw();
            }
            WinitEvent::Input(event) => {
                state.process_input_event(event);
                state.request_redraw();
            }
            WinitEvent::Redraw => {
                state.present_winit();
            }
            WinitEvent::CloseRequested => {
                state.loop_signal.stop();
            }
            _ => {}
        },
    )
    .map_err(super::wrap)?;

    Ok(())
}
