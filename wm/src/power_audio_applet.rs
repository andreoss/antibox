 use antibox_core::error::Result;
use crate::applet::Applet;
use crate::audio::{detect, AudioSystem};
use crate::audio_view::{AudioView, MicView};
use crate::battery_view::BatteryView;
use crate::mpris::{self, Command};
use crate::mpris_view::{MediaButton, MediaSlot, MprisView};
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_ui::theme;
use std::sync::Arc;
use std::time::{Duration, Instant};

const LIST_INTERVAL: Duration = Duration::from_secs(10);
const SEEK_STEP: i64 = 5_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Segment {
    Battery,
    Audio,
    Mic,
    Media(MediaSlot),
}

pub struct PowerAudioApplet {
    conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    tooltip: Option<crate::tooltip::ToolTip>,
    hovered: Option<usize>,
    battery: BatteryView,
    audio_system: Box<dyn AudioSystem>,
    audio: AudioView,
    mic: MicView,
    media: MprisView,
    listed: Instant,
    bg: antibox_core::colour::Colour,
    w: u16,
    h: u16,
}

impl PowerAudioApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
    ) -> Result<Self> {
        let h = crate::status_graph::pref_h() as u16;
        let battery = BatteryView::new(true);
        let audio_system = detect();
        let state = audio_system.read();
        let audio = AudioView::new(state.clone());
        let mic = MicView::new(state);
        let media = MprisView::new(mpris::current(), mpris::position());
        let slot = Self::slot_w(h);
        let present = [battery.present(), audio.present(), mic.present()]
            .iter()
            .filter(|p| **p)
            .count() as u16;
        let w = slot * present.max(1) + media.natural_width(h);
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::POINTER_MOTION
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE,
        )?;
        Ok(Self {
            conn: Arc::clone(conn),
            window,
            tooltip: None,
            hovered: None,
            battery,
            audio_system,
            audio,
            mic,
            media,
            listed: Instant::now(),
            bg: theme::tray_face(),
            w,
            h,
        })
    }

    const fn slot_w(h: u16) -> u16 {
        h
    }

    #[cfg(test)]
    pub(crate) fn audio_x(&self) -> i16 {
        self.segment_x(Segment::Audio)
    }

    fn segments(&self) -> Vec<(Segment, i16, u16)> {
        let slot = Self::slot_w(self.h);
        let mut out: Vec<(Segment, i16, u16)> = Vec::new();
        let mut x = 0i16;
        let mut push = |seg: Segment, w: u16, x: &mut i16| {
            out.push((seg, *x, w));
            *x += w as i16;
        };
        if self.battery.present() {
            push(Segment::Battery, slot, &mut x);
        }
        if self.audio.present() {
            push(Segment::Audio, slot, &mut x);
        }
        if self.mic.present() {
            push(Segment::Mic, slot, &mut x);
        }
        for media in self.media.slots() {
            let w = self.media.slot_width(media, self.h);
            push(Segment::Media(media), w, &mut x);
        }
        out
    }

    #[cfg(test)]
    fn segment_x(&self, want: Segment) -> i16 {
        self.segments()
            .iter()
            .find(|(seg, _, _)| *seg == want)
            .map_or(0, |(_, x, _)| *x)
    }

    fn segment_at(&self, x: i16) -> Option<(usize, Segment)> {
        self.segments()
            .iter()
            .enumerate()
            .find(|(_, (_, sx, w))| x >= *sx && x < *sx + *w as i16)
            .map(|(i, (seg, _, _))| (i, *seg))
    }

    fn wanted_width(&self) -> u32 {
        self.segments().iter().map(|(_, _, w)| u32::from(*w)).sum()
    }

    fn segment_tooltip(&self, seg: Segment) -> String {
        match seg {
            Segment::Battery => self.battery.tooltip(),
            Segment::Audio => self.audio.tooltip(),
            Segment::Mic => self.mic.tooltip(),
            Segment::Media(slot) => self.media.slot_tooltip(slot),
        }
    }

    pub fn set_colours(&mut self, bg: antibox_core::colour::Colour) {
        self.bg = bg;
    }

    pub fn present(&self) -> bool {
        !self.segments().is_empty()
    }

    pub fn update_battery(&mut self) -> bool {
        self.battery.update()
    }

    pub fn update(&mut self) -> bool {
        let battery_changed = self.battery.update();
        let next_audio = self.audio_system.read();
        let audio_changed = next_audio != self.audio.state;
        if audio_changed {
            self.mic.state = next_audio.clone();
            self.audio.state = next_audio;
        }
        battery_changed || audio_changed
    }

    pub fn update_media(&mut self) -> bool {
        if !crate::layout_preferences::taskbar_wants(crate::layout_preferences::Widget::Mpris) {
            let had = self.media.present();
            self.media = MprisView::default();
            return had;
        }
        if self.listed.elapsed() >= LIST_INTERVAL {
            self.listed = Instant::now();
            mpris::refresh();
        } else {
            mpris::poll();
        }
        let next = MprisView::new(mpris::current(), mpris::position());
        let changed = next.player != self.media.player || next.timing() != self.media.timing();
        self.media = next;
        changed
    }

    pub fn tooltip(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for seg in [Segment::Battery, Segment::Audio, Segment::Mic] {
            if self.segments().iter().any(|(s, _, _)| *s == seg) {
                parts.push(self.segment_tooltip(seg));
            }
        }
        let media = self.media.tooltip();
        if !media.is_empty() {
            parts.push(media);
        }
        parts.join("\n\n")
    }

    fn refresh_tip(&mut self, idx: usize) {
        let Some((seg, x, w)) = self.segments().get(idx).copied() else { return };
        let text = self.segment_tooltip(seg);
        crate::tooltip::show_rect_tip(
            &mut self.tooltip,
            self.conn.as_ref(),
            &*self.window,
            (x, 0i16, w, self.h),
            (self.w, self.h),
            &text,
        );
    }

    fn media_click(&mut self, slot: MediaSlot, button: u8) -> Option<u32> {
        match (slot, button) {
            (MediaSlot::Button(b), 1) => mpris::send(b.command()),
            (MediaSlot::Timing, 1) => mpris::send(Command::Raise),
            (MediaSlot::Button(MediaButton::PlayPause), 2) => mpris::send(Command::Stop),
            (_, 3) => mpris::cycle_player(),
            (_, 4) if self.media.can_seek() => mpris::seek(SEEK_STEP),
            (_, 5) if self.media.can_seek() => mpris::seek(-SEEK_STEP),
            _ => return None,
        }
        self.update_media();
        Some(self.window.id())
    }
}

impl Applet for PowerAudioApplet {
    fn set_theme_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.set_colours(tc.task_bar_colour);
    }
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }

    fn paint(&self, g: &dyn GraphicsContext) {
        let _ = g.set_foreground(self.bg);
        let _ = g.fill_rect(0, 0, self.w, self.h);
        theme::well(g, 0, 0, self.w, self.h);
        let _ = g.set_background(self.bg);
        let slot = Self::slot_w(self.h);
        for (seg, x, _) in self.segments() {
            match seg {
                Segment::Battery => self.battery.draw(g, x, 0, slot, self.h),
                Segment::Audio => self.audio.draw(g, x, self.h),
                Segment::Mic => self.mic.draw(g, x, self.h),
                Segment::Media(media) => self.media.draw_slot(g, media, x, self.h),
            }
        }
    }

    fn preferred_width(&self) -> u32 {
        self.wanted_width()
    }

    fn preferred_height(&self) -> u32 {
        crate::status_graph::pref_h()
    }

    fn handle_click(&mut self, x: i32, _y: i32, button: u8) -> Option<u32> {
        let (idx, seg) = self.segment_at(x as i16)?;
        if let Segment::Media(slot) = seg {
            return self.media_click(slot, button);
        }
        match (seg, button) {
            (Segment::Audio, 1) => self.audio_system.toggle_sink_mute(),
            (Segment::Audio | Segment::Mic, 2) | (Segment::Mic, 1) => {
                self.audio_system.toggle_source_mute();
            }
            (Segment::Audio, 4) => self.audio_system.nudge_sink_volume(5),
            (Segment::Audio, 5) => self.audio_system.nudge_sink_volume(-5),
            _ => return None,
        }
        self.update();
        if self.hovered == Some(idx) {
            self.refresh_tip(idx);
        }
        None
    }

    fn handle_enter(&mut self) {
        if self.segments().len() > 1 {
            return;
        }
        self.hovered = None;
        let text = self.tooltip();
        crate::tooltip::show_window_tip(
            &mut self.tooltip,
            self.conn.as_ref(),
            &*self.window,
            &text,
        );
    }

    fn handle_motion(&mut self, x: i32, _y: i32) {
        if self.segments().len() <= 1 {
            return;
        }
        let Some((idx, _)) = self.segment_at(x as i16) else { return };
        if self.hovered == Some(idx) {
            return;
        }
        self.hovered = Some(idx);
        self.refresh_tip(idx);
    }

    fn handle_leave(&mut self) {
        self.hovered = None;
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn tick_tooltip(&mut self) {
        crate::tooltip::pump_tip(&mut self.tooltip, self.conn.as_ref());
    }
    fn shutdown(&mut self) {
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn tooltip_pending(&self) -> bool {
        crate::tooltip::tip_pending(&self.tooltip)
    }

    impl_applet_set_geometry!(w, h);

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "power_audio_applet_tests.rs"]
mod tests;
