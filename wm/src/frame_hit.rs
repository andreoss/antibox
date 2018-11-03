impl FrameWindow {
    pub fn hit_test_button(&self, x: i32, y: i32) -> Option<u8> {
        let p = antibox_core::point::Point::new(x, y);
        for (id, _, _, rect) in self.title_button_layout() {
            if rect.contains(p) {
                return Some(id);
            }
        }
        None
    }

    pub fn handle_button_click(&mut self, button: u8) {
        match button {
            2 => self.close(),
            4 => self.maximize(),
            5 | 0 => self.minimize(),
            1 => {
                self.state.maximized = false;
                self.state.max_vert = false;
                self.state.max_horz = false;
                self.state.minimized = false;
            }
            6 | 3 => self.shade(),
            _ => {}
        }
    }

    pub fn hit_test_edge(&self, p: antibox_core::point::Point) -> ResizeEdge {
        if self.state().shaded {
            return ResizeEdge::None;
        }
        let bw = self.effective_border();
        if bw == 0 {
            return ResizeEdge::None;
        }
        let bb = self.effective_bottom_border();
        let (w, h) = (self.frame_rect.w, self.frame_rect.h);
        let top = 0;
        let corner = corner_len().max(bw);
        let grab = corner_len();
        let title_bottom = self.title_offset() && !self.title_on_left() && !self.title_on_right();
        let lg = if self.title_on_left() {
            grab.max(bw)
        } else {
            bw
        };
        let rg = if self.title_on_right() {
            grab.max(bw)
        } else {
            bw
        };
        let bg = if title_bottom { grab.max(bb) } else { bb };
        let mut gx: i32 = if p.x <= lg && p.y >= top {
            -1
        } else {
            i32::from(p.x >= w - rg && p.y >= top)
        };
        let mut gy: i32 = if p.y >= top && p.y <= top + bw {
            -1
        } else {
            i32::from(p.y >= h - bg)
        };
        if gy != 0 && gx == 0 {
            gx = if p.x < corner {
                -1
            } else {
                i32::from(p.x >= w - corner)
            };
        }
        if gx != 0 && gy == 0 {
            gy = if p.y < top + corner {
                -1
            } else {
                i32::from(p.y >= h - corner)
            };
        }
        match (gx, gy) {
            (-1, -1) => ResizeEdge::TopLeft,
            (1, -1) => ResizeEdge::TopRight,
            (-1, 1) => ResizeEdge::BottomLeft,
            (1, 1) => ResizeEdge::BottomRight,
            (-1, 0) => ResizeEdge::Left,
            (1, 0) => ResizeEdge::Right,
            (0, -1) => ResizeEdge::Top,
            (0, 1) => ResizeEdge::Bottom,
            _ => ResizeEdge::None,
        }
    }
}
