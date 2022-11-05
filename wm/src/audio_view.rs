use crate::audio::AudioState;
use crate::icon_dsl::{
    self, thick_line, COLOR_CRITICAL as COLOR_MUTED, COLOR_GOOD as COLOR_ACTIVE,
};
use antibox_core::backend::GraphicsContext;
use antibox_ui::theme;

fn margin() -> i16 {
    icon_dsl::slot_margin()
}

fn wave_thickness() -> i16 {
    icon_dsl::stroke_w()
}

fn glyph_h(h: u16) -> i16 {
    icon_dsl::glyph_zone_h(h)
}

fn content_w(h: u16) -> i16 {
    icon_dsl::slot_content_w(h)
}

fn box_w(h: u16) -> i16 {
    (content_w(h) * 2 / 9).max(2)
}

fn cone_w(h: u16) -> i16 {
    (content_w(h) / 3).max(3)
}

fn open_h(h: u16) -> i16 {
    (glyph_h(h) * 14 / 16).max(5)
}

fn inner_gap(h: u16) -> i16 {
    (content_w(h) / 9).max(1)
}

fn wave_zone_w(h: u16) -> i16 {
    (content_w(h) - box_w(h) - cone_w(h) - inner_gap(h) - wave_thickness()).max(2)
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
    let _ = g.set_foreground(colour);
    let _ = g.fill_polygon(&thick_line(x, y, x + w, y + h, t));
    let _ = g.fill_polygon(&thick_line(x, y + h, x + w, y, t));
}

fn draw_wave(g: &dyn GraphicsContext, cx: i16, cy: i16, r: i16, t: i16, colour: u32) {
    let _ = g.set_foreground(colour);
    let _ = g.fill_polygon(&thick_line(cx, cy - r, cx + r, cy, t));
    let _ = g.fill_polygon(&thick_line(cx + r, cy, cx, cy + r, t));
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
        let mut s = String::new();
        if state.sink_muted {
            s.push_str("Muted");
        } else {
            s.push_str(&format!("Volume: {}%", state.volume));
        }
        if state.source_muted {
            s.push_str("\nMic: muted");
        }
        if !state.readers.is_empty() {
            s.push_str(&format!("\nRecording: {}", state.readers.join(", ")));
        }
        s
    }

    pub fn draw(&self, g: &dyn GraphicsContext, x0: i16, h: u16) {
        let state = match &self.state {
            Some(s) => s,
            None => return,
        };
        let muted = state.sink_muted;
        let shape_colour = if muted { COLOR_MUTED } else { theme::text() };
        let outline = theme::shadow();

        let gh = glyph_h(h);
        let top = icon_dsl::glyph_top();
        let cy = top + gh / 2;
        let sw = box_w(h) + cone_w(h);
        let sh = open_h(h);
        let bx = x0 + margin();

        icon_dsl::SPEAKER_ICON.draw_outlined(
            g,
            bx,
            cy - sh / 2,
            sw as u16,
            sh as u16,
            shape_colour,
            outline,
        );

        let zone = wave_zone_w(h);
        let wave_x = bx + sw + inner_gap(h);

        if muted {
            draw_mute_x(g, wave_x, cy - zone, zone, zone * 2, COLOR_MUTED);
        } else {
            let lit = wave_count(state.volume);
            let t = wave_thickness();
            let min_r = (zone / 3).max(2);
            let step = ((zone - min_r) / 2).max(2);
            for i in 0..3 {
                let r = (min_r + step * i).min(zone).max(2);
                let colour = if i < lit { COLOR_ACTIVE } else { outline };
                draw_wave(g, wave_x, cy, r, t, colour);
            }
        }
    }
}

#[cfg(test)]
#[path = "audio_view_tests.rs"]
mod tests;
