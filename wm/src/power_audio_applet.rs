use crate::applet::Applet;
use crate::audio::{detect, AudioSystem};
use crate::audio_view::AudioView;
use crate::battery_view::BatteryView;
use antibox_core::backend::*;
use antibox_core::rect::Rect;
use antibox_ui::theme;
use std::sync::Arc;

pub struct PowerAudioApplet {
    conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    tooltip: Option<crate::tooltip::ToolTip>,
    hovered: Option<bool>,
    battery: BatteryView,
    audio_system: Box<dyn AudioSystem>,
    audio: AudioView,
    bg: antibox_core::colour::Colour,
    w: u16,
    h: u16,
}

impl PowerAudioApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let h = crate::status_graph::pref_h() as u16;
        let battery = BatteryView::new(true);
        let audio_system = detect();
        let audio = AudioView::new(audio_system.read());
        let slot = Self::slot_w(h);
        let w = match (battery.present(), audio.present()) {
            (false, false) => slot,
            (true, true) => slot * 2,
            _ => slot,
        };
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            EventMask::ENTER_WINDOW | EventMask::LEAVE_WINDOW | EventMask::POINTER_MOTION,
        )?;
        Ok(Self {
            conn: Arc::clone(conn),
            window,
            tooltip: None,
            hovered: None,
            battery,
            audio_system,
            audio,
            bg: theme::tray_face(),
            w,
            h,
        })
    }

    fn audio_x(&self) -> i16 {
        if self.battery.present() {
            Self::slot_w(self.h) as i16
        } else {
            0
        }
    }

    fn slot_w(h: u16) -> u16 {
        h
    }

    fn wanted_width(&self) -> u32 {
        let slot = Self::slot_w(self.h) as u32;
        match (self.battery.present(), self.audio.present()) {
            (false, false) => 0,
            (true, true) => slot * 2,
            _ => slot,
        }
    }

    pub fn set_colours(&mut self, bg: antibox_core::colour::Colour) {
        self.bg = bg;
    }

    pub fn present(&self) -> bool {
        self.battery.present() || self.audio.present()
    }

    pub fn update(&mut self) -> bool {
        let battery_changed = self.battery.update();
        let next_audio = self.audio_system.read();
        let audio_changed = next_audio != self.audio.state;
        if audio_changed {
            self.audio.state = next_audio;
        }
        battery_changed || audio_changed
    }

    pub fn tooltip(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.battery.present() {
            parts.push(self.battery.tooltip());
        }
        if self.audio.present() {
            parts.push(self.audio.tooltip());
        }
        parts.join("\n\n")
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
        let slot = Self::slot_w(self.h) as i16;
        let mut x = 0i16;
        if self.battery.present() {
            self.battery.draw(g, x, 0, slot as u16, self.h);
            x += slot;
        }
        if self.audio.present() {
            self.audio.draw(g, x, self.h);
        }
    }

    fn preferred_width(&self) -> u32 {
        self.wanted_width()
    }

    fn preferred_height(&self) -> u32 {
        crate::status_graph::pref_h()
    }

    fn handle_click(&mut self, _x: i32, _y: i32, _button: u8) -> Option<u32> {
        None
    }

    fn handle_enter(&mut self) {
        if self.battery.present() && self.audio.present() {
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
        if !(self.battery.present() && self.audio.present()) {
            return;
        }
        let audio_half = (x as i16) >= self.audio_x();
        if self.hovered == Some(audio_half) {
            return;
        }
        self.hovered = Some(audio_half);
        let slot = Self::slot_w(self.h) as i16;
        let (rect, text) = if audio_half {
            let ax = self.audio_x();
            (
                (ax, 0i16, (self.w as i16 - ax).max(0) as u16, self.h),
                self.audio.tooltip(),
            )
        } else {
            ((0i16, 0i16, slot as u16, self.h), self.battery.tooltip())
        };
        crate::tooltip::show_rect_tip(
            &mut self.tooltip,
            self.conn.as_ref(),
            &*self.window,
            rect,
            &text,
        );
    }

    fn handle_leave(&mut self) {
        self.hovered = None;
        crate::tooltip::hide_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn tick_tooltip(&mut self) {
        crate::tooltip::pump_tip(&mut self.tooltip, self.conn.as_ref());
    }

    fn tooltip_pending(&self) -> bool {
        crate::tooltip::tip_pending(&self.tooltip)
    }

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

#[cfg(test)]
#[path = "power_audio_applet_tests.rs"]
mod tests;
