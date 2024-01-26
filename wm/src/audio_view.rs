use crate::audio::AudioState;
use crate::icon_dsl::{
    self, thick_line, COLOR_CRITICAL as COLOR_MUTED, COLOR_GOOD as COLOR_ACTIVE,
};
use antibox_core::backend::GraphicsContext;
use antibox_ui::theme;

fn margin() -> i16 {
    icon_dsl::slot_margin()
}

fn glyph_h(h: u16) -> i16 {
    icon_dsl::glyph_zone_h(h)
}

fn content_w(h: u16) -> i16 {
    icon_dsl::slot_content_w(h)
}

fn spk_w(h: u16) -> i16 {
    (content_w(h) * 2 / 5).max(4)
}

fn inner_gap(h: u16) -> i16 {
    (content_w(h) / 12).max(1)
}

fn bar_zone_w(h: u16) -> i16 {
    (content_w(h) - spk_w(h) - inner_gap(h)).max(5)
}

fn bar_w(h: u16) -> i16 {
    ((bar_zone_w(h) - 2) / 3).max(2)
}

fn wave_count(volume: i32) -> i16 {
    if volume <= 0 {
        0
    } else if volume < 34 {
        1
    } else if volume < 67 {
        2
    } else {
        3
    }
}

fn draw_mute_x(g: &dyn GraphicsContext, x: i16, y: i16, w: i16, h: i16, colour: u32) {
    let t = (w.min(h) / 4).max(2);
    let m = (t + 2) / 2;
    let (x0, x1) = (x + m, x + w - m);
    let _ = g.set_foreground(theme::light());
    let _ = g.fill_polygon(&thick_line(x0, y, x1, y + h, t + 2));
    let _ = g.fill_polygon(&thick_line(x0, y + h, x1, y, t + 2));
    let _ = g.set_foreground(colour);
    let _ = g.fill_polygon(&thick_line(x0, y, x1, y + h, t));
    let _ = g.fill_polygon(&thick_line(x0, y + h, x1, y, t));
}

pub struct AudioView {
    pub state: Option<AudioState>,
}

impl AudioView {
    pub fn new(state: Option<AudioState>) -> Self {
        Self { state }
    }

    pub fn present(&self) -> bool {
        self.state.is_some()
    }

    pub fn natural_width(h: u16) -> u16 {
        h
    }

    pub fn tooltip(&self) -> String {
        let state = match &self.state {
            Some(s) => s,
            None => return String::new(),
        };
        if state.sink_muted {
            "Muted".to_string()
        } else {
            format!("Volume: {}%", state.volume)
        }
    }

    pub fn draw(&self, g: &dyn GraphicsContext, x0: i16, h: u16) {
        let state = match &self.state {
            Some(s) => s,
            None => return,
        };
        let muted = state.sink_muted;
        let shape_colour = theme::text();
        let outline = theme::shadow();

        let gh = glyph_h(h);
        let gy = icon_dsl::glyph_top();
        let sw = spk_w(h);
        let sx = x0 + margin();

        icon_dsl::SPEAKER_ICON.draw_outlined(
            g,
            sx,
            gy,
            sw as u16,
            gh as u16,
            shape_colour,
            outline,
        );

        let bx0 = sx + sw + inner_gap(h);

        if muted {
            let cw = content_w(h);
            draw_mute_x(g, sx, gy, cw, gh, COLOR_MUTED);
        } else {
            let lit = wave_count(state.volume);
            let bw = bar_w(h);
            let gap = (bw / 3).max(1);
            for i in 0..3i16 {
                let bh = (gh * (2 + i) / 4).max(3);
                let bx = bx0 + i * (bw + gap);
                let by = gy + gh - bh;
                let colour = if i < lit {
                    COLOR_ACTIVE
                } else if bw >= 5 {
                    theme::graph_bg()
                } else {
                    theme::shadow()
                };
                let _ = g.set_foreground(colour);
                let _ = g.fill_rect(bx, by, bw as u16, bh as u16);
                if bw >= 5 {
                    icon_dsl::stroke_rect(g, bx, by, bw as u16, bh as u16, outline);
                }
            }
        }
    }
}

pub struct MicView {
    pub state: Option<AudioState>,
}

impl MicView {
    pub fn new(state: Option<AudioState>) -> Self {
        Self { state }
    }

    pub fn present(&self) -> bool {
        self.state.as_ref().map_or(false, |s| !s.readers.is_empty())
    }

    pub fn natural_width(h: u16) -> u16 {
        h
    }

    pub fn tooltip(&self) -> String {
        let state = match &self.state {
            Some(s) => s,
            None => return String::new(),
        };
        let mut s = format!("Recording: {}", state.readers.join(", "));
        if state.source_muted {
            s.push_str("\nMic: muted");
        }
        s
    }

    pub fn draw(&self, g: &dyn GraphicsContext, x0: i16, h: u16) {
        let state = match &self.state {
            Some(s) => s,
            None => return,
        };
        if state.readers.is_empty() {
            return;
        }
        let gh = glyph_h(h);
        let gy = icon_dsl::glyph_top();
        let gw = gh.min(content_w(h)).max(6);
        let gx = x0 + (h as i16 - gw) / 2;
        let colour = if state.source_muted {
            theme::text()
        } else {
            COLOR_ACTIVE
        };
        if gw >= 8 {
            icon_dsl::MIC_ICON.draw_outlined(g, gx, gy, gw as u16, gh as u16, colour, theme::shadow());
        } else {
            icon_dsl::MIC_ICON.draw(g, gx, gy, gw as u16, gh as u16, colour);
        }
        if state.source_muted {
            let t = (gw / 5).max(2);
            let _ = g.set_foreground(theme::light());
            let _ = g.fill_polygon(&thick_line(gx, gy + gh, gx + gw, gy, t + 2));
            let _ = g.set_foreground(COLOR_MUTED);
            let _ = g.fill_polygon(&thick_line(gx, gy + gh, gx + gw, gy, t));
        }
    }
}

#[cfg(test)]
#[path = "audio_view_tests.rs"]
mod tests;
