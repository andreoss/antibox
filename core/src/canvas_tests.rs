use super::*;

fn px(c: &SoftCanvas, x: i16, y: i16) -> [u8; 4] {
    let o = (y as usize * c.width() as usize + x as usize) * 4;
    let d = c.as_rgba();
    [d[o], d[o + 1], d[o + 2], d[o + 3]]
}

#[test]
fn new_is_transparent() {
    let c = SoftCanvas::new(4, 4);
    assert_eq!(px(&c, 2, 2), [0, 0, 0, 0]);
}

#[test]
fn fill_rect_is_opaque_and_clipped() {
    let mut c = SoftCanvas::new(4, 4);
    c.fill_rect(2, 2, 10, 10, 0x00FF_8040);
    assert_eq!(px(&c, 3, 3), [0xFF, 0x80, 0x40, 0xFF]);
    assert_eq!(px(&c, 0, 0), [0, 0, 0, 0]);
}

#[test]
fn fill_rect_negative_origin_clips() {
    let mut c = SoftCanvas::new(4, 4);
    c.fill_rect(-2, -2, 4, 4, 0x0000_FF00);
    assert_eq!(px(&c, 0, 0), [0, 0xFF, 0, 0xFF]);
    assert_eq!(px(&c, 2, 2), [0, 0, 0, 0]);
}

#[test]
fn draw_line_diagonal() {
    let mut c = SoftCanvas::new(4, 4);
    c.draw_line(0, 0, 3, 3, 0x00FF_FFFF);
    for i in 0..4 {
        assert_eq!(px(&c, i, i), [0xFF, 0xFF, 0xFF, 0xFF]);
    }
}

#[test]
fn blend_half_coverage() {
    let mut c = SoftCanvas::new(2, 2);
    c.fill_rect(0, 0, 2, 2, 0x0000_0000);
    c.blend(0, 0, [255, 255, 255], 128);
    let p = px(&c, 0, 0);
    assert!(p[0] > 120 && p[0] < 135, "expected ~half blend, got {p:?}");
    assert_eq!(p[3], 0xFF);
}

#[test]
fn draw_pixmap_respects_alpha() {
    let mut c = SoftCanvas::new(2, 2);
    c.fill_rect(0, 0, 2, 2, 0x0000_0000);
    let pm = PixmapData::new(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 0]);
    c.draw_pixmap(0, 0, &pm);
    assert_eq!(px(&c, 0, 0), [255, 0, 0, 255]);
    assert_eq!(px(&c, 1, 0), [0, 0, 0, 255]);
}

