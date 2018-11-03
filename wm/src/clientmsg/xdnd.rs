use antibox_core::backend::DisplayBackend;
use antibox_core::rect::Rect;

use crate::manager::WindowManager;

pub fn handle_xdnd_enter<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    d: [u32; 5],
) {
    let source = d[0];
    let version = (d[1] >> 24) & 0xF;
    wm.xdnd_drop_target = None;
    const XDND_CURRENT_VERSION: u32 = 5;
    if version <= XDND_CURRENT_VERSION {
        wm.xdnd_source = Some(source);
    } else {
        wm.xdnd_source = None;
    }
    if let Some(backend) = wm.backend() {
        let accept = wm.xdnd_source.is_some();
        crate::ewmh::send_xdnd_status(backend, &wm.atoms, w, Rect::default(), source, accept);
    }
}

pub fn handle_xdnd_leave<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    d: [u32; 5],
) {
    let source = d[0];
    if wm.xdnd_source == Some(source) {
        wm.xdnd_drop_target = None;
        wm.xdnd_source = None;
    }
}

pub fn handle_xdnd_position<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    d: [u32; 5],
) {
    let source = d[0];
    let x = (d[2] >> 16) as i32;
    let y = (d[2] & 0xFFFF) as i32;
    let root_pt = antibox_core::point::Point::new(x, y);
    if wm.xdnd_source != Some(source) {
        wm.xdnd_source = Some(source);
    }
    let target = wm.frames.iter().find_map(|(id, fw)| {
        if fw.frame_rect().contains(root_pt) {
            Some(*id)
        } else {
            None
        }
    });
    if target != wm.xdnd_drop_target {
        wm.xdnd_drop_target = target;
    }
    if let Some(backend) = wm.backend() {
        let accept = wm.xdnd_drop_target.is_some();
        let (status_target, status_rect) = match wm.xdnd_drop_target.and_then(|id| wm.frame(id)) {
            Some(fw) => (fw.frame().id(), fw.frame_rect()),
            None => (w, Rect::default()),
        };
        crate::ewmh::send_xdnd_status(
            backend,
            &wm.atoms,
            status_target,
            status_rect,
            source,
            accept,
        );
    }
}

pub fn handle_xdnd_drop<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    d: [u32; 5],
) {
    let source = d[0];
    let timestamp = d[2];
    if wm.xdnd_source == Some(source) {
        if let Some(backend) = wm.backend() {
            crate::ewmh::send_xdnd_finished(backend, &wm.atoms, w, source, true);
            if let Some(xdnd_sel) = wm.atoms.get("XdndSelection") {
                let _ = backend.convert_selection(w, xdnd_sel, 0, 0, timestamp);
            }
        }
        wm.xdnd_source = None;
        wm.xdnd_drop_target = None;
    }
}
