use antibox_core::backend::{FontSpec, GraphicsContext};
use antibox_core::point::Dimension;
use antibox_core::rect::Rect;
use antibox_wayland::buffers::BufferStore;
use antibox_wayland::graphics::WaylandGraphics;
use antibox_ui::theme;

fn main() {
    antibox::wayland_glyphs::install();

    let store = BufferStore::new();
    let (w, h) = (520u16, 300u16);
    let id = store.create(w, h);
    let g = WaylandGraphics::new(store.clone(), id);

    let _ = g.set_font(&FontSpec::ui(13));

    let _ = g.set_foreground(0x0000_5A8C);
    let _ = g.fill_rect(0, 0, w, h);

    theme::window_frame(&g, 22, Dimension::px(360, 180), 0x00C0_C0C0, 0x0000_0080, 22, true);
    let _ = g.set_foreground(0x0000_0080);
    let _ = g.fill_rect(2, 2, 356, 20);
    let _ = g.set_foreground(0x00FF_FFFF);
    let _ = g.draw_text_transparent(8, 17, "antibox on wayland");

    theme::button_surface(&g, Rect::px(20, 60, 120, 28), theme::Fill::new(0x00C0_C0C0, false));
    let _ = g.set_foreground(0x0000_0000);
    let _ = g.draw_text_transparent(34, 79, "Button");

    theme::bevel(&g, 20, 110, 320, 50, true);
    let _ = g.set_foreground(0x0000_0000);
    let _ = g.draw_text_transparent(28, 140, "software canvas + freetype glyphs");

    let _ = g.set_foreground(0x00C0_C0C0);
    let _ = g.fill_rect(0, h as i16 - 24, w, 24);
    theme::bevel(&g, 4, h as i16 - 20, 90, 16, false);
    let _ = g.set_foreground(0x0000_0000);
    let _ = g.draw_text_transparent(10, h as i16 - 8, "Taskbar");

    let path = std::env::args().nth(1).unwrap_or_else(|| "wayland_shot.ppm".to_string());
    let pm = store.snapshot(id).expect("snapshot");
    let mut out = format!("P6\n{} {}\n255\n", pm.width, pm.height).into_bytes();
    for px in pm.data.chunks_exact(4) {
        out.extend_from_slice(&[px[0], px[1], px[2]]);
    }
    std::fs::write(&path, out).expect("write ppm");
    println!("wrote {path} ({}x{})", pm.width, pm.height);
}
