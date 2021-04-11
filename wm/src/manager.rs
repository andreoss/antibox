use crate::action::*;
use crate::bindings::KeyBindings;
use crate::drag;
use crate::frame::FrameWindow;
use crate::frame_store::FrameStore;
use crate::handler;
use crate::id::{ClientId, FrameId, XidIndex};
use crate::mouse_parser::{match_mouse_modifiers, mouse_button_from_state, MouseEntry};
use crate::option::WindowOptions;
use crate::render::ThemeColors;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::collections::HashMap;
use std::sync::Arc;

type LayoutSnapshot = HashMap<ClientId, Rect>;
pub struct Config {
    pub workspace_count: u32,
    pub focus_follows_mouse: bool,
    pub click_to_focus: bool,
    pub focus_mode: u32,
    pub gradients: bool,
    pub shapes_protect_client: bool,
    pub opaque_move: bool,
    pub mouse_follows_focus: bool,
    pub warp_pointer: bool,
    pub warp_pointer_on_edge_switch: bool,
}
impl Default for Config {
    fn default() -> Config {
        Config {
            workspace_count: 4,
            focus_follows_mouse: false,
            click_to_focus: true,
            focus_mode: 1,
            gradients: true,
            shapes_protect_client: true,
            opaque_move: true,
            mouse_follows_focus: false,
            warp_pointer: false,
            warp_pointer_on_edge_switch: false,
        }
    }
}
impl Config {
    pub fn set_focus_mode(&mut self, mode: u32) {
        self.focus_mode = mode;
        self.click_to_focus = mode == 1;
        self.focus_follows_mouse = matches!(mode, 2 | 4 | 5);
    }
}
type AfterEventCallback = Box<dyn FnMut(&BackendEvent) + Send>;

pub struct WindowManager<H: DisplayBackend + 'static + ?Sized> {
    pub config: Config,
    pub frames: FrameStore,
    pub xid_index: XidIndex,
    pub(crate) insertion_order: Vec<ClientId>,
    pub(crate) map_order: Vec<ClientId>,
    pub(crate) active_workspace: u32,
    pub focused_window: Option<ClientId>,
    pub(crate) last_focused_window: Option<ClientId>,
    pub(crate) workspace_names: Vec<String>,
    pub(crate) workspace_layouts: Vec<crate::layout::Layout>,
    pub(crate) backend: Option<Arc<H>>,
    pub atoms: AtomManager,
    pub drag_state: Option<(FrameId, Point, crate::wmstate::ResizeEdge, Rect)>,
    pub(crate) xdnd_source: Option<u32>,
    pub(crate) xdnd_drop_target: Option<ClientId>,
    pub(crate) monitors: Vec<MonitorInfo>,
    pub(crate) workareas: Vec<Rect>,
    pub(crate) key_bindings: KeyBindings,
    pub keymaps: crate::bindings::KeymapStack,
    pub(crate) pending_action: Option<Action>,
    pub(crate) after_window_event: Option<AfterEventCallback>,
    pub(crate) dock_manager: crate::dock::DockManager,
    pub(crate) render_backend: Option<Arc<dyn RenderBackend>>,
    pub(crate) self_windows: std::collections::HashSet<u32>,
    pub(crate) win_options: WindowOptions,
    pub(crate) win_menu: Option<crate::winmenu::WindowActionMenu>,
    pub(crate) dock_menu: Option<crate::dockmenu::DockMenu>,
    pub(crate) moveresize_popup: Option<Box<dyn WindowHandle>>,
    pub(crate) drag_outline: Option<Vec<Box<dyn WindowHandle>>>,
    pub(crate) drag_pending: Option<Rect>,
    pub(crate) snap_preview: Option<crate::snap::Preview>,
    pub(crate) saved_layout: LayoutSnapshot,
    pub(crate) mouse_bindings: Vec<MouseEntry>,
    pub(crate) above_windows: Vec<u32>,
    pub(crate) theme_colours: ThemeColors,
    pub(crate) cursors: Vec<u32>,
    pub(crate) pending_unmaps: HashMap<u32, u32>,
    pub(crate) reserved_strut: Strut,
    pub(crate) last_title_click: Option<(u32, ClientId)>,
    pub(crate) showing_desktop: bool,
    pub(crate) desktop_hidden: Vec<ClientId>,
    pub(crate) desktop_focus: Option<ClientId>,
    pub(crate) workspace_names_dirty: bool,
}
impl<H: DisplayBackend + 'static + ?Sized> WindowManager<H> {
    pub fn frame(&self, id: ClientId) -> Option<&FrameWindow> {
        self.frames.get(&id)
    }

    pub fn frame_mut(&mut self, id: ClientId) -> Option<&mut FrameWindow> {
        self.frames.get_mut(&id)
    }

    pub fn client(&self, id: ClientId) -> Option<&crate::client::ClientWindow> {
        self.frames.get(&id).map(FrameWindow::client)
    }

    pub fn frame_by_xid(&self, xid: u32) -> Option<&FrameWindow> {
        self.xid_index
            .client_id_for(xid)
            .and_then(|cid| self.frames.get(&cid))
    }

    pub fn frame_by_xid_mut(&mut self, xid: u32) -> Option<&mut FrameWindow> {
        let cid = self.xid_index.client_id_for(xid)?;
        self.frames.get_mut(&cid)
    }

    pub fn new(backend: &Arc<H>) -> Self {
        let mut wm = Self::new_test();
        wm.workspace_names = (0..Config::default().workspace_count)
            .map(|i| format!("W{}", i + 1))
            .collect();
        wm.backend = Some(Arc::clone(backend));
        wm
    }

    pub fn cid_for_xid(&self, xid: u32) -> Option<ClientId> {
        self.xid_index.client_id_for(xid)
    }

    pub fn with_config(
        backend: &Arc<H>,
        workspace_count: u32,
        workspace_names: Vec<String>,
        click_to_focus: bool,
        mouse_follows_focus: bool,
        colours: &ThemeColors,
    ) -> Self {
        let focus_mode = if click_to_focus { 1u32 } else { 2u32 };
        WindowManager {
            config: Config {
                workspace_count,
                focus_follows_mouse: !click_to_focus,
                click_to_focus,
                focus_mode,
                gradients: true,
                shapes_protect_client: true,
                opaque_move: true,
                mouse_follows_focus,
                ..Config::default()
            },
            frames: FrameStore::new(),
            xid_index: XidIndex::new(),
            insertion_order: Vec::new(),
            map_order: Vec::new(),
            active_workspace: 0,
            focused_window: None,
            last_focused_window: None,
            workspace_names,
            workspace_layouts: Vec::new(),
            backend: Some(Arc::clone(backend)),
            atoms: AtomManager::new(),
            drag_state: None,
            xdnd_source: None,
            xdnd_drop_target: None,
            monitors: Vec::new(),
            workareas: Vec::new(),
            key_bindings: KeyBindings::new(),
            keymaps: crate::bindings::KeymapStack::new(),
            pending_action: None,
            after_window_event: None,
            dock_manager: crate::dock::DockManager::new(),
            render_backend: None,
            self_windows: std::collections::HashSet::new(),
            win_options: WindowOptions::new(),
            win_menu: None,
            dock_menu: None,
            moveresize_popup: None,
            drag_outline: None,
            drag_pending: None,
            snap_preview: None,
            saved_layout: HashMap::new(),
            mouse_bindings: Vec::new(),
            above_windows: Vec::new(),
            theme_colours: *colours,
            cursors: vec![0u32; crate::cursors::idx::COUNT],
            pending_unmaps: HashMap::new(),
            reserved_strut: Strut::default(),
            last_title_click: None,
            showing_desktop: false,
            desktop_hidden: Vec::new(),
            desktop_focus: None,
            workspace_names_dirty: false,
        }
    }
    pub fn new_test() -> Self {
        WindowManager {
            config: Config::default(),
            frames: FrameStore::new(),
            xid_index: XidIndex::new(),
            insertion_order: Vec::new(),
            map_order: Vec::new(),
            monitors: Vec::new(),
            workareas: Vec::new(),
            active_workspace: 0,
            focused_window: None,
            last_focused_window: None,
            workspace_names: vec![],
            workspace_layouts: Vec::new(),
            backend: None,
            atoms: AtomManager::new(),
            drag_state: None,
            key_bindings: KeyBindings::new(),
            keymaps: crate::bindings::KeymapStack::new(),
            xdnd_source: None,
            xdnd_drop_target: None,
            pending_action: None,
            after_window_event: None,
            dock_manager: crate::dock::DockManager::new(),
            render_backend: None,
            self_windows: std::collections::HashSet::new(),
            win_options: WindowOptions::new(),
            win_menu: None,
            dock_menu: None,
            moveresize_popup: None,
            drag_outline: None,
            drag_pending: None,
            snap_preview: None,
            saved_layout: HashMap::new(),
            mouse_bindings: Vec::new(),
            above_windows: Vec::new(),
            theme_colours: ThemeColors::default(),
            cursors: vec![0u32; crate::cursors::idx::COUNT],
            pending_unmaps: HashMap::new(),
            reserved_strut: Strut::default(),
            last_title_click: None,
            showing_desktop: false,
            desktop_hidden: Vec::new(),
            desktop_focus: None,
            workspace_names_dirty: false,
        }
    }
    pub(crate) fn backend(&self) -> Option<&H> {
        self.backend.as_ref().map(|v| v.as_ref())
    }

    pub fn layout_for(&self, ws: u32) -> crate::layout::Layout {
        self.workspace_layouts
            .get(ws as usize)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_workspace_layouts(&mut self, layouts: Vec<crate::layout::Layout>) {
        self.workspace_layouts = layouts;
        let ws = self.active_workspace;
        self.layout_for(ws).arrange(self, ws);
    }

    pub fn set_layout(&mut self, ws: u32, layout: crate::layout::Layout) {
        let len = self
            .workspace_layouts
            .len()
            .max(ws as usize + 1)
            .max(self.config.workspace_count as usize);
        self.workspace_layouts
            .resize(len, crate::layout::Layout::default());
        self.workspace_layouts[ws as usize] = layout;
        layout.arrange(self, ws);
    }

    fn undock_and_close(&mut self, w: u32) {
        self.dock_manager.undock(w);
        if let Some(fw) = self.frame_by_xid(w) {
            if fw
                .client()
                .has_protocol(self.atoms.get("WM_DELETE_WINDOW").unwrap_or(0))
            {
                crate::ewmh::close_window(self.backend().expect("backend present"), &self.atoms, w);
            } else if let Some(b) = self.backend() {
                let _ = b.destroy_window(w);
            }
        }
        self.dock_manager
            .adapt_with(self.backend().expect("backend present"));
    }

    pub(crate) fn expect_client_unmap(&mut self, client: u32) {
        *self.pending_unmaps.entry(client).or_insert(0) += 1;
    }

    pub(crate) fn consume_expected_unmap(&mut self, client: u32) -> bool {
        if let Some(n) = self.pending_unmaps.get_mut(&client) {
            *n -= 1;
            if *n == 0 {
                self.pending_unmaps.remove(&client);
            }
            true
        } else {
            false
        }
    }
    pub fn handle_event(&mut self, e: &BackendEvent) {
        self.dispatch_event(e);
        if let Some(ref mut cb) = self.after_window_event {
            cb(e);
        }
    }

    fn dispatch_event(&mut self, e: &BackendEvent) {
        match e {
            BackendEvent::MapRequest { window } => handler::map_request(self, *window),
            BackendEvent::ConfigureRequest {
                window,
                rect,
                value_mask,
                ..
            } => handler::configure_request(self, *window, *rect, *value_mask),
            BackendEvent::DestroyNotify { window } => self.handle_destroy(*window),
            BackendEvent::ClientMessage {
                window,
                message_type,
                data,
                ..
            } => handler::client_message(self, *window, *message_type, *data),
            BackendEvent::UnmapNotify { window } => handler::unmap(self, *window),
            BackendEvent::Expose { window, rect } => {
                if let Some(menu) = self.win_menu.as_ref().filter(|m| m.visible) {
                    if menu.contains_window(*window) {
                        if let Some(b) = self.backend() {
                            menu.paint(b);
                        }
                        return;
                    }
                }
                if let Some(dock) = self.dock_menu.as_ref().filter(|m| m.visible) {
                    if dock.contains_window(*window) {
                        if let Some(b) = self.backend() {
                            dock.paint(b);
                        }
                        return;
                    }
                }
                handler::expose(self, *window, *rect);
            }
            BackendEvent::PropertyNotify { window, atom, .. } => {
                handler::property_notify(self, *window, *atom);
            }
            BackendEvent::ButtonPress {
                window,
                point,
                root,
                button,
                state,
                ..
            } => {
                if self.win_menu.as_ref().map_or(false, |m| m.visible) {
                    let mut menu = self.win_menu.take().expect("win_menu confirmed Some");
                    let backend = self.backend.clone();
                    if let Some(b) = backend.as_ref().map(|v| v.as_ref()) {
                        if menu.handle_bar_button(b, *window, *point, *button) {
                            self.win_menu = Some(menu);
                            return;
                        }
                    }
                    if menu.contains_window(*window) {
                        if let Some(b) = self.backend.clone() {
                            if menu.click_opens_submenu(&*b, *root) {
                                self.win_menu = Some(menu);
                                return;
                            }
                        }
                    }
                    let on_menu = menu.contains_window(*window);
                    let action = if on_menu {
                        menu.handle_click(*root)
                    } else {
                        None
                    };
                    if let Some(b) = self.backend() {
                        menu.hide(b);
                    }
                    if let Some(a) = action {
                        self.handle_action(&a);
                    }
                    handler::redraw_all_frames(self);
                    return;
                }
                if self.dock_menu.as_ref().map_or(false, |m| m.visible) {
                    let mut menu = self.dock_menu.take().expect("dock_menu confirmed Some");
                    let backend = self.backend.clone();
                    if let Some(b) = backend.as_ref().map(|v| v.as_ref()) {
                        if menu.handle_bar_button(b, *window, *point, *button) {
                            self.dock_menu = Some(menu);
                            return;
                        }
                    }
                    if menu.contains_window(*window) {
                        if let Some(b) = self.backend.clone() {
                            if menu.click_opens_submenu(&*b, *root) {
                                self.dock_menu = Some(menu);
                                return;
                            }
                        }
                    }
                    let win_id = if menu.contains_window(*window) {
                        menu.handle_click(*root)
                    } else {
                        None
                    };
                    if let Some(b) = self.backend() {
                        menu.hide(b);
                    }
                    if let Some(w) = win_id {
                        self.undock_and_close(w);
                    }
                    handler::redraw_all_frames(self);
                    return;
                }
                let root = self.backend().map(|b| b.root().read_id());
                if !self.mouse_bindings.is_empty() && root == Some(*window) {
                    if let Some(action) = self.lookup_mouse(*state, *button) {
                        self.handle_action(&action.clone());
                        return;
                    }
                }
                drag::button_press(self, *window, *button, *state, *point);
            }
            BackendEvent::ButtonRelease { window, point, .. } => {
                drag::button_release(self, *window, *point);
            }
            BackendEvent::MotionNotify {
                window,
                point,
                root,
                ..
            } => {
                if let Some(ref mut menu) = self.win_menu {
                    if menu.visible && menu.contains_window(*window) {
                        menu.handle_motion(
                            self.backend.as_ref().map(|v| v.as_ref()).expect("backend present"),
                            *root,
                        );
                        return;
                    }
                }
                if let Some(ref mut menu) = self.dock_menu {
                    if menu.visible && menu.contains_window(*window) {
                        menu.handle_motion(
                            self.backend.as_ref().map(|v| v.as_ref()).expect("backend present"),
                            *root,
                        );
                        return;
                    }
                }
                drag::motion_notify(self, *window, *point, *root);
            }
            BackendEvent::KeyPress { keycode, state, .. } => {
                if self.win_menu.as_ref().map_or(false, |m| m.visible) {
                    let ks = self.keysym_for(*keycode);
                    let mut menu = self.win_menu.take().expect("win_menu visible");
                    let backend = self.backend.clone();
                    let res = backend
                        .as_ref().map(|v| v.as_ref())
                        .map_or(crate::menu::MenuNav::Ignored, |b| {
                            let min = b.setup_min_keycode();
                            let max = b.setup_max_keycode();
                            match b.get_keyboard_mapping(min, max - min + 1) {
                                Ok(m) => menu.handle_key_input(b, *keycode, *state, &m, ks),
                                Err(_) => menu.handle_key(b, ks),
                            }
                        });
                    match res {
                        crate::menu::MenuNav::Ignored | crate::menu::MenuNav::Handled => {
                            self.win_menu = Some(menu);
                        }
                        crate::menu::MenuNav::Close => {
                            if let Some(b) = self.backend() {
                                menu.hide(b);
                            }
                            handler::redraw_all_frames(self);
                        }
                        crate::menu::MenuNav::Activate(a) => {
                            if let Some(b) = self.backend() {
                                menu.hide(b);
                            }
                            self.handle_action(&a);
                            handler::redraw_all_frames(self);
                        }
                    }
                    return;
                }
                if self.dock_menu.as_ref().map_or(false, |m| m.visible) {
                    use crate::menu::MenuNav;
                    let ks = self.keysym_for(*keycode);
                    let mut menu = self.dock_menu.take().expect("dock_menu visible");
                    let backend = self.backend.clone();
                    let res = backend.as_ref().map(|v| v.as_ref()).map_or(MenuNav::Ignored, |b| {
                        let min = b.setup_min_keycode();
                        let max = b.setup_max_keycode();
                        match b.get_keyboard_mapping(min, max - min + 1) {
                            Ok(m) => menu.handle_key_input(b, *keycode, *state, &m, ks),
                            Err(_) => menu.handle_key(b, ks),
                        }
                    });
                    match res {
                        MenuNav::Ignored | MenuNav::Handled => self.dock_menu = Some(menu),
                        MenuNav::Close => {
                            if let Some(b) = self.backend() {
                                menu.hide(b);
                            }
                            handler::redraw_all_frames(self);
                        }
                        MenuNav::Activate(w) => {
                            if let Some(b) = self.backend() {
                                menu.hide(b);
                            }
                            self.undock_and_close(w);
                            handler::redraw_all_frames(self);
                        }
                    }
                    return;
                }
                if self.keymaps.is_active() {
                    if self.drag_state.is_none() {
                        self.keymaps.clear();
                    } else {
                        let ks = self.keysym_for(*keycode);
                        match self.keymaps.lookup(ks) {
                            crate::bindings::KeymapLookup::Pass => {}
                            crate::bindings::KeymapLookup::Swallow => return,
                            crate::bindings::KeymapLookup::Pop => {
                                self.keymaps.pop();
                                return;
                            }
                            crate::bindings::KeymapLookup::Dispatch(a) => {
                                self.handle_action(&a);
                                return;
                            }
                            crate::bindings::KeymapLookup::Drag => {
                                drag::keyboard_drag(self, ks);
                                if self.drag_state.is_none() {
                                    self.keymaps.pop();
                                }
                                return;
                            }
                        }
                    }
                }
                if let Some(a) = self.key_bindings.lookup(*keycode as u8, *state).cloned() {
                    self.handle_action(&a);
                }
            }
            BackendEvent::EnterNotify { window, .. } => {
                self.dock_pointer_enter(*window);
                handler::enter_notify(self, *window);
            }
            BackendEvent::LeaveNotify { window, .. } => self.dock_pointer_leave(*window),
            BackendEvent::MappingNotify { .. } => handler::mapping_notify(self),
            BackendEvent::ShapeNotify { window, .. } => handler::shape_notify(self, *window),
            _ => {}
        }
    }

    fn keysym_for(&self, keycode: u32) -> u32 {
        let b = match self.backend() {
            Some(b) => b,
            None => return 0,
        };
        let min = b.setup_min_keycode();
        let max = b.setup_max_keycode();
        if let Ok(m) = b.get_keyboard_mapping(min, (max as usize - min as usize + 1) as u8) {
            let off =
                (keycode as usize).saturating_sub(min as usize) * (m.keysyms_per_keycode as usize);
            if off < m.keysyms.len() {
                return m.keysyms[off];
            }
        }
        0
    }

    pub fn lookup_mouse(&self, state: u16, button: u8) -> Option<&Action> {
        for entry in &self.mouse_bindings {
            if mouse_button_from_state(button, entry.button_keysym)
                && match_mouse_modifiers(state, entry.modifiers)
            {
                return Some(&entry.action);
            }
        }
        None
    }
    pub fn handle_destroy(&mut self, w: u32) {
        handler::destroy(self, w);
    }
    pub fn take_window_to_workspace(&mut self, target: u32) {
        if target >= self.config.workspace_count || target == self.active_workspace {
            return;
        }
        if let Some(w) = self.focused_window {
            let moved = if let Some(fw) = self.frames.get_mut(&w) {
                if fw.workspace() != !0 {
                    fw.set_workspace(target);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if moved {
                if let Some(b) = self.backend() {
                    crate::ewmh::set_wm_desktop(b, &self.atoms, self.xid_index.xid_of(w), target);
                }
            }
        }
        self.activate_workspace(target);
    }

    pub fn move_window_to_workspace(&mut self, target: u32) {
        if target >= self.config.workspace_count || target == self.active_workspace {
            return;
        }
        if let Some(w) = self.focused_window {
            let moved = if let Some(fw) = self.frames.get_mut(&w) {
                if fw.workspace() != !0 {
                    fw.set_workspace(target);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if moved {
                if let Some(b) = self.backend() {
                    crate::ewmh::set_wm_desktop(b, &self.atoms, self.xid_index.xid_of(w), target);
                }
                self.apply_workspace_visibility();
                crate::focus::recover_focus(self);
                self.layout_for(self.active_workspace)
                    .arrange(self, self.active_workspace);
            }
        }
    }

    pub fn activate_workspace(&mut self, i: u32) {
        if i < self.config.workspace_count && i != self.active_workspace {
            self.active_workspace = i;
            if let Some(c) = self.backend() {
                crate::ewmh::update_current_desktop(c, &self.atoms, i);
                crate::ewmh::update_desktop_names(c, &self.atoms, &self.workspace_names);
            }
            self.apply_workspace_visibility();
            crate::focus::recover_focus(self);
            self.layout_for(i).arrange(self, i);
        }
    }

    fn dock_window_client(&self, window: u32) -> Option<u32> {
        if self.dock_manager.is_dock_app(window) {
            return Some(window);
        }
        self.xid_index
            .client_id_for(window)
            .map(|cid| self.xid_index.xid_of(cid))
            .filter(|id| self.dock_manager.is_dock_app(*id))
    }

    pub(crate) fn dock_pointer_enter(&mut self, window: u32) {
        if self.dock_window_client(window).is_none() {
            return;
        }
        if self.dock_manager.expand() {
            if let Some(b) = self.backend.clone() {
                self.dock_manager.adapt_with(&*b);
            }
        } else {
            self.dock_manager.pointer_entered();
        }
    }

    pub(crate) fn dock_pointer_leave(&mut self, window: u32) {
        if self.dock_window_client(window).is_none() || self.dock_manager.is_dragging() {
            return;
        }
        let b = match self.backend.clone() {
            Some(b) => b,
            None => return,
        };
        let ptr = match b.query_pointer(b.root().read_id()) {
            Ok(p) => p,
            Err(_) => return,
        };
        let sw = b.screen_width() as i32;
        let sh = b.screen_height() as i32;
        if self
            .dock_manager
            .should_collapse(ptr.root_x as i32, ptr.root_y as i32, sw, sh)
            && self.dock_manager.collapse()
        {
            self.dock_manager.adapt_with(&*b);
        }
    }

    pub(crate) fn apply_workspace_visibility(&mut self) {
        let cur = self.active_workspace;
        let backend = self.backend.clone();
        let b = match backend {
            Some(b) => b,
            None => return,
        };
        let decisions: Vec<(u32, u32, bool)> = self
            .frames
            .values()
            .map(|fw| {
                let visible =
                    workspace_visible(fw.workspace(), fw.state().sticky, fw.state().minimized, cur)
                        || self.dock_manager.is_dock_app(fw.client_xid());
                (fw.frame().id(), fw.client_xid(), visible)
            })
            .collect();
        for (_fr, cl, visible) in &decisions {
            if !visible {
                self.expect_client_unmap(*cl);
            }
        }
        for (fr, cl, visible) in decisions {
            if visible {
                let _ = b.map_window(cl);
                let _ = b.map_window(fr);
            } else {
                let _ = b.unmap_window(fr);
                let _ = b.unmap_window(cl);
            }
        }
        let _ = b.flush();
    }
    pub fn cycle_focus(&mut self, forward: bool) {
        crate::focus::cycle_focus(self, forward);
    }
    pub fn raise_to_top(&mut self, id: ClientId) {
        if self.insertion_order.last() == Some(&id) {
            return;
        }
        self.insertion_order.retain(|&x| x != id);
        self.insertion_order.push(id);
    }
    pub fn lower_to_bottom(&mut self, id: ClientId) {
        if self.insertion_order.first() == Some(&id) {
            return;
        }
        self.insertion_order.retain(|&x| x != id);
        self.insertion_order.insert(0, id);
    }
    pub fn handle_action(&mut self, a: &Action) {
        crate::wmaction::handle_wm_action(self, a);
        match a {
            Action::Focus(FocusOp::Next) => self.cycle_focus(true),
            Action::Focus(FocusOp::Prev) => self.cycle_focus(false),
            Action::Workspace(WorkspaceOp::NextWorkspace) => {
                self.activate_workspace((self.active_workspace + 1) % self.config.workspace_count);
            }
            Action::Workspace(WorkspaceOp::PrevWorkspace) => {
                self.activate_workspace(if self.active_workspace == 0 {
                    self.config.workspace_count - 1
                } else {
                    self.active_workspace - 1
                });
            }
            Action::Workspace(WorkspaceOp::Workspace(i)) => {
                self.activate_workspace(*i);
            }
            Action::Workspace(WorkspaceOp::MoveWindowTo(i)) => {
                self.move_window_to_workspace(*i);
            }
            Action::Workspace(WorkspaceOp::WorkspaceNextTaken) => {
                if let Some(w) =
                    crate::focus::next_workspace_with_windows(self, self.active_workspace, true)
                {
                    self.activate_workspace(w);
                }
            }
            Action::Workspace(WorkspaceOp::WorkspacePrevTaken) => {
                if let Some(w) =
                    crate::focus::next_workspace_with_windows(self, self.active_workspace, false)
                {
                    self.activate_workspace(w);
                }
            }
            Action::Workspace(WorkspaceOp::WorkspaceNextTakeWin) => {
                self.take_window_to_workspace(
                    (self.active_workspace + 1) % self.config.workspace_count,
                );
            }
            Action::Workspace(WorkspaceOp::WorkspacePrevTakeWin) => {
                self.take_window_to_workspace(if self.active_workspace == 0 {
                    self.config.workspace_count - 1
                } else {
                    self.active_workspace - 1
                });
            }
            Action::Misc(MiscOp::Command(cmd)) => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if let Some((p, a)) = parts.split_first() {
                    let _ = std::process::Command::new(p).args(a).spawn();
                }
            }
            Action::Menu(MenuOp::WindowPickerList) | Action::Menu(MenuOp::Pager) => {
                self.pending_action = Some(a.clone());
            }
            Action::Menu(MenuOp::WindowActionMenu) => {
                let ws_count = self.config.workspace_count;
                let mut menu = crate::winmenu::WindowActionMenu::for_focused_client_opts(
                    ws_count,
                    &self.theme_colours,
                );
                if let Some(b) = self.backend() {
                    let pos = self.focused_window.and_then(|fwid| {
                        self.frames.get(&fwid).map(|fw| {
                            let fr = fw.frame_rect();
                            Point::new(fr.x + fr.w / 2, fr.y)
                        })
                    });
                    if let Some(pos) = pos {
                        menu.show(b, pos);
                        if let Some(rb) = self.render_backend.as_ref() {
                            menu.enable_filter(rb);
                        }
                        self.win_menu = Some(menu);
                    }
                }
            }
            _ => {}
        }
    }
    pub fn focused_window(&self) -> Option<ClientId> {
        self.focused_window
    }
    pub fn active_workspace(&self) -> u32 {
        self.active_workspace
    }
    pub fn workspace_count(&self) -> u32 {
        self.config.workspace_count
    }
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
    pub fn save_layout(&mut self) {
        self.saved_layout.clear();
        for (id, fw) in self.frames.iter() {
            self.saved_layout.insert(*id, fw.frame_rect());
        }
    }

    pub(crate) fn reposition_resize_handles(&self, client_id: ClientId) {
        if let Some(b) = self.backend() {
            if let Some(fw) = self.frames.get(&client_id) {
                fw.layout_pointer_windows(b);
            }
        }
    }

    pub fn restore_layout(&mut self) {
        let snap = std::mem::take(&mut self.saved_layout);
        for (id, rect) in snap {
            if let Some(fw) = self.frames.get_mut(&id) {
                fw.set_frame_rect(rect);
                let _ = fw.frame().configure(
                    Some(rect.x),
                    Some(rect.y),
                    Some(rect.w as u16),
                    Some(rect.h as u16),
                );
                if let Some(b) = self.backend() {
                    let _ = b.configure_window(
                        self.xid_index.xid_of(id),
                        &[rect.x as u32, rect.y as u32, rect.w as u32, rect.h as u32],
                    );
                }
            }
            self.reposition_resize_handles(id);
        }
        if let Some(b) = self.backend() {
            let _ = b.flush();
        }
    }
}

pub(crate) fn workspace_visible(
    workspace: u32,
    sticky: bool,
    minimized: bool,
    active: u32,
) -> bool {
    (workspace == !0 || workspace == active || sticky) && !minimized
}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod tests;
