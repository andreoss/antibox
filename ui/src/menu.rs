use crate::menu_tree::{self, MenuNode};
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;

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
            bg: crate::theme::menu_bg(),
            fg: crate::theme::text(),
            sel_bg: crate::theme::menu_sel_bg(),
            sel_fg: crate::theme::menu_sel_fg(),
        }
    }
}

impl MenuColors {
    pub fn with_bg(bg: antibox_core::colour::Colour) -> MenuColors {
        MenuColors {
            bg,
            fg: crate::theme::contrast(0x000000, bg),
            sel_bg: crate::theme::menu_sel_bg(),
            sel_fg: crate::theme::menu_sel_fg(),
        }
    }
}

pub fn item_h() -> u16 {
    crate::metrics::menu_item_height() as u16
}

fn band_h() -> i32 {
    (item_h() as i32) * 3 / 4
}

fn frame_pad() -> i32 {
    4
}

pub enum MenuNav<T> {
    Ignored,
    Handled,
    Close,
    Activate(T),
}

enum Hit {
    Item(usize),
    ScrollUp,
    ScrollDown,
    Inert,
}

pub struct MenuView<T> {
    window: Option<Box<dyn WindowHandle>>,
    items: Vec<MenuNode<T>>,
    all: Vec<MenuNode<T>>,
    bar: Option<crate::searchbar::SearchBar>,
    pos: Point,
    anchor: Point,
    last_pointer: Option<Point>,
    pub visible: bool,
    selected: Option<usize>,
    offset: usize,
    vis_rows: usize,
    scrollable: bool,
    submenu: Option<Box<MenuView<T>>>,
    pub colours: MenuColors,
}

impl<T: Clone> Default for MenuView<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn bar_h() -> i16 {
    crate::metrics::field_height() as i16
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
            offset: 0,
            vis_rows: 0,
            scrollable: false,
            submenu: None,
            colours: MenuColors::default(),
        }
    }

    pub fn with_nodes(nodes: Vec<MenuNode<T>>) -> Self {
        MenuView {
            items: nodes.clone(),
            all: nodes,
            ..Self::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.all.is_empty()
    }

    fn bar_off(&self) -> i16 {
        if self.bar.is_some() {
            bar_h() + crate::metrics::pad() as i16
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
        let m = crate::metrics::pad() as i16;
        let bw = (w as i16 - m * 2).max(1) as u16;
        if let Ok(mut bar) =
            crate::searchbar::SearchBar::new(rb, win_id, m, m, bw, bar_h() as u16)
        {
            bar.set_focus(true);
            bar.show();
            self.bar = Some(bar);
            self.reposition(rb.screen_width() as i32, rb.screen_height() as i32);
        }
    }

    fn fit_rows(&mut self, sh: i32) {
        let n = self.items.len();
        let ih = item_h() as i32;
        let avail = sh - frame_pad() * 2 - self.bar_off() as i32;
        if n as i32 * ih <= avail {
            self.vis_rows = n;
            self.scrollable = false;
        } else {
            self.vis_rows = ((avail - 2 * band_h()) / ih).max(1) as usize;
            self.scrollable = true;
        }
        self.clamp_offset();
    }

    fn clamp_offset(&mut self) {
        let max = self.items.len().saturating_sub(self.vis_rows.max(1));
        if self.offset > max {
            self.offset = max;
        }
    }

    fn reposition(&mut self, sw: i32, sh: i32) {
        self.fit_rows(sh);
        let win = match self.window {
            Some(ref w) => w,
            None => return,
        };
        let ph = self.height().max(1);
        let pos =
            crate::menurender::menu_clamp_pos(self.anchor, self.menu_w() as i32, ph, sw, sh, true);
        let _ = win.configure(Some(pos.x), Some(pos.y), None, Some(ph as u16));
        self.pos = pos;
    }

    fn apply_needle(&mut self, needle: &str) {
        let keep = self.selected.and_then(|s| {
            self.items
                .get(s)
                .map(|it| crate::menurender::parse_mnemonic(it.title()).0)
        });
        self.items = if needle.is_empty() {
            self.all.clone()
        } else {
            let mut out = Vec::new();
            menu_tree::collect_leaves(&menu_tree::filter_nodes(&self.all, needle), &mut out);
            out
        };
        self.offset = 0;
        self.selected = keep
            .and_then(|k| {
                self.items
                    .iter()
                    .position(|it| crate::menurender::parse_mnemonic(it.title()).0 == k)
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

    fn handle_key_input<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        keycode: u32,
        state: u16,
        mapping: &KeyboardMapping,
        ks: u32,
    ) -> MenuNav<T> {
        use crate::searchbar::SearchEvent;
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

    fn handle_bar_button<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        window: u32,
        p: Point,
        button: u8,
    ) -> bool {
        use crate::searchbar::SearchEvent;
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

    fn icon_col(&self) -> i16 {
        if self.items.iter().any(|it| it.icon().is_some()) {
            (crate::listview::row_icon_px() + crate::metrics::gap() as u16) as i16
        } else {
            0
        }
    }

    pub fn menu_w(&self) -> u16 {
        let labels = if self.all.is_empty() {
            &self.items
        } else {
            &self.all
        };
        crate::menurender::menu_content_width(
            labels
                .iter()
                .filter(|it| !it.is_separator())
                .map(|it| it.title()),
        ) + self.icon_col() as u16
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
        if self.items.is_empty() {
            return;
        }
        self.anchor = pos;
        let sw = conn.screen_width() as i32;
        let sh = conn.screen_height() as i32;
        self.fit_rows(sh);
        let ph = self.height();
        let pos = crate::menurender::menu_clamp_pos(pos, self.menu_w() as i32, ph, sw, sh, true);
        let mask = EventMask::BUTTON_PRESS
            | EventMask::EXPOSURE
            | EventMask::POINTER_MOTION
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW
            | EventMask::KEY_PRESS;
        if let Ok(win) = conn.create_window(
            conn.root().as_parent(),
            Rect::new(pos.x, pos.y, self.menu_w() as i32, ph),
            WmWindowClass::InputOutput,
            true,
            mask,
        ) {
            let _ = win.map();
            self.window = Some(win);
            self.pos = pos;
            self.visible = true;
            self.selected = None;
            self.offset = 0;
        }
    }

    fn height(&self) -> i32 {
        let ih = item_h() as i32;
        let rows = if self.scrollable {
            2 * band_h() + self.vis_rows as i32 * ih
        } else {
            (self.items.len().max(1) as i32) * ih
        };
        rows + frame_pad() * 2 + self.bar_off() as i32
    }

    fn contains(&self, root: Point) -> bool {
        root.x >= self.pos.x
            && root.x < self.pos.x + self.menu_w() as i32
            && root.y >= self.pos.y
            && root.y < self.pos.y + self.height()
    }

    fn tree_contains(&self, root: Point) -> bool {
        self.contains(root)
            || self
                .submenu
                .as_ref()
                .map_or(false, |s| s.tree_contains(root))
    }

    fn rows_top(&self) -> i32 {
        frame_pad()
            + self.bar_off() as i32
            + if self.scrollable { band_h() } else { 0 }
    }

    fn row_y(&self, idx: usize) -> i32 {
        self.rows_top() + (idx as i32 - self.offset as i32) * item_h() as i32
    }

    fn hit_at(&self, root: Point) -> Hit {
        if !self.contains(root) {
            return Hit::Inert;
        }
        let wy = root.y - self.pos.y - frame_pad() - self.bar_off() as i32;
        if wy < 0 {
            return Hit::Inert;
        }
        let ih = item_h() as i32;
        if self.scrollable {
            let rows_h = self.vis_rows as i32 * ih;
            if wy < band_h() {
                return Hit::ScrollUp;
            }
            if wy >= band_h() + rows_h {
                if wy < 2 * band_h() + rows_h {
                    return Hit::ScrollDown;
                }
                return Hit::Inert;
            }
            let idx = self.offset + ((wy - band_h()) / ih) as usize;
            if idx < self.items.len() {
                return Hit::Item(idx);
            }
            return Hit::Inert;
        }
        let idx = (wy / ih) as usize;
        if idx < self.items.len() {
            Hit::Item(idx)
        } else {
            Hit::Inert
        }
    }

    fn item_at(&self, root: Point) -> Option<usize> {
        match self.hit_at(root) {
            Hit::Item(idx) if !self.items[idx].is_separator() => Some(idx),
            _ => None,
        }
    }

    fn scroll_by<H: DisplayBackend + ?Sized>(&mut self, conn: &H, d: i32) {
        if !self.scrollable {
            return;
        }
        let max = self.items.len().saturating_sub(self.vis_rows) as i32;
        self.offset = (self.offset as i32 + d).clamp(0, max) as usize;
        self.paint(conn);
    }

    fn ensure_visible(&mut self) {
        let s = match self.selected {
            Some(s) => s,
            None => return,
        };
        if !self.scrollable {
            return;
        }
        if s < self.offset {
            self.offset = s;
        } else if s >= self.offset + self.vis_rows {
            self.offset = s + 1 - self.vis_rows;
        }
        self.clamp_offset();
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn hide<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        if let Some(ref mut sub) = self.submenu {
            sub.hide(conn);
            self.submenu = None;
        }
        crate::menurender::menu_destroy_window(&mut self.window, &mut self.visible);
        self.bar = None;
        self.selected = None;
        self.offset = 0;
    }

    fn motion_root<H: DisplayBackend + ?Sized>(&mut self, conn: &H, root: Point) {
        if !self.visible {
            return;
        }
        let prev = self.last_pointer.replace(root);
        if let Some(ref mut sub) = self.submenu {
            if sub.tree_contains(root) {
                sub.motion_root(conn, root);
                return;
            }
        }
        let new_sel = self.item_at(root);
        if new_sel != self.selected {
            if let (Some(p), Some(sub)) = (prev, self.submenu.as_ref()) {
                if crate::menurender::toward_submenu(
                    p,
                    root,
                    sub.pos,
                    sub.menu_w() as i32,
                    sub.height(),
                ) {
                    return;
                }
            }
            if !self.contains(root) && new_sel.is_none() {
                return;
            }
            if let Some(mut sub) = self.submenu.take() {
                sub.hide(conn);
            }
            self.selected = new_sel;
            self.paint(conn);
            if let Some(s) = self.selected {
                if self.items[s].children().is_some() {
                    self.show_submenu(conn, s);
                }
            }
        }
    }

    fn show_submenu<H: DisplayBackend + ?Sized>(&mut self, conn: &H, idx: usize) {
        let sub_items = match self.items[idx].children() {
            Some(s) => s.to_vec(),
            None => return,
        };
        if let Some(bar) = self.bar.as_mut() {
            if !bar.text().is_empty() {
                bar.set_text("");
            }
        }
        let mut sub = Self::with_nodes(sub_items);
        sub.colours = self.colours;
        let sub_pos = Point::new(
            self.pos.x + self.menu_w() as i32 - antibox_core::scale::scaled(2),
            self.pos.y + self.row_y(idx),
        );
        sub.show_at(conn, sub_pos);
        self.submenu = Some(Box::new(sub));
    }

    fn press_opens_submenu<H: DisplayBackend + ?Sized>(&mut self, conn: &H, root: Point) -> bool {
        if let Some(sub) = self.submenu.as_mut() {
            if sub.tree_contains(root) {
                return sub.press_opens_submenu(conn, root);
            }
        }
        let idx = match self.item_at(root) {
            Some(idx) => idx,
            None => return false,
        };
        if self.items[idx].children().is_none() {
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
            if sub.tree_contains(root) {
                return sub.click_root(root);
            }
        }
        let idx = self.item_at(root)?;
        self.items[idx].payload().cloned()
    }

    fn next_selectable(&self, from: Option<usize>, dir: i32) -> Option<usize> {
        let n = self.items.len();
        if n == 0 {
            return None;
        }
        let mut i = from.map_or(if dir > 0 { -1 } else { n as i32 }, |s| s as i32);
        for _ in 0..n {
            i = ((i + dir) % n as i32 + n as i32) % n as i32;
            if !self.items[i as usize].is_separator() {
                return Some(i as usize);
            }
        }
        None
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
        if d.items[s].children().is_none() {
            return MenuNav::Ignored;
        }
        d.show_submenu(conn, s);
        if let Some(ref mut sub) = d.submenu {
            sub.selected = sub.next_selectable(None, 1);
            sub.paint(conn);
        }
        MenuNav::Handled
    }

    fn move_selection<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        from_current: bool,
        dir: i32,
    ) -> MenuNav<T> {
        let d = self.deepest();
        let from = if from_current { d.selected } else { None };
        d.selected = d.next_selectable(from, dir);
        d.ensure_visible();
        d.paint(conn);
        MenuNav::Handled
    }

    fn handle_key<H: DisplayBackend + ?Sized>(&mut self, conn: &H, ks: u32) -> MenuNav<T> {
        match ks {
            0xFF52 | 0xFF54 => self.move_selection(conn, true, if ks == 0xFF52 { -1 } else { 1 }),
            0xFF50 | 0xFF57 => self.move_selection(conn, false, if ks == 0xFF50 { 1 } else { -1 }),
            0xFF55 | 0xFF56 => {
                let d = self.deepest();
                let step = d.vis_rows.max(1) as i32 * if ks == 0xFF55 { -1 } else { 1 };
                d.scroll_by(conn, step);
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
                        Some(s) => (
                            d.items[s].children().is_some(),
                            d.items[s].payload().cloned(),
                        ),
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
            _ if (0x21..=0x7e).contains(&ks) => {
                let key = ks as u8 as char;
                let (matched, count, is_sub, payload) = {
                    let d = self.deepest();
                    let hots: Vec<Option<char>> = d
                        .items
                        .iter()
                        .map(|it| {
                            if it.is_separator() {
                                None
                            } else {
                                crate::menurender::mnemonic_key(it.title())
                            }
                        })
                        .collect();
                    match crate::menurender::menu_hot_match(&hots, d.selected, key) {
                        Some((idx, count)) => {
                            d.selected = Some(idx);
                            d.ensure_visible();
                            d.paint(conn);
                            (
                                true,
                                count,
                                d.items[idx].children().is_some(),
                                d.items[idx].payload().cloned(),
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

    fn wheel<H: DisplayBackend + ?Sized>(&mut self, conn: &H, root: Point, up: bool) {
        if let Some(sub) = self.submenu.as_mut() {
            if sub.tree_contains(root) {
                return sub.wheel(conn, root, up);
            }
        }
        self.scroll_by(conn, if up { -3 } else { 3 });
    }

    pub fn handle_event<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        event: &BackendEvent,
    ) -> MenuNav<T> {
        if !self.visible {
            return MenuNav::Ignored;
        }
        match event {
            BackendEvent::ButtonPress {
                window,
                point,
                root,
                button,
                ..
            } => {
                if self.handle_bar_button(conn, *window, *point, *button) {
                    return MenuNav::Handled;
                }
                if *button == 4 || *button == 5 {
                    self.wheel(conn, *root, *button == 4);
                    return MenuNav::Handled;
                }
                if !self.tree_contains(*root) {
                    return MenuNav::Close;
                }
                if self.press_opens_submenu(conn, *root) {
                    return MenuNav::Handled;
                }
                match self.scroll_press(conn, *root) {
                    Some(nav) => nav,
                    None => match self.click_root(*root) {
                        Some(p) => MenuNav::Activate(p),
                        None => MenuNav::Handled,
                    },
                }
            }
            BackendEvent::MotionNotify { root, .. } => {
                self.motion_root(conn, *root);
                MenuNav::Handled
            }
            BackendEvent::Expose { window, .. } if self.contains_window(*window) => {
                self.paint(conn);
                MenuNav::Handled
            }
            BackendEvent::KeyPress { keycode, state, .. } => {
                let ks = crate::keymap::keysym_for_keycode(conn, *keycode);
                let nav = match crate::keymap::keymap(conn) {
                    Some(m) => self.handle_key_input(conn, *keycode, *state, &m, ks),
                    None => self.handle_key(conn, ks),
                };
                match nav {
                    MenuNav::Ignored => MenuNav::Handled,
                    nav => nav,
                }
            }
            _ => MenuNav::Ignored,
        }
    }

    fn scroll_press<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        root: Point,
    ) -> Option<MenuNav<T>> {
        if let Some(sub) = self.submenu.as_mut() {
            if sub.tree_contains(root) {
                return sub.scroll_press(conn, root);
            }
        }
        match self.hit_at(root) {
            Hit::ScrollUp => {
                self.scroll_by(conn, -1);
                Some(MenuNav::Handled)
            }
            Hit::ScrollDown => {
                self.scroll_by(conn, 1);
                Some(MenuNav::Handled)
            }
            _ => None,
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
            crate::metrics::font_pt(),
        ));
        let (light, dark) = crate::menurender::draw_menu_frame(&*g, w, h, c.bg);
        let ih = item_h();
        let icon_col = self.icon_col();
        let top = self.rows_top() as i16;
        let last = if self.scrollable {
            (self.offset + self.vis_rows).min(self.items.len())
        } else {
            self.items.len()
        };
        for (v, item) in self.items[self.offset..last].iter().enumerate() {
            let idx = self.offset + v;
            let y = top + (v as i16) * ih as i16;
            if item.is_separator() {
                crate::menurender::draw_menu_separator(
                    &*g,
                    frame_pad() as i16,
                    y + (ih / 2) as i16 - 1,
                    w - 2 * frame_pad() as u16,
                    light,
                    dark,
                );
                continue;
            }
            let sel = self.selected == Some(idx);
            if sel {
                crate::menurender::fill_menu_selection(&*g, 2, y, w - 4, ih, c.sel_bg);
            }
            let fg = if sel { c.sel_fg } else { c.fg };
            let _ = g.set_background(if sel { c.sel_bg } else { c.bg });
            let mut tx = 8;
            if let Some(ico) = item.icon() {
                let _ = g.draw_pixmap(tx, y + (ih as i16 - ico.height as i16) / 2, ico);
            }
            tx += icon_col;
            crate::menurender::draw_text_mnemonic(
                &*g,
                tx,
                crate::metrics::baseline(y as i32, ih as i32) as i16,
                item.title(),
                fg,
            );
            if item.children().is_some() {
                crate::menurender::draw_submenu_arrow(
                    &*g,
                    w as i16 - antibox_core::scale::scaled(12) as i16,
                    y + ih as i16 / 2,
                    antibox_core::scale::scaled(7) as i16,
                    fg,
                );
            }
        }
        if self.scrollable {
            self.paint_scroll_bands(&*g, w, h);
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

    fn paint_scroll_bands(&self, g: &dyn GraphicsContext, w: u16, _h: u16) {
        let c = self.colours;
        let bh = band_h();
        let ih = item_h() as i32;
        let top_y = frame_pad() + self.bar_off() as i32;
        let bot_y = top_y + bh + self.vis_rows as i32 * ih;
        let cx = (w / 2) as i16;
        let asz = antibox_core::scale::scaled(7).max(5) as i16;
        let can_up = self.offset > 0;
        let can_down = self.offset + self.vis_rows < self.items.len();
        let arrow = |y: i32, dir: crate::theme::Arrow, on: bool| {
            let colour = if on {
                c.fg
            } else {
                crate::theme::shadow()
            };
            crate::theme::arrow_glyph(
                g,
                cx,
                (y + bh / 2) as i16,
                asz,
                dir,
                colour,
            );
        };
        arrow(top_y, crate::theme::Arrow::Up, can_up);
        arrow(bot_y, crate::theme::Arrow::Down, can_down);
    }
}

#[cfg(test)]
mod menu_filter_tests {
    use crate::menu_tree::{collect_leaves, filter_nodes, MenuNode};

    fn leaf(label: &str) -> MenuNode<u32> {
        MenuNode::leaf(label, 0)
    }

    fn leaves(rows: &[MenuNode<u32>], needle: &str) -> Vec<String> {
        let mut out = Vec::new();
        collect_leaves(&filter_nodes(rows, needle), &mut out);
        out.iter().map(|n| n.title().to_string()).collect()
    }

    #[test]
    fn matching_descends_into_submenus() {
        let rows = vec![
            MenuNode::group("Network", vec![leaf("Firefox"), leaf("Thunderbird")]),
            MenuNode::group("Game", vec![leaf("Mines")]),
            MenuNode::separator(),
            leaf("Show Desktop"),
        ];
        assert_eq!(leaves(&rows, "fire"), ["Firefox"]);
    }

    #[test]
    fn matching_is_case_insensitive_and_returns_leaves_only() {
        let rows = vec![
            MenuNode::group("Utility", vec![leaf("Calculator"), leaf("Vim")]),
            leaf("Show Desktop"),
        ];
        assert_eq!(leaves(&rows, "DESKTOP"), ["Show Desktop"]);
        assert_eq!(leaves(&rows, "").len(), 3);
    }

    #[test]
    fn category_name_surfaces_its_children() {
        let rows = vec![
            MenuNode::group("Game", vec![leaf("Mines")]),
            MenuNode::group("Network", vec![leaf("Firefox")]),
        ];
        assert_eq!(leaves(&rows, "game"), ["Mines"]);
        assert_eq!(leaves(&rows, "network"), ["Firefox"]);
    }

    #[test]
    fn nested_category_name_matches_deeper_children() {
        let rows = vec![MenuNode::group(
            "Game",
            vec![MenuNode::group("BoardGame", vec![leaf("Chess")])],
        )];
        assert_eq!(leaves(&rows, "board"), ["Chess"]);
    }
}
