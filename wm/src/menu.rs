use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;

pub trait MenuItem {
    fn is_separator(&self) -> bool;
}

pub fn next_selectable<T: MenuItem>(items: &[T], from: Option<usize>, dir: i32) -> Option<usize> {
    let n = items.len();
    if n == 0 {
        return None;
    }
    let mut i = from.map_or(if dir > 0 { -1 } else { n as i32 }, |s| s as i32);
    for _ in 0..n {
        i = ((i + dir) % n as i32 + n as i32) % n as i32;
        if !items[i as usize].is_separator() {
            return Some(i as usize);
        }
    }
    None
}

#[derive(Clone, Copy)]
pub struct MenuColors {
    pub bg: antibox_core::colour::Colour,
    pub fg: antibox_core::colour::Colour,
    pub sel_bg: antibox_core::colour::Colour,
    pub sel_fg: antibox_core::colour::Colour,
}

impl Default for MenuColors {
    fn default() -> MenuColors {
        MenuColors {
            bg: antibox_ui::theme::menu_bg(),
            fg: antibox_ui::theme::text(),
            sel_bg: antibox_ui::theme::menu_sel_bg(),
            sel_fg: antibox_ui::theme::menu_sel_fg(),
        }
    }
}

impl MenuColors {
    pub fn from_theme(tc: &crate::render::ThemeColors) -> MenuColors {
        MenuColors {
            bg: tc.menu_bg,
            fg: antibox_ui::theme::contrast(0x000000, tc.menu_bg),
            sel_bg: antibox_ui::theme::menu_sel_bg(),
            sel_fg: antibox_ui::theme::menu_sel_fg(),
        }
    }

    pub const fn for_taskbar(tc: &crate::render::ThemeColors) -> MenuColors {
        MenuColors {
            bg: tc.task_bar_colour,
            fg: tc.button_fg,
            sel_bg: tc.workspace_active_bg,
            sel_fg: tc.workspace_active_fg,
        }
    }
}

pub fn item_h() -> u16 {
    antibox_ui::metrics::menu_item_height() as u16
}

#[derive(Clone)]
pub struct MenuRow<T> {
    pub label: String,
    pub payload: Option<T>,
    pub submenu: Option<Vec<MenuRow<T>>>,
    pub separator: bool,
}

impl<T> MenuRow<T> {
    pub fn item(label: impl Into<String>, payload: T) -> Self {
        MenuRow {
            label: label.into(),
            payload: Some(payload),
            submenu: None,
            separator: false,
        }
    }

    pub fn submenu(label: impl Into<String>, rows: Vec<Self>) -> Self {
        MenuRow {
            label: label.into(),
            payload: None,
            submenu: Some(rows),
            separator: false,
        }
    }

    pub fn separator() -> Self {
        MenuRow {
            label: String::new(),
            payload: None,
            submenu: None,
            separator: true,
        }
    }
}

impl<T> MenuItem for MenuRow<T> {
    fn is_separator(&self) -> bool {
        self.separator
    }
}

pub enum MenuNav<T> {
    Ignored,
    Handled,
    Close,
    Activate(T),
}

pub struct MenuView<T> {
    pub window: Option<Box<dyn WindowHandle>>,
    pub items: Vec<MenuRow<T>>,
    all: Vec<MenuRow<T>>,
    bar: Option<antibox_ui::searchbar::SearchBar>,
    pub pos: Point,
    anchor: Point,
    last_pointer: Option<Point>,
    pub visible: bool,
    pub selected: Option<usize>,
    pub submenu: Option<Box<MenuView<T>>>,
    pub colours: MenuColors,
    pub min_w: u16,
}

impl<T: Clone> Default for MenuView<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn bar_h() -> i16 {
    antibox_ui::metrics::field_height() as i16
}

impl<T: Clone> MenuView<T> {
    pub fn new() -> Self {
        MenuView {
            window: None,
            items: Vec::new(),
            all: Vec::new(),
            bar: None,
            pos: Point::new(0, 0),
            anchor: Point::new(0, 0),
            last_pointer: None,
            visible: false,
            selected: None,
            submenu: None,
            colours: MenuColors::default(),
            min_w: 0,
        }
    }

    pub fn with_items(items: Vec<MenuRow<T>>) -> Self {
        MenuView {
            items,
            ..Self::new()
        }
    }

    fn bar_off(&self) -> i16 {
        if self.bar.is_some() {
            bar_h() + antibox_ui::metrics::pad() as i16
        } else {
            0
        }
    }

    pub fn enable_filter(&mut self, rb: &std::sync::Arc<dyn RenderBackend>) {
        let win_id = match self.window.as_ref().map(|w| w.id()) {
            Some(id) => id,
            None => return,
        };
        if self.bar.is_some() {
            return;
        }
        let w = self.menu_w();
        let m = antibox_ui::metrics::pad() as i16;
        let bw = (w as i16 - m * 2).max(1) as u16;
        if let Ok(mut bar) =
            antibox_ui::searchbar::SearchBar::new(rb, win_id, m, m, bw, bar_h() as u16)
        {
            bar.set_placeholder("Filter");
            bar.set_focus(true);
            bar.show();
            self.bar = Some(bar);
            self.all = self.items.clone();
            self.reposition(rb.screen_width() as i32, rb.screen_height() as i32);
        }
    }

    fn reposition(&mut self, sw: i32, sh: i32) {
        let win = match self.window { Some(ref w) => w, None => return };
        let ph = self.height().max(1);
        let pos =
            crate::render::menu_clamp_pos(self.anchor, self.menu_w() as i32, ph, sw, sh, true);
        let _ = win.configure(Some(pos.x), Some(pos.y), None, Some(ph as u16));
        self.pos = pos;
    }

    fn apply_needle(&mut self, needle: &str) {
        let keep = self.selected.and_then(|s| {
            self.items
                .get(s)
                .map(|it| crate::render::parse_mnemonic(&it.label).0)
        });
        self.items = if needle.is_empty() {
            self.all.clone()
        } else {
            self.all
                .iter()
                .filter(|it| {
                    !it.separator
                        && crate::render::parse_mnemonic(&it.label)
                            .0
                            .to_lowercase()
                            .contains(needle)
                })
                .cloned()
                .collect()
        };
        self.selected = keep
            .and_then(|k| {
                self.items
                    .iter()
                    .position(|it| crate::render::parse_mnemonic(&it.label).0 == k)
            })
            .or_else(|| self.next_selectable(None, 1));
    }

    fn reset_filter<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        let bar = match self.bar.as_mut() {
            Some(bar) => bar,
            None => return,
        };
        if !bar.text().is_empty() {
            bar.set_text("");
        }
        self.refilter(conn);
    }

    fn refilter<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        let needle = self
            .bar
            .as_ref()
            .map(|b| b.text().to_lowercase())
            .unwrap_or_default();
        let sw = conn.screen_width() as i32;
        let sh = conn.screen_height() as i32;
        let d = self.deepest();
        d.apply_needle(&needle);
        d.reposition(sw, sh);
        d.paint(conn);
    }

    pub fn handle_key_input<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        keycode: u32,
        state: u16,
        mapping: &KeyboardMapping,
        ks: u32,
    ) -> MenuNav<T> {
        use antibox_ui::searchbar::SearchEvent;
        let bar = match self.bar.as_mut() {
            Some(bar) => bar,
            None => return self.handle_key(conn, ks),
        };
        if ks == 0xFF1B && !bar.text().is_empty() {
            bar.set_text("");
            self.refilter(conn);
            return MenuNav::Handled;
        }
        match bar.handle_key(keycode, state, mapping) {
            SearchEvent::Changed => {
                bar.repaint();
                self.refilter(conn);
                MenuNav::Handled
            }
            SearchEvent::Submitted => self.handle_key(conn, 0xFF0D),
            SearchEvent::Cancelled => self.handle_key(conn, 0xFF1B),
            SearchEvent::None => self.handle_key(conn, ks),
        }
    }

    pub fn handle_bar_button<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        window: u32,
        p: Point,
        button: u8,
    ) -> bool {
        use antibox_ui::searchbar::SearchEvent;
        let bar = match self.bar.as_mut() {
            Some(bar) => bar,
            None => return false,
        };
        if !bar.owns_window(window) {
            return false;
        }
        let changed = bar.handle_button(window, p.x, p.y, button) == SearchEvent::Changed;
        bar.repaint();
        if changed {
            self.refilter(conn);
        }
        true
    }

    pub fn menu_w(&self) -> u16 {
        let labels = if self.all.is_empty() {
            &self.items
        } else {
            &self.all
        };
        crate::render::menu_content_width(
            labels
                .iter()
                .filter(|it| !it.separator)
                .map(|it| it.label.as_str()),
        )
        .max(self.min_w)
    }

    pub fn contains_window(&self, window: u32) -> bool {
        self.window.as_ref().map_or(false, |w| w.id() == window)
            || self.bar.as_ref().map_or(false, |b| b.owns_window(window))
            || self
                .submenu
                .as_ref()
                .map_or(false, |s| s.contains_window(window))
    }

    pub fn show<H: DisplayBackend + ?Sized>(&mut self, conn: &H, pos: Point) {
        self.show_at(conn, pos);
        if let Some(ref win) = self.window {
            let _ = conn.grab_pointer(PointerGrab::new(
                win.id(),
                EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION,
            ));
            let _ = conn.set_input_focus(1, win.id(), 0);
        }
    }

    fn show_at<H: DisplayBackend + ?Sized>(&mut self, conn: &H, pos: Point) {
        let n = self.items.len();
        if n == 0 {
            return;
        }
        self.anchor = pos;
        let ph = (n as u16) * item_h() + 8;
        let sw = conn.screen_width() as i32;
        let sh = conn.screen_height() as i32;
        let pos = crate::render::menu_clamp_pos(pos, self.menu_w() as i32, ph as i32, sw, sh, true);
        let mask = EventMask::BUTTON_PRESS
            | EventMask::EXPOSURE
            | EventMask::POINTER_MOTION
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW
            | EventMask::KEY_PRESS;
        if let Ok(win) = conn.create_window(
            conn.root().as_parent(),
            Rect::new(pos.x, pos.y, self.menu_w() as i32, ph as i32),
            WmWindowClass::InputOutput,
            true,
            mask,
        ) {
            let _ = win.map();
            self.window = Some(win);
            self.pos = pos;
            self.visible = true;
            self.selected = None;
        }
    }

    fn height(&self) -> i32 {
        (self.items.len().max(1) as i32) * item_h() as i32 + 8 + self.bar_off() as i32
    }

    fn contains(&self, root: Point) -> bool {
        root.x >= self.pos.x
            && root.x < self.pos.x + self.menu_w() as i32
            && root.y >= self.pos.y
            && root.y < self.pos.y + self.height()
    }

    fn item_at(&self, root: Point) -> Option<usize> {
        if !self.contains(root) {
            return None;
        }
        let idx = crate::render::menu_item_at(
            root,
            self.pos,
            4 + self.bar_off() as i32,
            item_h() as i32,
            self.items.len(),
        )?;
        if self.items[idx].separator {
            return None;
        }
        Some(idx)
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn hide<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        if let Some(ref mut sub) = self.submenu {
            sub.hide(conn);
            self.submenu = None;
        }
        crate::render::menu_destroy_window(&mut self.window, &mut self.visible);
        self.bar = None;
        self.selected = None;
    }

    pub fn handle_motion<H: DisplayBackend + ?Sized>(&mut self, conn: &H, root: Point) {
        self.motion_root(conn, root);
    }

    fn motion_root<H: DisplayBackend + ?Sized>(&mut self, conn: &H, root: Point) {
        if !self.visible {
            return;
        }
        let prev = self.last_pointer.replace(root);
        if let Some(ref mut sub) = self.submenu {
            if sub.contains(root) {
                sub.motion_root(conn, root);
                return;
            }
        }
        let new_sel = self.item_at(root);
        if new_sel != self.selected {
            if let (Some(p), Some(sub)) = (prev, self.submenu.as_ref()) {
                if crate::render::toward_submenu(
                    p,
                    root,
                    sub.pos,
                    sub.menu_w() as i32,
                    sub.height(),
                ) {
                    return;
                }
            }
            if let Some(mut sub) = self.submenu.take() {
                sub.hide(conn);
            }
            self.selected = new_sel;
            self.paint(conn);
            if let Some(s) = self.selected {
                if self.items[s].submenu.is_some() {
                    self.show_submenu(conn, s);
                }
            }
        }
    }

    fn show_submenu<H: DisplayBackend + ?Sized>(&mut self, conn: &H, idx: usize) {
        let sub_items = match self.items[idx].submenu.clone() {
            Some(s) => s,
            None => return,
        };
        if let Some(bar) = self.bar.as_mut() {
            if !bar.text().is_empty() {
                bar.set_text("");
            }
        }
        let mut sub = Self::new();
        sub.all = sub_items.clone();
        sub.items = sub_items;
        sub.colours = self.colours;
        let sub_pos = Point::new(
            self.pos.x + self.menu_w() as i32,
            self.pos.y + 4 + self.bar_off() as i32 + (idx as i16 * item_h() as i16) as i32,
        );
        sub.show_at(conn, sub_pos);
        self.submenu = Some(Box::new(sub));
    }

    pub fn handle_click(&mut self, root: Point) -> Option<T> {
        self.click_root(root)
    }

    pub fn click_opens_submenu<H: DisplayBackend + ?Sized>(&mut self, conn: &H, root: Point) -> bool {
        self.press_opens_submenu(conn, root)
    }

    fn press_opens_submenu<H: DisplayBackend + ?Sized>(&mut self, conn: &H, root: Point) -> bool {
        if let Some(sub) = self.submenu.as_mut() {
            if sub.contains(root) {
                return sub.press_opens_submenu(conn, root);
            }
        }
        let idx = match self.item_at(root) {
            Some(idx) => idx,
            None => return false,
        };
        if self.items[idx].submenu.is_none() {
            return false;
        }
        if self.selected != Some(idx) {
            if let Some(mut old) = self.submenu.take() {
                old.hide(conn);
            }
            self.selected = Some(idx);
            self.paint(conn);
            self.show_submenu(conn, idx);
        } else if self.submenu.is_none() {
            self.show_submenu(conn, idx);
        }
        true
    }

    fn click_root(&self, root: Point) -> Option<T> {
        if !self.visible {
            return None;
        }
        if let Some(ref sub) = self.submenu {
            if sub.contains(root) {
                return sub.click_root(root);
            }
        }
        let idx = self.item_at(root)?;
        if self.items[idx].submenu.is_some() {
            return None;
        }
        self.items[idx].payload.clone()
    }

    fn next_selectable(&self, from: Option<usize>, dir: i32) -> Option<usize> {
        next_selectable(&self.items, from, dir)
    }

    #[allow(clippy::unnecessary_unwrap)]
    fn deepest(&mut self) -> &mut Self {
        if self.submenu.is_some() {
            self.submenu.as_mut().unwrap().deepest()
        } else {
            self
        }
    }

    fn close_innermost<H: DisplayBackend + ?Sized>(&mut self, conn: &H) -> bool {
        match self.submenu {
            Some(ref mut sub) if sub.submenu.is_some() => sub.close_innermost(conn),
            Some(ref mut sub) => {
                sub.hide(conn);
                self.submenu = None;
                true
            }
            None => false,
        }
    }

    fn open_selected_submenu<H: DisplayBackend + ?Sized>(&mut self, conn: &H) -> MenuNav<T> {
        let d = self.deepest();
        let s = match d.selected {
            Some(s) => s,
            None => return MenuNav::Ignored,
        };
        if d.items[s].submenu.is_none() {
            return MenuNav::Ignored;
        }
        d.show_submenu(conn, s);
        if let Some(ref mut sub) = d.submenu {
            sub.selected = sub.next_selectable(None, 1);
            sub.paint(conn);
        }
        MenuNav::Handled
    }

    pub fn handle_key<H: DisplayBackend + ?Sized>(&mut self, conn: &H, ks: u32) -> MenuNav<T> {
        match ks {
            0xFF52 | 0xFF54 => {
                let dir = if ks == 0xFF52 { -1 } else { 1 };
                let d = self.deepest();
                d.selected = d.next_selectable(d.selected, dir);
                d.paint(conn);
                MenuNav::Handled
            }
            0xFF50 | 0xFF57 => {
                let dir = if ks == 0xFF50 { 1 } else { -1 };
                let d = self.deepest();
                d.selected = d.next_selectable(None, dir);
                d.paint(conn);
                MenuNav::Handled
            }
            0xFF53 => self.open_selected_submenu(conn),
            0xFF51 => {
                if self.close_innermost(conn) {
                    self.reset_filter(conn);
                    MenuNav::Handled
                } else {
                    MenuNav::Ignored
                }
            }
            0xFF0D | 0xFF8D | 0x20 => {
                let (is_sub, payload) = {
                    let d = self.deepest();
                    match d.selected {
                        Some(s) => (d.items[s].submenu.is_some(), d.items[s].payload.clone()),
                        None => return MenuNav::Ignored,
                    }
                };
                if is_sub {
                    self.open_selected_submenu(conn)
                } else {
                    match payload {
                        Some(a) => MenuNav::Activate(a),
                        None => MenuNav::Ignored,
                    }
                }
            }
            0xFF1B => {
                if self.close_innermost(conn) {
                    self.reset_filter(conn);
                    MenuNav::Handled
                } else {
                    MenuNav::Close
                }
            }
            _ if crate::compat::in_range(ks, 0x21, 0x7e) => {
                let key = ks as u8 as char;
                let (matched, count, is_sub, payload) = {
                    let d = self.deepest();
                    let hots: Vec<Option<char>> = d
                        .items
                        .iter()
                        .map(|it| {
                            if it.separator {
                                None
                            } else {
                                crate::render::mnemonic_key(&it.label)
                            }
                        })
                        .collect();
                    match crate::render::menu_hot_match(&hots, d.selected, key) {
                        Some((idx, count)) => {
                            d.selected = Some(idx);
                            d.paint(conn);
                            (
                                true,
                                count,
                                d.items[idx].submenu.is_some(),
                                d.items[idx].payload.clone(),
                            )
                        }
                        None => (false, 0, false, None),
                    }
                };
                if !matched {
                    MenuNav::Ignored
                } else if count == 1 {
                    if is_sub {
                        self.open_selected_submenu(conn)
                    } else {
                        match payload {
                            Some(a) => MenuNav::Activate(a),
                            None => MenuNav::Handled,
                        }
                    }
                } else {
                    MenuNav::Handled
                }
            }
            _ => MenuNav::Ignored,
        }
    }

    pub fn paint<H: DisplayBackend + ?Sized>(&self, conn: &H) {
        let win = match &self.window {
            Some(win) => win,
            None => return,
        };
        let c = self.colours;
        let w = self.menu_w();
        let h = self.height().max(1) as u16;
        let pm = match conn.create_pixmap(w, h, conn.screen_depth()) {
            Ok(pm) => pm,
            Err(_) => return,
        };
        let g = match conn.create_graphics(pm) {
            Ok(g) => g,
            Err(_) => return,
        };
        let _ = g.set_font(&FontSpec::role(
            FontRole::Menu,
            antibox_ui::metrics::font_pt(),
        ));
        let (light, dark) = crate::render::draw_menu_frame(&*g, w, h, c.bg);
        for (i, item) in self.items.iter().enumerate() {
            let y = 4 + self.bar_off() + i as i16 * item_h() as i16;
            if item.separator {
                crate::render::draw_menu_separator(
                    &*g,
                    4,
                    y + (item_h() / 2) as i16,
                    w - 8,
                    light,
                    dark,
                );
            } else {
                let sel = self.selected == Some(i);
                if sel {
                    crate::render::fill_menu_selection(&*g, 2, y, w - 4, item_h(), c.sel_bg);
                }
                let fg = if sel { c.sel_fg } else { c.fg };
                let _ = g.set_background(if sel { c.sel_bg } else { c.bg });
                crate::render::draw_text_mnemonic(
                    &*g,
                    8,
                    antibox_ui::metrics::baseline(y as i32, item_h() as i32) as i16,
                    &item.label,
                    fg,
                );
                if item.submenu.is_some() {
                    crate::render::draw_submenu_arrow(
                        &*g,
                        w as i16 - 12,
                        y + item_h() as i16 / 2,
                        7,
                        if sel { c.sel_fg } else { c.fg },
                    );
                }
                if i + 1 < self.items.len() && !self.items[i + 1].separator {
                    crate::render::draw_menu_row_rule(&*g, 2, y + item_h() as i16 - 1, w - 4);
                }
            }
        }
        if let Ok(wg) = conn.create_graphics(win.id()) {
            let _ = wg.copy_from(pm, Rect::px(0, 0, w, h), Point::ZERO);
        }
        let _ = conn.free_pixmap(pm);
        if let Some(ref bar) = self.bar {
            bar.repaint();
        }
        if let Some(ref sub) = self.submenu {
            sub.paint(conn);
        }
    }
}
