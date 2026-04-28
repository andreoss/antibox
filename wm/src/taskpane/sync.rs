use super::data::{TaskButton, TaskSync};
use super::TaskPane;

impl TaskPane {
    pub(super) fn button_bg(face: antibox_core::colour::Colour, active: bool) -> u32 {
        if active && !antibox_ui::theme::pressed_dither(true) {
            antibox_ui::theme::pressed_face(face)
        } else {
            face
        }
    }

    pub fn add_button(&mut self, id: u32, label: &str) {
        let n = self.buttons.len() as i16;
        self.buttons.push(TaskButton {
            label: label.to_string(),
            window_id: id,
            members: vec![id],
            active: false,
            minimized: false,
            urgent: false,
            rect: (n * 120 + 2, 2, 116, 24),
            progress: None,
            icon: None,
            icon_bg: u32::MAX,
        });
    }

    pub fn remove_button(&mut self, id: u32) {
        self.buttons.retain(|b| b.window_id != id);
    }

    pub fn set_vertical(&mut self, vertical: bool) {
        self.vertical = vertical;
    }

    pub(crate) fn min_task_w() -> i32 {
        antibox_ui::metrics::text_w(6) + antibox_ui::metrics::pad() * 2
    }

    pub(super) fn max_button_w() -> i32 {
        antibox_ui::metrics::text_w(antibox_ui::theme::taskbar_item_chars() as usize)
            + antibox_ui::metrics::pad() * 2
            + antibox_ui::metrics::gap() * 2
    }

    pub(crate) fn spread_widths(avail: i32, n: i32, gap: i32, min_w: i32, max_w: i32) -> Vec<i32> {
        let per = (avail - gap * (n - 1)) / n.max(1);
        if per >= max_w {
            return vec![max_w; n as usize];
        }
        if per < min_w {
            return vec![(max_w + 1) / 2; n as usize];
        }
        let mut out = vec![per; n as usize];
        let extra = avail - gap * (n - 1) - per * n;
        for w in out.iter_mut().take(extra.max(0) as usize) {
            *w += 1;
        }
        out
    }

    fn overflow_window(n: usize, capacity: usize, active: usize) -> (usize, usize, usize, usize) {
        if n <= capacity {
            return (0, n, 0, 0);
        }
        let cap = capacity.max(1);
        if cap == 1 {
            return (0, 0, 0, n);
        }
        let win = cap - 2;
        let mut lo = if win == 0 { 0 } else { active.saturating_sub(win / 2) };
        if lo + win > n {
            lo = n - win;
        }
        let mut hi = lo + win;
        let mut before = lo;
        let mut after = n - hi;
        if before == 0 && after > 0 {
            hi = (cap - 1).min(n);
            after = n - hi;
        } else if after == 0 && before > 0 {
            lo = n - (cap - 1).min(n);
            before = lo;
        }
        (lo, hi, before, after)
    }

    pub fn layout_buttons(&mut self, pane_w: u16, pane_h: u16) {
        let n = self.buttons.len();
        self.overflow = None;
        self.overflow_ids.clear();
        self.overflow_before = None;
        self.overflow_before_ids.clear();
        self.scroll = 0;
        self.visible_count = n;
        if n == 0 {
            return;
        }
        let vin = antibox_ui::metrics::button_inset() as i16;
        let gap = antibox_ui::metrics::item_gap();
        let active = self.buttons.iter().position(|b| b.active).unwrap_or(0);

        if self.vertical {
            let cell = antibox_ui::metrics::panel_height() as i16;
            let width = pane_w.saturating_sub(vin as u16 * 2).max(1);
            let step = cell + gap as i16;
            let avail = (pane_h as i16 - vin * 2).max(1);
            let capacity = ((i32::from(avail) + gap) / i32::from(step).max(1)).max(1) as usize;
            let (lo, hi, before, after) = Self::overflow_window(n, capacity, active);
            let cellrect =
                |slot: i16| -> (i16, i16, u16, u16) { (vin, vin + slot * step, width, cell.max(1) as u16) };
            for btn in &mut self.buttons {
                btn.rect = (0, 0, 0, cell.max(1) as u16);
            }
            let mut slot = 0i16;
            if before > 0 {
                self.overflow_before = Some(cellrect(slot));
                self.overflow_before_ids = self.buttons[..lo].iter().map(|b| b.window_id).collect();
                slot += 1;
            }
            for i in lo..hi {
                self.buttons[i].rect = cellrect(slot);
                slot += 1;
            }
            if after > 0 {
                self.overflow = Some(cellrect(slot));
                self.overflow_ids = self.buttons[hi..].iter().map(|b| b.window_id).collect();
            }
            self.visible_count = hi - lo;
            return;
        }

        let btn_h = (pane_h.saturating_sub(vin as u16 * 2)).max(1);
        let min_w = Self::min_task_w().max(antibox_ui::metrics::panel_height());
        let max_w = match antibox_ui::theme::taskbar_item_pct() {
            0 => Self::max_button_w(),
            pct => (pane_w as i32 * pct as i32 / 100).max(min_w),
        };
        let rows = usize::from(crate::layout_preferences::taskbar_rows()).max(1);
        let fill = crate::layout_preferences::taskbar_fill();

        let per_row_cap = (((pane_w as i32 + gap) / (min_w + gap)).max(1)) as usize;
        let capacity = per_row_cap.saturating_mul(rows).max(1);
        let (lo, hi, before, after) = Self::overflow_window(n, capacity, active);
        let used = usize::from(before > 0) + (hi - lo) + usize::from(after > 0);

        let per_row = used.div_ceil(rows).max(1);
        let row_h = ((i32::from(btn_h) - gap * (rows as i32 - 1)) / rows as i32).max(1) as u16;
        let ni = per_row as i32;
        let widths = if fill {
            Self::spread_widths(pane_w as i32, ni, gap, 1, i32::from(pane_w))
        } else {
            Self::spread_widths(pane_w as i32, ni, gap, min_w, max_w)
        };
        let span: i32 = widths.iter().sum::<i32>() + gap * (ni - 1);
        let leftover = (pane_w as i32 - span).max(0);
        let start = if fill {
            0
        } else {
            match crate::layout_preferences::taskbar_align() {
                antibox_ui::widget::LabelAlign::Center => leftover / 2,
                antibox_ui::widget::LabelAlign::Right => leftover,
                antibox_ui::widget::LabelAlign::Left => 0,
            }
        };
        let cell_rect = |slot: usize| -> (i16, i16, u16, u16) {
            let (col, row) = (slot % per_row, slot / per_row);
            let x = start as i16 + widths[..col].iter().map(|w| *w + gap).sum::<i32>() as i16;
            let y = vin + row as i16 * (row_h as i16 + gap as i16);
            (x, y, widths[col].max(1) as u16, row_h)
        };
        for btn in &mut self.buttons {
            btn.rect = (0, 0, 0, row_h);
        }
        let mut slot = 0usize;
        if before > 0 {
            self.overflow_before = Some(cell_rect(slot));
            self.overflow_before_ids = self.buttons[..lo].iter().map(|b| b.window_id).collect();
            slot += 1;
        }
        for i in lo..hi {
            self.buttons[i].rect = cell_rect(slot);
            slot += 1;
        }
        if after > 0 {
            self.overflow = Some(cell_rect(slot));
            self.overflow_ids = self.buttons[hi..].iter().map(|b| b.window_id).collect();
        }
        self.visible_count = hi - lo;
    }

    pub fn set_active(&mut self, id: u32) {
        for btn in &mut self.buttons {
            btn.active = btn.window_id == id;
        }
    }

    pub fn sync_from_frames(
        &mut self,
        frames: &crate::frame_store::FrameStore,
        focused: Option<crate::id::ClientId>,
        active_ws: u32,
        xid_index: &crate::id::XidIndex,
    ) -> TaskSync {
        use std::collections::{HashMap, HashSet};
        let on_bar = |fw: &crate::frame::FrameWindow| {
            let ws = fw.workspace();
            !fw.state().skip_taskbar && (ws == active_ws || ws == !0)
        };
        self.focused_id = focused.map_or(0, |f| xid_index.xid_of(f));
        let new_focused = self.focused_id;

        let mut group_of: HashMap<u32, Vec<u32>> = HashMap::new();
        if crate::layout_preferences::taskbar_grouping() {
            let mut by_class: HashMap<String, Vec<u32>> = HashMap::new();
            for (id, fw) in frames.iter().filter(|(_, fw)| on_bar(fw)) {
                let key = fw.client().class_instance().unwrap_or("").to_string();
                by_class.entry(key).or_default().push(xid_index.xid_of(*id));
            }
            for (_, mut members) in by_class {
                members.sort_unstable();
                group_of.insert(members[0], members);
            }
        } else {
            for (id, _) in frames.iter().filter(|(_, fw)| on_bar(fw)) {
                let xid = xid_index.xid_of(*id);
                group_of.insert(xid, vec![xid]);
            }
        }

        let new_ids: HashSet<u32> = group_of.keys().copied().collect();
        let old_ids: HashSet<u32> = self.buttons.iter().map(|b| b.window_id).collect();
        let mut membership_changed = false;
        for id in old_ids.difference(&new_ids) {
            self.remove_button(*id);
            membership_changed = true;
        }
        let existing: HashSet<u32> = self.buttons.iter().map(|b| b.window_id).collect();
        let mut to_add: Vec<u32> = new_ids.difference(&existing).copied().collect();
        to_add.sort_unstable();
        for rep in to_add {
            self.add_button(rep, "");
            membership_changed = true;
        }

        let mut title_changed = false;
        let mut active_changed = false;
        for btn in &mut self.buttons {
            let Some(members) = group_of.get(&btn.window_id) else { continue };
            if btn.members != *members {
                btn.members = members.clone();
                membership_changed = true;
            }
            let shown = members
                .iter()
                .copied()
                .find(|&m| m == new_focused)
                .unwrap_or(btn.window_id);
            if let Some(fw) = xid_index
                .client_id_for(shown)
                .and_then(|cid| frames.get(&cid))
            {
                let base = fw.client().title();
                let label = if members.len() > 1 {
                    format!("{}  ({})", base, members.len())
                } else {
                    base.to_string()
                };
                if btn.label != label {
                    btn.label = label;
                    title_changed = true;
                }
                let new_progress = fw.client().progress();
                if btn.progress != new_progress {
                    btn.progress = new_progress;
                    title_changed = true;
                }
            }
            let want_active = members.contains(&new_focused);
            if btn.active != want_active {
                btn.active = want_active;
                active_changed = true;
            }
            let want_urgent = !want_active
                && members
                    .iter()
                    .filter_map(|m| xid_index.client_id_for(*m).and_then(|cid| frames.get(&cid)))
                    .any(|fw| fw.state().urgent);
            if btn.urgent != want_urgent {
                btn.urgent = want_urgent;
                active_changed = true;
            }
            let mut any_member = false;
            let mut all_minimized = true;
            for fw in members
                .iter()
                .filter_map(|m| xid_index.client_id_for(*m).and_then(|cid| frames.get(&cid)))
            {
                any_member = true;
                if !fw.state().minimized {
                    all_minimized = false;
                }
            }
            let want_minimized = any_member && all_minimized;
            if btn.minimized != want_minimized {
                btn.minimized = want_minimized;
                active_changed = true;
            }
            let want_bg = Self::button_bg(self.colours.task_bar_colour, btn.active);
            let btn_h = self.pane_h as i32 - 2 * antibox_ui::metrics::button_inset();
            let px = (btn_h - 2 * antibox_ui::metrics::gap())
                .min(antibox_ui::metrics::icon())
                .max(8) as u16;
            let want_new = match btn.icon {
                Some(ref ic) => ic.width != px || btn.icon_bg != want_bg,
                None => true,
            };
            if want_new {
                if let Some(fw) = xid_index
                    .client_id_for(shown)
                    .and_then(|cid| frames.get(&cid))
                {
                    btn.icon = Some(crate::icon_render::resolve_client_icon(
                        fw.client().icons(),
                        px,
                        want_bg,
                    ));
                    btn.icon_bg = want_bg;
                    title_changed = true;
                }
            }
        }

        if membership_changed {
            TaskSync::Layout
        } else if active_changed || title_changed {
            TaskSync::Content
        } else {
            TaskSync::Unchanged
        }
    }

    pub(super) fn cycle_target(&self, btn: &TaskButton) -> u32 {
        if btn.members.len() <= 1 {
            return btn.window_id;
        }
        if let Some(pos) = btn.members.iter().position(|&m| m == self.focused_id) {
            return btn.members[(pos + 1) % btn.members.len()];
        }
        btn.members.first().copied().unwrap_or(btn.window_id)
    }

    pub fn find_by_pos(&self, x: i32, y: i32) -> Option<u32> {
        let i = self.button_at(x, y)?;
        Some(self.cycle_target(&self.buttons[i]))
    }

    pub fn button_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.vertical && crate::layout_preferences::taskbar_rows() <= 1 {
            return self.button_index_at(x);
        }
        self.buttons.iter().position(|b| {
            let (bx, by, bw, bh) = b.rect;
            x >= i32::from(bx)
                && x < i32::from(bx) + i32::from(bw)
                && y >= i32::from(by)
                && y < i32::from(by) + i32::from(bh)
        })
    }

    pub fn button_index_at(&self, along: i32) -> Option<usize> {
        self.buttons.iter().position(|b| {
            let (bx, by, bw, bh) = b.rect;
            let (start, span) = if self.vertical { (by, bh) } else { (bx, bw) };
            along >= start as i32 && along < (start + span as i16) as i32
        })
    }

    pub const fn along(&self, x: i32, y: i32) -> i32 {
        if self.vertical {
            y
        } else {
            x
        }
    }

    pub fn overflow_at(&self, x: i32, y: i32) -> Option<Vec<u32>> {
        let inside = |r: (i16, i16, u16, u16)| {
            let (ox, oy, ow, oh) = r;
            x >= i32::from(ox)
                && x < i32::from(ox) + i32::from(ow)
                && y >= i32::from(oy)
                && y < i32::from(oy) + i32::from(oh)
        };
        if let Some(r) = self.overflow_before {
            if inside(r) && !self.overflow_before_ids.is_empty() {
                return Some(self.overflow_before_ids.clone());
            }
        }
        if let Some(r) = self.overflow {
            if inside(r) && !self.overflow_ids.is_empty() {
                return Some(self.overflow_ids.clone());
            }
        }
        None
    }

    pub fn members_at(&self, x: i32, y: i32) -> Option<Vec<u32>> {
        self.button_at(x, y).map(|i| self.buttons[i].members.clone())
    }
}
