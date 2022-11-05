use antibox_core::backend::GraphicsContext;
use antibox_core::colour::Colour;

pub const COLOR_CRITICAL: Colour = 0xCC0000;
pub const COLOR_LOW: Colour = 0xCC8800;
pub const COLOR_MEDIUM: Colour = 0xBBBB00;
pub const COLOR_GOOD: Colour = 0x00AA00;

#[derive(Clone, Copy)]
pub enum Shape {
    Rect(f32, f32, f32, f32),
    Poly(&'static [(f32, f32)]),
    Line(f32, f32, f32, f32, f32),
}

pub struct Icon(pub &'static [Shape]);

fn scale(x: i16, y: i16, w: u16, h: u16, nx: f32, ny: f32) -> (i16, i16) {
    (
        x + (nx * w as f32).round() as i16,
        y + (ny * h as f32).round() as i16,
    )
}

pub fn thick_line(x1: i16, y1: i16, x2: i16, y2: i16, t: i16) -> [(i16, i16); 4] {
    let dx = (x2 - x1) as f32;
    let dy = (y2 - y1) as f32;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let ox = (-dy / len * t as f32 / 2.0).round() as i16;
    let oy = (dx / len * t as f32 / 2.0).round() as i16;
    [
        (x1 + ox, y1 + oy),
        (x2 + ox, y2 + oy),
        (x2 - ox, y2 - oy),
        (x1 - ox, y1 - oy),
    ]
}

pub fn stroke_rect(g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, colour: Colour) {
    let _ = g.set_foreground(colour);
    let _ = g.draw_rect(x, y, w, h);
}

pub fn stroke_polygon(g: &dyn GraphicsContext, pts: &[(i16, i16)], colour: Colour) {
    let _ = g.set_foreground(colour);
    for i in 0..pts.len() {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % pts.len()];
        let _ = g.draw_line(x1, y1, x2, y2);
    }
}

fn fill_rect_px(
    data: &mut [u8],
    stride: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    colour: Colour,
) {
    let (r, g, b) = (
        ((colour >> 16) & 0xff) as u8,
        ((colour >> 8) & 0xff) as u8,
        (colour & 0xff) as u8,
    );
    for py in y..(y + h).min(stride) {
        for px in x..(x + w).min(stride) {
            let i = (py * stride + px) * 4;
            data[i] = r;
            data[i + 1] = g;
            data[i + 2] = b;
            data[i + 3] = 255;
        }
    }
}

impl Icon {
    pub fn draw(&self, g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, colour: Colour) {
        self.draw_shaded(g, x, y, w, h, &[colour]);
    }

    pub fn draw_shaded(
        &self,
        g: &dyn GraphicsContext,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        colours: &[Colour],
    ) {
        for (i, shape) in self.0.iter().enumerate() {
            let _ = g.set_foreground(colours[i % colours.len().max(1)]);
            match *shape {
                Shape::Rect(nx, ny, nw, nh) => {
                    let (rx, ry) = scale(x, y, w, h, nx, ny);
                    let (rx1, ry1) = scale(x, y, w, h, nx + nw, ny + nh);
                    let _ = g.fill_rect(rx, ry, (rx1 - rx).max(1) as u16, (ry1 - ry).max(1) as u16);
                }
                Shape::Poly(pts) => {
                    let mapped: Vec<(i16, i16)> = pts
                        .iter()
                        .map(|&(nx, ny)| scale(x, y, w, h, nx, ny))
                        .collect();
                    let _ = g.fill_polygon(&mapped);
                }
                Shape::Line(x1, y1, x2, y2, t) => {
                    let (px1, py1) = scale(x, y, w, h, x1, y1);
                    let (px2, py2) = scale(x, y, w, h, x2, y2);
                    let thickness = (t * w.min(h) as f32).round().max(1.0) as i16;
                    let _ = g.fill_polygon(&thick_line(px1, py1, px2, py2, thickness));
                }
            }
        }
    }

    pub fn stroke(&self, g: &dyn GraphicsContext, x: i16, y: i16, w: u16, h: u16, colour: Colour) {
        for shape in self.0 {
            match *shape {
                Shape::Rect(nx, ny, nw, nh) => {
                    let (rx, ry) = scale(x, y, w, h, nx, ny);
                    let (rx1, ry1) = scale(x, y, w, h, nx + nw, ny + nh);
                    stroke_rect(
                        g,
                        rx,
                        ry,
                        (rx1 - rx).max(1) as u16,
                        (ry1 - ry).max(1) as u16,
                        colour,
                    );
                }
                Shape::Poly(pts) => {
                    let mapped: Vec<(i16, i16)> = pts
                        .iter()
                        .map(|&(nx, ny)| scale(x, y, w, h, nx, ny))
                        .collect();
                    stroke_polygon(g, &mapped, colour);
                }
                Shape::Line(x1, y1, x2, y2, t) => {
                    let (px1, py1) = scale(x, y, w, h, x1, y1);
                    let (px2, py2) = scale(x, y, w, h, x2, y2);
                    let thickness = (t * w.min(h) as f32).round().max(1.0) as i16;
                    stroke_polygon(g, &thick_line(px1, py1, px2, py2, thickness), colour);
                }
            }
        }
    }

    pub fn draw_outlined(
        &self,
        g: &dyn GraphicsContext,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
        fill: Colour,
        outline: Colour,
    ) {
        self.draw(g, x, y, w, h, fill);
        self.stroke(g, x, y, w, h, outline);
    }

    pub fn rasterize(&self, size: u16, bg: Colour, colours: &[Colour]) -> Vec<u8> {
        let s = size as usize;
        let mut data = vec![0u8; s * s * 4];
        fill_rect_px(&mut data, s, 0, 0, s, s, bg);
        for (i, shape) in self.0.iter().enumerate() {
            if let Shape::Rect(nx, ny, nw, nh) = *shape {
                let colour = colours[i % colours.len().max(1)];
                let x0 = (nx * size as f32).round() as usize;
                let y0 = (ny * size as f32).round() as usize;
                let x1 = ((nx + nw) * size as f32).round() as usize;
                let y1 = ((ny + nh) * size as f32).round() as usize;
                fill_rect_px(
                    &mut data,
                    s,
                    x0,
                    y0,
                    x1.saturating_sub(x0).max(1),
                    y1.saturating_sub(y0).max(1),
                    colour,
                );
            }
        }
        data
    }
}

pub fn slot_margin() -> i16 {
    antibox_ui::metrics::gap() as i16
}

pub fn stroke_w() -> i16 {
    (antibox_core::scale::scaled(2) as i16).max(2)
}

pub fn glyph_top() -> i16 {
    slot_margin()
}

pub fn glyph_zone_h(h: u16) -> i16 {
    ((h as i16) - 2 * slot_margin()).max(6)
}

pub fn slot_content_w(h: u16) -> i16 {
    ((h as i16) - 2 * slot_margin()).max(6)
}

const BOLT_PTS: [(f32, f32); 6] = [
    (0.60, 0.0),
    (0.05, 0.58),
    (0.45, 0.58),
    (0.30, 1.0),
    (0.95, 0.40),
    (0.55, 0.40),
];

pub const POWER_ICON: Icon = Icon(&[Shape::Poly(&BOLT_PTS)]);

pub const MIC_ICON: Icon = Icon(&[
    Shape::Rect(0.30, 0.05, 0.40, 0.45),
    Shape::Line(0.5, 0.50, 0.5, 0.75, 0.14),
    Shape::Line(0.25, 0.85, 0.75, 0.85, 0.14),
]);

const SPEAKER_PTS: [(f32, f32); 6] = [
    (0.0, 0.30),
    (0.43, 0.30),
    (1.0, 0.0),
    (1.0, 1.0),
    (0.43, 0.70),
    (0.0, 0.70),
];

pub const SPEAKER_ICON: Icon = Icon(&[Shape::Poly(&SPEAKER_PTS)]);

const BATTERY_PTS: [(f32, f32); 8] = [
    (0.32, 0.0),
    (0.68, 0.0),
    (0.68, 0.10),
    (0.84, 0.10),
    (0.84, 1.0),
    (0.16, 1.0),
    (0.16, 0.10),
    (0.32, 0.10),
];

pub const BATTERY_ICON: Icon = Icon(&[Shape::Poly(&BATTERY_PTS)]);

pub const WINDOW_ICON: Icon = Icon(&[
    Shape::Rect(0.125, 0.125, 0.75, 0.75),
    Shape::Rect(0.215, 0.215, 0.57, 0.16),
    Shape::Rect(0.215, 0.375, 0.57, 0.41),
]);

#[cfg(test)]
#[path = "icon_dsl_tests.rs"]
mod tests;
