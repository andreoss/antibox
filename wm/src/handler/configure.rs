use crate::frame::{border_width, bottom_border_width, title_bar_height, top_for};
use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
use antibox_core::rect::Rect;

pub fn configure_request<H: DisplayBackend + 'static + ?Sized>(
    wm: &mut WindowManager<H>,
    w: u32,
    r: Rect,
    value_mask: u16,
) {
    const CFG_X: u16 = 1 << 0;
    const CFG_Y: u16 = 1 << 1;
    const CFG_WIDTH: u16 = 1 << 2;
    const CFG_HEIGHT: u16 = 1 << 3;

    let key = wm.cid_for_xid(w);
    let id = match key {
        Some(id) => id,
        None => {
            if let Ok(xw) = wm.backend().unwrap().wrap_window(w) {
                let _ = xw.configure(
                    if value_mask & CFG_X != 0 { Some(r.x) } else { None },
                    if value_mask & CFG_Y != 0 { Some(r.y) } else { None },
                    if value_mask & CFG_WIDTH != 0 {
                        Some(r.w.max(1) as u16)
                    } else {
                        None
                    },
                    if value_mask & CFG_HEIGHT != 0 {
                        Some(r.h.max(1) as u16)
                    } else {
                        None
                    },
                );
            }
            return;
        }
    };

    let cur = wm
        .frame(id)
        .map_or(r, super::super::frame::FrameWindow::client_rect);
    let nx = if value_mask & CFG_X != 0 { r.x } else { cur.x };
    let ny = if value_mask & CFG_Y != 0 { r.y } else { cur.y };
    let nw = if value_mask & CFG_WIDTH != 0 {
        r.w
    } else {
        cur.w
    }
    .max(1);
    let nh = if value_mask & CFG_HEIGHT != 0 {
        r.h
    } else {
        cur.h
    }
    .max(1);
    let (nw, nh) = wm
        .frames
        .get(&id)
        .and_then(|f| f.client().size_hints())
        .map_or((nw, nh), |h| {
            let (cw, ch) = h.constrain(nw, nh);
            (cw.max(1), ch.max(1))
        });
    let (decorated, inset) = wm.frames.get(&id).map_or((true, [0; 4]), |f| {
        (
            f.decorated(),
            if f.client().is_csd() {
                f.client().csd_extents()
            } else {
                [0; 4]
            },
        )
    });
    let bw = if decorated { border_width() } else { 0 };
    let bb = if decorated { bottom_border_width() } else { 0 };
    let th = if decorated { title_bar_height() } else { 0 };
    let top = top_for(bw, th);
    let [il, ir, it, ib] = inset;
    let client_rect = Rect::new(nx - il, ny - it, nw, nh);
    let frame_rect = Rect::new(
        nx - bw,
        ny - top,
        (nw + bw * 2 - il - ir).max(1),
        (nh + top + bb - it - ib).max(1),
    );
    let frame_id = if let Some(fw) = wm.frame_mut(id) {
        fw.set_frame_rect(frame_rect);
        fw.client_rect = client_rect;
        Some(fw.frame().id())
    } else {
        None
    };
    if let Some(b) = wm.backend() {
        if let Some(fid) = frame_id {
            let _ = b.configure_window(
                fid,
                &[
                    frame_rect.x as u32,
                    frame_rect.y as u32,
                    frame_rect.w as u32,
                    frame_rect.h as u32,
                ],
            );
        }
        let _ = b.configure_window(
            w,
            &[
                (bw - il) as u32,
                (top_for(bw, th) - it) as u32,
                nw as u32,
                nh as u32,
            ],
        );
        let _ = b.flush();
    }
}
