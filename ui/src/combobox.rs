use crate::editcore::EditCore;
use crate::listcore::ListCore;

pub struct ComboBox {
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub open: bool,
    pub open_upward: bool,
    pub editable: bool,
    pub edit: EditCore,
    pub focused: bool,
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
    pub max_visible: usize,
    pub first_visible: usize,
}

impl ComboBox {
    pub fn new(items: Vec<String>) -> Self {
        let selected = if !items.is_empty() { Some(0) } else { None };
        Self {
            items,
            selected,
            open: false,
            open_upward: false,
            editable: false,
            edit: EditCore::new(),
            focused: false,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            max_visible: 8,
            first_visible: 0,
        }
    }

    pub fn editable(items: Vec<String>) -> Self {
        let mut c = Self::new(items);
        c.set_editable(true);
        c
    }

    pub fn set_editable(&mut self, on: bool) {
        self.editable = on;
        if on {
            let seed = self.selected_text().to_string();
            self.edit.set_text(&seed);
        }
    }

    pub fn set_rect(&mut self, x: i16, y: i16, w: u16, h: u16) {
        self.x = x;
        self.y = y;
        self.w = w;
        self.h = h;
    }

    pub fn selected_text(&self) -> &str {
        self.selected
            .and_then(|i| self.items.get(i))
            .map_or("", String::as_str)
    }

    pub fn dropdown_height(&self) -> u16 {
        let rows = self.items.len().min(self.max_visible).max(1) as i16;
        (rows * self.h as i16) as u16
    }

    pub fn dropdown_rect(&self) -> (i16, i16, u16, u16) {
        let dh = self.dropdown_height();
        let dy = if self.open_upward {
            self.y - dh as i16
        } else {
            self.y + self.h as i16
        };
        (self.x, dy, self.w, dh)
    }

    pub fn item_at(&self, py: i32) -> Option<usize> {
        if !self.open {
            return None;
        }
        let (_, dy, _, dh) = self.dropdown_rect();
        if py < dy as i32 || py >= (dy + dh as i16) as i32 {
            return None;
        }
        let row = ((py - dy as i32) / self.h.max(1) as i32) as usize;
        let idx = self.first_visible + row;
        if idx < self.items.len() { Some(idx) } else { None }
    }

    fn core(&self) -> ListCore {
        let mut core = ListCore::new();
        core.scroll = self.first_visible as i32;
        core.selected = self.selected;
        core.set_counts(self.items.len(), 1, self.max_visible.max(1) as i32);
        core
    }

    pub fn scroll(&mut self, delta: i32) {
        if !self.open {
            return;
        }
        let mut core = self.core();
        core.scroll_by(delta);
        self.first_visible = core.first_visible();
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let mut core = self.core();
        core.select_next();
        self.selected = core.selected;
        self.first_visible = core.first_visible();
    }

    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let mut core = self.core();
        core.select_prev();
        self.selected = core.selected;
        self.first_visible = core.first_visible();
    }

}

#[cfg(test)]
#[path = "combobox_tests.rs"]
mod tests;
