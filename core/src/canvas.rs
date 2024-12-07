use crate::backend::PixmapData;
use crate::rect::Rect;

#[inline]
pub const fn rgb(pixel: u32) -> [u8; 3] {
    [
        ((pixel >> 16) & 0xFF) as u8,
        ((pixel >> 8) & 0xFF) as u8,
        (pixel & 0xFF) as u8,
    ]
}

#[derive(Clone)]
pub struct SoftCanvas {
    width: u16,
    height: u16,
    data: Vec<u8>,
    clip: Vec<Rect>,
}

impl SoftCanvas {
    pub fn new(width: u16, height: u16) -> Self {
        let w = width.max(1);
        let h = height.max(1);
        Self {
            width: w,
            height: h,
            data: vec![0u8; w as usize * h as usize * 4],
            clip: Vec::new(),
        }
    }

    pub const fn width(&self) -> u16 {
        self.width
    }

    pub const fn height(&self) -> u16 {
        self.height
    }

    pub fn as_rgba(&self) -> &[u8] {
        &self.data
    }

    pub fn to_pixmap_data(&self) -> PixmapData {
        PixmapData::new(self.width, self.height, self.data.clone())
    }

    pub fn push_clip(&mut self, rect: Rect) {
        let next = match self.clip.last() {
            Some(top) => top.intersection(&rect),
            None => rect,
        };
        self.clip.push(next);
    }

    pub fn pop_clip(&mut self) {
        self.clip.pop();
    }

    #[inline]
    fn clipped_out(&self, x: i32, y: i32) -> bool {
        match self.clip.last() {
            Some(r) => x < r.x || y < r.y || x >= r.x + r.w || y >= r.y + r.h,
            None => false,
        }
    }

    #[inline]
    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        if self.clipped_out(x, y) {
            return None;
        }
        Some((y as usize * self.width as usize + x as usize) * 4)
    }

    #[inline]
    pub fn put(&mut self, x: i32, y: i32, colour: [u8; 3]) {
        if let Some(o) = self.idx(x, y) {
            self.data[o] = colour[0];
            self.data[o + 1] = colour[1];
            self.data[o + 2] = colour[2];
            self.data[o + 3] = 0xFF;
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, colour: [u8; 3], a: u32) {
        if a == 0 {
            return;
        }
        if a >= 255 {
            self.put(x, y, colour);
            return;
        }
        if let Some(o) = self.idx(x, y) {
            let ia = 255 - a;
            let dst = &mut self.data[o..o + 4];
            for (d, &s) in dst[..3].iter_mut().zip(colour.iter()) {
                *d = ((s as u32 * a + *d as u32 * ia) / 255) as u8;
            }
            let da = dst[3] as u32;
            dst[3] = (a + da * ia / 255).min(255) as u8;
        }
    }

    pub fn fill_rect(&mut self, x: i16, y: i16, w: u16, h: u16, colour: u32) {
        let c = rgb(colour);
        let x0 = x as i32;
        let y0 = y as i32;
        let x1 = x0 + w as i32;
        let y1 = y0 + h as i32;
        let mut cx0 = x0.max(0);
        let mut cy0 = y0.max(0);
        let mut cx1 = x1.min(self.width as i32);
        let mut cy1 = y1.min(self.height as i32);
        if let Some(r) = self.clip.last() {
            cx0 = cx0.max(r.x);
            cy0 = cy0.max(r.y);
            cx1 = cx1.min(r.x + r.w);
            cy1 = cy1.min(r.y + r.h);
        }
        for py in cy0..cy1 {
            let row = py as usize * self.width as usize;
            for px in cx0..cx1 {
                let o = (row + px as usize) * 4;
                self.data[o] = c[0];
                self.data[o + 1] = c[1];
                self.data[o + 2] = c[2];
                self.data[o + 3] = 0xFF;
            }
        }
    }

    pub fn draw_rect(&mut self, x: i16, y: i16, w: u16, h: u16, colour: u32) {
        let c = rgb(colour);
        let (x0, y0) = (x as i32, y as i32);
        let x1 = x0 + w as i32;
        let y1 = y0 + h as i32;
        for px in x0..=x1 {
            self.put(px, y0, c);
            self.put(px, y1, c);
        }
        for py in y0..=y1 {
            self.put(x0, py, c);
            self.put(x1, py, c);
        }
    }

    pub fn draw_point(&mut self, x: i16, y: i16, colour: u32) {
        self.put(x as i32, y as i32, rgb(colour));
    }

    pub fn fill_polygon(&mut self, pts: &[(i16, i16)], colour: u32) {
        if pts.len() < 3 {
            return;
        }
        let c = rgb(colour);
        let min_y = pts.iter().map(|p| p.1).min().unwrap() as i32;
        let max_y = pts.iter().map(|p| p.1).max().unwrap() as i32;
        for row in min_y..=max_y {
            let yr = row as f64;
            let mut xs: Vec<f64> = Vec::new();
            for i in 0..pts.len() {
                let (ax, ay) = (pts[i].0 as f64, pts[i].1 as f64);
                let j = (i + 1) % pts.len();
                let (bx, by) = (pts[j].0 as f64, pts[j].1 as f64);
                if (ay <= yr && yr <= by) || (by <= yr && yr <= ay) {
                    if (by - ay).abs() < f64::EPSILON {
                        xs.push(ax.min(bx));
                        xs.push(ax.max(bx));
                    } else {
                        xs.push(ax + (yr - ay) / (by - ay) * (bx - ax));
                    }
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            xs.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
            for span in xs.chunks(2) {
                let [a, b] = span else { continue };
                let x0 = a.ceil() as i32;
                let x1 = b.ceil() as i32 - 1;
                for x in x0..=x1 {
                    self.put(x, row, c);
                }
            }
        }
    }

    pub fn draw_arc(&mut self, r: Rect, a1: i32, a2: i32, colour: u32) {
        let (x, y, w, h) = r.as_px();
        if w == 0 || h == 0 {
            return;
        }
        let c = rgb(colour);
        if w == h && w % 2 == 0 {
            self.draw_circle_arc(x, y, w, a1, a2, c);
            return;
        }
        let cx = x as f64 + w as f64 / 2.0;
        let cy = y as f64 + h as f64 / 2.0;
        let rx = w as f64 / 2.0;
        let ry = h as f64 / 2.0;
        let steps = ((rx.max(ry) * 8.0).ceil() as i32).max(16);
        let start = a1 as f64 / 64.0;
        let extent = a2 as f64 / 64.0;
        for i in 0..=steps {
            let ang = (start + extent * i as f64 / steps as f64).to_radians();
            self.put(
                (cx + rx * ang.cos() - 0.5).round() as i32,
                (cy - ry * ang.sin() - 0.5).round() as i32,
                c,
            );
        }
    }

    fn draw_circle_arc(&mut self, x: i16, y: i16, d: u16, a1: i32, a2: i32, c: [u8; 3]) {
        let r = (d / 2) as i32;
        let cx = x as i32 + r;
        let cy = y as i32 + r;
        let full = a2.abs() >= 360 * 64;
        let s = a1 as f64 / 64.0;
        let e = s + a2 as f64 / 64.0;
        let (lo, hi) = if a2 >= 0 { (s, e) } else { (e, s) };
        let mut pts: Vec<(i32, i32)> = Vec::with_capacity(r as usize * 8 + 8);
        let mut xx = r;
        let mut yy = 0;
        let mut err = 1 - r;
        while xx >= yy {
            pts.extend_from_slice(&[
                (cx + xx, cy + yy),
                (cx - xx, cy + yy),
                (cx + xx, cy - yy),
                (cx - xx, cy - yy),
                (cx + yy, cy + xx),
                (cx - yy, cy + xx),
                (cx + yy, cy - xx),
                (cx - yy, cy - xx),
            ]);
            yy += 1;
            if err < 0 {
                err += 2 * yy + 1;
            } else {
                xx -= 1;
                err += 2 * (yy - xx) + 1;
            }
        }
        for (px, py) in pts {
            if !full {
                let mut ang = ((cy - py) as f64).atan2((px - cx) as f64).to_degrees();
                while ang < lo - 1e-6 {
                    ang += 360.0;
                }
                if ang > hi + 1e-6 {
                    continue;
                }
            }
            self.put(px, py, c);
        }
    }

    pub fn fill_arc(&mut self, r: Rect, a1: i32, a2: i32, colour: u32) {
        let (x, y, w, h) = r.as_px();
        if w == 0 || h == 0 {
            return;
        }
        let c = rgb(colour);
        let cx = x as f64 + w as f64 / 2.0;
        let cy = y as f64 + h as f64 / 2.0;
        let rx = w as f64 / 2.0;
        let ry = h as f64 / 2.0;
        let full = a2.abs() >= 360 * 64;
        let (s, e) = if a2 >= 0 {
            (a1 as f64 / 64.0, (a1 + a2) as f64 / 64.0)
        } else {
            ((a1 + a2) as f64 / 64.0, a1 as f64 / 64.0)
        };
        for row in y as i32..y as i32 + h as i32 {
            for col in x as i32..x as i32 + w as i32 {
                let dx = (col as f64 + 0.5 - cx) / rx;
                let dy = (cy - (row as f64 + 0.5)) / ry;
                if dx * dx + dy * dy > 1.0 {
                    continue;
                }
                if !full {
                    let mut ang = dy.atan2(dx).to_degrees();
                    while ang < s {
                        ang += 360.0;
                    }
                    if ang > e {
                        continue;
                    }
                }
                self.put(col, row, c);
            }
        }
    }

    pub fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, colour: u32) {
        let c = rgb(colour);
        let (mut x0, mut y0) = (x1 as i32, y1 as i32);
        let (xe, ye) = (x2 as i32, y2 as i32);
        let dx = (xe - x0).abs();
        let dy = -(ye - y0).abs();
        let sx = if x0 < xe { 1 } else { -1 };
        let sy = if y0 < ye { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.put(x0, y0, c);
            if x0 == xe && y0 == ye {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn draw_3d_rect(&mut self, r: Rect, sunken: bool, fg: u32, bg: u32) {
        let (x, y, w, h) = r.as_px();
        if w == 0 || h == 0 {
            return;
        }
        let (tl, br) = if sunken { (fg, bg) } else { (bg, fg) };
        let x1 = x + w as i16 - 1;
        let y1 = y + h as i16 - 1;
        self.draw_line(x, y, x1, y, tl);
        self.draw_line(x, y, x, y1, tl);
        self.draw_line(x, y1, x1, y1, br);
        self.draw_line(x1, y, x1, y1, br);
    }

    pub fn fill_gradient_v(&mut self, x: i16, y: i16, w: u16, h: u16, top: u32, bottom: u32) {
        if h == 0 {
            return;
        }
        for (off, bh, colour) in crate::backend::gradient_v_bands(h, top, bottom) {
            let bh = bh.min(h.saturating_sub(off));
            self.fill_rect(x, y + off as i16, w, bh, colour);
        }
    }

    pub fn fill_gradient_h(&mut self, x: i16, y: i16, w: u16, h: u16, left: u32, right: u32) {
        if w == 0 {
            return;
        }
        for (off, bw, colour) in crate::backend::gradient_h_bands(w, left, right) {
            self.fill_rect(x + off as i16, y, bw, h, colour);
        }
    }

    pub fn draw_pixmap(&mut self, x: i16, y: i16, pm: &PixmapData) {
        let stride = pm.width as usize * 4;
        for row in 0..pm.height as usize {
            let dy = y as i32 + row as i32;
            for col in 0..pm.width as usize {
                let so = row * stride + col * 4;
                let a = pm.data[so + 3] as u32;
                if a == 0 {
                    continue;
                }
                let c = [pm.data[so], pm.data[so + 1], pm.data[so + 2]];
                self.blend(x as i32 + col as i32, dy, c, a);
            }
        }
    }

}

#[cfg(test)]
#[path = "canvas_tests.rs"]
mod tests;
