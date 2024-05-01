 use antibox_core::error::Result;
use crate::client::ClientWindow;
use crate::cursors::idx as cursor_idx;
use crate::id::{ClientId, FrameId};
use crate::wmstate::{ResizeEdge, WinLayer, WindowState};
use antibox_core::backend::{
    AtomManager, ButtonGrabSpec, DisplayBackend, EventMask, GrabMode, GraphicsContext, ShapeOp,
    WindowHandle, WmWindowClass,
};
use antibox_core::rect::Rect;

const CORNER_LEN_BASE: i32 = 24;

static TABS_ON_BOTTOM: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

pub fn tabs_on_bottom() -> bool {
    TABS_ON_BOTTOM.load(core::sync::atomic::Ordering::Relaxed)
}

pub fn set_tabs_on_bottom(v: bool) {
    TABS_ON_BOTTOM.store(v, core::sync::atomic::Ordering::Relaxed);
}

pub fn title_bar_height() -> i32 {
    let base = crate::layout_preferences::title_height_override()
        .unwrap_or_else(antibox_ui::theme::title_height_base);
    antibox_core::scale::scaled(base as i32)
        .max(antibox_ui::metrics::font_px() + antibox_core::scale::scaled(4))
}
pub fn border_width() -> i32 {
    antibox_core::scale::scaled(antibox_ui::theme::border_base() as i32)
}
pub fn bottom_border_width() -> i32 {
    antibox_core::scale::scaled(antibox_ui::theme::border_bottom_base() as i32)
}
pub fn corner_len() -> i32 {
    antibox_core::scale::scaled(CORNER_LEN_BASE)
}
pub fn title_top_inset() -> i32 {
    antibox_core::scale::scaled(antibox_ui::theme::title_inset_base() as i32)
}

pub fn title_side_inset() -> i32 {
    let ti = title_top_inset();
    if ti > 0 {
        return ti;
    }
    let ov = antibox_core::scale::scaled(antibox_ui::theme::title_overlap_base() as i32);
    (border_width() - ov).max(0)
}

pub fn client_insets_free(decorated: bool) -> [i32; 4] {
    if !decorated {
        return [0; 4];
    }
    let bw = border_width();
    let bb = bottom_border_width();
    if !antibox_ui::theme::title_offset_side() {
        return [bw, title_block_height(), bw, bb];
    }
    let band = title_bar_height();
    if antibox_ui::theme::title_on_left() {
        [bw + band, bw, bw, bb]
    } else if antibox_ui::theme::title_on_right() {
        [bw, bw, bw + band, bb]
    } else {
        [bw, bw, bw, bb + band]
    }
}

pub fn top_for(border: i32, title: i32) -> i32 {
    if antibox_ui::theme::title_offset_side() {
        return border;
    }
    if title == 0 {
        border
    } else if border == 0 {
        title
    } else {
        title_block_height()
    }
}

pub fn title_block_height() -> i32 {
    if antibox_ui::theme::title_offset_side() {
        return border_width();
    }
    let ti = title_top_inset();
    if ti > 0 {
        ti + title_bar_height()
    } else {
        let ov = antibox_core::scale::scaled(antibox_ui::theme::title_overlap_base() as i32);
        border_width() + title_bar_height() - ov
    }
}

pub fn title_button_h() -> i32 {
    if antibox_ui::theme::title_overlap_base() > 0 {
        return title_bar_height();
    }
    let inset = antibox_core::scale::scaled(antibox_ui::theme::title_button_inset_base() as i32);
    (title_bar_height() - inset).max(antibox_core::scale::scaled(9))
}

pub fn button_width() -> i32 {
    antibox_core::scale::scaled(antibox_ui::theme::button_base() as i32)
        .max(title_button_h() + antibox_core::scale::scaled(2))
}

pub type TitleButton = (u8, &'static str, &'static str, Rect);

pub struct FrameWindow {
    pub(crate) client: ClientWindow,
    pub(crate) frame: Box<dyn WindowHandle>,
    pub(crate) state: WindowState,
    pub(crate) workspace: u32,
    pub(crate) frame_rect: Rect,
    pub(crate) client_rect: Rect,
    pub(crate) first_transient: Option<ClientId>,
    pub(crate) next_transient: Option<ClientId>,
    pub(crate) prev_transient: Option<ClientId>,
    pub(crate) layer: WinLayer,
    pub(crate) pressed_button: Option<u8>,
    pub(crate) saved_rect: Option<Rect>,
    pub(crate) saved_shade_height: Option<i32>,
    pub(crate) saved_fullscreen_rect: Option<Rect>,
    pub(crate) snap_saved: Option<Rect>,
    pub(crate) snap_zone: Option<crate::snap::SnapZone>,
    pub(crate) decorated: bool,
    pub(crate) f_shape_width: i32,
    pub(crate) f_shape_height: i32,
    pub(crate) f_shape_title_y: i32,
    pub(crate) f_shape_border_x: i32,
    pub(crate) f_shape_border_y: i32,
    pub(crate) shape_known: bool,
    pub(crate) shapes_protect: bool,
    rounded: std::cell::Cell<bool>,
    pub(crate) pointer_windows: Vec<u32>,
    pub(crate) gfx: std::cell::RefCell<Option<Box<dyn GraphicsContext>>>,
    pub(crate) tabbed_clients: Vec<u32>,
    pub(crate) tab_order: Vec<u32>,
    pub(crate) tab_titles: std::cell::RefCell<std::collections::HashMap<u32, String>>,
    pub(crate) tab_hit: std::cell::RefCell<Vec<(Rect, u32)>>,
    pub(crate) tab_close_hit: std::cell::RefCell<Vec<(Rect, u32)>>,
}

impl FrameWindow {
    pub fn new(client: ClientWindow, frame: Box<dyn WindowHandle>) -> Self {
        let layer = WinLayer::default_for_window_type(client.window_type());
        Self {
            client,
            frame,
            state: WindowState::default(),
            workspace: 0,
            frame_rect: Rect::new(0, 0, 0, 0),
            client_rect: Rect::new(0, 0, 0, 0),
            first_transient: None,
            next_transient: None,
            prev_transient: None,
            layer,
            pressed_button: None,
            saved_rect: None,
            saved_shade_height: None,
            saved_fullscreen_rect: None,
            snap_saved: None,
            snap_zone: None,
            decorated: true,
            f_shape_width: 0,
            f_shape_height: 0,
            f_shape_title_y: 0,
            f_shape_border_x: 0,
            f_shape_border_y: 0,
            shape_known: false,
            shapes_protect: true,
            rounded: std::cell::Cell::new(false),
            pointer_windows: Vec::new(),
            gfx: std::cell::RefCell::new(None),
            tabbed_clients: Vec::new(),
            tab_order: Vec::new(),
            tab_titles: std::cell::RefCell::new(std::collections::HashMap::new()),
            tab_hit: std::cell::RefCell::new(Vec::new()),
            tab_close_hit: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub fn create_frame_ex<H: DisplayBackend + 'static + ?Sized>(
        backend: &H,
        client_id: u32,
        client_rect: Rect,
        decorated: bool,
        frame_bg: u32,
    ) -> Result<(Box<dyn WindowHandle>, Rect)> {
        Self::create_frame_inset(backend, client_id, client_rect, decorated, frame_bg, [0; 4])
    }

    pub fn create_frame_inset<H: DisplayBackend + 'static + ?Sized>(
        backend: &H,
        client_id: u32,
        client_rect: Rect,
        decorated: bool,
        frame_bg: u32,
        inset: [i32; 4],
    ) -> Result<(Box<dyn WindowHandle>, Rect)> {
        let [dl, dt, dr, db] = client_insets_free(decorated);
        let [il, ir, it, ib] = inset;
        let fw = (client_rect.w + dl + dr - il - ir).max(1);
        let fh = (client_rect.h + dt + db - it - ib).max(1);
        let fx = client_rect.x - dl;
        let fy = client_rect.y - dt;
        let frame = backend.create_window(
            backend.root().as_parent(),
            Rect::new(fx, fy, fw, fh),
            WmWindowClass::InputOutput,
            true,
            EventMask::EXPOSURE
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::BUTTON_MOTION
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::ENTER_WINDOW
                | EventMask::LEAVE_WINDOW,
        )?;
        backend.reparent_window(client_id, frame.id(), antibox_core::point::Point::new(dl - il, dt - it))?;

        const CFG_BACK_PIXEL: u32 = 1 << 1;
        const CFG_BACKING_STORE: u32 = 1 << 6;
        const BACKING_WHEN_MAPPED: u32 = 1;
        let _ = backend.change_window_attributes(
            frame.id(),
            &[
                CFG_BACK_PIXEL | CFG_BACKING_STORE,
                frame_bg,
                BACKING_WHEN_MAPPED,
            ],
        );

        let _ = backend.change_save_set(client_id, true);
        for button in 1..=3 {
            let _ = backend.grab_button(ButtonGrabSpec {
                button,
                modifiers: 0x8000,
                window: client_id,
                owner_events: false,
                event_mask: EventMask::BUTTON_PRESS,
                pointer_mode: GrabMode::Sync,
                keyboard_mode: GrabMode::Async,
                confine_to: 0,
                cursor: 0,
            });
        }
        frame.map()?;
        backend.map_window(client_id)?;
        Ok((frame, Rect::new(fx, fy, fw, fh)))
    }

    pub const fn decorated(&self) -> bool {
        self.decorated
    }

    pub fn effective_border(&self) -> i32 {
        if !self.decorated || self.state.maximized || self.state.fullscreen {
            0
        } else {
            border_width()
        }
    }

    pub fn effective_bottom_border(&self) -> i32 {
        if !self.decorated || self.state.maximized || self.state.fullscreen {
            0
        } else {
            bottom_border_width()
        }
    }

    pub fn effective_title(&self) -> i32 {
        if self.decorated && !self.state.fullscreen {
            title_bar_height()
        } else {
            0
        }
    }

    pub fn effective_top(&self) -> i32 {
        if self.title_offset() {
            return self.effective_border();
        }
        if self.effective_title() == 0 {
            return self.effective_border();
        }
        if self.effective_border() == 0 {
            return self.effective_title();
        }
        title_block_height()
    }

    pub fn title_offset(&self) -> bool {
        antibox_ui::theme::title_offset_side() && self.effective_title() > 0
    }

    pub fn title_on_left(&self) -> bool {
        self.title_offset() && antibox_ui::theme::title_on_left()
    }

    pub fn title_on_right(&self) -> bool {
        self.title_offset() && antibox_ui::theme::title_on_right()
    }

    pub fn title_on_bottom(&self) -> bool {
        self.title_offset() && antibox_ui::theme::title_on_bottom()
    }

    pub fn title_vertical(&self) -> bool {
        self.title_offset() && antibox_ui::theme::title_vertical()
    }

    pub fn band_thick(&self) -> i32 {
        if self.title_offset() {
            title_bar_height()
        } else {
            0
        }
    }

    pub fn tab_strip_h(&self) -> i32 {
        if self.tabbed_clients.is_empty() || self.state.fullscreen {
            0
        } else {
            title_bar_height()
        }
    }

    pub fn tab_strip_rect(&self) -> Rect {
        let strip = self.tab_strip_h();
        let bw = self.effective_border();
        let rel = if tabs_on_bottom() {
            self.client_rect.y - self.frame_rect.y + self.client_rect.h
        } else {
            self.client_rect.y - self.frame_rect.y - strip
        };
        Rect::new(bw, rel, (self.frame_rect.w - bw * 2).max(1), strip)
    }

    pub fn tab_order_synced(&self) -> Vec<u32> {
        let mut members = vec![self.client.xid()];
        members.extend(self.tabbed_clients.iter().copied());
        let mut out: Vec<u32> = self
            .tab_order
            .iter()
            .copied()
            .filter(|id| members.contains(id))
            .collect();
        let mut rest: Vec<u32> = members.into_iter().filter(|id| !out.contains(id)).collect();
        rest.sort_unstable();
        out.extend(rest);
        out
    }

    pub fn tab_display(&self) -> Vec<(u32, String)> {
        let titles = self.tab_titles.borrow();
        let active = self.client.xid();
        self.tab_order_synced()
            .into_iter()
            .map(|id| {
                let title = if id == active {
                    self.client.title().to_string()
                } else {
                    titles.get(&id).cloned().unwrap_or_default()
                };
                (id, title)
            })
            .collect()
    }

    pub fn tab_at_point(&self, x: i32, y: i32) -> Option<u32> {
        self.tab_hit
            .borrow()
            .iter()
            .find(|(r, _)| r.contains_xy(x, y))
            .map(|(_, id)| *id)
    }

    pub fn tab_close_at_point(&self, x: i32, y: i32) -> Option<u32> {
        self.tab_close_hit
            .borrow()
            .iter()
            .find(|(r, _)| r.contains_xy(x, y))
            .map(|(_, id)| *id)
    }

    pub fn sync_hidden_tab_sizes<H: DisplayBackend + 'static + ?Sized>(&self, backend: &H) {
        let cr = self.client_rect;
        let fr = self.frame_rect;
        for &xid in &self.tabbed_clients {
            let _ = backend.configure_window(
                xid,
                &[
                    (cr.x - fr.x).max(0) as u32,
                    (cr.y - fr.y).max(0) as u32,
                    cr.w.max(1) as u32,
                    cr.h.max(1) as u32,
                ],
            );
        }
    }

    pub fn client_insets(&self) -> [i32; 4] {
        let bw = self.effective_border();
        let strip = self.tab_strip_h();
        let (ts, bs) = if tabs_on_bottom() { (0, strip) } else { (strip, 0) };
        let bb = self.effective_bottom_border() + bs;
        if !self.title_offset() {
            let t = self.effective_top();
            return [bw, t + ts, bw, bb];
        }
        let band = self.band_thick();
        if self.title_on_left() {
            [bw + band, bw + ts, bw, bb]
        } else if self.title_on_right() {
            [bw, bw + ts, bw + band, bb]
        } else {
            [bw, bw + ts, bw, bb + band]
        }
    }

    pub fn band_rect(&self) -> Rect {
        let bw = self.effective_border();
        let bb = self.effective_bottom_border();
        let band = self.band_thick();
        let w = self.frame_rect.w;
        let h = self.frame_rect.h;
        if self.title_on_left() {
            Rect::new(bw, bw, band, (h - bw - bb).max(1))
        } else if self.title_on_right() {
            Rect::new(w - bw - band, bw, band, (h - bw - bb).max(1))
        } else {
            Rect::new(bw, h - bb - band, (w - bw * 2).max(1), band)
        }
    }

    pub fn title_bar_rect(&self) -> Rect {
        if self.title_offset() {
            return self.band_rect();
        }
        let bw = self.effective_border();
        let ti = if bw > 0 { title_side_inset() } else { 0 };
        Rect::new(ti, ti, self.frame_rect.w - ti * 2, title_bar_height())
    }

    pub fn sys_menu_rect(&self) -> Rect {
        let bw = self.effective_border();
        let ti = if bw > 0 { title_side_inset() } else { 0 };
        let sz = (title_bar_height() - 6).max(8);
        Rect::new(ti, ti, sz + 6, title_bar_height())
    }

    pub const fn frame_rect(&self) -> Rect {
        self.frame_rect
    }
    pub fn set_frame_rect(&mut self, r: Rect) {
        self.frame_rect = r;
        let [il, it, ir, ib] = self.client_insets();
        self.client_rect = Rect::new(
            r.x + il,
            r.y + it,
            (r.w - il - ir).max(1),
            (r.h - it - ib).max(1),
        );
        self.layout_shape();
    }
    pub const fn client_rect(&self) -> Rect {
        self.client_rect
    }
    pub const fn client(&self) -> &ClientWindow {
        &self.client
    }
    pub const fn client_id(&self) -> ClientId {
        self.client.id()
    }
    pub fn client_xid(&self) -> u32 {
        self.client.xid()
    }
    pub fn frame_id(&self) -> FrameId {
        FrameId(self.frame.id())
    }
    pub fn client_mut(&mut self) -> &mut ClientWindow {
        &mut self.client
    }
    pub fn frame(&self) -> &dyn WindowHandle {
        &*self.frame
    }
    pub const fn state(&self) -> &WindowState {
        &self.state
    }
    pub fn state_mut(&mut self) -> &mut WindowState {
        &mut self.state
    }
    pub const fn workspace(&self) -> u32 {
        self.workspace
    }
    pub fn set_workspace(&mut self, ws: u32) {
        self.workspace = ws;
    }

    pub const fn layer(&self) -> WinLayer {
        self.layer
    }
    pub fn set_layer(&mut self, layer: WinLayer) {
        self.layer = layer;
    }

    pub const fn pressed_button(&self) -> Option<u8> {
        self.pressed_button
    }
    pub fn set_pressed_button(&mut self, btn: Option<u8>) {
        self.pressed_button = btn;
    }

    pub fn sync_state_from_ewmh(&mut self, atoms: &AtomManager) {
        let client = &self.client;
        let has = |name: &str| client.has_net_state(atoms, name);
        self.state.max_vert = has("_NET_WM_STATE_MAXIMIZED_VERT");
        self.state.max_horz = has("_NET_WM_STATE_MAXIMIZED_HORZ");
        self.state.maximized = self.state.max_vert && self.state.max_horz;
        self.state.shaded = has("_NET_WM_STATE_SHADED");
        self.state.fullscreen = has("_NET_WM_STATE_FULLSCREEN");
        let net_urgent = has("_NET_WM_STATE_DEMANDS_ATTENTION");
        let hint_urgent = self.client.wm_hints().is_some_and(|h| h.urgency);
        self.state.urgent = net_urgent || hint_urgent;
        self.state.above = has("_NET_WM_STATE_ABOVE");
        self.state.below = has("_NET_WM_STATE_BELOW");
        self.state.sticky = has("_NET_WM_STATE_STICKY");
        self.state.skip_taskbar = has("_NET_WM_STATE_SKIP_TASKBAR")
            || crate::handler::hide_from_taskbar_on_map(&self.client);
        self.state.skip_pager = has("_NET_WM_STATE_SKIP_PAGER");
        let type_layer = WinLayer::default_for_window_type(self.client.window_type());
        self.layer = if self.state.fullscreen {
            WinLayer::Fullscreen
        } else {
            WinLayer::from_ewmh_state(self.state.above, self.state.below, type_layer)
        };
    }

    pub const fn transient_for(&self) -> Option<u32> {
        self.client.transient_for
    }

    pub const fn first_transient(&self) -> Option<ClientId> {
        self.first_transient
    }

    pub const fn next_transient(&self) -> Option<ClientId> {
        self.next_transient
    }

    pub const fn prev_transient(&self) -> Option<ClientId> {
        self.prev_transient
    }

    pub fn close(&mut self) {
        self.state.minimized = false;
        self.state.maximized = false;
        self.state.max_vert = false;
        self.state.max_horz = false;
    }
    pub fn maximize(&mut self) {
        let full = !self.state.maximized;
        self.state.maximized = full;
        self.state.max_vert = full;
        self.state.max_horz = full;
    }
    pub fn minimize(&mut self) {
        self.state.minimized = !self.state.minimized;
    }
    pub fn shade(&mut self) {
        self.state.shaded = !self.state.shaded;
    }

    pub fn title_button_for(&self, code: char) -> Option<(u8, &'static str, &'static str)> {
        use antibox_core::backend::hints::mwm_func;
        match code {
            'x' => Some((2, "X", "close")),
            'm' if self
                .client()
                .mwm_hints()
                .is_some_and(|h| !h.allows(mwm_func::MAXIMIZE)) =>
            {
                None
            }
            'm' if self.state.fullscreen => None,
            'm' if self.state.maximized => Some((1, "O", "restore")),
            'm' => Some((4, "D", "maximize")),
            'i' if self
                .client()
                .mwm_hints()
                .is_some_and(|h| !h.allows(mwm_func::MINIMIZE)) =>
            {
                None
            }
            'i' if !self.state.minimized => Some((5, "_", "minimize")),
            'r' if self.state.shaded => Some((3, "=", "rolldown")),
            'h' => Some((0, "0", "hide")),
            's' => Some((7, "S", "menu")),
            'p' if self.workspace() == !0 => Some((8, "P", "pinned")),
            'p' => Some((8, "P", "pin")),
            _ => None,
        }
    }

    fn build_title_layout_vertical(&self) -> (Vec<TitleButton>, (i32, i32)) {
        let btn = button_width();
        let bandr = self.band_rect();
        let band_x = bandr.x + ((bandr.w - btn) / 2).max(0);
        let gap = antibox_core::scale::scaled(2);
        let mut y = bandr.y + antibox_core::scale::scaled(3);
        let mut out = Vec::new();
        let mut seen: Vec<char> = Vec::new();
        for code in antibox_ui::theme::title_buttons().chars() {
            if code == 's' || seen.contains(&code) {
                continue;
            }
            seen.push(code);
            if let Some((id, sym, pix)) = self.title_button_for(code) {
                out.push((id, sym, pix, Rect::new(band_x, y, btn, btn)));
                y += btn + gap;
            }
        }
        let text_start = y + antibox_core::scale::scaled(3);
        let text_end = (bandr.y + bandr.h - antibox_core::scale::scaled(3)).max(text_start);
        (out, (text_start, text_end))
    }

    fn build_title_layout(&self) -> (Vec<TitleButton>, (i32, i32)) {
        if self.title_vertical() {
            return self.build_title_layout_vertical();
        }
        if self.title_on_bottom() {
            let (mut out, span) = self.build_title_layout_horizontal();
            let delta = self.band_rect().y - self.effective_border();
            for b in &mut out {
                b.3.y += delta;
            }
            return (out, span);
        }
        self.build_title_layout_horizontal()
    }

    fn build_title_layout_horizontal(&self) -> (Vec<TitleButton>, (i32, i32)) {
        let btn = button_width();
        let mut out = Vec::new();
        let supported = antibox_ui::theme::title_buttons();
        let allowed = |code: char| code == 's' || code == 'r' || supported.contains(code);
        let bw = if self.effective_border() > 0 {
            title_side_inset()
        } else {
            0
        };
        let btn_h = title_button_h();
        let flush = antibox_ui::theme::title_overlap_base() > 0;
        let close_gap =
            antibox_core::scale::scaled(antibox_ui::theme::close_gap_base().max(2) as i32);
        let pad = if flush {
            0
        } else {
            2 + antibox_core::scale::scaled(antibox_ui::theme::title_end_pad_base() as i32)
        };
        let btn_y = bw + ((title_bar_height() - btn_h) / 2).max(0);
        let mut seen: Vec<char> = Vec::new();
        let mut lx = bw + pad;
        for code in crate::layout_preferences::title_buttons_left().chars() {
            if !allowed(code) || seen.contains(&code) {
                continue;
            }
            seen.push(code);
            if let Some((id, sym, pix)) = self.title_button_for(code) {
                out.push((id, sym, pix, Rect::new(lx, btn_y, btn, btn_h)));
                lx += btn;
                if code == 'x' {
                    lx += close_gap;
                }
            }
        }
        let mut x = self.frame_rect.w - bw - pad - btn;
        let mut span_right = self.frame_rect.w - bw - pad;
        for code in crate::layout_preferences::title_buttons_right().chars() {
            if !allowed(code) || seen.contains(&code) {
                continue;
            }
            seen.push(code);
            if let Some((id, sym, pix)) = self.title_button_for(code) {
                out.push((id, sym, pix, Rect::new(x, btn_y, btn, btn_h)));
                span_right = span_right.min(x);
                x -= btn;
                if code == 'x' {
                    x -= close_gap;
                }
            }
        }

        (out, (lx + antibox_core::scale::scaled(2), span_right))
    }

    pub fn title_button_layout(&self) -> Vec<TitleButton> {
        self.build_title_layout().0
    }

    pub fn title_text_span(&self) -> (i32, i32) {
        self.build_title_layout().1
    }

    pub fn set_shape(&self) {
        let client_id = self.client.xid();
        if self.client.f_shaped && self.shapes_protect {
            let r = self.frame_rect;
            let bw = self.effective_border() as i16;
            let th = title_bar_height() as i16;
            if self.state.shaded || self.state.minimized {
                let full = [(0i16, 0i16, r.w as u16, r.h as u16)];
                let _ = self.frame.set_shape_rectangles(&full, ShapeOp::Set);
            } else {
                let mut rects: Vec<(i16, i16, u16, u16)> = Vec::new();
                let bb = self.effective_bottom_border() as i16;
                if bw > 0 {
                    rects.push((0, 0, r.w as u16, bw as u16));
                    rects.push((0, r.h as i16 - bb, r.w as u16, bb as u16));
                    let side_h = (r.h as i16 - bw - bb).max(0) as u16;
                    rects.push((0, bw, bw as u16, side_h));
                    rects.push((r.w as i16 - bw, bw, bw as u16, side_h));
                }
                let ti = if bw > 0 { title_side_inset() as i16 } else { 0 };
                rects.push((ti, ti, (r.w as i16 - ti * 2) as u16, th as u16));
                let _ = self.frame.set_shape_rectangles(&rects, ShapeOp::Set);
                let cr = self.client_rect;
                let off = ((cr.x - r.x) as i16, (cr.y - r.y) as i16);
                let _ = self.frame.combine_shape(client_id, off, ShapeOp::Union);
            }
            return;
        }
        self.apply_corner_shape();
    }

    fn apply_corner_shape(&self) {
        let r = self.frame_rect;
        let square = !self.decorated() || self.state.maximized || self.state.fullscreen;
        let region = if square {
            None
        } else {
            antibox_ui::theme::rounded_region(r.w as u16, r.h as u16)
        };
        match region {
            Some(rows) => {
                let _ = self.frame.set_shape_rectangles(&rows, ShapeOp::Set);
                self.rounded.set(true);
            }
            None => {
                if self.rounded.get() {
                    let full = [(0i16, 0i16, r.w as u16, r.h as u16)];
                    let _ = self.frame.set_shape_rectangles(&full, ShapeOp::Set);
                    self.rounded.set(false);
                }
            }
        }
    }

    pub fn create_pointer_windows<H: DisplayBackend + 'static + ?Sized>(
        &mut self,
        backend: &H,
        cursors: &[u32],
    ) {
        if !self.pointer_windows.is_empty() {
            return;
        }
        const SPECS: [(u32, usize); 8] = [
            (2, cursor_idx::SIZE_TOP),
            (4, cursor_idx::SIZE_LEFT),
            (6, cursor_idx::SIZE_RIGHT),
            (8, cursor_idx::SIZE_BOTTOM),
            (1, cursor_idx::SIZE_TOP_LEFT),
            (3, cursor_idx::SIZE_TOP_RIGHT),
            (7, cursor_idx::SIZE_BOTTOM_LEFT),
            (9, cursor_idx::SIZE_BOTTOM_RIGHT),
        ];
        let frame_id = self.frame.id();
        for &(gravity, cur_idx) in &SPECS {
            if let Ok(win) = backend.create_window(
                frame_id,
                Rect::new(0, 0, 1, 1),
                WmWindowClass::InputOnly,
                false,
                EventMask::NO_EVENT,
            ) {
                let xid = win.id();
                let _ = backend.set_win_gravity(xid, gravity);
                let cursor = cursors.get(cur_idx).copied().unwrap_or(0);
                if cursor != 0 {
                    let _ = backend.define_cursor(xid, cursor);
                }
                let _ = backend.map_window(xid);
                self.pointer_windows.push(xid);
            }
        }
        self.layout_pointer_windows(backend);
    }

    pub fn destroy_pointer_windows<H: DisplayBackend + 'static + ?Sized>(&mut self, backend: &H) {
        for &xid in &self.pointer_windows {
            let _ = backend.destroy_window(xid);
        }
        self.pointer_windows.clear();
    }

    pub fn layout_pointer_windows<H: DisplayBackend + 'static + ?Sized>(&self, backend: &H) {
        if self.pointer_windows.len() < 8 {
            return;
        }
        let bw = self.effective_border().max(1);
        let bb = self.effective_bottom_border().max(1);
        let top = 0;
        let grab = corner_len();
        let title_bottom = self.title_offset() && !self.title_on_left() && !self.title_on_right();
        let lg = if self.title_on_left() {
            grab.max(bw)
        } else {
            bw
        };
        let rg = if self.title_on_right() {
            grab.max(bw)
        } else {
            bw
        };
        let bg = if title_bottom { grab.max(bb) } else { bb };
        let w = self.frame_rect.w.max(lg + rg + 1);
        let h = self.frame_rect.h.max(top + bw + bg + 1);
        let iw = (w - lg - rg).max(1);
        let ih = (h - top - bw - bg).max(1);
        let tz = if title_top_inset() > 0 {
            title_top_inset().max(1)
        } else {
            bw
        };
        let rects: [(i32, i32, i32, i32); 8] = [
            (lg, top, iw, tz),
            (0, top + bw, lg, ih),
            (w - rg, top + bw, rg, ih),
            (lg, h - bg, iw, bg),
            (0, top, lg, tz),
            (w - rg, top, rg, tz),
            (0, h - bg, lg, bg),
            (w - rg, h - bg, rg, bg),
        ];
        for (i, &(x, y, ww, hh)) in rects.iter().enumerate() {
            let _ = backend.configure_window(
                self.pointer_windows[i],
                &[x as u32, y as u32, ww.max(1) as u32, hh.max(1) as u32],
            );
        }
    }

    pub fn layout_shape(&mut self) {
        let r = self.frame_rect;
        if self.f_shape_width != r.w
            || self.f_shape_height != r.h
            || self.f_shape_title_y != title_bar_height()
            || self.f_shape_border_x != self.effective_border()
            || self.f_shape_border_y != self.effective_border()
            || !self.shape_known
        {
            self.f_shape_width = r.w;
            self.f_shape_height = r.h;
            self.f_shape_title_y = title_bar_height();
            self.f_shape_border_x = self.effective_border();
            self.f_shape_border_y = self.effective_border();
            self.shape_known = true;
            self.set_shape();
        }
    }
}

include!("frame_hit.rs");

#[cfg(test)]
#[path = "frame_test.rs"]
mod tests;
