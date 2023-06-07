 use antibox_core::error::Result;
use crate::applet::Applet;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::sync::Arc;

fn menu_item_h() -> u16 {
    antibox_ui::metrics::menu_item_height() as u16
}

fn capture_output(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

fn run_ok(cmd: &str, args: &[&str]) -> bool {
    std::process::Command::new(cmd)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn detect_layout(conn: &Arc<dyn DisplayBackend>) -> (String, String) {
    if let Some(info) = conn.keyboard_info() {
        let active = info
            .active_layout()
            .filter(|s| !s.is_empty())
            .map_or_else(|| "US".to_string(), str::to_uppercase);
        return (active, format_xkb_tooltip(&info));
    }
    if let Some(l) = conn.keyboard_layout() {
        let up = l.to_uppercase();
        return (up.clone(), format!("Layout: {}", up));
    }
    detect_layout_info()
}

fn format_xkb_tooltip(info: &KeyboardInfo) -> String {
    let mut lines = Vec::new();
    if let Some(active) = info.active_layout().filter(|s| !s.is_empty()) {
        lines.push(format!("Layout: {}", active.to_uppercase()));
    }
    if !info.layouts.is_empty() {
        lines.push(format!("Layouts: {}", info.layouts));
    }
    if info.variants.chars().any(|c| c != ',') {
        lines.push(format!("Variants: {}", info.variants));
    }
    if !info.options.is_empty() {
        lines.push(format!("Options: {}", info.options));
    }
    lines.join("\n")
}

fn detect_layout_info() -> (String, String) {
    let mut layout = String::new();
    let mut tooltip = String::new();
    if let Some(s) = capture_output("setxkbmap", &["-query"]) {
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("layout:") {
                layout = rest.trim().to_uppercase();
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if !tooltip.is_empty() {
                    tooltip.push('\n');
                }
                tooltip.push_str(trimmed);
            }
        }
    }
    if layout.is_empty() {
        if let Ok(v) = std::env::var("LANG") {
            if v.len() >= 2 {
                layout = v[..2].to_uppercase();
                tooltip = format!("layout: {}", layout);
            }
        }
    }
    if layout.is_empty() {
        layout = "US".to_string();
        tooltip = "layout: US".to_string();
    }
    (layout, tooltip)
}

fn estimate_width() -> u16 {
    (antibox_ui::metrics::text_w(4) + antibox_ui::metrics::pad() * 2) as u16
}

fn xkb_layouts(conn: &Arc<dyn DisplayBackend>) -> Vec<String> {
    conn.keyboard_info().map_or_else(Vec::new, |i| {
        i.layouts
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

pub struct KeyboardApplet {
    pub(crate) window: Box<dyn WindowHandle>,
    conn: Arc<dyn DisplayBackend>,
    layout: String,
    tooltip_text: String,
    layouts: Vec<String>,
    xkb_groups: bool,
    index: usize,
    menu_window: Option<Box<dyn WindowHandle>>,
    menu_selected: Option<usize>,
    applet_abs: (i32, i32),
    menu_abs: (i32, i32),
    opened_at: Option<std::time::Instant>,
    width: u16,
    height: u16,
    face: antibox_core::colour::Colour,
    text: u32,
    sel_bg: antibox_core::colour::Colour,
    sel_fg: antibox_core::colour::Colour,
    tooltip: Option<crate::tooltip::ToolTip>,
}

impl KeyboardApplet {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        parent: u32,
        layouts: Vec<String>,
        colours: &crate::render::ThemeColors,
    ) -> Result<Self> {
        let (layout, tooltip_text) = detect_layout(conn);
        let (layouts, xkb_groups) = if layouts.is_empty() {
            (xkb_layouts(conn), true)
        } else {
            (layouts, false)
        };
        let w = estimate_width();
        let h = antibox_ui::metrics::panel_height() as u16;
        let window = conn.create_window(
            parent,
            Rect::new(0, 0, w as i32, h as i32),
            WmWindowClass::InputOutput,
            true,
            EventMask::BUTTON_PRESS
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW
                | EventMask::POINTER_MOTION,
        )?;
        let index = layouts
            .iter()
            .position(|l| l.eq_ignore_ascii_case(&layout))
            .unwrap_or(0);
        Ok(KeyboardApplet {
            window,
            conn: Arc::clone(conn),
            layout,
            tooltip_text,
            layouts,
            xkb_groups,
            index,
            menu_window: None,
            menu_selected: None,
            applet_abs: (0, 0),
            menu_abs: (0, 0),
            opened_at: None,
            width: w,
            height: h,
            face: colours.task_bar_colour,
            text: colours.button_fg,
            sel_bg: colours.workspace_active_bg,
            sel_fg: colours.workspace_active_fg,
            tooltip: None,
        })
    }

    pub fn set_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.face = tc.task_bar_colour;
        self.text = tc.button_fg;
        self.sel_bg = tc.workspace_active_bg;
        self.sel_fg = tc.workspace_active_fg;
    }

    pub fn next_layout(&mut self) {
        if self.layouts.is_empty() {
            return;
        }
        self.apply_layout((self.index + 1) % self.layouts.len());
    }

    fn apply_layout(&mut self, index: usize) {
        if index >= self.layouts.len() {
            return;
        }
        self.index = index;
        let new_layout = self.layouts[index].clone();
        if self.xkb_groups {
            self.conn.set_keyboard_group(index);
        } else {
            let _ = run_ok("setxkbmap", &["-layout", &new_layout]);
        }
        self.layout = new_layout.to_uppercase();
        self.tooltip_text = format!("Layout: {}", self.layout);
        if let Ok(g) = self.conn.create_graphics(self.window.id()) {
            self.paint(&*g);
        }
    }

    pub fn update(&mut self) -> bool {
        let old_layout = self.layout.clone();
        let (layout, tooltip_text) = detect_layout(&self.conn);
        self.layout = layout;
        self.tooltip_text = tooltip_text;
        if self.xkb_groups {
            let fresh = xkb_layouts(&self.conn);
            if !fresh.is_empty() && fresh != self.layouts {
                self.layouts = fresh;
            }
        }
        if let Some(i) = self
            .layouts
            .iter()
            .position(|l| l.eq_ignore_ascii_case(&self.layout))
        {
            self.index = i;
        }
        self.layout != old_layout
    }

    pub fn switch_to(&mut self, index: usize) {
        self.apply_layout(index);
        self.hide_menu();
    }

    pub fn shutdown(&mut self) {
        self.hide_menu();
        let _ = self.window.unmap();
        let _ = self.window.destroy();
    }

    fn show_menu(&mut self) {
        self.hide_menu();
        let n = self.layouts.len();
        if n == 0 {
            return;
        }
        let pad = antibox_ui::metrics::pad() as u16;
        let ph = (n as u16) * menu_item_h() + pad * 2;
        let (ax, ay) = match self.window.translate_coords(Point::new(0, 0)) {
            Ok(p) => (p.x, p.y),
            Err(_) => (0, 0),
        };
        let my = if ay - ph as i32 >= 0 {
            ay - ph as i32
        } else {
            ay + self.height as i32
        };
        let mask = EventMask::BUTTON_PRESS
            | EventMask::EXPOSURE
            | EventMask::POINTER_MOTION
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW;
        if let Ok(win) = self.conn.create_window(
            self.conn.root().as_parent(),
            Rect::new(ax, my, self.width as i32, ph as i32),
            WmWindowClass::InputOutput,
            true,
            mask,
        ) {
            let _ = win.map();
            let menu_id = win.id();
            self.menu_window = Some(win);
            self.menu_selected = None;
            self.applet_abs = (ax, ay);
            self.menu_abs = (ax, my);
            self.opened_at = Some(std::time::Instant::now());
            let _ = self.conn.grab_pointer(PointerGrab::new(
                menu_id,
                EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION,
            ));
            let _ = self
                .conn
                .grab_keyboard(false, menu_id, 0, GrabMode::Async, GrabMode::Async);
            self.paint_menu();
        }
    }

    fn hide_menu(&mut self) {
        if let Some(ref win) = self.menu_window {
            let _ = self.conn.ungrab_pointer(0);
            let _ = self.conn.ungrab_keyboard(0);
            let _ = win.unmap();
            let _ = win.destroy();
        }
        self.menu_window = None;
        self.menu_selected = None;
    }

    fn menu_move(&mut self, dir: i32) {
        let n = self.layouts.len() as i32;
        if n == 0 {
            return;
        }
        let cur = self.menu_selected.map_or(self.index as i32, |s| s as i32);
        self.menu_selected = Some((((cur + dir) % n + n) % n) as usize);
        self.paint_menu();
    }

    fn paint_menu(&self) {
        let win = match self.menu_window {
            Some(ref win) => win,
            None => return,
        };
        use antibox_ui::metrics;
        let mw = self.width;
        let ih = menu_item_h();
        let pad = metrics::pad() as i16;
        let h = (self.layouts.len() as u16) * ih + pad as u16 * 2;
        let pm = match self.conn.create_pixmap(mw, h, self.conn.screen_depth()) {
            Ok(pm) => pm,
            Err(_) => return,
        };
        if let Ok(g) = self.conn.create_graphics(pm) {
            let _ = g.set_font(&FontSpec::ui(metrics::font_pt()));
            crate::render::draw_menu_frame(&*g, mw, h, self.face);
            for (i, layout) in self.layouts.iter().enumerate() {
                let y = pad + i as i16 * ih as i16;
                let sel = self.menu_selected == Some(i);
                if sel {
                    let _ = g.set_foreground(self.sel_bg);
                    let _ = g.fill_rect(2, y, mw - 4, ih);
                } else if i == self.index {
                    let _ = g.set_foreground(self.sel_bg);
                    let _ = g.draw_rect(2, y, mw - 4, ih - 1);
                }
                let fg = if sel { self.sel_fg } else { self.text };
                let label = layout.to_uppercase();
                let tw = g
                    .text_width(&label)
                    .unwrap_or(metrics::text_w(label.chars().count()) as u32)
                    as i16;
                let x = ((mw as i16 - tw) / 2).max(2);
                let baseline = metrics::baseline(y as i32, ih as i32) as i16;
                let _ = g.set_foreground(fg);
                let _ = g.draw_text_transparent(x, baseline, &label);
            }
            if let Ok(wg) = self.conn.create_graphics(win.id()) {
                let _ = wg.copy_from(pm, Rect::new(0, 0, mw as i32, h as i32), Point::ZERO);
            }
        }
        let _ = self.conn.free_pixmap(pm);
    }

    fn menu_index_at(&self, p: Point) -> Option<usize> {
        let pad = antibox_ui::metrics::pad();
        if p.x < 0 || p.x >= self.width as i32 || p.y < pad {
            return None;
        }
        let idx = ((p.y - pad) / menu_item_h() as i32) as usize;
        if idx < self.layouts.len() {
            Some(idx)
        } else {
            None
        }
    }

    fn point_on_applet(&self, p: Point) -> bool {
        let (abs_x, abs_y) = (self.menu_abs.0 + p.x, self.menu_abs.1 + p.y);
        abs_x >= self.applet_abs.0
            && abs_x < self.applet_abs.0 + self.width as i32
            && abs_y >= self.applet_abs.1
            && abs_y < self.applet_abs.1 + self.height as i32
    }

    fn handle_menu_click(&mut self, p: Point) {
        if let Some(idx) = self.menu_index_at(p) {
            self.switch_to(idx);
            return;
        }
        let double = self.point_on_applet(p)
            && self.opened_at.map_or(false, |t| {
                t.elapsed() < std::time::Duration::from_millis(400)
            });
        if double {
            self.next_layout();
        }
        self.hide_menu();
    }

    fn handle_menu_motion(&mut self, p: Point) {
        let new_sel = self.menu_index_at(p);
        if new_sel != self.menu_selected {
            self.menu_selected = new_sel;
            self.paint_menu();
        }
    }

    fn tooltip(&self) -> String {
        if self.tooltip_text.is_empty() {
            format!("Layout: {}", self.layout)
        } else {
            self.tooltip_text.clone()
        }
    }
}

impl Applet for KeyboardApplet {
    fn set_theme_colours(&mut self, tc: &crate::render::ThemeColors) {
        self.set_colours(tc);
    }
    fn window(&self) -> &dyn WindowHandle {
        &*self.window
    }
    fn paint(&self, g: &dyn GraphicsContext) {
        use antibox_ui::metrics;
        let _ = g.set_foreground(self.face);
        let _ = g.fill_rect(0, 0, self.width, self.height);
        antibox_ui::theme::well(g, 0, 0, self.width, self.height);
        let _ = g.set_font(&FontSpec::ui(metrics::font_pt()));
        let _ = g.set_foreground(self.text);
        let _ = g.set_background(self.face);
        let pad = metrics::pad() as u16;
        let avail = self.width.saturating_sub(pad * 2);
        let label = antibox_ui::widget::fit_label(g, &self.layout, avail);
        let tw = g
            .text_width(&label)
            .unwrap_or(metrics::text_w(label.chars().count()) as u32) as i16;
        let x = ((self.width as i16 - tw) / 2).max(pad as i16);
        let baseline = metrics::baseline(0, self.height as i32) as i16;
        let _ = g.draw_text_transparent(x, baseline, &label);
    }
    fn preferred_width(&self) -> u32 {
        use antibox_ui::metrics;
        if self.layouts.len() <= 1 {
            return 0;
        }
        let spec = FontSpec::ui(metrics::font_pt());
        let measured = global_text_width(&spec, &self.layout).map_or_else(
            || metrics::text_w(self.layout.chars().count().max(2)),
            |w| w as i32,
        );
        (measured + metrics::pad() * 2).max(estimate_width() as i32) as u32
    }
    fn preferred_height(&self) -> u32 {
        self.height as u32
    }
    fn handle_click(&mut self, _x: i32, _y: i32, button: u8) -> Option<u32> {
        if button == 1 || button == 3 {
            if self.menu_window.is_some() {
                self.hide_menu();
            } else {
                self.show_menu();
            }
        }
        None
    }
    impl_applet_tooltip!(tooltip);
    fn set_geometry(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.width = w;
        self.height = h;
        let _ = self
            .window
            .configure(Some(x as i32), Some(y as i32), Some(w), Some(h));
    }
    fn owns_window(&self, id: u32) -> bool {
        self.window.id() == id || self.menu_window.as_ref().map_or(false, |w| w.id() == id)
    }
    fn handle_other_event(&mut self, event: &BackendEvent, conn: &Arc<dyn DisplayBackend>) {
        match *event {
            BackendEvent::Expose { .. } => {
                self.paint_menu();
            }
            BackendEvent::ButtonPress { point, .. } => {
                self.handle_menu_click(point);
            }
            BackendEvent::MotionNotify { point, .. } => {
                self.handle_menu_motion(point);
            }
            BackendEvent::KeyPress { keycode, .. } => {
                match crate::bindings::keysym_for_keycode(conn.as_ref(), keycode) {
                    0xFF52 | 0xFF97 => self.menu_move(-1),
                    0xFF54 | 0xFF99 => self.menu_move(1),
                    0xFF0D | 0xFF8D => {
                        let i = self.menu_selected.unwrap_or(self.index);
                        self.switch_to(i);
                    }
                    0xFF1B => self.hide_menu(),
                    _ => {}
                }
            }
            BackendEvent::LeaveNotify { .. } => {
                self.menu_selected = None;
                self.paint_menu();
            }
            _ => {}
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "keyboard_applet_tests.rs"]
mod tests;
