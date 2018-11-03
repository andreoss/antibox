use antibox_gfx::scale::scaled;

pub fn min_len() -> i32 {
    scaled(8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thumb {
    pub pos: i32,
    pub page: i32,
    pub first: i32,
    pub total: i32,
}

impl Thumb {
    pub fn new(pos: i32, page: i32, first: i32, total: i32) -> Self {
        let page = page.max(1);
        let total = total.max(page);
        let first = first.max(0).min(total - page);
        Self {
            pos,
            page,
            first,
            total,
        }
    }

    pub fn max_first(&self) -> i32 {
        (self.total - self.page).max(0)
    }

    pub fn len(&self, track: i32) -> i32 {
        if track <= 0 {
            return 0;
        }
        let total = self.total.max(self.page).max(1);
        let l = (track as i64 * self.page.max(1) as i64 / total as i64) as i32;
        l.max(min_len().min(track)).min(track)
    }

    pub fn start(&self, track: i32) -> i32 {
        let span = (track - self.len(track)).max(0);
        let m = self.max_first();
        if m == 0 {
            return self.pos;
        }
        self.pos + (span as i64 * self.first.max(0).min(m) as i64 / m as i64) as i32
    }

    pub fn rect(&self, track: i32) -> (i32, i32) {
        (self.start(track), self.len(track))
    }

    pub fn contains(&self, track: i32, p: i32) -> bool {
        let (s, l) = self.rect(track);
        p >= s && p < s + l
    }

    pub fn first_at(&self, track: i32, start: i32) -> i32 {
        let span = (track - self.len(track)).max(0);
        let m = self.max_first();
        if span == 0 || m == 0 {
            return 0;
        }
        ((start - self.pos).max(0).min(span) as i64 * m as i64 / span as i64) as i32
    }

    pub fn grab(&self, track: i32, p: i32) -> i32 {
        let (s, l) = self.rect(track);
        if p >= s && p < s + l {
            p - s
        } else {
            l / 2
        }
    }

    pub fn press(&self, track: i32, p: i32) -> (i32, i32) {
        let (s, l) = self.rect(track);
        if p >= s && p < s + l {
            (p - s, self.first)
        } else {
            let g = l / 2;
            (g, self.first_at(track, p - g))
        }
    }

    pub fn drag(&self, track: i32, p: i32, grab: i32) -> i32 {
        self.first_at(track, p - grab)
    }
}

#[cfg(test)]
#[path = "thumb_tests.rs"]
mod tests;
