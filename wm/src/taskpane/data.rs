#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskSync {
    Unchanged,
    Content,
    Layout,
}

pub(crate) struct TaskButton {
    pub label: String,
    pub window_id: u32,
    pub members: Vec<u32>,
    pub active: bool,
    pub minimized: bool,
    pub urgent: bool,
    pub rect: (i16, i16, u16, u16),
    pub progress: Option<u8>,
    pub icon: Option<antibox_core::backend::PixmapData>,
    pub icon_bg: u32,
}

pub(super) struct Drag {
    pub index: usize,
    pub start_x: i32,
    pub grab_dx: i32,
    pub cur_x: i32,
    pub bw: u16,
    pub window_id: u32,
    pub moved: bool,
}
