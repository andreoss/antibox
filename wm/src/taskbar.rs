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
    LeftVertical,
    RightVertical,
    Disabled,
}

impl TaskBarPosition {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "left-vertical" | "leftvertical" => Some(Self::LeftVertical),
            "right-vertical" | "rightvertical" => Some(Self::RightVertical),
            "disabled" | "none" | "off" => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn is_side(self) -> bool {
        matches!(
            self,
            Self::Left | Self::Right | Self::LeftVertical | Self::RightVertical
        )
    }

    pub const fn is_rotated(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    pub const fn on_right(self) -> bool {
        matches!(self, Self::Right | Self::RightVertical)
    }

    pub const fn is_upright(self) -> bool {
        matches!(self, Self::LeftVertical | Self::RightVertical)
    }

    pub const fn code(self) -> u8 {
        self as u8
    }

    pub const fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Top,
            2 => Self::Left,
            3 => Self::Right,
            4 => Self::LeftVertical,
            5 => Self::RightVertical,
            6 => Self::Disabled,
            _ => Self::Bottom,
        }
    }
}

pub fn rotate_cw(src: &[u8], w: u16, h: u16) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let s = (y * w + x) * 4;
            let d = (x * h + (h - 1 - y)) * 4;
            out[d..d + 4].copy_from_slice(&src[s..s + 4]);
        }
    }
    out
}

pub fn rotate_ccw(src: &[u8], w: u16, h: u16) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let s = (y * w + x) * 4;
            let d = ((w - 1 - x) * h + y) * 4;
            out[d..d + 4].copy_from_slice(&src[s..s + 4]);
        }
    }
    out
}

pub const fn logical_to_window(
    position: TaskBarPosition,
    rect: (i32, i32, i32, i32),
    len: i32,
    thick: i32,
) -> (i32, i32, i32, i32) {
    let (x, y, w, h) = rect;
    match position {
        TaskBarPosition::Left => (thick - y - h, x, h, w),
        TaskBarPosition::Right => (y, len - x - w, h, w),
        _ => rect,
    }
}

pub fn applet_anchor(
    window: &dyn WindowHandle,
    local: (i16, i16, u16, u16),
    logical: (u16, u16),
) -> (Rect, bool) {
    let position = crate::layout_preferences::taskbar_position();
    let rect = (
        i32::from(local.0),
        i32::from(local.1),
        i32::from(local.2),
        i32::from(local.3),
    );
    let (x, y, w, h) = logical_to_window(
        position,
        rect,
        i32::from(logical.0),
        i32::from(logical.1),
    );
    let at = window
        .translate_coords(Point::new(x, y))
        .unwrap_or_else(|_| Point::new(x, y));
    (Rect::new(at.x, at.y, w, h), position.is_side())
}

pub const fn unrotate_point(
    position: TaskBarPosition,
    p: (i32, i32),
    len: i32,
    thick: i32,
) -> (i32, i32) {
    match position {
        TaskBarPosition::Left => (p.1, thick - 1 - p.0),
        TaskBarPosition::Right => (len - 1 - p.1, p.0),
        _ => p,
    }
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
    well: std::cell::Cell<Option<(i16, i16, u16, u16)>>,
    pub(crate) resizing: std::cell::Cell<bool>,
}

impl TaskBar {
    pub fn new(
        conn: &Arc<dyn DisplayBackend>,
        position: TaskBarPosition,
        strut_atom: u32,
    ) -> Result<Self> {
        let screen_w = conn.screen_width();
        let screen_h = conn.screen_height();
        let bar_height = Self::thickness_for(position);
        let (length, place) = Self::place_on_screen(position, screen_w, screen_h, bar_height);
        let window = conn.create_window(
            conn.root().as_parent(),
            Rect::new(place.0, place.1, place.2 as i32, place.3 as i32),
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
            width: if position.is_rotated() { length } else { place.2 },
            height: if position.is_rotated() { bar_height } else { place.3 },
            gradients_enabled: false,
            task_bar_colour: crate::render::ThemeColors::default().task_bar_colour,
            menu_colours: crate::menu::MenuColors::default(),
            window_x: place.0,
            window_y: place.1,
            menu: None,
            pending_action: None,
            strut_atom,
            gfx_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            geom_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            tray_x: std::cell::Cell::new(-1),
            well: std::cell::Cell::new(None),
            resizing: std::cell::Cell::new(false),
        })
    }

    pub(crate) fn upright_min_thickness() -> u16 {
        (antibox_ui::metrics::text_w(14) + antibox_ui::metrics::pad() * 2).max(1) as u16
    }

    pub(crate) fn upright_thickness() -> u16 {
        let min = Self::upright_min_thickness();
        let want =
            antibox_core::scale::scaled(i32::from(crate::layout_preferences::taskbar_width_px()))
                as u16;
        want.max(min)
    }

    pub(crate) fn thickness_for(position: TaskBarPosition) -> u16 {
        match position {
            TaskBarPosition::LeftVertical | TaskBarPosition::RightVertical => {
                Self::upright_thickness()
            }
            _ => Self::bar_height(),
        }
    }

    pub(crate) const fn place_on_screen(
        position: TaskBarPosition,
        screen_w: u16,
        screen_h: u16,
        thickness: u16,
    ) -> (u16, (i32, i32, u16, u16)) {
        match position {
            TaskBarPosition::Bottom => (
                screen_w,
                (0, screen_h as i32 - thickness as i32, screen_w, thickness),
            ),
            TaskBarPosition::Top | TaskBarPosition::Disabled => {
                (screen_w, (0, 0, screen_w, thickness))
            }
            TaskBarPosition::Left | TaskBarPosition::LeftVertical => {
                (screen_h, (0, 0, thickness, screen_h))
            }
            TaskBarPosition::Right | TaskBarPosition::RightVertical => (
                screen_h,
                (screen_w as i32 - thickness as i32, 0, thickness, screen_h),
            ),
        }
    }

    fn bar_height_for(double: bool) -> u16 {
        let row = antibox_ui::metrics::panel_height() as u16;
        let row = if double { row * 2 } else { row };
        let rows = crate::layout_preferences::taskbar_rows();
        row * rows + antibox_ui::theme::panel_edge_height()
    }

    pub(crate) fn bar_height() -> u16 {
        Self::bar_height_for(crate::layout_preferences::taskbar_double_height())
    }

    pub fn show(&self) -> Result<()> {
        self.window.map()
    }


    pub fn edge_grab(&self, x: i32, y: i32) -> bool {
        let grab = i32::from(antibox_ui::theme::panel_edge_height())
            + antibox_core::scale::scaled(3);
        match self.position {
            TaskBarPosition::Bottom => y < grab,
            TaskBarPosition::Top => y >= i32::from(self.height) - grab,
            TaskBarPosition::LeftVertical => x >= i32::from(self.width) - grab,
            TaskBarPosition::RightVertical => x < grab,
            _ => false,
        }
    }

    pub fn drag_rows(&self, y: i32) -> i64 {
        let row = antibox_ui::metrics::panel_height().max(1);
        let span = match self.position {
            TaskBarPosition::Bottom => i32::from(self.height) - y,
            _ => y,
        };
        i64::from(((span + row / 2) / row).clamp(1, 4))
    }

    pub fn drag_width(&self, x: i32) -> i64 {
        let want = match self.position {
            TaskBarPosition::RightVertical => i32::from(self.width) - x,
            _ => x,
        };
        i64::from(want.clamp(1, 1000))
    }

    pub fn close(&mut self) {
        if let Some(menu) = self.menu.as_mut() {
            menu.hide(&*self.conn);
        }
        self.menu = None;
        for a in &mut self.applets {
            a.shutdown();
        }
        self.applets.clear();
        self.gfx_cache.borrow_mut().clear();
        self.geom_cache.borrow_mut().clear();
        let _ = self.window.unmap();
        let _ = self.window.destroy();
        let _ = self.conn.flush();
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

    pub(crate) const fn map_rect(&self, x: i16, y: i16, w: u16, h: u16) -> (i16, i16, u16, u16) {
        match self.position {
            TaskBarPosition::Left => (self.height as i16 - y - h as i16, x, h, w),
            TaskBarPosition::Right => (y, self.width as i16 - x - w as i16, h, w),
            _ => (x, y, w, h),
        }
    }

    fn place_applet(&mut self, index: usize, x: i16, y: i16, w: u16, h: u16) {
        self.applets[index].set_geometry(x, y, w, h);
        if self.position.is_rotated() {
            let (wx, wy, ww, wh) = self.map_rect(x, y, w, h);
            let _ = self.applets[index].window().configure(
                Some(i32::from(wx)),
                Some(i32::from(wy)),
                Some(ww),
                Some(wh),
            );
        }
        self.geom_cache
            .borrow_mut()
            .insert(self.applets[index].window().id(), (w, h));
    }

    pub(crate) fn applet_size(&self, window: u32) -> (u16, u16) {
        self.geom_cache
            .borrow()
            .get(&window)
            .copied()
            .unwrap_or((1, 1))
    }

    pub(crate) fn applet_point(&self, window: u32, x: i32, y: i32) -> (i32, i32) {
        if !self.position.is_rotated() {
            return (x, y);
        }
        let (w, h) = self
            .geom_cache
            .borrow()
            .get(&window)
            .copied()
            .unwrap_or((1, 1));
        unrotate_point(self.position, (x, y), i32::from(w), i32::from(h))
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
            self.blit(&mut cache, wid, pm, w, h);
        }
        let _ = self.conn.flush();
        for &(_, pm, _, _) in &blits {
            let _ = self.conn.free_pixmap(pm);
        }
        Ok(())
    }

    fn blit(
        &self,
        cache: &mut std::collections::HashMap<u32, Box<dyn GraphicsContext>>,
        wid: u32,
        pm: u32,
        w: u16,
        h: u16,
    ) {
        let turned = if self.position.is_rotated() {
            self.conn.read_pixels(pm, w, h).ok().map(|px| {
                if self.position == TaskBarPosition::Left {
                    rotate_cw(&px, w, h)
                } else {
                    rotate_ccw(&px, w, h)
                }
            })
        } else {
            None
        };
        let Some(wg) = Self::gfx(cache, &self.conn, wid) else { return };
        match turned {
            Some(px) => {
                let _ = wg.draw_image(0, 0, h, w, &px);
            }
            None => {
                let _ = wg.copy_from(pm, Rect::px(0, 0, w, h), Point::ZERO);
            }
        }
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
            self.blit(cache, wid, pm, w, h);
        }
        let _ = self.conn.free_pixmap(pm);
    }

    fn note_well(&self, i: usize, x: i16, y: i16, w: u16, h: u16) {
        let a = self.applets[i].as_ref();
        if !(PanelSlot::Tray.matches(a) || PanelSlot::Clock.matches(a)) {
            return;
        }
        let add = (x, y, w, h);
        let merged = match self.well.get() {
            None => add,
            Some((ux, uy, uw, uh)) => {
                let x0 = ux.min(add.0);
                let y0 = uy.min(add.1);
                let x1 = (i32::from(ux) + i32::from(uw)).max(i32::from(add.0) + i32::from(add.2));
                let y1 = (i32::from(uy) + i32::from(uh)).max(i32::from(add.1) + i32::from(add.3));
                (x0, y0, (x1 - i32::from(x0)) as u16, (y1 - i32::from(y0)) as u16)
            }
        };
        self.well.set(Some(merged));
    }

    fn draw_background_to(&self, g: &dyn GraphicsContext) {
        antibox_ui::theme::panel_surface(g, self.width, self.height, self.task_bar_colour);
        match self.position {
            TaskBarPosition::LeftVertical => {
                antibox_ui::theme::panel_edge_right(g, self.width, self.height);
            }
            TaskBarPosition::RightVertical => {
                antibox_ui::theme::panel_edge_left(g, self.height);
            }
            _ => antibox_ui::theme::panel_edge(g, self.width),
        }
        let tx = self.tray_x.get();
        if tx > 0 {
            let edge = antibox_ui::theme::panel_edge_height() as i16;
            let ew = antibox_ui::theme::tray_edge_width() as i16;
            antibox_ui::theme::tray_edge(g, tx - ew, edge, self.height.saturating_sub(edge as u16));
        }
        if let Some((x, y, w, h)) = self.well.get() {
            let s = antibox_core::scale::scaled(2).max(2) as i16;
            antibox_ui::theme::well(
                g,
                x - s,
                y - s,
                w + s as u16 * 2,
                h + s as u16 * 2,
            );
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
                self.blit(cache, self.window.id(), pm, self.width, self.height);
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

    pub fn group_members_at(&mut self, window: u32, x: i32, y: i32) -> Option<Vec<u32>> {
        for applet in &mut self.applets {
            if applet.window().id() != window {
                continue;
            }
            let pane = applet
                .as_any_mut()
                .downcast_mut::<crate::taskpane::TaskPane>()?;
            let members = pane.members_at(x, y)?;
            if members.len() > 1 {
                return Some(members);
            }
            return None;
        }
        None
    }

    pub fn handle_click(&mut self, window: u32, x: i32, y: i32, button: u8) -> Option<u32> {
        let (x, y) = self.applet_point(window, x, y);
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
        let (x, y) = self.applet_point(window, x, y);
        let mut id = None;
        let mut relayout = false;
        for applet in &mut self.applets {
            if applet.window().id() == window {
                id =
                    guard_applet("release", || applet.handle_release(x, y, button)).and_then(|v| v);
                if let Some(action) =
                    guard_applet("take_action", || applet.take_action()).and_then(|v| v)
                {
                    self.pending_action = Some(action);
                }
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
            .map_or_else(
                |_| Point::new(x + self.window_x, y + self.window_y),
                |p| Point::new(p.root_x as i32, p.root_y as i32),
            );
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
            .map_or_else(
                |_| Point::new(self.window_x, self.window_y),
                |p| Point::new(p.root_x as i32, p.root_y as i32),
            );
        self.open_menu(nodes, pos);
    }

    fn open_menu(&mut self, nodes: Vec<crate::menu_tree::MenuNode<Action>>, pos: Point) {
        let mut menu = crate::menu::MenuView::with_nodes(nodes);
        menu.colours = self.menu_colours;
        menu.show(self.conn.as_ref(), pos);
        self.menu = Some(menu);
    }

    pub fn update_height(&mut self) -> bool {
        if self.position.is_upright() {
            return false;
        }
        let h = Self::bar_height();
        if h == self.height {
            return false;
        }
        self.height = h;
        self.fit_to_screen();
        self.relayout();
        true
    }

    pub(crate) const fn thickness(&self) -> u16 {
        match self.position {
            TaskBarPosition::LeftVertical | TaskBarPosition::RightVertical => self.width,
            _ => self.height,
        }
    }

    pub fn strut(&self) -> Strut {
        let t = self.thickness() as u32;
        match self.position {
            TaskBarPosition::Bottom => Strut { bottom: t, ..Default::default() },
            TaskBarPosition::Top => Strut { top: t, ..Default::default() },
            TaskBarPosition::Left | TaskBarPosition::LeftVertical => {
                Strut { left: t, ..Default::default() }
            }
            TaskBarPosition::Right | TaskBarPosition::RightVertical => {
                Strut { right: t, ..Default::default() }
            }
            TaskBarPosition::Disabled => Strut::default(),
        }
    }

    pub fn update_strut(&self) {
        if self.strut_atom == 0 {
            return;
        }
        let t = self.thickness() as u32;
        let strut = match self.position {
            TaskBarPosition::Bottom => [0u32, 0, 0, t],
            TaskBarPosition::Top => [0u32, 0, t, 0],
            TaskBarPosition::Left | TaskBarPosition::LeftVertical => [t, 0, 0, 0],
            TaskBarPosition::Right | TaskBarPosition::RightVertical => [0, t, 0, 0],
            TaskBarPosition::Disabled => [0, 0, 0, 0],
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
        let (length, place) =
            Self::place_on_screen(self.position, screen_w, screen_h, self.thickness());
        self.width = if self.position.is_rotated() { length } else { place.2 };
        if !self.position.is_rotated() {
            self.height = place.3;
        }
        self.window_x = place.0;
        self.window_y = place.1;
        let _ = self.window.configure(
            Some(place.0),
            Some(place.1),
            Some(place.2),
            Some(place.3),
        );
        self.update_strut();
        self.relayout();
        let _ = self.paint();
        let _ = self.conn.flush();
    }

    pub fn set_active_workspace(&mut self, ws: u32) -> Option<u32> {
        for applet in &mut self.applets {
            if let Some(pane) = applet.as_any_mut().downcast_mut::<WorkspacesPane>() {
                let changed = pane.set_active(ws);
                return changed.then(|| applet.window().id());
            }
        }
        None
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

    pub fn update_mpris(&mut self) -> Vec<u32> {
        let mut relayout = false;
        let mut changed = Vec::new();
        for applet in &mut self.applets {
            let Some(pa) = applet
                .as_any_mut()
                .downcast_mut::<crate::power_audio_applet::PowerAudioApplet>() else { continue };
            let was_w = pa.preferred_width();
            if guard_applet("update", || pa.update_media()).unwrap_or(false) {
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
    Launch,
    Workspaces,
    Task,
    Tray,
    Cpu,
    Mem,
    Net,
    PowerAudio,
    Keyboard,
    Clock,
    ShowDesktop,
}

const DEFAULT_LEFT: [PanelSlot; 3] = [PanelSlot::Menu, PanelSlot::Launch, PanelSlot::Workspaces];
const DEFAULT_RIGHT: [PanelSlot; 8] = [
    PanelSlot::Cpu,
    PanelSlot::Mem,
    PanelSlot::Net,
    PanelSlot::Keyboard,
    PanelSlot::PowerAudio,
    PanelSlot::Tray,
    PanelSlot::Clock,
    PanelSlot::ShowDesktop,
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
            Widget::Mpris | Widget::PowerAudio => Self::PowerAudio,
            Widget::Keyboard => Self::Keyboard,
            Widget::Clock => Self::Clock,
            Widget::Launch => Self::Launch,
            Widget::ShowDesktop => Self::ShowDesktop,
        }
    }

    fn matches(self, a: &dyn Applet) -> bool {
        let any = a.as_any();
        match self {
            Self::Menu => any.is::<crate::menu_applet::MenuApplet>(),
            Self::Launch => any.is::<crate::launch_applet::LaunchApplet>(),
            Self::ShowDesktop => any.is::<crate::show_desktop_applet::ShowDesktopApplet>(),
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

pub fn shares_slot(
    a: crate::layout_preferences::Widget,
    b: crate::layout_preferences::Widget,
) -> bool {
    PanelSlot::from_widget(a) == PanelSlot::from_widget(b)
}

fn slot_wanted(slot: PanelSlot) -> bool {
    crate::layout_preferences::Widget::ALL
        .iter()
        .copied()
        .any(|w| PanelSlot::from_widget(w) == slot && crate::layout_preferences::taskbar_wants(w))
}

fn upright_cell() -> i16 {
    antibox_ui::metrics::panel_height() as i16
}

fn effective_slots(
    layout: Option<Vec<crate::layout_preferences::Widget>>,
) -> (Vec<PanelSlot>, Vec<PanelSlot>) {
    match layout {
        None => (DEFAULT_LEFT.to_vec(), DEFAULT_RIGHT.to_vec()),
        Some(widgets) => {
            let mut slots: Vec<PanelSlot> = Vec::new();
            for slot in widgets.iter().map(|w| PanelSlot::from_widget(*w)) {
                if !slots.contains(&slot) {
                    slots.push(slot);
                }
            }
            match slots.iter().position(|s| *s == PanelSlot::Task) {
                Some(p) => (slots[..p].to_vec(), slots[p + 1..].to_vec()),
                None => (slots, Vec::new()),
            }
        }
    }
}

pub fn flow_rows(sizes: &[(i16, i16)], avail: i16, gap: i16) -> (Vec<(i16, i16)>, i16) {
    let avail = avail.max(1);
    let mut spots = Vec::with_capacity(sizes.len());
    let (mut x, mut y, mut row_h) = (0i16, 0i16, 0i16);
    for &(w, h) in sizes {
        let w = w.clamp(1, avail);
        if x > 0 && x + w > avail {
            y += row_h + gap;
            x = 0;
            row_h = 0;
        }
        spots.push((x, y));
        x += w + gap;
        row_h = row_h.max(h);
    }
    (spots, y + row_h)
}

type Placed = (usize, i16, i16);
type Band = Vec<Placed>;

pub fn row_offsets(
    sizes: &[(i16, i16)],
    spots: &[(i16, i16)],
    avail: i16,
    gap: i16,
) -> std::collections::HashMap<i16, i16> {
    let mut width: std::collections::HashMap<i16, i16> = std::collections::HashMap::new();
    for ((w, _), (x, y)) in sizes.iter().zip(spots) {
        let end = x + w.clamp(&1, &avail);
        let slot = width.entry(*y).or_insert(0);
        *slot = (*slot).max(end);
    }
    let _ = gap;
    width
        .into_iter()
        .map(|(y, used)| (y, (avail - used).max(0) / 2))
        .collect()
}

pub fn fit_row(widths: &[i16], avail: i16, gap: i16, floor: i16) -> Option<Vec<i16>> {
    if widths.is_empty() {
        return Some(Vec::new());
    }
    let n = widths.len() as i16;
    let room = avail - gap * (n - 1);
    if room < floor * n {
        return None;
    }
    let want: i32 = widths.iter().map(|w| i32::from(*w).max(1)).sum();
    if want <= i32::from(room) {
        return Some(widths.iter().map(|w| (*w).max(1)).collect());
    }
    let mut out: Vec<i16> = widths
        .iter()
        .map(|w| {
            let scaled = i32::from(*w).max(1) * i32::from(room) / want;
            (scaled as i16).max(floor)
        })
        .collect();
    let mut over: i32 = out.iter().map(|w| i32::from(*w)).sum::<i32>() - i32::from(room);
    while over > 0 {
        let Some(i) = out
            .iter()
            .enumerate()
            .filter(|(_, w)| **w > floor)
            .max_by_key(|(_, w)| **w)
            .map(|(i, _)| i)
        else {
            break;
        };
        out[i] -= 1;
        over -= 1;
    }
    Some(out)
}

const fn band_of(slot: PanelSlot) -> u8 {
    match slot {
        PanelSlot::Cpu | PanelSlot::Mem | PanelSlot::Net => 1,
        PanelSlot::PowerAudio | PanelSlot::Keyboard => 2,
        PanelSlot::Tray => 3,
        PanelSlot::Clock => 4,
        _ => 0,
    }
}

fn bands_of(slots: &[PanelSlot]) -> Vec<Vec<PanelSlot>> {
    let mut bands: Vec<Vec<PanelSlot>> = Vec::new();
    for slot in slots {
        let band = band_of(*slot);
        match bands.last_mut() {
            Some(last) if band != 0 && band_of(last[0]) == band => last.push(*slot),
            _ => bands.push(vec![*slot]),
        }
    }
    bands
}

impl TaskBar {
    fn upright_group(&mut self, slots: &[PanelSlot], avail: i16) -> Vec<Placed> {
        let mut group = Vec::new();
        for slot in slots {
            let Some(i) = self.applets.iter().position(|a| slot.matches(a.as_ref())) else {
                continue;
            };
            if let Some(p) = self.applets[i]
                .as_any_mut()
                .downcast_mut::<WorkspacesPane>()
            {
                p.set_wrap_width(Some(avail.max(1) as u16));
            }
            if let Some(c) = self.applets[i].as_any_mut().downcast_mut::<ClockApplet>() {
                c.set_wrap_width(Some(avail.max(1) as u16));
            }
            let w = self.applets[i].preferred_width() as i16;
            if w <= 0 {
                self.place_applet(i, 0, 0, 1, 1);
                let _ = self.applets[i].window().unmap();
                continue;
            }
            let h = (self.applets[i].preferred_height() as i16).max(upright_cell());
            group.push((i, w.min(avail), h));
        }
        group
    }

    fn place_upright_band(
        &mut self,
        band: &[Placed],
        left: i16,
        top: i16,
        avail: i16,
        gap: i16,
    ) -> i16 {
        let widths: Vec<i16> = band.iter().map(|(_, w, _)| *w).collect();
        let floor = antibox_ui::metrics::icon() as i16;
        if let Some(fitted) = fit_row(&widths, avail, gap, floor) {
            let span: i16 = fitted.iter().sum::<i16>() + gap * (band.len() as i16 - 1).max(0);
            let mut x = left + (avail - span).max(0) / 2;
            let mut height = 0i16;
            for ((i, _, h), w) in band.iter().zip(fitted) {
                self.place_applet(*i, x, top, w as u16, *h as u16);
                self.note_well(*i, x, top, w as u16, *h as u16);
                let _ = self.applets[*i].window().map();
                x += w + gap;
                height = height.max(*h);
            }
            return height;
        }
        let sizes: Vec<(i16, i16)> = band.iter().map(|(_, w, h)| (*w, *h)).collect();
        let (spots, height) = flow_rows(&sizes, avail, gap);
        let rows = row_offsets(&sizes, &spots, avail, gap);
        for ((i, w, h), (dx, dy)) in band.iter().zip(spots) {
            let centre = rows.get(&dy).copied().unwrap_or(0);
            self.place_applet(*i, left + centre + dx, top + dy, *w as u16, *h as u16);
            self.note_well(*i, left + centre + dx, top + dy, *w as u16, *h as u16);
            let _ = self.applets[*i].window().map();
        }
        height
    }

    fn upright_bands(&mut self, slots: &[PanelSlot], avail: i16) -> Vec<Band> {
        bands_of(slots)
            .into_iter()
            .map(|band| self.upright_group(&band, avail))
            .filter(|band| !band.is_empty())
            .collect()
    }

    fn band_stack_height(bands: &[Band], gap: i16) -> i16 {
        let mut total = 0i16;
        for band in bands {
            let tall = band.iter().map(|(_, _, h)| *h).max().unwrap_or(0);
            total += tall + gap;
        }
        (total - gap).max(0)
    }

    fn relayout_upright(&mut self) {
        let mut inner_pad = 0i16;
        if !crate::layout_preferences::taskbar_locked() {
            inner_pad = antibox_core::scale::scaled(3) as i16;
        }
        let edge = antibox_ui::theme::panel_edge_height() as i16;
        let gap = (antibox_ui::metrics::gap() + antibox_core::scale::scaled(2)) as i16;
        let margin = antibox_ui::metrics::pad() as i16;
        let left = if matches!(self.position, TaskBarPosition::RightVertical) {
            margin + inner_pad
        } else {
            margin
        };
        let avail = (self.width as i16 - margin * 2 - inner_pad).max(1);
        let (top_slots, bottom_slots) =
            effective_slots(crate::layout_preferences::taskbar_layout());

        let top_bands = self.upright_bands(&top_slots, avail);
        let top_start = edge + margin;
        let mut y = top_start;
        for band in &top_bands {
            let h = self.place_upright_band(band, left, y, avail, gap);
            y += h + gap;
        }
        let top_end = (y - gap).max(top_start);

        let bottom_bands = self.upright_bands(&bottom_slots, avail);
        let stack = Self::band_stack_height(&bottom_bands, gap);
        let mut by = (self.height as i16 - margin - stack).max(top_end + gap);
        let bottom_start = by;
        for band in &bottom_bands {
            let h = self.place_upright_band(band, left, by, avail, gap);
            by += h + gap;
        }

        if let Some(i) = self.applets.iter().position(|a| PanelSlot::Task.matches(a.as_ref())) {
            let task_y = top_end + gap;
            let task_h = (bottom_start - gap - task_y).max(upright_cell());
            self.place_applet(i, left, task_y, avail as u16, task_h as u16);
            let _ = self.applets[i].window().map();
        }
    }

    pub fn sync_applets<F>(&mut self, mut make: F)
    where
        F: FnMut(crate::layout_preferences::Widget) -> Option<Box<dyn Applet>>,
    {
        for w in crate::layout_preferences::Widget::ALL.iter().copied() {
            let slot = PanelSlot::from_widget(w);
            let at = self.applets.iter().position(|a| slot.matches(a.as_ref()));
            let wanted = slot_wanted(slot);
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
        self.well.set(None);
        if matches!(
            self.position,
            TaskBarPosition::LeftVertical | TaskBarPosition::RightVertical
        ) {
            self.relayout_upright();
            return;
        }
        let bar_w = self.width as i16;
        let mut edge = antibox_ui::theme::panel_edge_height() as i16;
        if !crate::layout_preferences::taskbar_locked() {
            edge += antibox_core::scale::scaled(3) as i16;
        }
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
                    self.place_applet(i, left_x, ay, 1, ah);
                    let _ = self.applets[i].window().unmap();
                    continue;
                }
                self.place_applet(i, left_x, ay, w, ah);
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
                    self.place_applet(i, rx, inset_y, 1, inset_h);
                    let _ = self.applets[i].window().unmap();
                    continue;
                }
                self.place_applet(i, rx, inset_y, w, inset_h);
                self.note_well(i, rx, inset_y, w, inset_h);
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
            self.place_applet(i, left_x, edge, w, bar_h);
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
