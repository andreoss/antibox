use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use antibox_core::scale::scaled;
use antibox_ui::menurender::draw_button_bevel;
use antibox_ui::searchbar::{SearchBar, SearchEvent};
use antibox_ui::{metrics, theme};
use antibox_wm::settings_io::{self, SettingValue};
use antibox_wm::wmconfig;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Text,
    Int,
    Bool,
    Choice(&'static [&'static str]),
}

struct Field {
    section: &'static str,
    key: &'static str,
    label: &'static str,
    kind: Kind,
    text: String,
    on: bool,
    bar: Option<SearchBar>,
}

fn fields(p: &wmconfig::Prefs) -> Vec<Field> {
    let f = |section, key, label, kind, text: String, on| Field {
        section,
        key,
        label,
        kind,
        text,
        on,
        bar: None,
    };
    vec![
        f(
            "theme",
            "name",
            "Theme",
            Kind::Choice(&["nt", "2k3"]),
            p.theme.name.clone(),
            false,
        ),
        f("font", "name", "Font", Kind::Text, p.font.name.clone(), false),
        f(
            "workspace",
            "count",
            "Workspaces",
            Kind::Int,
            p.workspace.count.to_string(),
            false,
        ),
        f(
            "workspace",
            "layouts",
            "Workspace layouts",
            Kind::Text,
            p.workspace.layouts.clone(),
            false,
        ),
        f(
            "keyboard",
            "layouts",
            "Keyboard layouts",
            Kind::Text,
            p.keyboard.layouts.clone(),
            false,
        ),
        f(
            "taskbar",
            "layout",
            "Taskbar layout",
            Kind::Text,
            p.taskbar.layout.clone(),
            false,
        ),
        f(
            "clock",
            "format",
            "Clock format",
            Kind::Text,
            p.clock.format.clone(),
            false,
        ),
        f(
            "net",
            "device",
            "Network device",
            Kind::Text,
            p.net.device.clone(),
            false,
        ),
        f(
            "cpu",
            "width",
            "CPU graph width",
            Kind::Int,
            p.cpu.width.to_string(),
            false,
        ),
        f(
            "mem",
            "width",
            "Memory graph width",
            Kind::Int,
            p.mem.width.to_string(),
            false,
        ),
        f(
            "net",
            "width",
            "Net graph width",
            Kind::Int,
            p.net.width.to_string(),
            false,
        ),
        f(
            "winlist",
            "position",
            "Window list position",
            Kind::Choice(&["centre", "pointer"]),
            p.winlist.position.clone(),
            false,
        ),
        f(
            "tabs",
            "position",
            "Tab position",
            Kind::Choice(&["top", "bottom"]),
            p.tabs.position.clone(),
            false,
        ),
        f(
            "taskbar",
            "menu_on_super_tap",
            "Menu on Super tap",
            Kind::Bool,
            String::new(),
            p.taskbar.menu_on_super_tap,
        ),
        f(
            "ticker",
            "enabled",
            "Scrolling titles",
            Kind::Bool,
            String::new(),
            p.ticker.enabled,
        ),
        f(
            "pointer",
            "warp",
            "Warp pointer",
            Kind::Bool,
            String::new(),
            p.pointer.warp,
        ),
    ]
}

struct Layout {
    label_w: i16,
    row_h: i16,
    top: i16,
    pad: i16,
    field_h: u16,
    w: u16,
    h: u16,
}

fn layout(n: usize) -> Layout {
    let pad = scaled(10) as i16;
    let field_h = metrics::field_height() as u16;
    let row_h = field_h as i16 + scaled(6) as i16;
    let label_w = scaled(170) as i16;
    let w = (label_w + scaled(240) as i16 + pad * 2) as u16;
    let h = (pad + row_h * n as i16 + scaled(8) as i16 + field_h as i16 + pad) as u16;
    Layout {
        label_w,
        row_h,
        top: pad,
        pad,
        field_h,
        w,
        h,
    }
}

fn row_y(l: &Layout, i: usize) -> i16 {
    l.top + l.row_h * i as i16
}

fn cell_rect(l: &Layout, i: usize) -> (i16, i16, u16, u16) {
    (
        l.pad + l.label_w,
        row_y(l, i),
        (l.w as i16 - l.label_w - l.pad * 2) as u16,
        l.field_h,
    )
}

fn button_rects(l: &Layout, _n: usize) -> ((i16, i16, u16, u16), (i16, i16, u16, u16)) {
    let bw = scaled(80) as u16;
    let bh = l.field_h;
    let y = l.h as i16 - l.pad - bh as i16;
    let save_x = l.w as i16 - l.pad - bw as i16 * 2 - scaled(8) as i16;
    let quit_x = l.w as i16 - l.pad - bw as i16;
    ((save_x, y, bw, bh), (quit_x, y, bw, bh))
}

struct App {
    conn: Arc<dyn DisplayBackend>,
    win: Box<dyn WindowHandle>,
    fields: Vec<Field>,
    l: Layout,
    focus: Option<usize>,
    pressed: Option<usize>,
    status: String,
    delete_atom: u32,
    protocols_atom: u32,
}

impl App {
    fn text_indices(&self) -> Vec<usize> {
        self.fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.bar.is_some())
            .map(|(i, _)| i)
            .collect()
    }

    fn resize(&mut self, w: u16, h: u16) {
        if w == 0 || h == 0 || (w == self.l.w && h == self.l.h) {
            return;
        }
        self.l.w = w;
        self.l.h = h;
        for i in 0..self.fields.len() {
            let (cx, cy, cw, ch) = cell_rect(&self.l, i);
            if let Some(bar) = self.fields[i].bar.as_mut() {
                bar.set_rect(cx, cy, cw, ch);
                bar.repaint();
            }
        }
        self.paint();
    }

    fn set_focus(&mut self, idx: Option<usize>) {
        self.focus = idx;
        for (i, f) in self.fields.iter_mut().enumerate() {
            if let Some(bar) = f.bar.as_mut() {
                bar.set_focus(Some(i) == idx);
                bar.repaint();
            }
        }
    }

    fn focus_next(&mut self, dir: i32) {
        let order = self.text_indices();
        if order.is_empty() {
            return;
        }
        let pos = self
            .focus
            .and_then(|f| order.iter().position(|&i| i == f))
            .map(|p| p as i32)
            .unwrap_or(-1);
        let next = (pos + dir).rem_euclid(order.len() as i32) as usize;
        self.set_focus(Some(order[next]));
    }

    fn save(&mut self) {
        let mut values: Vec<(&str, &str, SettingValue)> = Vec::new();
        for f in &self.fields {
            let v = match f.kind {
                Kind::Text | Kind::Choice(_) => {
                    let text = f
                        .bar
                        .as_ref()
                        .map(|b| b.text().to_string())
                        .unwrap_or_else(|| f.text.clone());
                    SettingValue::Text(text)
                }
                Kind::Int => {
                    let text = f
                        .bar
                        .as_ref()
                        .map(|b| b.text().to_string())
                        .unwrap_or_else(|| f.text.clone());
                    match text.trim().parse::<i64>() {
                        Ok(n) => SettingValue::Int(n),
                        Err(_) => continue,
                    }
                }
                Kind::Bool => SettingValue::Bool(f.on),
            };
            values.push((f.section, f.key, v));
        }
        self.status = match settings_io::save(&values) {
            Ok(path) => format!("Saved to {}", path.display()),
            Err(e) => format!("Save failed: {}", e),
        };
        self.paint();
    }

    fn paint(&self) {
        let g = match self.conn.create_graphics(self.win.id()) {
            Ok(g) => g,
            Err(_) => return,
        };
        let _ = g.set_foreground(theme::face());
        let _ = g.fill_rect(0, 0, self.l.w, self.l.h);
        let _ = g.set_font(&FontSpec::ui(metrics::font_pt()));
        let _ = g.set_background(theme::face());
        for (i, f) in self.fields.iter().enumerate() {
            let y = row_y(&self.l, i);
            let base = metrics::baseline(y as i32, self.l.field_h as i32) as i16;
            let _ = g.set_foreground(theme::text());
            let _ = g.set_background(theme::face());
            let _ = g.draw_text(self.l.pad, base, f.label);
            let (cx, cy, cw, ch) = cell_rect(&self.l, i);
            match f.kind {
                Kind::Bool => {
                    let side = ch.min(self.l.field_h);
                    let _ = g.set_foreground(theme::field());
                    let _ = g.fill_rect(cx, cy, side, side);
                    let _ = draw_button_bevel(&*g, cx, cy, side, side, theme::face(), true);
                    if f.on {
                        let m = (side as i16 / 4).max(2);
                        let _ = g.set_foreground(theme::text());
                        let _ = g.fill_rect(
                            cx + m,
                            cy + m,
                            (side as i16 - 2 * m).max(2) as u16,
                            (side as i16 - 2 * m).max(2) as u16,
                        );
                    }
                }
                Kind::Choice(_) => {
                    let _ = g.set_foreground(theme::face());
                    let _ = g.fill_rect(cx, cy, cw, ch);
                    let _ = draw_button_bevel(&*g, cx, cy, cw, ch, theme::face(), false);
                    let _ = g.set_foreground(theme::text());
                    let _ = g.set_background(theme::face());
                    let _ = g.draw_text(
                        cx + scaled(6) as i16,
                        metrics::baseline(cy as i32, ch as i32) as i16,
                        &f.text,
                    );
                }
                _ => {}
            }
        }
        let n = self.fields.len();
        let (save, quit) = button_rects(&self.l, n);
        for (i, (r, label)) in [(save, "Save"), (quit, "Quit")].into_iter().enumerate() {
            let (x, y, w, h) = r;
            let sunken = self.pressed == Some(i);
            let _ = g.set_foreground(theme::face());
            let _ = g.fill_rect(x, y, w, h);
            let _ = draw_button_bevel(&*g, x, y, w, h, theme::face(), sunken);
            let _ = g.set_foreground(theme::text());
            let _ = g.set_background(theme::face());
            let off = if sunken { 1 } else { 0 };
            let tw = g.text_width(label).unwrap_or(0) as i16;
            let _ = g.draw_text(
                x + (w as i16 - tw) / 2 + off,
                metrics::baseline(y as i32, h as i32) as i16 + off,
                label,
            );
        }
        if !self.status.is_empty() {
            let (save, _) = button_rects(&self.l, n);
            let clip = antibox_core::rect::Rect::new(
                self.l.pad as i32,
                save.1 as i32,
                (save.0 - self.l.pad - scaled(8) as i16).max(1) as i32,
                self.l.field_h as i32,
            );
            let _ = g.push_clip(&clip);
            let _ = g.set_foreground(theme::shadow());
            let _ = g.set_background(theme::face());
            let _ = g.draw_text(
                self.l.pad,
                metrics::baseline(save.1 as i32, self.l.field_h as i32) as i16,
                &self.status,
            );
            let _ = g.pop_clip();
        }
    }

    fn hit(&self, p: Point) -> Option<usize> {
        for i in 0..self.fields.len() {
            let (cx, cy, cw, ch) = cell_rect(&self.l, i);
            let inside = p.x >= cx as i32
                && p.x < cx as i32 + cw as i32
                && p.y >= cy as i32
                && p.y < cy as i32 + ch as i32;
            if inside {
                return Some(i);
            }
        }
        None
    }
}

fn in_rect(p: Point, r: (i16, i16, u16, u16)) -> bool {
    p.x >= r.0 as i32
        && p.x < r.0 as i32 + r.2 as i32
        && p.y >= r.1 as i32
        && p.y < r.1 as i32 + r.3 as i32
}

fn main() {
    let (conn, render, mut event_loop, _tray) = match antibox_x11::xcb::build_backend(None) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("antibox-settings: cannot open display: {}", e);
            std::process::exit(1);
        }
    };
    let (mm_w, mm_h) = conn.screen_size_mm();
    let dpi = antibox_core::scale::detect(
        conn.screen_width() as u32,
        conn.screen_height() as u32,
        mm_w,
        mm_h,
        conn.preferred_scale(),
    );
    antibox_core::scale::set_dpi(dpi);

    let prefs = wmconfig::Config::load_prefs();
    antibox_ui::theme::install_named(&prefs.theme.name);
    antibox_wm::wmapp::apply_font_prefs(&conn, &prefs);
    let mut fields = fields(&prefs);
    let l = layout(fields.len());

    let win = conn
        .create_window(
            conn.root().as_parent(),
            Rect::new(0, 0, l.w as i32, l.h as i32),
            WmWindowClass::InputOutput,
            false,
            EventMask::EXPOSURE
                | EventMask::KEY_PRESS
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::STRUCTURE_NOTIFY,
        )
        .expect("create window");
    let _ = conn.change_property8(PropMode::Replace, win.id(), 39, 31, b"Settings");
    let _ = conn.change_property8(
        PropMode::Replace,
        win.id(),
        67,
        31,
        b"antibox-settings\0Antibox\0",
    );
    let protocols_atom = conn.intern_atom("WM_PROTOCOLS").unwrap_or(0);
    let delete_atom = conn.intern_atom("WM_DELETE_WINDOW").unwrap_or(0);
    if protocols_atom != 0 && delete_atom != 0 {
        let _ = conn.change_property32(
            PropMode::Replace,
            win.id(),
            protocols_atom,
            4,
            &[delete_atom],
        );
    }
    let _ = win.map();

    let rb: Arc<dyn RenderBackend> = render;
    for (i, f) in fields.iter_mut().enumerate() {
        if matches!(f.kind, Kind::Text | Kind::Int) {
            let (cx, cy, cw, ch) = cell_rect(&l, i);
            if let Ok(mut bar) = SearchBar::new(&rb, win.id(), cx, cy, cw, ch) {
                bar.set_text(&f.text);
                bar.show();
                f.bar = Some(bar);
            }
        }
    }
    let _ = conn.flush();

    let mut app = App {
        conn: Arc::clone(&conn),
        win,
        fields,
        l,
        focus: None,
        pressed: None,
        status: String::new(),
        delete_atom,
        protocols_atom,
    };
    app.set_focus(app.text_indices().first().copied());

    let min = conn.setup_min_keycode();
    let max = conn.setup_max_keycode();
    let mapping = conn.get_keyboard_mapping(min, max - min + 1).ok();

    let mut running = true;
    while running {
        let pending = event_loop.process_pending().unwrap_or_default();
        if pending.is_empty() {
            let _ = conn.flush();
            match event_loop.wait_for_one_event(Duration::from_millis(250)) {
                Ok(Some(ev)) => running = handle(&mut app, &ev, mapping.as_ref()),
                Ok(None) => {}
                Err(_) => break,
            }
            continue;
        }
        for ev in &pending {
            if !handle(&mut app, ev, mapping.as_ref()) {
                running = false;
                break;
            }
        }
        let _ = conn.flush();
    }
}

fn handle(app: &mut App, ev: &BackendEvent, mapping: Option<&KeyboardMapping>) -> bool {
    match ev {
        BackendEvent::ClientMessage {
            window,
            message_type,
            data,
            ..
        } if *window == app.win.id()
            && *message_type == app.protocols_atom
            && data.first() == Some(&app.delete_atom) =>
        {
            return false;
        }
        BackendEvent::DestroyNotify { window } if *window == app.win.id() => {
            return false;
        }
        BackendEvent::ConfigureNotify { window, rect, .. } if *window == app.win.id() => {
            app.resize(rect.w as u16, rect.h as u16);
        }
        BackendEvent::Expose { window, .. } => {
            if *window == app.win.id() {
                app.paint();
            } else {
                for f in &app.fields {
                    if let Some(bar) = &f.bar {
                        if bar.owns_window(*window) {
                            bar.repaint();
                        }
                    }
                }
            }
        }
        BackendEvent::ButtonPress {
            window,
            point,
            button,
            ..
        } => {
            if *button != 1 {
                return true;
            }
            let mut bar_hit = None;
            for (i, f) in app.fields.iter_mut().enumerate() {
                if let Some(bar) = f.bar.as_mut() {
                    if bar.owns_window(*window) {
                        let _ = bar.handle_button(*window, point.x, point.y, *button);
                        bar_hit = Some(i);
                    }
                }
            }
            if let Some(i) = bar_hit {
                app.set_focus(Some(i));
                return true;
            }
            if *window != app.win.id() {
                return true;
            }
            let n = app.fields.len();
            let (save, quit) = button_rects(&app.l, n);
            if in_rect(*point, save) {
                app.pressed = Some(0);
                app.paint();
                return true;
            }
            if in_rect(*point, quit) {
                app.pressed = Some(1);
                app.paint();
                return true;
            }
            if let Some(i) = app.hit(*point) {
                match app.fields[i].kind {
                    Kind::Bool => {
                        app.fields[i].on = !app.fields[i].on;
                        app.paint();
                    }
                    Kind::Choice(opts) => {
                        let cur = app.fields[i].text.clone();
                        let pos = opts.iter().position(|o| *o == cur).unwrap_or(0);
                        app.fields[i].text = opts[(pos + 1) % opts.len()].to_string();
                        app.paint();
                    }
                    _ => app.set_focus(Some(i)),
                }
            }
        }
        BackendEvent::ButtonRelease { window, point, .. } => {
            if let Some(idx) = app.pressed.take() {
                app.paint();
                if *window == app.win.id() {
                    let (save, quit) = button_rects(&app.l, app.fields.len());
                    if idx == 0 && in_rect(*point, save) {
                        app.save();
                    } else if idx == 1 && in_rect(*point, quit) {
                        return false;
                    }
                }
            }
        }
        BackendEvent::KeyPress { keycode, state, .. } => {
            let ks = antibox_ui::keymap::keysym_for_keycode(app.conn.as_ref(), *keycode);
            match ks {
                0xFF1B => return false,
                0xFF09 => app.focus_next(if *state & 0x01 != 0 { -1 } else { 1 }),
                0xFF0D | 0xFF8D => app.save(),
                _ => {
                    if let (Some(i), Some(m)) = (app.focus, mapping) {
                        if let Some(bar) = app.fields[i].bar.as_mut() {
                            if bar.handle_key(*keycode, *state, m) == SearchEvent::Changed {
                                bar.repaint();
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    true
}
