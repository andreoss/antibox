use crate::thumb::Thumb;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListCore {
    pub rows: usize,
    pub row_h: i32,
    pub view_h: i32,
    pub scroll: i32,
    pub selected: Option<usize>,
}

impl ListCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_counts(&mut self, rows: usize, row_h: i32, view_h: i32) {
        self.rows = rows;
        self.row_h = row_h.max(1);
        self.view_h = view_h.max(0);
        self.scroll = self.scroll.max(0).min(self.max_scroll());
        self.selected = match self.selected {
            Some(_) if rows == 0 => None,
            Some(s) => Some(s.min(rows - 1)),
            None => None,
        };
    }

    pub fn content_h(&self) -> i32 {
        self.rows as i32 * self.row_h.max(1)
    }

    pub fn max_scroll(&self) -> i32 {
        (self.content_h() - self.view_h).max(0)
    }

    pub fn first_visible(&self) -> usize {
        (self.scroll.max(0) / self.row_h.max(1)) as usize
    }

    pub fn ensure_visible(&mut self, row: usize) {
        if self.rows == 0 {
            return;
        }
        let row = row.min(self.rows - 1) as i32;
        let rh = self.row_h.max(1);
        let top = row * rh;
        let bottom = top + rh;
        if top < self.scroll {
            self.scroll = top;
        } else if bottom > self.scroll + self.view_h {
            self.scroll = bottom - self.view_h;
        }
        self.scroll = self.scroll.max(0).min(self.max_scroll());
    }

    fn select(&mut self, row: usize) {
        if self.rows == 0 {
            self.selected = None;
            return;
        }
        let row = row.min(self.rows - 1);
        self.selected = Some(row);
        self.ensure_visible(row);
    }

    pub fn select_next(&mut self) {
        match self.selected {
            Some(s) => self.select(s.saturating_add(1)),
            None => self.select(0),
        }
    }

    pub fn select_prev(&mut self) {
        match self.selected {
            Some(s) => self.select(s.saturating_sub(1)),
            None => self.select(0),
        }
    }

    pub fn home(&mut self) {
        self.select(0);
    }

    pub fn end(&mut self) {
        if self.rows > 0 {
            self.select(self.rows - 1);
        }
    }

    pub fn scroll_by(&mut self, px: i32) {
        self.scroll = self.scroll.saturating_add(px).max(0).min(self.max_scroll());
    }

    pub fn thumb(&self, track: i32) -> Thumb {
        Thumb::new(
            track,
            self.view_h.max(1),
            self.scroll,
            self.content_h().max(1),
        )
    }
}

#[cfg(test)]
#[path = "listcore_tests.rs"]
mod tests;
