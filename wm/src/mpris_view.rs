use crate::icon_dsl::{self, COLOR_GOOD as COLOR_ACTIVE};
use crate::mpris::{format_time, Command, Playback, Player};
use antibox_core::backend::{global_text_width, FontSpec, GraphicsContext};
use antibox_ui::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaButton {
    Previous,
    PlayPause,
    Next,
}

impl MediaButton {
    pub const fn command(self) -> Command {
        match self {
            Self::Previous => Command::Previous,
            Self::PlayPause => Command::PlayPause,
            Self::Next => Command::Next,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaSlot {
    Button(MediaButton),
    Timing,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MprisView {
    pub player: Option<Player>,
    pub position: i64,
}

fn text_width(text: &str) -> i32 {
    let spec = FontSpec::ui(antibox_ui::metrics::font_pt());
    global_text_width(&spec, text).map_or_else(
        || antibox_ui::metrics::text_w(text.chars().count()),
        |w| w as i32,
    )
}

impl MprisView {
    pub const fn new(player: Option<Player>, position: i64) -> Self {
        Self { player, position }
    }

    pub fn present(&self) -> bool {
        !self.slots().is_empty()
    }

    pub fn can_seek(&self) -> bool {
        self.player.as_ref().is_some_and(|p| p.can_seek)
    }

    pub fn playing(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(|p| p.status == Playback::Playing)
    }

    pub fn slots(&self) -> Vec<MediaSlot> {
        let Some(player) = &self.player else { return Vec::new() };
        let mut out = Vec::new();
        if player.can_control && player.can_previous {
            out.push(MediaSlot::Button(MediaButton::Previous));
        }
        if player.can_control {
            out.push(MediaSlot::Button(MediaButton::PlayPause));
        }
        if !self.timing().is_empty() {
            out.push(MediaSlot::Timing);
        }
        if player.can_control && player.can_next {
            out.push(MediaSlot::Button(MediaButton::Next));
        }
        out
    }

    pub fn timing(&self) -> String {
        let Some(player) = &self.player else { return String::new() };
        let length = player.track.length;
        let position = self.position.max(0);
        if length > 0 && position > 0 {
            format!("{} / {}", format_time(position), format_time(length))
        } else if length > 0 {
            format_time(length)
        } else if position > 0 {
            format_time(position)
        } else {
            String::new()
        }
    }

    pub fn slot_width(&self, slot: MediaSlot, h: u16) -> u16 {
        match slot {
            MediaSlot::Button(_) => h,
            MediaSlot::Timing => {
                (text_width(&self.timing()) + antibox_ui::metrics::pad() * 2).max(1) as u16
            }
        }
    }

    pub fn natural_width(&self, h: u16) -> u16 {
        self.slots()
            .into_iter()
            .map(|s| self.slot_width(s, h))
            .sum()
    }

    pub fn slot_at(&self, x: i16, h: u16) -> Option<MediaSlot> {
        let mut edge = 0i16;
        for slot in self.slots() {
            let w = self.slot_width(slot, h) as i16;
            if x >= edge && x < edge + w {
                return Some(slot);
            }
            edge += w;
        }
        None
    }

    pub fn tooltip(&self) -> String {
        self.player
            .as_ref()
            .map_or_else(String::new, Player::tooltip)
    }

    pub fn slot_tooltip(&self, slot: MediaSlot) -> String {
        match slot {
            MediaSlot::Button(MediaButton::Previous) => "Previous track".to_string(),
            MediaSlot::Button(MediaButton::Next) => "Next track".to_string(),
            _ => self.tooltip(),
        }
    }

    fn icon(&self, button: MediaButton) -> &'static icon_dsl::Icon {
        match button {
            MediaButton::Previous => &icon_dsl::MEDIA_PREVIOUS_ICON,
            MediaButton::Next => &icon_dsl::MEDIA_NEXT_ICON,
            MediaButton::PlayPause if self.playing() => &icon_dsl::MEDIA_PAUSE_ICON,
            MediaButton::PlayPause => &icon_dsl::MEDIA_PLAY_ICON,
        }
    }

    pub fn draw_slot(&self, g: &dyn GraphicsContext, slot: MediaSlot, x0: i16, h: u16) {
        match slot {
            MediaSlot::Button(button) => self.draw_button(g, button, x0, h),
            MediaSlot::Timing => self.draw_timing(g, x0, h),
        }
    }

    fn draw_button(&self, g: &dyn GraphicsContext, button: MediaButton, x0: i16, h: u16) {
        let gh = icon_dsl::glyph_zone_h(h);
        let gy = icon_dsl::glyph_top();
        let gw = (gh * 3 / 4).max(5).min(icon_dsl::slot_content_w(h));
        let gx = x0 + (h as i16 - gw) / 2;
        let colour = if button == MediaButton::PlayPause && self.playing() {
            COLOR_ACTIVE
        } else {
            theme::text()
        };
        if gw >= 8 {
            self.icon(button)
                .draw_outlined(g, gx, gy, gw as u16, gh as u16, colour, theme::shadow());
        } else {
            self.icon(button)
                .draw(g, gx, gy, gw as u16, gh as u16, colour);
        }
    }

    fn draw_timing(&self, g: &dyn GraphicsContext, x0: i16, h: u16) {
        let text = self.timing();
        if text.is_empty() {
            return;
        }
        let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
        let _ = g.set_foreground(theme::text());
        let x = x0 + antibox_ui::metrics::pad() as i16;
        let baseline = antibox_ui::metrics::baseline(0, h as i32) as i16;
        let _ = g.draw_text_transparent(x, baseline, &text);
    }

    pub fn draw(&self, g: &dyn GraphicsContext, x0: i16, h: u16) {
        let mut x = x0;
        for slot in self.slots() {
            self.draw_slot(g, slot, x, h);
            x += self.slot_width(slot, h) as i16;
        }
    }
}

#[cfg(test)]
#[path = "mpris_view_tests.rs"]
mod tests;
