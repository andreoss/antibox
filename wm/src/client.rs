use crate::id::ClientId;
use antibox_core::backend::{
    AtomManager, BackendEvent, DisplayBackend, IconData, MwmHints, PropMode, SizeHints, Strut,
    WindowHandle, WmHints,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    Normal,
}

impl WindowType {
    pub const fn default_for_transient() -> Self {
        Self::Dialog
    }
}

pub struct ClientWindow {
    pub(crate) id: ClientId,
    pub(crate) xwindow: Box<dyn WindowHandle>,
    pub(crate) window_type: WindowType,
    pub(crate) transient_for: Option<u32>,
    pub(crate) pid: u32,
    pub(crate) is_xpra: bool,
    pub(crate) xpra_resolved: bool,
    pub(crate) title: String,
    pub(crate) class_instance: Option<String>,
    pub(crate) client_id: Option<String>,
    pub(crate) window_role: Option<String>,
    pub(crate) leader_window: u32,
    pub(crate) wm_state: Vec<u32>,
    pub(crate) protocols: Vec<u32>,
    pub(crate) wm_hints: Option<WmHints>,
    pub(crate) size_hints: Option<SizeHints>,
    pub(crate) mwm_hints: Option<MwmHints>,
    pub(crate) user_time: u32,
    pub(crate) user_time_set: bool,
    pub(crate) strut: Option<Strut>,
    pub fullscreen_monitors: Option<[u32; 4]>,
    pub(crate) f_shaped: bool,
    pub(crate) startup_id: Option<String>,
    pub(crate) csd: bool,
    pub(crate) csd_extents: [i32; 4],
    pub(crate) progress: Option<u8>,
    pub(crate) icon: Vec<IconData>,
}

impl ClientWindow {
    pub fn new(xwindow: Box<dyn WindowHandle>) -> Self {
        let _id = xwindow.id();
        Self {
            id: ClientId::allocate(),
            xwindow,
            window_type: WindowType::Normal,
            transient_for: None,
            pid: 0,
            is_xpra: false,
            xpra_resolved: false,
            title: String::new(),
            class_instance: None,
            client_id: None,
            window_role: None,
            leader_window: 0,
            wm_state: Vec::new(),
            protocols: Vec::new(),
            wm_hints: None,
            size_hints: None,
            mwm_hints: None,
            user_time: 0,
            user_time_set: false,
            strut: None,
            fullscreen_monitors: None,
            f_shaped: false,
            startup_id: None,
            csd: false,
            csd_extents: [0; 4],
            progress: None,
            icon: Vec::new(),
        }
    }

    pub fn icons(&self) -> &[IconData] {
        &self.icon
    }

    pub const fn csd_extents(&self) -> [i32; 4] {
        self.csd_extents
    }

    pub const fn progress(&self) -> Option<u8> {
        self.progress
    }

    pub const fn id(&self) -> ClientId {
        self.id
    }

    pub fn xid(&self) -> u32 {
        self.xwindow.id()
    }

    pub const fn window_type(&self) -> WindowType {
        self.window_type
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn protocols(&self) -> &[u32] {
        &self.protocols
    }

    pub const fn wm_hints(&self) -> Option<&WmHints> {
        self.wm_hints.as_ref()
    }

    pub const fn size_hints(&self) -> Option<&SizeHints> {
        self.size_hints.as_ref()
    }

    pub const fn mwm_hints(&self) -> Option<&MwmHints> {
        self.mwm_hints.as_ref()
    }

    pub const fn is_csd(&self) -> bool {
        self.csd
    }

    pub const fn user_time(&self) -> u32 {
        self.user_time
    }

    pub const fn suppresses_map_focus(&self) -> bool {
        self.user_time_set && self.user_time == 0
    }

    pub fn has_protocol(&self, atom: u32) -> bool {
        self.protocols.contains(&atom)
    }

    pub fn wm_state(&self) -> &[u32] {
        &self.wm_state
    }

    pub fn has_net_state(&self, atoms: &AtomManager, name: &str) -> bool {
        atoms.get(name).is_some_and(|a| self.wm_state.contains(&a))
    }

    pub const fn strut(&self) -> Option<&Strut> {
        self.strut.as_ref()
    }

    pub const fn transient_for(&self) -> Option<u32> {
        self.transient_for
    }

    pub fn class_instance(&self) -> Option<&str> {
        self.class_instance.as_ref().map(AsRef::as_ref)
    }

    pub const fn is_xpra(&self) -> bool {
        self.is_xpra
    }

    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_ref().map(AsRef::as_ref)
    }

    pub fn startup_id(&self) -> Option<&str> {
        self.startup_id.as_ref().map(AsRef::as_ref)
    }

    pub fn window_role(&self) -> Option<&str> {
        self.window_role.as_ref().map(AsRef::as_ref)
    }

    pub const fn leader_window(&self) -> u32 {
        self.leader_window
    }
}

include!("client_props.rs");

impl Default for ClientWindow {
    fn default() -> Self {
        Self::new(Box::new(antibox_core::mock::MockWindow::new(0)))
    }
}

#[cfg(test)]
#[path = "client_test.rs"]
mod tests;
