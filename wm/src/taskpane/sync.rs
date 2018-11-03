use super::data::{TaskButton, TaskSync};
use super::TaskPane;
use crate::compat::ClampExt;

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
        });
    }

    pub fn remove_button(&mut self, id: u32) {
        self.buttons.retain(|b| b.window_id != id);
    }

    pub(super) fn max_button_w() -> i32 {
        antibox_ui::metrics::text_w(antibox_ui::theme::taskbar_item_chars() as usize)
            + antibox_ui::metrics::pad() * 2
            + antibox_ui::metrics::gap() * 2
    }

    pub fn layout_buttons(&mut self, pane_w: u16, pane_h: u16) {
        let n = self.buttons.len();
        if n == 0 {
            return;
        }
        let vin = antibox_ui::metrics::button_inset() as i16;
        let btn_h = (pane_h.saturating_sub(vin as u16 * 2)).max(1);
        let gap = antibox_ui::metrics::item_gap();
        let min_w = antibox_ui::metrics::panel_height();
        let max_w = match antibox_ui::theme::taskbar_item_pct() {
            0 => Self::max_button_w(),
            pct => (pane_w as i32 * pct as i32 / 100).max(min_w),
        };
        let ni = n as i32;
        let fill = crate::layout_preferences::taskbar_fill();
        let btn_w = if fill {
            ((pane_w as i32 - gap * (ni - 1)) / ni).max(min_w)
        } else {
            ((pane_w as i32 - gap.max(0) * (ni - 1)) / ni).clamped(min_w, max_w)
        };
        let span = btn_w * ni + gap * (ni - 1);
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
        let btn_w = btn_w as u16;
        let mut x = start as i16;
        for btn in &mut self.buttons {
            btn.rect = (x, vin, btn_w, btn_h);
            x += btn_w as i16 + gap as i16;
        }
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
            for (_, mut members) in by_class.into_iter() {
                members.sort_unstable();
                group_of.insert(members[0], members);
            }
        } else {
            for (id, _) in frames.iter().filter(|(_, fw)| on_bar(fw)) {
                let xid = xid_index.xid_of(*id);
                group_of.insert(xid, vec![xid]);
            }
        }

        let new_ids: HashSet<u32> = group_of.keys().cloned().collect();
        let old_ids: HashSet<u32> = self.buttons.iter().map(|b| b.window_id).collect();
        let mut membership_changed = false;
        for id in old_ids.difference(&new_ids) {
            self.remove_button(*id);
            membership_changed = true;
        }
        let existing: HashSet<u32> = self.buttons.iter().map(|b| b.window_id).collect();
        let mut to_add: Vec<u32> = new_ids.difference(&existing).cloned().collect();
        to_add.sort_unstable();
        for rep in to_add {
            self.add_button(rep, "");
            membership_changed = true;
        }

        let mut title_changed = false;
        let mut active_changed = false;
        for btn in &mut self.buttons {
            let members = match group_of.get(&btn.window_id) {
                Some(members) => members,
                None => continue,
            };
            if btn.members != *members {
                btn.members = members.clone();
                membership_changed = true;
            }
            let shown = members
                .iter().cloned()
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
        btn.members.first().cloned().unwrap_or(btn.window_id)
    }

    pub fn find_by_pos(&self, x: i32) -> Option<u32> {
        for btn in &self.buttons {
            let (bx, _, bw, _) = btn.rect;
            if x >= bx as i32 && x < (bx + bw as i16) as i32 {
                return Some(self.cycle_target(btn));
            }
        }
        None
    }

    pub fn button_index_at(&self, x: i32) -> Option<usize> {
        self.buttons.iter().position(|b| {
            let (bx, _, bw, _) = b.rect;
            x >= bx as i32 && x < (bx + bw as i16) as i32
        })
    }

    pub fn members_at(&self, x: i32) -> Option<Vec<u32>> {
        self.button_index_at(x)
            .map(|i| self.buttons[i].members.clone())
    }
}
