#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskSync {
    Unchanged,
    Content,
    Layout,
}

pub(crate) struct TaskButton {
    pub(crate) label: String,
    pub(crate) window_id: u32,
    pub(crate) members: Vec<u32>,
    pub(crate) active: bool,
    pub(crate) minimized: bool,
    pub(crate) urgent: bool,
    pub(crate) rect: (i16, i16, u16, u16),
    pub(crate) progress: Option<u8>,
    pub(crate) icon: Option<antibox_core::backend::PixmapData>,
    pub(crate) icon_bg: u32,
}

pub(super) struct Drag {
    pub(super) index: usize,
    pub(super) start_x: i32,
    pub(super) grab_dx: i32,
    pub(super) cur_x: i32,
    pub(super) bw: u16,
    pub(super) window_id: u32,
    pub(super) moved: bool,
}
