use antibox_gfx::backend::keys::{keysym_to_char, NormalizedKey};
use antibox_gfx::keysyms::{
    KEY_BackSpace, KEY_Delete, KEY_End, KEY_Escape, KEY_Home, KEY_KP_Enter, KEY_Left, KEY_Return,
    KEY_Right, KEY_Tab,
};
use std::borrow::Cow;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditOutcome {
    Consumed,
    Changed,
    Ignored,
    Submit,
    Cancel,
    Tab,
}

#[derive(Default)]
pub struct EditCore {
    text: String,
    cursor: usize,
    sel_start: Option<usize>,
    pub secret: bool,
    pub readonly: bool,
}

impl EditCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub const fn sel_start(&self) -> Option<usize> {
        self.sel_start
    }

    pub fn set_sel_start(&mut self, s: Option<usize>) {
        self.sel_start = s.map(|i| self.prev_boundary(i));
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.len();
        self.sel_start = None;
    }

    pub fn prev_boundary(&self, mut i: usize) -> usize {
        i = i.min(self.text.len());
        while i > 0 && !self.text.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    pub fn next_boundary(&self, mut i: usize) -> usize {
        let len = self.text.len();
        i = i.min(len);
        while i < len && !self.text.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    pub fn set_cursor(&mut self, i: usize) {
        self.cursor = self.prev_boundary(i);
    }

    pub fn char_left(&self) -> usize {
        if self.cursor == 0 {
            0
        } else {
            self.prev_boundary(self.cursor - 1)
        }
    }

    pub fn char_right(&self) -> usize {
        if self.cursor >= self.text.len() {
            self.text.len()
        } else {
            self.next_boundary(self.cursor + 1)
        }
    }

    pub fn replace_range(&mut self, lo: usize, hi: usize, s: &str) {
        if self.readonly {
            return;
        }
        let lo = self.prev_boundary(lo);
        let hi = self.prev_boundary(hi).max(lo);
        self.text.replace_range(lo..hi, s);
        self.set_cursor(lo + s.len());
        self.sel_start = None;
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.readonly {
            return;
        }
        self.delete_selection();
        let pos = self.cursor;
        self.replace_range(pos, pos, ch.encode_utf8(&mut [0u8; 4]));
    }

    pub fn delete_prev(&mut self) {
        if self.readonly {
            return;
        }
        if !self.delete_selection() && self.cursor > 0 {
            self.replace_range(self.char_left(), self.cursor, "");
        }
        self.sel_start = None;
    }

    pub fn delete_next(&mut self) {
        if self.readonly {
            return;
        }
        if !self.delete_selection() && self.cursor < self.text.len() {
            self.replace_range(self.cursor, self.char_right(), "");
        }
        self.sel_start = None;
    }

    pub fn update_selection(&mut self, shift: bool) {
        if shift {
            if self.sel_start.is_none() {
                self.sel_start = Some(self.cursor);
            }
        } else {
            self.sel_start = None;
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        if self.readonly {
            return false;
        }
        let (lo, hi) = match self.sel_start {
            Some(s) if s != self.cursor => {
                if s < self.cursor {
                    (s, self.cursor)
                } else {
                    (self.cursor, s)
                }
            }
            _ => return false,
        };
        self.replace_range(lo, hi, "");
        true
    }

    pub fn delete_prev_word(&mut self) {
        let p = self.prev_word();
        if p < self.cursor {
            self.replace_range(p, self.cursor, "");
        }
    }

    pub fn prev_word(&self) -> usize {
        let mut p = self.cursor;
        if p == 0 {
            return 0;
        }
        p -= 1;
        while p > 0 && self.text.as_bytes()[p] == b' ' {
            p -= 1;
        }
        while p > 0 && self.text.as_bytes()[p] != b' ' {
            p -= 1;
        }
        if p > 0 {
            p + 1
        } else {
            0
        }
    }

    pub fn next_word(&self) -> usize {
        let mut p = self.cursor;
        let b = self.text.as_bytes();
        while p < b.len() && b[p] == b' ' {
            p += 1;
        }
        while p < b.len() && b[p] != b' ' {
            p += 1;
        }
        p
    }

    pub fn select_all(&mut self) {
        if self.text.is_empty() {
            return;
        }
        self.sel_start = Some(0);
        self.cursor = self.text.len();
    }

    pub fn copy_selection(&self) -> Option<String> {
        if self.secret {
            return None;
        }
        let (lo, hi) = match self.sel_start {
            Some(s) if s != self.cursor => {
                if s < self.cursor {
                    (s, self.cursor)
                } else {
                    (self.cursor, s)
                }
            }
            _ => return None,
        };
        Some(self.text[lo..hi].to_string())
    }

    pub fn cut_selection(&mut self) {
        if self.copy_selection().is_some() {
            self.delete_selection();
        }
    }

    pub fn paste_selection(&mut self) {}

    pub fn display_text(&self) -> Cow<'_, str> {
        if self.secret {
            Cow::Owned("*".repeat(self.text.chars().count()))
        } else {
            Cow::Borrowed(self.text.as_str())
        }
    }

    pub fn display_index(&self, byte: usize) -> usize {
        let byte = self.prev_boundary(byte);
        if self.secret {
            self.text[..byte].chars().count()
        } else {
            byte
        }
    }

    pub fn logical_index(&self, display_byte: usize) -> usize {
        if self.secret {
            self.text
                .char_indices()
                .nth(display_byte)
                .map_or(self.text.len(), |(b, _)| b)
        } else {
            self.prev_boundary(display_byte)
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn handle_key(&mut self, key: &NormalizedKey) -> EditOutcome {
        let ctrl = key.ctrl;
        let shift = key.shift;

        match key.original_keysym {
            KEY_Return | KEY_KP_Enter => return EditOutcome::Submit,
            KEY_Escape => return EditOutcome::Cancel,
            KEY_Tab => return EditOutcome::Tab,
            KEY_BackSpace => {
                if self.readonly {
                    return EditOutcome::Consumed;
                }
                if ctrl {
                    self.delete_prev_word();
                    self.sel_start = None;
                } else {
                    self.delete_prev();
                }
                return EditOutcome::Changed;
            }
            KEY_Delete => {
                if self.readonly {
                    return EditOutcome::Consumed;
                }
                self.delete_next();
                return EditOutcome::Changed;
            }
            KEY_Left => {
                self.update_selection(shift);
                if self.cursor > 0 {
                    let target = if ctrl {
                        self.prev_word()
                    } else {
                        self.char_left()
                    };
                    self.set_cursor(target);
                }
                return EditOutcome::Consumed;
            }
            KEY_Right => {
                self.update_selection(shift);
                if self.cursor < self.text.len() {
                    let target = if ctrl {
                        self.next_word()
                    } else {
                        self.char_right()
                    };
                    self.set_cursor(target);
                }
                return EditOutcome::Consumed;
            }
            KEY_Home => {
                self.update_selection(shift);
                self.set_cursor(0);
                return EditOutcome::Consumed;
            }
            KEY_End => {
                self.update_selection(shift);
                let end = self.text.len();
                self.set_cursor(end);
                return EditOutcome::Consumed;
            }
            _ => {}
        }

        if ctrl {
            let letter = keysym_to_char(key.keysym)
                .unwrap_or('\0')
                .to_ascii_lowercase();
            return match letter {
                'a' => {
                    self.select_all();
                    EditOutcome::Consumed
                }
                'c' => {
                    let _ = self.copy_selection();
                    EditOutcome::Consumed
                }
                'x' => {
                    self.cut_selection();
                    EditOutcome::Changed
                }
                'v' => {
                    self.paste_selection();
                    EditOutcome::Changed
                }
                _ => EditOutcome::Ignored,
            };
        }

        if let Some(ch) = key.ch {
            if !ch.is_control() {
                if self.readonly {
                    return EditOutcome::Consumed;
                }
                self.insert_char(ch);
                return EditOutcome::Changed;
            }
            return EditOutcome::Consumed;
        }
        EditOutcome::Ignored
    }

}

#[cfg(test)]
#[path = "editcore_tests.rs"]
mod tests;
