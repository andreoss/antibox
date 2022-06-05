pub const MAX_SAMPLES: usize = 256;
pub const INSET: i16 = 3;

fn col_w() -> u16 {
    antibox_core::scale::scaled(1).max(1) as u16
}

pub struct Plot {
    pub cw: u16,
    pub inset: i16,
    pub gh: i16,
    n: usize,
    cols: usize,
}

pub fn plot(w: u16, h: u16, count: usize) -> Option<Plot> {
    let cw = col_w();
    if cw == 0 {
        return None;
    }
    let inset = INSET;
    let gh = (h as i16 - inset * 2).max(1);
    let gw = (w as i16 - inset * 2).max(1) as u16;
    let cols = (gw / cw).max(1) as usize;
    let n = count.min(cols);
    Some(Plot {
        cw,
        inset,
        gh,
        n,
        cols,
    })
}

impl Plot {
    pub fn count(&self) -> usize {
        self.n
    }
    pub fn top(&self) -> i16 {
        self.inset
    }
    pub fn bottom(&self) -> i16 {
        self.inset + self.gh
    }
    pub fn col_x(&self, col: usize) -> i16 {
        self.inset + ((self.cols - self.n + col) as u16 * self.cw) as i16
    }

    pub fn bar_up_heat(
        &self,
        g: &dyn antibox_core::backend::GraphicsContext,
        x: i16,
        y: i16,
        barh: i16,
        colour: antibox_core::colour::Colour,
    ) -> i16 {
        if barh <= 0 {
            return y;
        }
        let gh = self.gh.max(1) as f32;
        let bottom = self.bottom() - 1;
        for row in (y - barh + 1)..=y {
            let f = (bottom - row) as f32 / gh;
            let _ = g.set_foreground(antibox_core::colour::lerp(
                colour,
                antibox_ui::theme::graph_heat(),
                f,
            ));
            let _ = g.fill_rect(x, row, self.cw, 1);
        }
        y - barh
    }

    pub fn bar_down_heat(
        &self,
        g: &dyn antibox_core::backend::GraphicsContext,
        x: i16,
        barh: i16,
        colour: antibox_core::colour::Colour,
    ) {
        if barh <= 0 {
            return;
        }
        let gh = self.gh.max(1) as f32;
        let top = self.top();
        for row in top..top + barh {
            let f = (row - top) as f32 / gh;
            let _ = g.set_foreground(antibox_core::colour::lerp(
                colour,
                antibox_ui::theme::graph_heat(),
                f,
            ));
            let _ = g.fill_rect(x, row, self.cw, 1);
        }
    }

    pub fn bar_up(
        &self,
        g: &dyn antibox_core::backend::GraphicsContext,
        x: i16,
        y: i16,
        barh: i16,
        colour: antibox_core::colour::Colour,
    ) -> i16 {
        if barh > 0 {
            let _ = g.set_foreground(colour);
            let _ = g.fill_rect(x, y - barh + 1, self.cw, barh as u16);
            y - barh
        } else {
            y
        }
    }
}

pub struct Samples<T> {
    buf: [T; MAX_SAMPLES],
    head: usize,
    count: usize,
}

impl<T: Copy + Default> Default for Samples<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Default> Samples<T> {
    pub fn new() -> Self {
        Samples {
            buf: [T::default(); MAX_SAMPLES],
            head: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, v: T) {
        self.buf[self.head] = v;
        self.head = (self.head + 1) % MAX_SAMPLES;
        if self.count < MAX_SAMPLES {
            self.count += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn latest(&self) -> Option<&T> {
        if self.count == 0 {
            None
        } else {
            Some(&self.buf[(self.head + MAX_SAMPLES - 1) % MAX_SAMPLES])
        }
    }

    pub fn at(&self, col: usize, n: usize) -> &T {
        &self.buf[(self.head + MAX_SAMPLES - n + col) % MAX_SAMPLES]
    }
}

pub fn pref_h() -> u32 {
    antibox_ui::metrics::button_height() as u32
}

macro_rules! impl_status_applet {
    ($t:ty) => {
        impl crate::applet::Applet for $t {
            fn window(&self) -> &dyn antibox_core::backend::WindowHandle {
                &*self.window
            }
            fn paint(&self, g: &dyn antibox_core::backend::GraphicsContext) {
                let _ = g.set_foreground(antibox_ui::theme::graph_bg());
                let _ = g.fill_rect(0, 0, self.w, self.h);
                antibox_ui::theme::well(g, 0, 0, self.w, self.h);
                self.paint_graph(g);
            }
            fn preferred_width(&self) -> u32 {
                self.pref_w as u32
            }
            fn preferred_height(&self) -> u32 {
                crate::status_graph::pref_h()
            }
            fn handle_click(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
                None
            }
            fn set_graph_width(&mut self, w: u16) {
                let w = antibox_core::scale::scaled(w as i32) as u16;
                self.pref_w = w;
                self.w = w;
            }
            impl_applet_tooltip!(tooltip);
            fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
                self.w = w;
                self.h = h;
                let _ = self
                    .window
                    .configure(Some(x as i32), Some(y as i32), Some(w), Some(h));
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

pub(crate) use impl_status_applet;

#[cfg(test)]
#[path = "status_graph_tests.rs"]
mod tests;
