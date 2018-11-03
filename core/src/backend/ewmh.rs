#[derive(Debug, Clone, Copy, Default)]
pub struct Strut {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Strut {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }
        let words: Vec<u32> = data
            .chunks_exact(4)
            .map(|c| (c[0] as u32) | ((c[1] as u32) << 8) | ((c[2] as u32) << 16) | ((c[3] as u32) << 24))
            .collect();
        if words.len() < 4 {
            return None;
        }
        Some(Self {
            left: words[0],
            right: words[1],
            top: words[2],
            bottom: words[3],
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StrutPartial {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
    pub left_start_y: u32,
    pub left_end_y: u32,
    pub right_start_y: u32,
    pub right_end_y: u32,
    pub top_start_x: u32,
    pub top_end_x: u32,
    pub bottom_start_x: u32,
    pub bottom_end_x: u32,
}

impl StrutPartial {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 48 {
            return None;
        }
        let words: Vec<u32> = data
            .chunks_exact(4)
            .map(|c| (c[0] as u32) | ((c[1] as u32) << 8) | ((c[2] as u32) << 16) | ((c[3] as u32) << 24))
            .collect();
        if words.len() < 12 {
            return None;
        }
        Some(Self {
            left: words[0],
            right: words[1],
            top: words[2],
            bottom: words[3],
            left_start_y: words[4],
            left_end_y: words[5],
            right_start_y: words[6],
            right_end_y: words[7],
            top_start_x: words[8],
            top_end_x: words[9],
            bottom_start_x: words[10],
            bottom_end_x: words[11],
        })
    }
}

#[cfg(test)]
#[path = "ewmh_tests.rs"]
mod tests;
