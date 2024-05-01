use antibox_core::backend::*;
use antibox_core::point::Point;

pub enum DockButtonResult {
    None,
    GrabPointer,
    CloseWindow(u32),
    RotateForward,
    RotateBackward,
    ShowMenu,
}

const DOCK_SIZE: i32 = 64;
const DOCK_PAD: i32 = 2;

struct DockEntry {
    window: u32,
}

pub struct DockManager {
    apps: Vec<DockEntry>,
    right_side: bool,
    drag_idx: Option<usize>,
    drag_start: Option<Point>,
    drag_origin: Option<(i32, i32)>,
    collapsed: bool,
    expand_guard: bool,
}

pub fn is_dockapp_class(class_instance: &str) -> bool {
    let (instance, class) = match class_instance.find('.') {
        Some(i) => (&class_instance[..i], &class_instance[i + 1..]),
        None => (class_instance, ""),
    };
    class.eq_ignore_ascii_case("dockapp") || instance.to_ascii_lowercase().ends_with("dock")
}

impl DockManager {
    pub fn compute_drop_target(
        &self,
        root_x: i16,
        root_y: i16,
        sw: i32,
        sh: i32,
    ) -> Option<usize> {
        let n = self.apps.len() as i32;
        if n == 0 {
            return None;
        }
        let (cols, rows, total_w, total_h) = self.layout_dims();
        let start_x = if self.right_side {
            sw - total_w - DOCK_PAD
        } else {
            DOCK_PAD
        };
        let start_y = (sh - total_h) / 2;
        let rel_x = root_x as i32 - start_x;
        let rel_y = root_y as i32 - start_y;
        if rel_x >= 0 && rel_y >= 0 && cols > 0 && rows > 0 {
            let target_col = rel_x / (DOCK_SIZE + DOCK_PAD);
            let target_row = rel_y / (DOCK_SIZE + DOCK_PAD);
            let target = target_col * rows + target_row;
            if target >= 0 && target < n {
                return Some(target as usize);
            }
        }
        None
    }
}

impl Default for DockManager {
    fn default() -> Self {
        Self {
            apps: Vec::new(),
            right_side: true,
            drag_idx: None,
            drag_start: None,
            drag_origin: None,
            collapsed: false,
            expand_guard: false,
        }
    }
}

impl DockManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dock(&mut self, client_window: u32) -> bool {
        if self.apps.iter().any(|a| a.window == client_window) {
            return false;
        }
        self.apps.push(DockEntry {
            window: client_window,
        });
        true
    }

    pub fn undock(&mut self, client_window: u32) -> bool {
        let before = self.apps.len();
        self.apps.retain(|a| a.window != client_window);
        if self.apps.is_empty() {
            self.collapsed = false;
            self.expand_guard = false;
        }
        self.apps.len() != before
    }

    pub fn layout_dims(&self) -> (i32, i32, i32, i32) {
        let n = self.apps.len() as i32;
        if n == 0 {
            return (0, 0, 0, 0);
        }
        let cols = 1;
        let rows = n;
        let total_w = cols * (DOCK_SIZE + DOCK_PAD) - DOCK_PAD;
        let total_h = rows * (DOCK_SIZE + DOCK_PAD) - DOCK_PAD;
        (cols, rows, total_w, total_h)
    }

    pub fn grid_pos<H: DisplayBackend + ?Sized>(&self, backend: &H, i: usize) -> (i32, i32) {
        let sw = backend.screen_width() as i32;
        let sh = backend.screen_height() as i32;
        if self.collapsed {
            return self.collapsed_pos(sw);
        }
        let (cols, rows, total_w, total_h) = self.layout_dims();
        if cols == 0 || i >= self.apps.len() {
            return (0, 0);
        }
        let start_x = if self.right_side {
            sw - total_w - DOCK_PAD
        } else {
            DOCK_PAD
        };
        let start_y = (sh - total_h) / 2;
        let col = cols - 1 - (i as i32 / rows);
        let row = i as i32 % rows;
        let x = start_x + col * (DOCK_SIZE + DOCK_PAD);
        let y = start_y + row * (DOCK_SIZE + DOCK_PAD);
        (x, y)
    }

    pub fn adapt_with<H: DisplayBackend + ?Sized>(&self, backend: &H) {
        for i in 0..self.apps.len() {
            let (x, y) = self.grid_pos(backend, i);
            let _ = backend.configure_window(self.apps[i].window, &[x as u32, y as u32]);
        }
        let _ = backend.flush();
    }

    pub fn is_dock_app(&self, window: u32) -> bool {
        self.apps.iter().any(|a| a.window == window)
    }

    pub const fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn collapse(&mut self) -> bool {
        if self.collapsed || self.apps.is_empty() {
            return false;
        }
        self.collapsed = true;
        self.expand_guard = false;
        true
    }

    pub fn expand(&mut self) -> bool {
        if !self.collapsed {
            return false;
        }
        self.collapsed = false;
        self.expand_guard = true;
        true
    }

    pub fn pointer_entered(&mut self) {
        self.expand_guard = false;
    }

    pub const fn collapsed_pos(&self, sw: i32) -> (i32, i32) {
        let x = if self.right_side {
            sw - DOCK_SIZE - DOCK_PAD
        } else {
            DOCK_PAD
        };
        (x, DOCK_PAD)
    }

    fn expanded_bounds(&self, sw: i32, sh: i32) -> (i32, i32, i32, i32) {
        let (_, _, total_w, total_h) = self.layout_dims();
        let start_x = if self.right_side {
            sw - total_w - DOCK_PAD
        } else {
            DOCK_PAD
        };
        let start_y = (sh - total_h) / 2;
        (start_x, start_y, total_w, total_h)
    }

    const fn tile_contains(&self, x: i32, y: i32, sw: i32) -> bool {
        let (cx, cy) = self.collapsed_pos(sw);
        let m = DOCK_PAD;
        x >= cx - m && x < cx + DOCK_SIZE + m && y >= cy - m && y < cy + DOCK_SIZE + m
    }

    pub fn hover_zone(&self, x: i32, y: i32, sw: i32, sh: i32) -> bool {
        if self.apps.is_empty() {
            return false;
        }
        if self.collapsed {
            return self.tile_contains(x, y, sw);
        }
        let (bx, by, bw, bh) = self.expanded_bounds(sw, sh);
        let m = DOCK_PAD;
        if x >= bx - m && x < bx + bw + m && y >= by - m && y < by + bh + m {
            return true;
        }
        self.expand_guard && self.tile_contains(x, y, sw)
    }

    pub fn should_collapse(&self, x: i32, y: i32, sw: i32, sh: i32) -> bool {
        !self.collapsed && !self.apps.is_empty() && !self.hover_zone(x, y, sw, sh)
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.apps.iter().map(|e| e.window)
    }

    pub fn try_dock(&mut self, class_instance: Option<&str>) -> bool {
        let Some(ci) = class_instance else { return false };
        is_dockapp_class(ci)
    }

    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.apps.len() || to >= self.apps.len() || from == to {
            return;
        }
        let app = self.apps.remove(from);
        self.apps.insert(to, app);
    }

    pub fn rotate(&mut self, forward: bool) {
        if self.apps.len() < 2 {
            return;
        }
        if forward {
            let app = self.apps.remove(0);
            self.apps.push(app);
        } else if let Some(app) = self.apps.pop() {
            self.apps.insert(0, app);
        }
    }

    pub const fn is_dragging(&self) -> bool {
        self.drag_idx.is_some()
    }

    pub fn handle_button(
        &mut self,
        window: u32,
        button: u8,
        state: u16,
        point: Point,
    ) -> DockButtonResult {
        let Some(idx) = self.apps.iter().position(|a| a.window == window) else {
            return DockButtonResult::None;
        };
        let ctrl = state & 0x4 != 0;

        match (button, ctrl) {
            (1, true) => {
                self.drag_idx = Some(idx);
                self.drag_start = Some(point);
                self.drag_origin = None;
                DockButtonResult::GrabPointer
            }
            (2, _) => {
                self.undock(window);
                DockButtonResult::CloseWindow(window)
            }
            (3, _) => DockButtonResult::ShowMenu,
            (4, _) => {
                self.rotate(true);
                DockButtonResult::RotateForward
            }
            (5, _) => {
                self.rotate(false);
                DockButtonResult::RotateBackward
            }
            _ => DockButtonResult::None,
        }
    }

    pub fn handle_motion(&mut self, point: Point) -> Option<(u32, i32, i32)> {
        let idx = self.drag_idx?;
        if idx >= self.apps.len() {
            return None;
        }
        let app = self.apps[idx].window;
        let (orig_x, orig_y) = self.drag_origin?;
        let start = self.drag_start?;
        let dx = point.x - start.x;
        let dy = point.y - start.y;
        Some((app, orig_x + dx, orig_y + dy))
    }

    pub fn set_drag_origin(&mut self, x: i32, y: i32) {
        self.drag_origin = Some((x, y));
    }

    pub fn dragged_grid_pos<H: DisplayBackend + ?Sized>(&self, backend: &H) -> (i32, i32) {
        if let Some(idx) = self.drag_idx {
            self.grid_pos(backend, idx)
        } else {
            (0, 0)
        }
    }

    pub fn handle_release(&mut self, drop_target: Option<usize>) -> bool {
        let Some(idx) = self.drag_idx.take() else { return false };
        self.drag_start = None;
        self.drag_origin = None;

        if idx >= self.apps.len() {
            return false;
        }

        if let Some(to) = drop_target {
            self.reorder(idx, to);
        }
        true
    }
}

#[cfg(test)]
#[path = "dock_tests.rs"]
mod tests;
