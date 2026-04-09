use crate::expose_layout::{self, Cell};
use crate::id::ClientId;
use crate::manager::WindowManager;
use crate::wmconfig::ExposePrefs;
use antibox_core::backend::*;
use antibox_core::point::Point;
use antibox_core::rect::Rect;
use antibox_ui::theme;
use std::sync::Arc;

const KEY_ESCAPE: u32 = 0xFF1B;
const KEY_RETURN: u32 = 0xFF0D;
const KEY_KP_ENTER: u32 = 0xFF8D;
const KEY_SPACE: u32 = 0x0020;
const KEY_TAB: u32 = 0xFF09;
const KEY_LEFT: u32 = 0xFF51;
const KEY_UP: u32 = 0xFF52;
const KEY_RIGHT: u32 = 0xFF53;
const KEY_DOWN: u32 = 0xFF54;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExposeAction {
    None,
    Repaint,
    Cancel,
    Activate(ClientId),
    Client(ClientOp, ClientId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientOp {
    None,
    Focus,
    Iconify,
    Shade,
    Close,
    Kill,
}

impl ClientOp {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "no" | "none" => Some(Self::None),
            "focus" => Some(Self::Focus),
            "iconify" | "minimize" => Some(Self::Iconify),
            "shade" | "rollup" => Some(Self::Shade),
            "close" => Some(Self::Close),
            "kill" | "destroy" => Some(Self::Kill),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispMode {
    Thumbnail,
    ThumbnailIcon,
    Icon,
    Filled,
    None,
}

impl DispMode {
    fn from_word(word: &str) -> Option<Self> {
        match word.to_ascii_lowercase().as_str() {
            "thumbnail" => Some(Self::Thumbnail),
            "thumbnail-icon" => Some(Self::ThumbnailIcon),
            "icon" => Some(Self::Icon),
            "filled" => Some(Self::Filled),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    fn words(text: &str) -> Vec<Self> {
        text.split_whitespace().filter_map(Self::from_word).collect()
    }

    pub fn parse_list(text: &str) -> Vec<Self> {
        let modes = Self::words(text);
        if modes.is_empty() {
            Self::words(crate::wmconfig::DEFAULT_EXPOSE_MODES)
        } else {
            modes
        }
    }
}

pub fn pick_mode(modes: &[DispMode], thumb: bool, icon: bool) -> DispMode {
    for mode in modes {
        match mode {
            DispMode::ThumbnailIcon if thumb && icon => return *mode,
            DispMode::Thumbnail if thumb => return *mode,
            DispMode::Icon if icon => return *mode,
            DispMode::Filled | DispMode::None => return *mode,
            _ => {}
        }
    }
    DispMode::None
}

enum Art {
    Thumb(Vec<u8>),
    ThumbIcon(Vec<u8>, PixmapData),
    Icon(PixmapData),
    Filled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileStyle {
    pub tint: u32,
    pub tint_opacity: u8,
    pub opacity: u8,
}

impl TileStyle {
    pub const fn is_neutral(&self) -> bool {
        self.tint_opacity == 0 && self.opacity == 255
    }

    pub fn of(prefs: &ExposePrefs, selected: bool) -> Self {
        let colour = |text: &str| crate::wmconfig::parse_colour(text).unwrap_or(0);
        if selected {
            Self {
                tint: colour(&prefs.highlight_tint),
                tint_opacity: prefs.highlight_tint_opacity,
                opacity: prefs.highlight_opacity,
            }
        } else {
            Self {
                tint: colour(&prefs.tint),
                tint_opacity: prefs.tint_opacity,
                opacity: prefs.opacity,
            }
        }
    }

    pub fn apply(&self, colour: u32, under: u32) -> u32 {
        let tinted =
            antibox_core::colour::lerp(colour, self.tint, f32::from(self.tint_opacity) / 255.0);
        antibox_core::colour::lerp(under, tinted, f32::from(self.opacity) / 255.0)
    }
}

pub fn blend_pixels(src: &[u8], style: TileStyle, under: u32) -> Option<Vec<u8>> {
    if style.is_neutral() {
        return None;
    }
    let mut out = src.to_vec();
    for px in out.chunks_exact_mut(4) {
        let colour = (u32::from(px[2]) << 16) | (u32::from(px[1]) << 8) | u32::from(px[0]);
        let mixed = style.apply(colour, under);
        px[0] = (mixed & 0xff) as u8;
        px[1] = ((mixed >> 8) & 0xff) as u8;
        px[2] = ((mixed >> 16) & 0xff) as u8;
    }
    Some(out)
}

pub struct Built {
    pub tiles: Vec<Tile>,
    pub missing_live: usize,
}

struct Candidate {
    client: ClientId,
    title: String,
    rect: Rect,
    mode: DispMode,
    live: Option<(u16, u16, Vec<u8>)>,
}

pub struct Tile {
    pub client: ClientId,
    pub title: String,
    pub cell: Cell,
    art: Art,
    dim: Option<Vec<u8>>,
}

pub struct Expose {
    window: Option<Box<dyn WindowHandle>>,
    pub visible: bool,
    tiles: Vec<Tile>,
    selected: usize,
    area: Rect,
    shot: Option<(u16, u16, Vec<u8>)>,
    pointer: Point,
    warp_to: Option<Point>,
    awaiting_capture: bool,
    prefs: ExposePrefs,
}

impl Default for Expose {
    fn default() -> Self {
        Self::new()
    }
}

fn gap(prefs: &ExposePrefs) -> i32 {
    antibox_core::scale::scaled(prefs.distance)
}

pub fn monitor_at(monitors: &[MonitorInfo], at: Point) -> Option<Rect> {
    monitors
        .iter()
        .map(|m| Rect::new(m.x as i32, m.y as i32, m.width as i32, m.height as i32))
        .find(|r| r.contains_xy(at.x, at.y))
}

pub const fn on_monitor(rect: Rect, area: Rect) -> bool {
    let cx = rect.x + rect.w / 2;
    let cy = rect.y + rect.h / 2;
    area.contains_xy(cx, cy)
}

fn backdrop() -> u32 {
    antibox_core::colour::scale(theme::face(), 0.3)
}

fn margin() -> i32 {
    antibox_core::scale::scaled(32)
}

fn icon_px(prefs: &ExposePrefs) -> u16 {
    antibox_core::scale::scaled(i32::from(prefs.icon_size)) as u16
}

pub fn crop_scale(
    src: &[u8],
    src_w: u16,
    src_h: u16,
    area: Rect,
    dst_w: u16,
    dst_h: u16,
) -> Option<Vec<u8>> {
    if dst_w == 0 || dst_h == 0 || src_w == 0 || src_h == 0 {
        return None;
    }
    let sx0 = area.x.clamp(0, src_w as i32);
    let sy0 = area.y.clamp(0, src_h as i32);
    let sx1 = (area.x + area.w).clamp(0, src_w as i32);
    let sy1 = (area.y + area.h).clamp(0, src_h as i32);
    let (aw, ah) = (sx1 - sx0, sy1 - sy0);
    if aw <= 0 || ah <= 0 {
        return None;
    }
    if src.len() < src_w as usize * src_h as usize * 4 {
        return None;
    }
    let mut out = vec![0u8; dst_w as usize * dst_h as usize * 4];
    for ty in 0..dst_h as i32 {
        let y0 = sy0 + ty * ah / dst_h as i32;
        let y1 = (sy0 + (ty + 1) * ah / dst_h as i32).max(y0 + 1).min(sy1);
        for tx in 0..dst_w as i32 {
            let x0 = sx0 + tx * aw / dst_w as i32;
            let x1 = (sx0 + (tx + 1) * aw / dst_w as i32).max(x0 + 1).min(sx1);
            let (mut b, mut g, mut r, mut n) = (0u32, 0u32, 0u32, 0u32);
            for y in y0..y1 {
                let row = y as usize * src_w as usize;
                for x in x0..x1 {
                    let i = (row + x as usize) * 4;
                    b += u32::from(src[i]);
                    g += u32::from(src[i + 1]);
                    r += u32::from(src[i + 2]);
                    n += 1;
                }
            }
            if n == 0 {
                continue;
            }
            let o = (ty as usize * dst_w as usize + tx as usize) * 4;
            out[o] = (b / n) as u8;
            out[o + 1] = (g / n) as u8;
            out[o + 2] = (r / n) as u8;
            out[o + 3] = 0xff;
        }
    }
    Some(out)
}

impl Expose {
    pub fn new() -> Self {
        Self {
            window: None,
            visible: false,
            tiles: Vec::new(),
            selected: 0,
            area: Rect::ZERO,
            shot: None,
            pointer: Point::ZERO,
            warp_to: None,
            awaiting_capture: false,
            prefs: ExposePrefs::default(),
        }
    }

    pub fn set_prefs(&mut self, prefs: ExposePrefs) {
        self.prefs = prefs;
    }

    pub const fn awaiting_capture(&self) -> bool {
        self.visible && self.awaiting_capture
    }

    pub const fn prefs(&self) -> &ExposePrefs {
        &self.prefs
    }

    fn aim_at_selected(&mut self) {
        if let Some(tile) = self.tiles.get(self.selected) {
            let (x, y) = tile.cell.centre();
            self.warp_to = Some(Point::new(x, y));
        }
    }

    pub fn pump(&mut self, conn: &Arc<dyn DisplayBackend>) {
        let Some(p) = self.warp_to.take() else { return };
        self.pointer = p;
        if let Some(win) = &self.window {
            let _ = conn.warp_pointer(0, win.id(), Rect::ZERO, p);
            let _ = conn.flush();
        }
    }

    pub fn owns_window(&self, window: u32) -> bool {
        self.visible && self.window.as_ref().is_some_and(|w| w.id() == window)
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub const fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_client(&self) -> Option<ClientId> {
        self.tiles.get(self.selected).map(|t| t.client)
    }

    pub fn show(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &WindowManager<dyn DisplayBackend>,
    ) -> bool {
        if self.visible {
            return false;
        }
        let screen = Rect::new(0, 0, conn.screen_width() as i32, conn.screen_height() as i32);
        self.area = if self.prefs.all_monitors {
            screen
        } else {
            let at = conn
                .query_pointer(conn.root().read_id())
                .map_or(Point::ZERO, |p| Point::new(i32::from(p.root_x), i32::from(p.root_y)));
            monitor_at(&wm.monitors, at).unwrap_or(screen)
        };
        self.shot = conn.grab_root_window().ok();
        let built = Self::build_tiles(wm, self.area, self.shot.as_ref(), &self.prefs);
        self.awaiting_capture = built.missing_live > 0;
        self.tiles = built.tiles;
        if self.tiles.is_empty() {
            return false;
        }
        self.selected = wm
            .focused_window()
            .and_then(|id| self.tiles.iter().position(|t| t.client == id))
            .unwrap_or(0);
        let Ok(window) = conn.create_window(
            conn.root().as_parent(),
            self.area,
            WmWindowClass::InputOutput,
            true,
            EventMask::BUTTON_PRESS
                | EventMask::POINTER_MOTION
                | EventMask::KEY_PRESS
                | EventMask::EXPOSURE,
        ) else {
            self.tiles.clear();
            return false;
        };
        let _ = window.set_title("Expose");
        let _ = window.set_class("antibox-expose", "Antibox");
        let _ = window.map();
        let _ = window.raise();
        let _ = conn.grab_keyboard(
            false,
            conn.root().read_id(),
            0,
            GrabMode::Async,
            GrabMode::Async,
        );
        self.window = Some(window);
        self.visible = true;
        if self.prefs.warp_on_show {
            self.aim_at_selected();
            self.pump(conn);
        }
        self.paint(conn);
        let _ = conn.flush();
        true
    }

    fn build_tiles(
        wm: &WindowManager<dyn DisplayBackend>,
        area: Rect,
        shot: Option<&(u16, u16, Vec<u8>)>,
        prefs: &ExposePrefs,
    ) -> Built {
        let ws = wm.active_workspace();
        let modes = DispMode::parse_list(&prefs.modes);
        let mut wanted: Vec<Candidate> = wm
            .frames
            .values()
            .filter(|f| prefs.all_desktops || f.workspace() == ws)
            .filter(|f| prefs.show_skip_taskbar || !f.state().skip_taskbar)
            .filter(|f| prefs.show_minimized || !f.state().minimized)
            .filter(|f| prefs.all_monitors || on_monitor(f.frame_rect(), area))
            .map(|f| {
                let rect = if prefs.include_frame { f.frame_rect() } else { f.client_rect() };
                let live = if prefs.live_thumbnails && !f.state().minimized {
                    wm.backend().and_then(|b| {
                        let client = || b.capture_window(f.client().xid()).ok();
                        if prefs.include_frame {
                            b.capture_window(f.frame().id()).ok().or_else(client)
                        } else {
                            client()
                        }
                    })
                } else {
                    None
                };
                let thumb = live.is_some() || (shot.is_some() && !f.state().minimized);
                let mode = pick_mode(&modes, thumb, !f.client().icons().is_empty());
                Candidate {
                    client: f.client_id(),
                    title: f.client().title().to_string(),
                    rect,
                    mode,
                    live,
                }
            })
            .filter(|c| c.mode != DispMode::None)
            .collect();
        wanted.sort_by(|a, b| (a.rect.x, a.rect.y).cmp(&(b.rect.x, b.rect.y)));
        if wanted.is_empty() {
            return Built { tiles: Vec::new(), missing_live: 0 };
        }
        let missing_live = if prefs.live_thumbnails {
            wanted.iter().filter(|c| c.live.is_none()).count()
        } else {
            0
        };
        let sizes: Vec<(i32, i32)> = wanted
            .iter()
            .map(|c| (c.rect.w.max(1), c.rect.h.max(1)))
            .collect();
        let cells = expose_layout::layout(
            &sizes,
            (area.w, area.h),
            gap(prefs),
            margin(),
            prefs.allow_upscale,
        );
        let tiles = wanted
            .into_iter()
            .zip(cells)
            .map(|(cand, cell)| {
                let art = Self::art_for(wm, &cand, cell, shot, prefs);
                let dim = match &art {
                    Art::Thumb(px) | Art::ThumbIcon(px, _) => {
                        blend_pixels(px, TileStyle::of(prefs, false), backdrop())
                    }
                    _ => None,
                };
                Tile {
                    client: cand.client,
                    title: cand.title,
                    cell,
                    art,
                    dim,
                }
            })
            .collect();
        Built { tiles, missing_live }
    }

    fn icon_of(
        wm: &WindowManager<dyn DisplayBackend>,
        client: ClientId,
        prefs: &ExposePrefs,
    ) -> PixmapData {
        let icons = wm
            .frames
            .values()
            .find(|f| f.client_id() == client)
            .map_or(&[][..], |f| f.client().icons());
        crate::icon_render::resolve_client_icon(icons, icon_px(prefs), theme::field())
    }

    fn art_for(
        wm: &WindowManager<dyn DisplayBackend>,
        cand: &Candidate,
        cell: Cell,
        shot: Option<&(u16, u16, Vec<u8>)>,
        prefs: &ExposePrefs,
    ) -> Art {
        let thumb = || {
            let (dw, dh) = (cell.w as u16, cell.h as u16);
            if let Some((cw, ch, data)) = &cand.live {
                let whole = Rect::new(0, 0, i32::from(*cw), i32::from(*ch));
                if let Some(px) = crop_scale(data, *cw, *ch, whole, dw, dh) {
                    return Some(px);
                }
            }
            shot.and_then(|(sw, sh, data)| crop_scale(data, *sw, *sh, cand.rect, dw, dh))
        };
        let icon = || Self::icon_of(wm, cand.client, prefs);
        match cand.mode {
            DispMode::ThumbnailIcon => match thumb() {
                Some(px) => Art::ThumbIcon(px, icon()),
                None => Art::Icon(icon()),
            },
            DispMode::Thumbnail => match thumb() {
                Some(px) => Art::Thumb(px),
                None => Art::Filled,
            },
            DispMode::Icon => Art::Icon(icon()),
            DispMode::Filled | DispMode::None => Art::Filled,
        }
    }

    pub fn rebuild(
        &mut self,
        conn: &Arc<dyn DisplayBackend>,
        wm: &WindowManager<dyn DisplayBackend>,
    ) {
        if !self.visible {
            return;
        }
        let was = self.selected_client();
        let built = Self::build_tiles(wm, self.area, self.shot.as_ref(), &self.prefs);
        self.awaiting_capture = built.missing_live > 0;
        self.tiles = built.tiles;
        if self.tiles.is_empty() {
            self.hide(conn);
            return;
        }
        self.selected = was
            .and_then(|id| self.tiles.iter().position(|t| t.client == id))
            .unwrap_or_else(|| self.selected.min(self.tiles.len() - 1));
        self.paint(conn);
        let _ = conn.flush();
    }

    pub fn hide(&mut self, conn: &Arc<dyn DisplayBackend>) {
        if let Some(win) = self.window.take() {
            let _ = win.unmap();
            let _ = win.destroy();
        }
        if self.visible {
            let _ = conn.ungrab_keyboard(0);
        }
        self.visible = false;
        self.tiles.clear();
        self.shot = None;
        self.warp_to = None;
        self.awaiting_capture = false;
        self.selected = 0;
        let _ = conn.flush();
    }

    pub fn paint(&self, conn: &Arc<dyn DisplayBackend>) {
        let Some(win) = &self.window else { return };
        let (w, h) = (self.area.w as u16, self.area.h as u16);
        let depth = conn.screen_depth();
        let Ok(pm) = conn.create_pixmap(w, h, depth) else { return };
        if let Ok(g) = conn.create_graphics(pm) {
            self.draw(&*g, w, h);
        }
        if let Ok(wg) = conn.create_graphics(win.id()) {
            let _ = wg.copy_from(pm, Rect::px(0, 0, w, h), Point::ZERO);
        }
        let _ = conn.free_pixmap(pm);
        let _ = conn.flush();
    }

    fn draw(&self, g: &dyn GraphicsContext, w: u16, h: u16) {
        let _ = g.set_foreground(backdrop());
        let _ = g.fill_rect(0, 0, w, h);
        for (i, tile) in self.tiles.iter().enumerate() {
            self.draw_tile(g, tile, i == self.selected);
        }
        if let Some(tile) = self.tiles.get(self.selected) {
            self.draw_title(g, tile, w);
        }
    }

    fn draw_tile(&self, g: &dyn GraphicsContext, tile: &Tile, selected: bool) {
        let c = tile.cell;
        let (x, y, cw, ch) = (c.x as i16, c.y as i16, c.w as u16, c.h as u16);
        let style = TileStyle::of(&self.prefs, selected);
        let under = backdrop();
        let _ = g.set_foreground(style.apply(theme::field(), under));
        let _ = g.fill_rect(x, y, cw, ch);
        let centred = |pm: &PixmapData| {
            let ix = x + (cw as i16 - pm.width as i16) / 2;
            let iy = y + (ch as i16 - pm.height as i16) / 2;
            let _ = g.draw_pixmap(ix, iy, pm);
        };
        let raw = match &tile.art {
            Art::Thumb(data) | Art::ThumbIcon(data, _) => Some(data),
            _ => None,
        };
        let highlighted = match raw {
            Some(data) if selected => blend_pixels(data, style, under),
            _ => None,
        };
        if let Some(data) = raw {
            let shown = highlighted.as_deref().unwrap_or_else(|| {
                if selected {
                    data
                } else {
                    tile.dim.as_deref().unwrap_or(data)
                }
            });
            let _ = g.draw_image(x, y, cw, ch, shown);
        }
        match &tile.art {
            Art::ThumbIcon(_, pm) | Art::Icon(pm) => centred(pm),
            Art::Thumb(_) | Art::Filled => {}
        }
        let border = if selected {
            theme::title_active()
        } else {
            theme::shadow()
        };
        let _ = g.set_foreground(border);
        let thick = if selected { 3i16 } else { 1i16 };
        for i in 0..thick {
            let _ = g.draw_rect(
                x - i,
                y - i,
                (cw as i16 + i * 2) as u16,
                (ch as i16 + i * 2) as u16,
            );
        }
    }

    fn draw_title(&self, g: &dyn GraphicsContext, tile: &Tile, screen_w: u16) {
        if tile.title.is_empty() || !self.prefs.tooltip {
            return;
        }
        let pad = antibox_ui::metrics::pad() as i16;
        let fh = antibox_ui::metrics::font_px() as i16;
        let bar_h = (fh + pad * 2) as u16;
        let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
        let max_w = (tile.cell.w.max(antibox_core::scale::scaled(160)) as u16).min(screen_w);
        let label =
            antibox_ui::widget::fit_label(g, &tile.title, max_w.saturating_sub(pad as u16 * 2));
        let tw = g
            .text_width(&label)
            .unwrap_or_else(|_| u32::from(max_w))
            .min(u32::from(max_w)) as i16;
        let box_w = (tw + pad * 2) as u16;
        let (bx, by) = self.tip_origin(tile, box_w, bar_h, screen_w);
        let _ = g.set_foreground(theme::menu_bg());
        let _ = g.fill_rect(bx, by, box_w, bar_h);
        let _ = g.set_foreground(theme::shadow());
        let _ = g.draw_rect(bx, by, box_w, bar_h);
        let _ = g.set_foreground(theme::text());
        let baseline = antibox_ui::metrics::baseline(by as i32, bar_h as i32) as i16;
        let _ = g.draw_text_transparent(bx + pad, baseline, &label);
    }

    fn tip_origin(&self, tile: &Tile, box_w: u16, box_h: u16, screen_w: u16) -> (i16, i16) {
        let pad = antibox_ui::metrics::pad();
        let (x, y) = if self.prefs.tooltip_follows_mouse {
            let anchor = self.pointer.x + antibox_core::scale::scaled(self.prefs.tooltip_offset_x);
            let x = match self.prefs.tooltip_align.as_str() {
                "mid" => anchor - i32::from(box_w) / 2,
                "right" => anchor - i32::from(box_w),
                _ => anchor,
            };
            (
                x,
                self.pointer.y + antibox_core::scale::scaled(self.prefs.tooltip_offset_y),
            )
        } else {
            (
                tile.cell.x + (tile.cell.w - i32::from(box_w)) / 2,
                tile.cell.y + tile.cell.h + pad,
            )
        };
        let max_x = i32::from(screen_w) - i32::from(box_w);
        let max_y = self.area.h - i32::from(box_h);
        (
            x.clamp(0, max_x.max(0)) as i16,
            y.clamp(0, max_y.max(0)) as i16,
        )
    }

    fn cells(&self) -> Vec<Cell> {
        self.tiles.iter().map(|t| t.cell).collect()
    }

    fn select(&mut self, idx: usize) -> ExposeAction {
        if idx == self.selected || idx >= self.tiles.len() {
            return ExposeAction::None;
        }
        self.selected = idx;
        ExposeAction::Repaint
    }

    pub fn handle_key(&mut self, keysym: u32, shift: bool) -> ExposeAction {
        if self.tiles.is_empty() {
            return ExposeAction::Cancel;
        }
        let step = |s: &mut Self, dx: i32, dy: i32| {
            let next = expose_layout::nearest(&s.cells(), s.selected, dx, dy);
            let moved = s.select(next);
            if moved == ExposeAction::Repaint && s.prefs.warp_on_select {
                s.aim_at_selected();
            }
            moved
        };
        match keysym {
            KEY_ESCAPE | 0x0071 => ExposeAction::Cancel,
            KEY_RETURN | KEY_KP_ENTER | KEY_SPACE => self
                .selected_client()
                .map_or(ExposeAction::Cancel, ExposeAction::Activate),
            KEY_LEFT | 0x0068 => step(self, -1, 0),
            KEY_RIGHT | 0x006c => step(self, 1, 0),
            KEY_UP | 0x006b => step(self, 0, -1),
            KEY_DOWN | 0x006a => step(self, 0, 1),
            KEY_TAB => {
                let len = self.tiles.len();
                let next = if shift {
                    (self.selected + len - 1) % len
                } else {
                    (self.selected + 1) % len
                };
                let moved = self.select(next);
                if moved == ExposeAction::Repaint && self.prefs.warp_on_select {
                    self.aim_at_selected();
                }
                moved
            }
            _ => ExposeAction::None,
        }
    }

    pub fn handle_motion(&mut self, p: Point) -> ExposeAction {
        self.pointer = p;
        let moved = match expose_layout::cell_at(&self.cells(), p.x, p.y) {
            Some(idx) => self.select(idx),
            None => ExposeAction::None,
        };
        if moved == ExposeAction::None && self.prefs.tooltip && self.prefs.tooltip_follows_mouse {
            return ExposeAction::Repaint;
        }
        moved
    }

    pub fn mouse_op(&self, button: u8) -> ClientOp {
        let bound = match button {
            1 => &self.prefs.mouse1,
            2 => &self.prefs.mouse2,
            3 => &self.prefs.mouse3,
            _ => return ClientOp::None,
        };
        ClientOp::parse(bound).unwrap_or(ClientOp::None)
    }

    pub fn handle_click(&mut self, p: Point, button: u8) -> ExposeAction {
        let Some(idx) = expose_layout::cell_at(&self.cells(), p.x, p.y) else {
            return ExposeAction::Cancel;
        };
        self.selected = idx;
        let Some(client) = self.selected_client() else {
            return ExposeAction::Cancel;
        };
        match self.mouse_op(button) {
            ClientOp::None => ExposeAction::None,
            ClientOp::Focus => ExposeAction::Activate(client),
            op => ExposeAction::Client(op, client),
        }
    }
}

#[cfg(test)]
#[path = "expose_tests.rs"]
mod tests;
