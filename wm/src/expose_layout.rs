#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Cell {
    pub const fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub const fn centre(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

fn pack_slots(sizes: &[(i32, i32)], gap: i32, max_h: i32) -> Vec<Vec<usize>> {
    let mut slots: Vec<Vec<usize>> = Vec::new();
    for (i, (_, h)) in sizes.iter().enumerate() {
        let mut placed = false;
        for slot in &mut slots {
            let used: i32 = slot.iter().map(|&s| sizes[s].1 + gap).sum::<i32>() - gap;
            if used + gap + h < max_h {
                slot.push(i);
                placed = true;
                break;
            }
        }
        if !placed {
            slots.push(vec![i]);
        }
    }
    slots
}

pub fn arrange(sizes: &[(i32, i32)], gap: i32) -> (Vec<Cell>, i32, i32) {
    if sizes.is_empty() {
        return (Vec::new(), 0, 0);
    }
    let sum_w: i32 = sizes.iter().map(|(w, _)| *w).sum();
    let max_h = sizes.iter().map(|(_, h)| *h).max().unwrap_or(1);
    let mut cells: Vec<Cell> = sizes
        .iter()
        .map(|&(w, h)| Cell { x: 0, y: 0, w, h })
        .collect();

    let slots = pack_slots(sizes, gap, max_h);
    let max_row_w = ((sum_w as f64 * max_h as f64).sqrt()) as i32;
    let mut rows: Vec<Vec<usize>> = vec![Vec::new()];
    let (mut total_w, mut total_h) = (0, 0);
    let (mut row_y, mut x, mut row_h) = (0, 0, 0);
    for slot in &slots {
        let slot_max_w = slot.iter().map(|&i| sizes[i].0).max().unwrap_or(0);
        let mut y = row_y;
        for &i in slot {
            cells[i].x = x + (slot_max_w - sizes[i].0) / 2;
            cells[i].y = y;
            y += sizes[i].1 + gap;
            if let Some(row) = rows.last_mut() {
                row.push(i);
            }
        }
        row_h = row_h.max(y - row_y);
        total_h = total_h.max(y);
        x += slot_max_w + gap;
        total_w = total_w.max(x);
        if x > max_row_w {
            x = 0;
            row_y += row_h;
            row_h = 0;
            rows.push(Vec::new());
        }
    }
    total_w -= gap;
    total_h -= gap;

    for row in &rows {
        let row_w = row
            .iter()
            .map(|&i| cells[i].x + cells[i].w)
            .max()
            .unwrap_or(0);
        let off = (total_w - row_w) / 2;
        for &i in row {
            cells[i].x += off;
        }
    }
    (cells, total_w.max(1), total_h.max(1))
}

pub fn scale_to(
    cells: &mut [Cell],
    total: (i32, i32),
    screen: (i32, i32),
    margin: i32,
    allow_upscale: bool,
) -> f64 {
    let avail_w = (screen.0 - margin * 2).max(1);
    let avail_h = (screen.1 - margin * 2).max(1);
    let fit = (avail_w as f64 / total.0 as f64).min(avail_h as f64 / total.1 as f64);
    let ceiling = if allow_upscale { f64::MAX } else { 1.0 };
    let scale = fit.clamp(0.02, ceiling);
    let scaled_w = (total.0 as f64 * scale) as i32;
    let scaled_h = (total.1 as f64 * scale) as i32;
    let off_x = (screen.0 - scaled_w) / 2;
    let off_y = (screen.1 - scaled_h) / 2;
    for c in cells.iter_mut() {
        c.x = off_x + (c.x as f64 * scale) as i32;
        c.y = off_y + (c.y as f64 * scale) as i32;
        c.w = ((c.w as f64 * scale) as i32).max(1);
        c.h = ((c.h as f64 * scale) as i32).max(1);
    }
    scale
}

pub fn layout(
    sizes: &[(i32, i32)],
    screen: (i32, i32),
    gap: i32,
    margin: i32,
    allow_upscale: bool,
) -> Vec<Cell> {
    let (mut cells, tw, th) = arrange(sizes, gap);
    scale_to(&mut cells, (tw, th), screen, margin, allow_upscale);
    cells
}

pub fn nearest(cells: &[Cell], from: usize, dx: i32, dy: i32) -> usize {
    let Some(origin) = cells.get(from) else { return from };
    let (ox, oy) = origin.centre();
    let mut best = from;
    let mut best_score = i64::MAX;
    for (i, c) in cells.iter().enumerate() {
        if i == from {
            continue;
        }
        let (cx, cy) = c.centre();
        let (ax, ay) = ((cx - ox) as i64, (cy - oy) as i64);
        let along = ax * dx as i64 + ay * dy as i64;
        if along <= 0 {
            continue;
        }
        let across = (ax * dy as i64 - ay * dx as i64).abs();
        let score = along + across * 3;
        if score < best_score {
            best_score = score;
            best = i;
        }
    }
    best
}

pub fn cell_at(cells: &[Cell], x: i32, y: i32) -> Option<usize> {
    cells.iter().position(|c| c.contains(x, y))
}

#[cfg(test)]
#[path = "expose_layout_tests.rs"]
mod tests;
