 use antibox_core::error::Result;
use crate::action::*;
use crate::applet::{Applet, AppletContainer};
use crate::clock_applet::ClockApplet;
use crate::workspace_pane::WorkspacesPane;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskBarPosition {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BarRepaint {
    None,
    Applet(u32),
    Full,
}

pub struct TaskBar {
    pub(crate) conn: Arc<dyn DisplayBackend>,
    pub(crate) window: Box<dyn WindowHandle>,
    pub(crate) position: TaskBarPosition,
    pub(crate) applets: Vec<Box<dyn Applet>>,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub gradients_enabled: bool,
    pub task_bar_colour: antibox_core::colour::Colour,
    pub menu_colours: crate::menu::MenuColors,
    pub(crate) window_x: i32,
    pub(crate) window_y: i32,
    pub(crate) menu: Option<crate::menu::MenuView<Action>>,
    pub(crate) pending_action: Option<Action>,
    pub(crate) strut_atom: u32,
    gfx_cache: std::cell::RefCell<std::collections::HashMap<u32, Box<dyn GraphicsContext>>>,
    geom_cache: std::cell::RefCell<std::collections::HashMap<u32, (u16, u16)>>,
    tray_x: std::cell::Cell<i16>,
}

impl TaskBar {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        position: TaskBarPosition,
        strut_atom: u32,
    ) -> Result<Self> {
        let screen_w = conn.screen_width();
        let screen_h = conn.screen_height();
        let bar_height = Self::bar_height();
        let y = match position {
            TaskBarPosition::Bottom => screen_h as i16 - bar_height as i16,
            _ => 0,
        };
        let window = conn.create_window(
            conn.root().as_parent(),
            Rect::new(0, y as i32, screen_w as i32, bar_height as i32),
            WmWindowClass::InputOutput,
            true,
            EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::EXPOSURE,
        )?;
        Ok(Self {
            conn: Arc::clone(conn),
            window,
            position,
            applets: Vec::new(),
            width: screen_w,
            height: bar_height,
            gradients_enabled: false,
            task_bar_colour: crate::render::ThemeColors::default().task_bar_colour,
            menu_colours: crate::menu::MenuColors::default(),
            window_x: 0,
            window_y: y as i32,
            menu: None,
            pending_action: None,
            strut_atom,
            gfx_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            geom_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            tray_x: std::cell::Cell::new(-1),
        })
    }

    fn bar_height_for(double: bool) -> u16 {
        let content = antibox_ui::metrics::panel_height() as u16;
        let content = if double { content * 2 } else { content };
        content + antibox_ui::theme::panel_edge_height()
    }

    pub(crate) fn bar_height() -> u16 {
        Self::bar_height_for(crate::layout_preferences::taskbar_double_height())
    }

    pub fn show(&self) -> Result<()> {
        self.window.map()
    }

    pub(crate) fn apply_background(&self, win: u32) {
        const CFG_BACK_PIXEL: u32 = 2;
        let _ = self
            .conn
            .change_window_attributes(win, &[CFG_BACK_PIXEL, self.task_bar_colour]);
    }

    pub(crate) fn set_taskbar_colour(&mut self, colour: antibox_core::colour::Colour) {
        self.task_bar_colour = colour;
        self.apply_background(self.window.id());
        let ids: Vec<u32> = self.applets.iter().map(|a| a.window().id()).collect();
        for id in ids {
            self.apply_background(id);
        }
    }

    pub(crate) fn apply_theme_colours(&mut self, tc: &crate::render::ThemeColors, gradients: bool) {
        self.gradients_enabled = gradients;
        self.set_taskbar_colour(tc.task_bar_colour);
        self.menu_colours = crate::menu::MenuColors::with_bg(tc.menu_bg);
    }

    fn applet_geom(&self, applet: &dyn Applet) -> Option<(u16, u16)> {
        let wid = applet.window().id();
        if wid == 0 {
            return None;
        }
        if let Some((w, h)) = self.geom_cache.borrow().get(&wid).copied() {
            if w > 0 && h > 0 {
                return Some((w, h));
            }
        }
        match applet.window().get_geometry() {
            Ok((w, h)) if w > 0 && h > 0 => Some((w, h)),
            _ => None,
        }
    }

    pub fn paint(&self) -> Result<()> {
        let mut cache = self.gfx_cache.borrow_mut();
        let tb_id = self.window.id();
        cache.retain(|&id, _| id == tb_id || self.applets.iter().any(|a| a.window().id() == id));

        let depth = self.conn.screen_depth();
        let mut blits: Vec<(u32, u32, u16, u16)> = Vec::new();
        if let Ok(pm) = self.conn.create_pixmap(self.width, self.height, depth) {
            if let Ok(pg) = self.conn.create_graphics(pm) {
                self.draw_background_to(&*pg);
            }
            blits.push((tb_id, pm, self.width, self.height));
        }
        for applet in &self.applets {
            let Some((w, h)) = self.applet_geom(applet.as_ref()) else { continue };
            let wid = applet.window().id();
            if let Ok(pm) = self.conn.create_pixmap(w, h, depth) {
                if let Ok(pg) = self.conn.create_graphics(pm) {
                    let _ = guard_applet("paint", || applet.paint(&*pg));
                }
                blits.push((wid, pm, w, h));
            }
        }
        for &(wid, pm, w, h) in &blits {
            if let Some(wg) = Self::gfx(&mut cache, &self.conn, wid) {
                let _ = wg.copy_from(pm, Rect::px(0, 0, w, h), Point::ZERO);
            }
        }
        let _ = self.conn.flush();
        for &(_, pm, _, _) in &blits {
            let _ = self.conn.free_pixmap(pm);
        }
        Ok(())
    }

    fn paint_applet_buffered(
        &self,
        cache: &mut std::collections::HashMap<u32, Box<dyn GraphicsContext>>,
        applet: &dyn Applet,
    ) {
        let wid = applet.window().id();
        let Some((w, h)) = self.applet_geom(applet) else { return };
        let depth = self.conn.screen_depth();
        let Ok(pm) = self.conn.create_pixmap(w, h, depth) else { return };
        if let Ok(pg) = self.conn.create_graphics(pm) {
            let _ = guard_applet("paint", || applet.paint(&*pg));
            if let Some(wg) = Self::gfx(cache, &self.conn, wid) {
                let _ = wg.copy_from(pm, Rect::px(0, 0, w, h), Point::ZERO);
            }
        }
        let _ = self.conn.free_pixmap(pm);
    }

    fn draw_background_to(&self, g: &dyn GraphicsContext) {
        antibox_ui::theme::panel_surface(g, self.width, self.height, self.task_bar_colour);
        antibox_ui::theme::panel_edge(g, self.width);
        let tx = self.tray_x.get();
        if tx > 0 {
            let edge = antibox_ui::theme::panel_edge_height() as i16;
            let ew = antibox_ui::theme::tray_edge_width() as i16;
            antibox_ui::theme::tray_edge(g, tx - ew, edge, self.height.saturating_sub(edge as u16));
        }
    }

    fn paint_background(
        &self,
        cache: &mut std::collections::HashMap<u32, Box<dyn GraphicsContext>>,
    ) {
        let depth = self.conn.screen_depth();
        if let Ok(pm) = self.conn.create_pixmap(self.width, self.height, depth) {
            if let Ok(pg) = self.conn.create_graphics(pm) {
                self.draw_background_to(&*pg);
                if let Some(wg) = Self::gfx(cache, &self.conn, self.window.id()) {
                    let _ = wg.copy_from(pm, Rect::px(0, 0, self.width, self.height), Point::ZERO);
                }
            }
            let _ = self.conn.free_pixmap(pm);
        } else if let Some(g) = Self::gfx(cache, &self.conn, self.window.id()) {
            self.draw_background_to(g);
        }
    }

    pub fn paint_window(&self, window: u32) -> Result<()> {
        let mut cache = self.gfx_cache.borrow_mut();
        if window == self.window.id() {
            self.paint_background(&mut cache);
        } else if let Some(applet) = self.applets.iter().find(|a| a.window().id() == window) {
            self.paint_applet_buffered(&mut cache, applet.as_ref());
        }
        let _ = self.conn.flush();
        Ok(())
    }

    fn gfx<'a>(
        cache: &'a mut std::collections::HashMap<u32, Box<dyn GraphicsContext>>,
        conn: &Arc<dyn DisplayBackend>,
        wid: u32,
    ) -> Option<&'a dyn GraphicsContext> {
        use std::collections::hash_map::Entry;
        match cache.entry(wid) {
            Entry::Occupied(e) => Some(&**e.into_mut()),
            Entry::Vacant(e) => Some(&**e.insert(conn.create_graphics(wid).ok()?)),
        }
    }

    pub fn group_members_at(&mut self, window: u32, x: i32) -> Option<Vec<u32>> {
        for applet in &mut self.applets {
            if applet.window().id() != window {
                continue;
            }
            let pane = applet
                .as_any_mut()
                .downcast_mut::<crate::taskpane::TaskPane>()?;
            let members = pane.members_at(x)?;
            if members.len() > 1 {
                return Some(members);
            }
            return None;
        }
        None
    }

    pub fn handle_click(&mut self, window: u32, x: i32, y: i32, button: u8) -> Option<u32> {
        let mut hit = None;
        for applet in &mut self.applets {
            if applet.window().id() == window {
                let id =
                    guard_applet("click", || applet.handle_click(x, y, button)).and_then(|v| v);
                if let Some(action) =
                    guard_applet("take_action", || applet.take_action()).and_then(|v| v)
                {
                    self.pending_action = Some(action);
                }
                hit = Some(id);
                break;
            }
        }
        if let Some(id) = hit {
            self.relayout();
            return id;
        }

        if button == 3 {
            self.show_menu(x, y);
        }
        None
    }

    pub fn handle_release(&mut self, window: u32, x: i32, y: i32, button: u8) -> Option<u32> {
        let mut id = None;
        let mut relayout = false;
        for applet in &mut self.applets {
            if applet.window().id() == window {
                id =
                    guard_applet("release", || applet.handle_release(x, y, button)).and_then(|v| v);
                relayout = applet.take_relayout();
                break;
            }
        }
        if relayout {
            self.relayout();
        }
        id
    }

    pub fn owns_window(&self, id: u32) -> bool {
        if self.window.id() == id {
            return true;
        }
        self.menu.as_ref().is_some_and(|m| m.contains_window(id))
    }

    pub fn handle_menu_event(&mut self, event: &BackendEvent, conn: &dyn DisplayBackend) -> bool {
        use crate::menu::MenuNav;
        let Some(menu) = self.menu.as_mut() else { return false };
        if !menu.visible {
            return false;
        }
        match menu.handle_event(conn, event) {
            MenuNav::Ignored => false,
            MenuNav::Handled => true,
            MenuNav::Close => {
                menu.hide(conn);
                self.menu = None;
                true
            }
            MenuNav::Activate(a) => {
                menu.hide(conn);
                self.menu = None;
                self.pending_action = Some(a);
                true
            }
        }
    }

    pub fn take_pending_action(&mut self) -> Option<Action> {
        self.pending_action.take()
    }

    pub fn show_menu(&mut self, x: i32, y: i32) {
        use crate::menu_tree::MenuNode;
        let nodes: Vec<MenuNode<Action>> = vec![
            MenuNode::leaf("_Cascade", Action::Tile(TileOp::Cascade)),
            MenuNode::leaf("Tile _Vertically", Action::Tile(TileOp::TileVertical)),
            MenuNode::leaf("Tile _Horizontally", Action::Tile(TileOp::TileHorizontal)),
            MenuNode::leaf("_Arrange", Action::Tile(TileOp::Arrange)),
            MenuNode::leaf("_Undo", Action::Tile(TileOp::UndoArrange)),
            MenuNode::separator(),
            MenuNode::leaf(
                "_Minimize All",
                Action::Workspace(WorkspaceOp::MinimizeAll),
            ),
            MenuNode::leaf("Hi_de All", Action::Workspace(WorkspaceOp::HideAll)),
            MenuNode::separator(),
            MenuNode::leaf("_Window List", Action::Menu(MenuOp::WindowPickerList)),
        ];
        let pos = self
            .conn
            .query_pointer(self.conn.root().read_id())
            .map(|p| Point::new(p.root_x as i32, p.root_y as i32))
            .unwrap_or_else(|_| Point::new(x + self.window_x, y + self.window_y));
        self.open_menu(nodes, pos);
    }

    pub fn show_workspace_menu(&mut self, ws: u32, current: crate::layout::Layout) {
        use crate::menu_tree::MenuNode;
        let nodes: Vec<MenuNode<Action>> = crate::layout::Layout::ALL
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let mark = if *l == current { "\u{2022} " } else { "  " };
                MenuNode::leaf(
                    format!("{}{}", mark, l.title()),
                    Action::Workspace(WorkspaceOp::SetLayout(ws, i as u8)),
                )
            })
            .collect();
        let pos = self
            .conn
            .query_pointer(self.conn.root().read_id())
            .map(|p| Point::new(p.root_x as i32, p.root_y as i32))
            .unwrap_or_else(|_| Point::new(self.window_x, self.window_y));
        self.open_menu(nodes, pos);
    }

    fn open_menu(&mut self, nodes: Vec<crate::menu_tree::MenuNode<Action>>, pos: Point) {
        let mut menu = crate::menu::MenuView::with_nodes(nodes);
        menu.colours = self.menu_colours;
        menu.show(self.conn.as_ref(), pos);
        self.menu = Some(menu);
    }

    pub fn update_height(&mut self) -> bool {
        let h = Self::bar_height();
        if h == self.height {
            return false;
        }
        self.height = h;
        self.fit_to_screen();
        self.relayout();
        true
    }

    pub fn strut(&self) -> Strut {
        let h = self.height as u32;
        match self.position {
            TaskBarPosition::Bottom => Strut {
                bottom: h,
                ..Default::default()
            },
            TaskBarPosition::Top => Strut {
                top: h,
                ..Default::default()
            },
            _ => Strut::default(),
        }
    }

    pub fn update_strut(&self) {
        if self.strut_atom == 0 {
            return;
        }
        let h = self.height as u32;
        let strut = match self.position {
            TaskBarPosition::Bottom => [0u32, 0, 0, h],
            TaskBarPosition::Top => [0u32, 0, h, 0],
            _ => [0u32, 0, 0, 0],
        };
        if strut.iter().any(|&s| s > 0) {
            let _ = self.conn.change_property32(
                PropMode::Replace,
                self.window.id(),
                self.strut_atom,
                6,
                &strut,
            );
        }
    }

    pub fn fit_to_screen(&mut self) {
        let screen_w = self.conn.screen_width();
        let screen_h = self.conn.screen_height();
        let y = match self.position {
            TaskBarPosition::Bottom => screen_h as i32 - self.height as i32,
            _ => 0,
        };
        self.width = screen_w;
        self.window_x = 0;
        self.window_y = y;
        let _ = self
            .window
            .configure(Some(0), Some(y), Some(screen_w), Some(self.height));
        self.update_strut();
        self.relayout();
        let _ = self.paint();
        let _ = self.conn.flush();
    }

    pub fn set_active_workspace(&mut self, ws: u32) {
        for applet in &mut self.applets {
            if let Some(pane) = applet.as_any_mut().downcast_mut::<WorkspacesPane>() {
                pane.set_active(ws);
                return;
            }
        }
    }

    pub fn set_workspace_names(&mut self, names: &[String]) -> bool {
        let mut found = false;
        for applet in &mut self.applets {
            if let Some(pane) = applet.as_any_mut().downcast_mut::<WorkspacesPane>() {
                pane.set_names(names);
                found = true;
                break;
            }
        }
        if found {
            self.relayout();
        }
        found
    }

    pub fn sync_task_pane(
        &mut self,
        wm: &crate::manager::WindowManager<dyn DisplayBackend>,
    ) -> BarRepaint {
        let mut layout = false;
        let mut content: Option<u32> = None;
        for applet in &mut self.applets {
            let wid = applet.window().id();
            if let Some(pane) = applet
                .as_any_mut()
                .downcast_mut::<crate::taskpane::TaskPane>()
            {
                match pane.sync_from_frames(
                    &wm.frames,
                    wm.focused_window(),
                    wm.active_workspace(),
                    &wm.xid_index,
                ) {
                    crate::taskpane::TaskSync::Layout => layout = true,
                    crate::taskpane::TaskSync::Content => content = Some(wid),
                    crate::taskpane::TaskSync::Unchanged => {}
                }
            }
        }
        if layout {
            self.relayout();
            BarRepaint::Full
        } else if let Some(wid) = content {
            BarRepaint::Applet(wid)
        } else {
            BarRepaint::None
        }
    }

    pub fn sync_pager(
        &mut self,
        wm: &crate::manager::WindowManager<dyn DisplayBackend>,
    ) -> Option<u32> {
        for applet in &mut self.applets {
            let wid = applet.window().id();
            if let Some(ws) = applet.as_any_mut().downcast_mut::<WorkspacesPane>() {
                if ws.sync_from_frames(&wm.frames, wm.focused_window(), &wm.insertion_order) {
                    return Some(wid);
                }
            }
        }
        None
    }

    fn update_applets<T: 'static, F: FnMut(&mut T) -> bool>(&mut self, mut f: F) -> Vec<u32> {
        let mut changed = Vec::new();
        for applet in &mut self.applets {
            let hit = if let Some(t) = applet.as_any_mut().downcast_mut::<T>() {
                guard_applet("update", || f(t)).unwrap_or(false)
            } else {
                false
            };
            if hit {
                changed.push(applet.window().id());
            }
        }
        changed
    }

    pub fn update_clocks(&mut self) -> Vec<u32> {
        self.update_applets::<ClockApplet, _>(ClockApplet::update)
    }

    pub fn update_cpu(&mut self) -> Vec<u32> {
        self.update_applets::<crate::cpu_status_applet::CpuStatusApplet, _>(
            crate::cpu_status_applet::CpuStatusApplet::update,
        )
    }

    pub fn update_mem(&mut self) -> Vec<u32> {
        self.update_applets::<crate::mem_status_applet::MemStatusApplet, _>(|mem| {
            mem.update();
            true
        })
    }

    pub fn update_net(&mut self) -> Vec<u32> {
        self.update_applets::<crate::net_status_applet::NetStatusApplet, _>(|net| {
            net.update();
            true
        })
    }

    pub fn update_power_audio(&mut self) -> Vec<u32> {
        self.pump_power_audio(false)
    }

    pub fn update_power_audio_battery(&mut self) -> Vec<u32> {
        self.pump_power_audio(true)
    }

    fn pump_power_audio(&mut self, battery_only: bool) -> Vec<u32> {
        let mut relayout = false;
        let mut changed = Vec::new();
        for applet in &mut self.applets {
            let Some(pa) = applet
                .as_any_mut()
                .downcast_mut::<crate::power_audio_applet::PowerAudioApplet>() else { continue };
            let was_w = pa.preferred_width();
            let dirty = if battery_only {
                guard_applet("update", || pa.update_battery())
            } else {
                guard_applet("update", || pa.update())
            };
            if dirty.unwrap_or(false) {
                changed.push(pa.window().id());
            }
            if pa.preferred_width() != was_w {
                relayout = true;
            }
        }
        if relayout {
            self.relayout();
        }
        changed
    }

    pub fn update_keyboard(&mut self) -> Vec<u32> {
        let mut relayout = false;
        let mut changed = Vec::new();
        for applet in &mut self.applets {
            let Some(kb) = applet
                .as_any_mut()
                .downcast_mut::<crate::keyboard_applet::KeyboardApplet>() else { continue };
            let was_present = kb.preferred_width() > 0;
            if guard_applet("update", || kb.update()).unwrap_or(false) {
                changed.push(kb.window().id());
            }
            if (kb.preferred_width() > 0) != was_present {
                relayout = true;
            }
        }
        if relayout {
            self.relayout();
        }
        changed
    }

    pub fn reflow(&mut self) {
        self.relayout();
    }

    pub fn pump_tooltips(&mut self) {
        for a in &mut self.applets {
            let _ = guard_applet("tick_tooltip", || a.tick_tooltip());
        }
    }

    pub fn update_urgent(&mut self) -> Vec<u32> {
        let mut v = Vec::new();
        for a in &self.applets {
            if let Some(p) = a.as_any().downcast_ref::<crate::taskpane::TaskPane>() {
                if p.buttons.iter().any(|b| b.urgent) {
                    v.push(a.window().id());
                }
            }
        }
        if !v.is_empty() {
            crate::taskpane::tick_urgent_phase();
        }
        v
    }

    pub fn any_tooltip_pending(&self) -> bool {
        self.applets.iter().any(|a| a.tooltip_pending())
    }

    #[cfg(feature = "tray")]
    pub fn tray_forget(&mut self, window: u32) -> bool {
        let mut changed = false;
        for applet in &mut self.applets {
            if let Some(tray) = applet
                .as_any_mut()
                .downcast_mut::<crate::tray_applet::TrayApplet>()
            {
                if tray.forget_window(window) {
                    changed = true;
                }
            }
        }
        if changed {
            self.relayout();
        }
        changed
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PanelSlot {
    Menu,
    Workspaces,
    Task,
    Tray,
    Cpu,
    Mem,
    Net,
    PowerAudio,
    Keyboard,
    Clock,
}

const DEFAULT_LEFT: [PanelSlot; 1] = [PanelSlot::Workspaces];
const DEFAULT_RIGHT: [PanelSlot; 7] = [
    PanelSlot::Cpu,
    PanelSlot::Mem,
    PanelSlot::Net,
    PanelSlot::PowerAudio,
    PanelSlot::Keyboard,
    PanelSlot::Tray,
    PanelSlot::Clock,
];

impl PanelSlot {
    const fn from_widget(w: crate::layout_preferences::Widget) -> Self {
        use crate::layout_preferences::Widget;
        match w {
            Widget::Menu => Self::Menu,
            Widget::Workspaces => Self::Workspaces,
            Widget::Windows => Self::Task,
            Widget::Tray => Self::Tray,
            Widget::Cpu => Self::Cpu,
            Widget::Mem => Self::Mem,
            Widget::Net => Self::Net,
            Widget::PowerAudio => Self::PowerAudio,
            Widget::Keyboard => Self::Keyboard,
            Widget::Clock => Self::Clock,
        }
    }

    fn matches(self, a: &dyn Applet) -> bool {
        let any = a.as_any();
        match self {
            Self::Menu => any.is::<crate::menu_applet::MenuApplet>(),
            Self::Workspaces => any.is::<WorkspacesPane>(),
            Self::Task => any.is::<crate::taskpane::TaskPane>(),
            Self::Cpu => any.is::<crate::cpu_status_applet::CpuStatusApplet>(),
            Self::Mem => any.is::<crate::mem_status_applet::MemStatusApplet>(),
            Self::Net => any.is::<crate::net_status_applet::NetStatusApplet>(),
            Self::PowerAudio => any.is::<crate::power_audio_applet::PowerAudioApplet>(),
            Self::Keyboard => any.is::<crate::keyboard_applet::KeyboardApplet>(),
            Self::Clock => any.is::<ClockApplet>(),
            #[cfg(feature = "tray")]
            Self::Tray => any.is::<crate::tray_applet::TrayApplet>(),
            #[cfg(not(feature = "tray"))]
            PanelSlot::Tray => false,
        }
    }
}

fn effective_slots(
    layout: Option<Vec<crate::layout_preferences::Widget>>,
) -> (Vec<PanelSlot>, Vec<PanelSlot>) {
    match layout {
        None => (DEFAULT_LEFT.to_vec(), DEFAULT_RIGHT.to_vec()),
        Some(widgets) => {
            let slots: Vec<PanelSlot> =
                widgets.iter().map(|w| PanelSlot::from_widget(*w)).collect();
            match slots.iter().position(|s| *s == PanelSlot::Task) {
                Some(p) => (slots[..p].to_vec(), slots[p + 1..].to_vec()),
                None => (slots, Vec::new()),
            }
        }
    }
}

impl TaskBar {
    pub fn sync_applets<F>(&mut self, mut make: F)
    where
        F: FnMut(crate::layout_preferences::Widget) -> Option<Box<dyn Applet>>,
    {
        for w in crate::layout_preferences::Widget::ALL.iter().copied() {
            let slot = PanelSlot::from_widget(w);
            let at = self.applets.iter().position(|a| slot.matches(a.as_ref()));
            let wanted = crate::layout_preferences::taskbar_wants(w);
            match (at, wanted) {
                (Some(i), false) => self.remove_applet(i),
                (None, true) => {
                    if let Some(a) = make(w) {
                        self.add_applet(a);
                    }
                }
                _ => {}
            }
        }
    }
}

impl AppletContainer for TaskBar {
    fn relayout(&mut self) {
        let bar_w = self.width as i16;
        let edge = antibox_ui::theme::panel_edge_height() as i16;
        let bar_h = self.height.saturating_sub(edge as u16);
        let gap = antibox_ui::metrics::pad() as i16;
        let tray_gap = gap;

        let idx = |this: &Self, which: PanelSlot| -> Option<usize> {
            this.applets.iter().position(|a| which.matches(a.as_ref()))
        };
        let task_idx = idx(self, PanelSlot::Task);
        let (left_slots, right_slots) =
            effective_slots(crate::layout_preferences::taskbar_layout());

        let vmargin = antibox_core::scale::scaled(3) as i16;
        let mut left_x = vmargin + tray_gap;
        for slot in left_slots {
            if let Some(i) = idx(self, slot) {
                let w = self.applets[i].preferred_width() as u16;
                let (ay, ah) = (edge, bar_h);
                if w == 0 {
                    self.applets[i].set_geometry(left_x, ay, 1, ah);
                    let _ = self.applets[i].window().unmap();
                    continue;
                }
                self.applets[i].set_geometry(left_x, ay, w, ah);
                self.geom_cache
                    .borrow_mut()
                    .insert(self.applets[i].window().id(), (w, ah));
                let _ = self.applets[i].window().map();
                left_x += w as i16 + gap;
            }
        }

        let vin = antibox_ui::metrics::button_inset() as i16;
        let inset_h = (bar_h as i16 - 2 * vin).max(8) as u16;
        let inset_y = edge + (bar_h as i16 - inset_h as i16) / 2;

        let mut right_total = 0i16;
        for &slot in &right_slots {
            if let Some(i) = idx(self, slot) {
                let w = self.applets[i].preferred_width() as i16;
                if w > 0 {
                    right_total += w + tray_gap;
                }
            }
        }
        let right_start = (bar_w - vmargin - right_total).max(left_x);
        self.tray_x
            .set(if right_total > 0 { right_start } else { -1 });
        let mut rx = right_start;
        for &slot in &right_slots {
            if let Some(i) = idx(self, slot) {
                let w = self.applets[i].preferred_width() as u16;
                if w == 0 {
                    self.applets[i].set_geometry(rx, inset_y, 1, inset_h);
                    let _ = self.applets[i].window().unmap();
                    continue;
                }
                self.applets[i].set_geometry(rx, inset_y, w, inset_h);
                self.geom_cache
                    .borrow_mut()
                    .insert(self.applets[i].window().id(), (w, inset_h));
                let _ = self.applets[i].window().map();
                rx += w as i16 + tray_gap;
            }
        }

        if let Some(i) = task_idx {
            let tew = antibox_ui::theme::tray_edge_width() as i16;
            let w = (right_start - tew - left_x - gap).max(1) as u16;
            self.applets[i].set_geometry(left_x, edge, w, bar_h);
            self.geom_cache
                .borrow_mut()
                .insert(self.applets[i].window().id(), (w, bar_h));
            let _ = self.applets[i].window().map();
        }
    }
    fn add_applet(&mut self, applet: Box<dyn Applet>) {
        self.apply_background(applet.window().id());
        self.applets.push(applet);
        self.relayout();
    }
    fn remove_applet(&mut self, idx: usize) {
        if idx < self.applets.len() {
            self.applets.remove(idx);
            self.relayout();
        }
    }
}

fn guard_applet<R, F: FnOnce() -> R>(what: &str, f: F) -> Option<R> {
    if let Ok(r) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Some(r)
    } else {
        eprintln!("applet {what} panicked; ignoring this cycle");
        None
    }
}

#[cfg(test)]
#[path = "taskbar_tests.rs"]
mod tests;
