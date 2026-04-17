use super::TaskPane;

impl TaskPane {
    pub(super) fn drag_motion(&mut self, x: i32, y: i32) {
        let (cur, start_x, start_y, grab_dx, grab_dy, bw, bh) = match &self.drag {
            Some(d) => (d.index, d.start_x, d.start_y, d.grab_dx, d.grab_dy, d.bw, d.bh),
            None => return,
        };
        let slack = antibox_ui::metrics::gap();
        if let Some(d) = self.drag.as_mut() {
            d.cur_x = x;
            d.cur_y = y;
            if (x - start_x).abs() > slack || (y - start_y).abs() > slack {
                d.moved = true;
            }
        }
        let centre_x = (x - grab_dx) + bw as i32 / 2;
        let centre_y = (y - grab_dy) + bh as i32 / 2;
        if let Some(target) = self.button_at(centre_x, centre_y) {
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

    pub(super) fn float_y(&self, d: &super::data::Drag) -> i16 {
        if !self.vertical {
            return self.buttons[d.index].rect.1;
        }
        let h = self.buttons[d.index].rect.3;
        (d.cur_y - d.grab_dy).clamp(0, (self.pane_h.saturating_sub(h)) as i32) as i16
    }
}
