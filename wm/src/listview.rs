use crate::menu_tree::{self, FlatEntry, FlatRow, MenuNode};
use crate::render;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use antibox_core::scale::scaled;
use std::sync::Arc;

pub(crate) fn row_h() -> i16 {
    antibox_ui::metrics::menu_item_height() as i16
}

pub(crate) fn row_icon_px() -> u16 {
    antibox_ui::metrics::icon().min(row_h() as i32 - 2).max(8) as u16
}

pub(crate) fn bar_h() -> i16 {
    (antibox_ui::metrics::field_height() + antibox_ui::metrics::gap()) as i16
}

pub(crate) fn pad() -> i16 {
    antibox_ui::metrics::pad() as i16
}

pub(crate) fn sb_w() -> i16 {
    scaled(16) as i16
}

pub(crate) fn indent_w() -> i16 {
    scaled(12) as i16
}

const TOP_PAD: i16 = 3;

pub enum ListNav<T> {
    Ignored,
    Handled,
    Close,
    Activate(T),
}

pub enum Place {
    Anchor(Point),
    At(Point),
    Pointer,
    Centre,
}

pub struct ListView<T> {
    pub window: Option<Box<dyn WindowHandle>>,
    pub items: Vec<FlatRow<T>>,
    tree: Vec<MenuNode<T>>,
    bar: Option<antibox_ui::searchbar::SearchBar>,
    pub visible: bool,
    pub selected: Option<usize>,
    pub offset: i16,
    pub scroll_drag: bool,
    drag_grab: i16,
    pos: Point,
    anchor: Option<(Point, bool)>,
    pub w: u16,
    pub h: u16,
    pub colours: crate::menu::MenuColors,
    fixed: bool,
    managed: bool,
    cap: i32,
}

impl<T: Clone> Default for ListView<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> ListView<T> {
    pub fn new() -> Self {
        ListView {
            window: None,
            items: Vec::new(),
            tree: Vec::new(),
            bar: None,
            visible: false,
            selected: None,
            offset: 0,
            scroll_drag: false,
            drag_grab: 0,
            pos: Point::new(0, 0),
            anchor: None,
            w: 0,
            h: 0,
            colours: crate::menu::MenuColors::default(),
            fixed: false,
            managed: false,
            cap: 0,
        }
    }

    pub fn bar_off(&self) -> i16 {
        if self.bar.is_some() {
            pad() * 2 + bar_h()
        } else {
            TOP_PAD
        }
    }

    pub fn list_top(&self) -> i16 {
        self.bar_off()
    }

    pub fn total_rows(&self) -> i16 {
        self.items.len() as i16
    }

    pub fn vis_rows(&self) -> i16 {
        ((self.h as i16 - self.list_top() - TOP_PAD) / row_h()).max(1)
    }

    pub fn needs_sb(&self) -> bool {
        self.total_rows() > self.vis_rows()
    }

    pub fn list_w(&self) -> i16 {
        (self.w as i16 - if self.needs_sb() { sb_w() } else { 0 }).max(1)
    }

    pub fn max_offset(&self) -> i16 {
        (self.total_rows() - self.vis_rows()).max(0)
    }

    pub fn clamp_offset(&mut self) {
        self.offset = self.offset.clamp(0, self.max_offset());
    }

    pub fn sb_x(&self) -> i16 {
        self.list_w()
    }

    pub fn sb_top(&self) -> i16 {
        self.list_top()
    }

    pub fn trough_h(&self) -> i16 {
        (self.h as i16 - self.sb_top() - 2 * sb_w()).max(1)
    }

    pub fn thumb(&self) -> (i16, i16) {
        let total = self.total_rows().max(1);
        let th = ((self.trough_h() as i32 * self.vis_rows() as i32 / total as i32) as i16).max(12);
        let mo = self.max_offset();
        let ty = self.sb_top()
            + sb_w()
            + if mo > 0 {
                (self.trough_h() - th) * self.offset / mo
            } else {
                0
            };
        (ty, th)
    }

    pub fn set_size(&mut self, w: u16, h: u16) {
        self.w = w;
        self.h = h;
        self.fixed = true;
        self.clamp_offset();
        self.sync_bar_rect();
        if let Some(win) = &self.window {
            let _ = win.configure(None, None, Some(self.w), Some(self.h));
        }
    }

    fn sync_bar_rect(&mut self) {
        let w = self.w;
        if let Some(bar) = self.bar.as_mut() {
            let bw = (w as i16 - pad() * 2).max(1) as u16;
            bar.set_rect(pad(), pad(), bw, bar_h() as u16);
        }
    }

    pub fn sync_geometry<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        if !self.fixed {
            let sh = conn.screen_height() as i32;
            let cap = if self.cap > 0 { self.cap } else { sh * 3 / 4 };
            self.h = self
                .desired_height()
                .clamp(scaled(120), cap.max(scaled(120))) as u16;
        }
        self.clamp_offset();
        if let Some((a, up)) = self.anchor {
            self.pos = if up {
                Point::new(a.x, (a.y - self.h as i32).max(0))
            } else {
                let sw = conn.screen_width() as i32;
                let sh = conn.screen_height() as i32;
                Point::new(
                    a.x.min(sw - self.w as i32).max(0),
                    a.y.min(sh - self.h as i32).max(0),
                )
            };
        }
        if let Some(win) = &self.window {
            if !self.managed {
                let _ = win.configure(
                    Some(self.pos.x),
                    Some(self.pos.y),
                    Some(self.w),
                    Some(self.h),
                );
            }
        }
    }

    pub fn desired_height(&self) -> i32 {
        let rows = self.total_rows().max(1) as i32 * row_h() as i32;
        pad() as i32 * 3 + bar_h() as i32 + rows
    }

    pub fn needle(&self) -> String {
        self.bar
            .as_ref()
            .map(|b| b.text().to_lowercase())
            .unwrap_or_default()
    }

    pub fn set_tree(&mut self, nodes: Vec<MenuNode<T>>) {
        self.tree = nodes;
        self.apply_filter();
        self.clamp_offset();
    }

    fn apply_filter(&mut self) {
        let needle = self.needle();
        self.items = if needle.is_empty() {
            menu_tree::flatten_nodes(&self.tree)
        } else {
            menu_tree::flatten_nodes(&menu_tree::filter_nodes(&self.tree, &needle))
        };
    }

    pub fn owns_window(&self, id: u32) -> bool {
        self.window.as_ref().map_or(false, |w| w.id() == id)
            || self.bar.as_ref().map_or(false, |b| b.owns_window(id))
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.pos.x
            && p.x < self.pos.x + self.w as i32
            && p.y >= self.pos.y
            && p.y < self.pos.y + self.h as i32
    }

    pub fn show<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        nodes: Vec<MenuNode<T>>,
        place: Place,
        w: u16,
        max_h: i32,
    ) {
        self.show_with(conn, nodes, place, w, max_h, true, |_, _| {});
    }

    pub fn show_with<H: DisplayBackend + ?Sized, F>(
        &mut self,
        conn: &H,
        nodes: Vec<MenuNode<T>>,
        place: Place,
        w: u16,
        max_h: i32,
        override_redirect: bool,
        on_created: F,
    ) where
        F: FnOnce(&H, u32),
    {
        self.set_tree(nodes);
        if self.items.is_empty() {
            return;
        }
        self.managed = !override_redirect;
        self.w = if w > 0 { w } else { scaled(280) as u16 };
        self.cap = max_h;
        self.anchor = match place {
            Place::Anchor(a) => Some((a, true)),
            Place::At(a) => Some((a, false)),
            _ => None,
        };
        self.sync_geometry(conn);
        let sw = conn.screen_width() as i32;
        let sh = conn.screen_height() as i32;
        let (ww, hh) = (self.w as i32, self.h as i32);
        let (x, y) = match place {
            Place::Anchor(_) | Place::At(_) => (self.pos.x, self.pos.y),
            Place::Pointer => match conn.query_pointer(conn.root().read_id()) {
                Ok(p) => (
                    (p.root_x as i32 - ww / 2).clamp(0, (sw - ww).max(0)),
                    (p.root_y as i32 + scaled(8)).clamp(0, (sh - hh).max(0)),
                ),
                Err(_) => (((sw - ww) / 2).max(0), ((sh - hh) / 2).max(0)),
            },
            Place::Centre => (((sw - ww) / 2).max(0), ((sh - hh) / 2).max(0)),
        };
        let mask = EventMask::EXPOSURE
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::POINTER_MOTION
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW
            | EventMask::KEY_PRESS
            | EventMask::STRUCTURE_NOTIFY;
        let win = match conn.create_window(
            conn.root().as_parent(),
            Rect::new(x, y, ww, hh),
            WmWindowClass::InputOutput,
            override_redirect,
            mask,
        ) {
            Ok(win) => win,
            Err(_) => return,
        };
        let id = win.id();
        on_created(conn, id);
        let _ = win.map();
        self.window = Some(win);
        self.pos = Point::new(x, y);
        self.visible = true;
        self.offset = 0;
        self.select_first();
        self.sync_geometry(conn);
    }

    pub fn set_bar(&mut self, bar: antibox_ui::searchbar::SearchBar) {
        self.bar = Some(bar);
    }

    pub fn bar(&self) -> Option<&antibox_ui::searchbar::SearchBar> {
        self.bar.as_ref()
    }

    pub fn bar_mut(&mut self) -> Option<&mut antibox_ui::searchbar::SearchBar> {
        self.bar.as_mut()
    }

    pub fn pos(&self) -> Point {
        self.pos
    }

    pub fn enable_filter(&mut self, rb: &Arc<dyn RenderBackend>) {
        let win_id = match self.window.as_ref().map(|w| w.id()) {
            Some(id) => id,
            None => return,
        };
        if self.bar.is_some() {
            return;
        }
        let bw = (self.w as i16 - pad() * 2).max(1) as u16;
        if let Ok(mut bar) =
            antibox_ui::searchbar::SearchBar::new(rb, win_id, pad(), pad(), bw, bar_h() as u16)
        {
            bar.set_focus(true);
            bar.show();
            self.bar = Some(bar);
        }
    }

    pub fn hide<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        render::menu_destroy_window(&mut self.window, &mut self.visible);
        self.bar = None;
        self.selected = None;
        self.offset = 0;
        self.scroll_drag = false;
        self.anchor = None;
        self.items.clear();
        self.tree.clear();
        let _ = conn.flush();
    }

    pub fn row_at(&self, p: Point) -> Option<usize> {
        let rel = p.y - self.pos.y - self.list_top() as i32;
        if rel < 0 {
            return None;
        }
        let vr = rel / row_h() as i32;
        if vr >= self.vis_rows() as i32 {
            return None;
        }
        let row = (self.offset + vr as i16) as usize;
        if row < self.items.len() {
            Some(row)
        } else {
            None
        }
    }

    pub fn select_first(&mut self) {
        self.selected = self.next_selectable(None, 1);
    }

    pub fn next_selectable(&self, from: Option<usize>, dir: i32) -> Option<usize> {
        self.next_matching(from, dir, |r| r.selectable())
    }

    pub fn next_leaf(&self, from: Option<usize>, dir: i32) -> Option<usize> {
        self.next_matching(from, dir, |r| r.payload().is_some())
    }

    fn next_matching<F>(&self, from: Option<usize>, dir: i32, keep: F) -> Option<usize>
    where
        F: Fn(&FlatRow<T>) -> bool,
    {
        let n = self.items.len();
        if n == 0 {
            return None;
        }
        let mut i = match from {
            Some(f) => f as i32 + dir,
            None => {
                if dir > 0 {
                    0
                } else {
                    n as i32 - 1
                }
            }
        };
        let mut steps = 0;
        while steps <= n as i32 {
            if i < 0 {
                i = n as i32 - 1;
            }
            if i >= n as i32 {
                i = 0;
            }
            if keep(&self.items[i as usize]) {
                return Some(i as usize);
            }
            i += dir;
            steps += 1;
        }
        None
    }

    pub fn scroll_by(&mut self, d: i16) {
        self.offset += d;
        self.clamp_offset();
    }

    pub fn scroll_to_selected(&mut self) {
        let s = match self.selected {
            Some(s) => s as i16,
            None => return,
        };
        if s < self.offset {
            self.offset = s;
        } else if s >= self.offset + self.vis_rows() {
            self.offset = s - self.vis_rows() + 1;
        }
        self.clamp_offset();
    }

    pub fn sb_hit(&mut self, p: Point) -> bool {
        if !self.needs_sb() {
            return false;
        }
        let (px, py) = (p.x - self.pos.x, p.y - self.pos.y);
        if px < self.sb_x() as i32 || px >= self.sb_x() as i32 + sb_w() as i32 {
            return false;
        }
        let bwid = sb_w();
        if py < (self.sb_top() + bwid) as i32 {
            self.scroll_by(-1);
            return true;
        }
        if py >= self.h as i32 - bwid as i32 {
            self.scroll_by(1);
            return true;
        }
        let (ty, th) = self.thumb();
        if py >= ty as i32 && py < ty as i32 + th as i32 {
            self.scroll_drag = true;
            self.drag_grab = py as i16 - ty;
            return true;
        }
        self.scroll_drag = true;
        self.drag_grab = th / 2;
        true
    }

    pub fn end_drag(&mut self) {
        self.scroll_drag = false;
    }

    pub fn drag_to(&mut self, p: Point) {
        if !self.scroll_drag {
            return;
        }
        let py = p.y - self.pos.y;
        let (_, th) = self.thumb();
        let rel = py as i16 - self.sb_top() - sb_w() - self.drag_grab;
        let span = (self.trough_h() - th).max(1);
        self.offset = (rel as i32 * self.max_offset() as i32 / span as i32) as i16;
        self.clamp_offset();
    }

    pub fn toggle_row<H: DisplayBackend + ?Sized>(&mut self, conn: &H, idx: usize) -> bool {
        let path = match self.items.get(idx).map(|r| &r.entry) {
            Some(FlatEntry::Group { path, .. }) => path.clone(),
            _ => return false,
        };
        if self.needle().is_empty() && menu_tree::toggle_at(&mut self.tree, &path) {
            self.apply_filter();
            self.selected = self.items.iter().position(|r| {
                matches!(&r.entry, FlatEntry::Group { path: p, .. } if *p == path)
            });
            self.scroll_to_selected();
            self.clamp_offset();
            self.sync_geometry(conn);
        }
        self.paint(conn);
        true
    }

    pub fn handle_click<H: DisplayBackend + ?Sized>(&mut self, conn: &H, p: Point) -> ListNav<T> {
        if self.sb_hit(p) {
            self.paint(conn);
            return ListNav::Handled;
        }
        let idx = match self.row_at(p) {
            Some(i) => i,
            None => return ListNav::Ignored,
        };
        match &self.items[idx].entry {
            FlatEntry::Separator => ListNav::Handled,
            FlatEntry::Group { .. } => {
                self.selected = Some(idx);
                self.toggle_row(conn, idx);
                ListNav::Handled
            }
            FlatEntry::Leaf(payload) => {
                let payload = payload.clone();
                self.selected = Some(idx);
                ListNav::Activate(payload)
            }
        }
    }

    pub fn handle_motion<H: DisplayBackend + ?Sized>(&mut self, conn: &H, p: Point) {
        if self.scroll_drag {
            self.drag_to(p);
            self.paint(conn);
            return;
        }
        let next = self
            .row_at(p)
            .filter(|i| self.items.get(*i).map_or(false, |r| r.selectable()));
        if next != self.selected {
            self.selected = next;
            self.paint(conn);
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

    pub fn handle_key<H: DisplayBackend + ?Sized>(&mut self, conn: &H, ks: u32, wheel: bool) -> ListNav<T> {
        if wheel {
            match ks {
                4 => {
                    self.scroll_by(-3);
                    self.paint(conn);
                    return ListNav::Handled;
                }
                5 => {
                    self.scroll_by(3);
                    self.paint(conn);
                    return ListNav::Handled;
                }
                _ => {}
            }
        }
        match ks {
            0xFF1B => ListNav::Close,
            0xFF52 | 0xFF51 => {
                self.selected = self.next_selectable(self.selected, -1);
                self.scroll_to_selected();
                self.paint(conn);
                ListNav::Handled
            }
            0xFF54 | 0xFF53 => {
                self.selected = self.next_selectable(self.selected, 1);
                self.scroll_to_selected();
                self.paint(conn);
                ListNav::Handled
            }
            0xFF50 | 0xFF57 => {
                self.selected = self.next_selectable(None, if ks == 0xFF50 { 1 } else { -1 });
                self.scroll_to_selected();
                self.paint(conn);
                ListNav::Handled
            }
            0xFF55 | 0xFF56 => {
                self.scroll_by(if ks == 0xFF55 {
                    -self.vis_rows()
                } else {
                    self.vis_rows()
                });
                self.paint(conn);
                ListNav::Handled
            }
            0xFF0D | 0xFF8D => {
                let idx = match self.selected {
                    Some(i) => i,
                    None => return ListNav::Handled,
                };
                match self.items.get(idx).map(|r| &r.entry) {
                    Some(FlatEntry::Leaf(payload)) => ListNav::Activate(payload.clone()),
                    Some(FlatEntry::Group { .. }) => {
                        self.toggle_row(conn, idx);
                        ListNav::Handled
                    }
                    _ => ListNav::Handled,
                }
            }
            _ => ListNav::Ignored,
        }
    }

    pub fn handle_key_input<H: DisplayBackend + ?Sized>(
        &mut self,
        conn: &H,
        keycode: u32,
        state: u16,
        mapping: &KeyboardMapping,
        ks: u32,
    ) -> ListNav<T> {
        match self.bar_key(keycode, state, mapping) {
            1 => {
                self.refilter(conn);
                ListNav::Handled
            }
            2 => self.handle_key(conn, 0xFF0D, false),
            3 => ListNav::Close,
            _ => self.handle_key(conn, ks, false),
        }
    }

    pub fn refilter<H: DisplayBackend + ?Sized>(&mut self, conn: &H) {
        self.apply_filter();
        self.offset = 0;
        self.selected = self.next_selectable(None, 1);
        self.sync_geometry(conn);
        self.paint(conn);
    }

    pub fn pos_set(&mut self, p: Point) {
        self.pos = p;
    }

    pub fn window_id(&self) -> u32 {
        self.window.as_ref().map(|w| w.id()).unwrap_or(0)
    }

    pub fn bar_key(&mut self, keycode: u32, state: u16, mapping: &KeyboardMapping) -> u8 {
        use antibox_ui::searchbar::SearchEvent;
        let bar = match self.bar.as_mut() {
            Some(bar) => bar,
            None => return 0,
        };
        let ev = bar.handle_key(keycode, state, mapping);
        if ev != SearchEvent::None {
            bar.repaint();
        }
        match ev {
            SearchEvent::Changed => 1,
            SearchEvent::Submitted => 2,
            SearchEvent::Cancelled => 3,
            SearchEvent::None => 0,
        }
    }

    pub fn paint<H: DisplayBackend + ?Sized>(&self, conn: &H) {
        let win = match &self.window {
            Some(win) => win,
            None => return,
        };
        crate::paintbuf::buffered(conn, win.id(), self.w.max(1), self.h.max(1), |g| {
            self.paint_to(g)
        });
        if let Some(bar) = &self.bar {
            bar.repaint();
        }
        let _ = conn.flush();
    }

    fn paint_to(&self, g: &dyn GraphicsContext) {
        let (w, h) = (self.w.max(1), self.h.max(1));
        let c = self.colours;
        let _ = g.set_font(&FontSpec::role(
            FontRole::Menu,
            antibox_ui::metrics::font_pt(),
        ));
        let (light, dark) = render::draw_menu_frame(g, w, h, c.bg);
        let top = self.list_top();
        let vis = self.vis_rows();
        for v in 0..vis {
            let idx = (self.offset + v) as usize;
            let row = match self.items.get(idx) {
                Some(r) => r,
                None => break,
            };
            let y = top + v * row_h();
            let x0 = 4 + row.depth as i16 * indent_w();
            if row.is_separator() {
                render::draw_menu_separator(
                    g,
                    4,
                    y + row_h() / 2,
                    (self.list_w() - 8).max(1) as u16,
                    light,
                    dark,
                );
                continue;
            }
            let sel = self.selected == Some(idx);
            if sel {
                render::fill_menu_selection(
                    g,
                    2,
                    y,
                    (self.list_w() - 4).max(1) as u16,
                    row_h() as u16,
                    c.sel_bg,
                );
            }
            let fg = if sel { c.sel_fg } else { c.fg };
            let _ = g.set_background(if sel { c.sel_bg } else { c.bg });
            let mut tx = x0;
            if let FlatEntry::Group { expanded, .. } = &row.entry {
                draw_expander(g, x0, y, *expanded, fg);
                tx += expander_w() + antibox_ui::metrics::gap() as i16;
            }
            if let Some(ref ico) = row.icon {
                let _ = g.draw_pixmap(tx, y + (row_h() - ico.height as i16) / 2, ico);
                tx += ico.width as i16 + antibox_ui::metrics::gap() as i16;
            }
            render::draw_text_mnemonic(
                g,
                tx,
                antibox_ui::metrics::baseline(y as i32, row_h() as i32) as i16,
                &row.title,
                fg,
            );
        }
        if self.needs_sb() {
            draw_scrollbar(g, self.sb_x(), self.sb_top(), self.h as i16, self.thumb());
        }
    }
}

fn expander_w() -> i16 {
    scaled(9) as i16
}

fn draw_expander(g: &dyn GraphicsContext, x: i16, y: i16, expanded: bool, fg: antibox_core::colour::Colour) {
    let e = expander_w();
    let by = y + (row_h() - e) / 2;
    let _ = g.set_foreground(fg);
    let _ = g.draw_rect(x, by, e as u16, e as u16);
    let cy = by + e / 2;
    let cx = x + e / 2;
    let _ = g.draw_line(x + 2, cy, x + e - 2, cy);
    if !expanded {
        let _ = g.draw_line(cx, by + 2, cx, by + e - 2);
    }
}

pub(crate) fn draw_scrollbar(
    g: &dyn GraphicsContext,
    x: i16,
    top: i16,
    bottom: i16,
    thumb: (i16, i16),
) {
    let face = antibox_ui::theme::face();
    let bwid = sb_w();
    let w = bwid as u16;

    let _ = g.set_foreground(crate::render::bevel_light(face));
    let _ = g.fill_rect(x, top + bwid, w, (bottom - top - 2 * bwid).max(0) as u16);

    let _ = g.set_foreground(face);
    let _ = g.fill_rect(x, top, w, bwid as u16);
    let _ = crate::render::draw_button_bevel(g, x, top, w, bwid as u16, face, false);
    sb_arrow(g, x, top, bwid, true);

    let _ = g.set_foreground(face);
    let _ = g.fill_rect(x, bottom - bwid, w, bwid as u16);
    let _ = crate::render::draw_button_bevel(g, x, bottom - bwid, w, bwid as u16, face, false);
    sb_arrow(g, x, bottom - bwid, bwid, false);

    let (ty, tht) = thumb;
    let _ = g.set_foreground(face);
    let _ = g.fill_rect(x, ty, w, tht as u16);
    let _ = crate::render::draw_button_bevel(g, x, ty, w, tht as u16, face, false);
}

fn sb_arrow(g: &dyn GraphicsContext, x: i16, y: i16, bwid: i16, up: bool) {
    let cx = x + bwid / 2;
    let cy = y + bwid / 2;
    let r = (bwid / 4).max(2);
    let _ = g.set_foreground(antibox_ui::theme::text());
    if up {
        let _ = g.fill_polygon(&[(cx, cy - r), (cx - r, cy + r), (cx + r, cy + r)]);
    } else {
        let _ = g.fill_polygon(&[(cx, cy + r), (cx - r, cy - r), (cx + r, cy - r)]);
    }
}

#[cfg(test)]
#[path = "listview_tests.rs"]
mod tests;
