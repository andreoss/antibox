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
    let _ = g.set_foreground(colour);
    let _ = g.fill_polygon(&thick_line(x, y, x + w, y + h, t));
    let _ = g.fill_polygon(&thick_line(x, y + h, x + w, y, t));
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

    fn draw_mic_badge(&self, g: &dyn GraphicsContext, x0: i16, h: u16, state: &AudioState) {
        if !state.source_muted && state.readers.is_empty() {
            return;
        }
        let badge = (glyph_h(h) * 3 / 4).max(8);
        let bx = x0 + h as i16 - margin() - badge;
        let by = (icon_dsl::glyph_top() + glyph_h(h) - badge).max(0);
        let colour = if state.source_muted {
            COLOR_MUTED
        } else {
            COLOR_ACTIVE
        };
        let _ = g.set_foreground(theme::tray_face());
        let _ = g.fill_rect(bx, by, badge as u16, badge as u16);
        icon_dsl::stroke_rect(g, bx, by, badge as u16, badge as u16, theme::shadow());
        icon_dsl::MIC_ICON.draw_outlined(
            g,
            bx,
            by,
            badge as u16,
            badge as u16,
            colour,
            theme::shadow(),
        );
        if state.source_muted {
            let t = (badge / 3).max(2);
            let _ = g.set_foreground(theme::text());
            let _ = g.fill_polygon(&thick_line(bx, by + badge, bx + badge, by, t));
        }
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
        let zone = bar_zone_w(h);

        if muted {
            let mh = (gh * 3 / 4).max(5);
            draw_mute_x(g, bx0, gy + (gh - mh) / 2, zone, mh, COLOR_MUTED);
        } else {
            let lit = wave_count(state.volume);
            let bw = bar_w(h);
            for i in 0..3i16 {
                let bh = (gh * (2 + i) / 4).max(3);
                let bx = bx0 + i * (bw + 1);
                let by = gy + gh - bh;
                let colour = if i < lit {
                    COLOR_ACTIVE
                } else {
                    theme::graph_bg()
                };
                let _ = g.set_foreground(colour);
                let _ = g.fill_rect(bx, by, bw as u16, bh as u16);
                icon_dsl::stroke_rect(g, bx, by, bw as u16, bh as u16, outline);
            }
        }

        self.draw_mic_badge(g, x0, h, state);
    }
}

#[cfg(test)]
#[path = "audio_view_tests.rs"]
mod tests;
