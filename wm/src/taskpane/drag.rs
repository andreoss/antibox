use super::TaskPane;

impl TaskPane {
    pub(super) fn drag_motion(&mut self, x: i32) {
        let (cur, start_x, grab_dx, bw) = match &self.drag {
            Some(d) => (d.index, d.start_x, d.grab_dx, d.bw),
            None => return,
        };
        if let Some(d) = self.drag.as_mut() {
            d.cur_x = x;
            if (x - start_x).abs() > antibox_ui::metrics::gap() {
                d.moved = true;
            }
        }
        let center = (x - grab_dx) + bw as i32 / 2;
        if let Some(target) = self.button_index_at(center) {
            if target != cur {
                self.buttons.swap(cur, target);
                if let Some(d) = self.drag.as_mut() {
                    d.index = target;
                    d.moved = true;
                }
                let (pw, ph) = (self.pane_w, self.pane_h);
                self.layout_buttons(pw, ph);
            }
        }
        self.repaint();
    }

    pub(super) fn float_x(&self, d: &super::data::Drag) -> i16 {
        let w = self.buttons[d.index].rect.2;
        (d.cur_x - d.grab_dx).clamp(0, (self.pane_w.saturating_sub(w)) as i32) as i16
    }
}
