use crate::applet::Applet;
use crate::applet::AppletContainer;
use crate::clock_applet::ClockApplet;
use crate::manager::WindowManager;
use crate::preview::PreviewWindow;
use crate::switcher::SwitcherWindow;
use crate::taskbar::TaskBar;
use crate::taskpane::TaskPane;
#[cfg(feature = "tray")]
use crate::tray_applet::TrayApplet;
use crate::winlist::WinListMenu;
use crate::wmconfig;
use crate::workspace_pane::WorkspacesPane;
use antibox_core::backend::*;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
enum AppletTick {
    Clock,
    Cpu,
    Mem,
    Net,
}

impl AppletTick {
    const COUNT: usize = 4;
    const ALL: [Self; AppletTick::COUNT] = [
        AppletTick::Clock,
        AppletTick::Cpu,
        AppletTick::Mem,
        AppletTick::Net,
    ];

    fn interval(self) -> Duration {
        match self {
            AppletTick::Clock => Duration::from_secs(1),
            AppletTick::Cpu | AppletTick::Net => Duration::from_secs(2),
            AppletTick::Mem => Duration::from_secs(5),
        }
    }

    fn update(self, tb: &mut TaskBar) -> Vec<u32> {
        match self {
            AppletTick::Clock => tb.update_clocks(),
            AppletTick::Cpu => tb.update_cpu(),
            AppletTick::Mem => tb.update_mem(),
            AppletTick::Net => tb.update_net(),
        }
    }
}

#[derive(Default)]
struct LoopWork {
    events: usize,
    repaint_taskbar: bool,
    repaint_applets: Vec<u32>,
}

impl LoopWork {
    fn any(&self) -> bool {
        self.events > 0 || self.repaint_taskbar || !self.repaint_applets.is_empty()
    }
}

fn apply_font_prefs(b: &Arc<dyn DisplayBackend>, prefs: &wmconfig::Prefs) {
    antibox_core::backend::set_ui_font(&prefs.font.name);
    if let Ok(g) = b.create_graphics(b.root().read_id()) {
        let _ = g.set_font(&FontSpec::ui(antibox_ui::metrics::font_pt()));
        let (_, _, fh) = g.font_metrics();
        if fh > 0 {
            let logical = fh as i32 * 96 / antibox_core::scale::dpi();
            antibox_ui::metrics::set_font_pt((logical * 3 / 4).max(6) as u16);
        }
    }
}

struct AppletTickers {
    last: [Instant; AppletTick::COUNT],
    last_clock_sec: u64,
}

impl AppletTickers {
    fn new() -> AppletTickers {
        let now = Instant::now();
        AppletTickers {
            last: [now; AppletTick::COUNT],
            last_clock_sec: Self::wall_secs(),
        }
    }

    fn wall_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }

    fn wall_to_next_second() -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(Duration::from_secs(1), |d| {
                Duration::from_secs(1) - Duration::from_nanos(d.subsec_nanos() as u64)
            })
    }

    fn tick(&mut self, tb: &mut TaskBar) -> Vec<u32> {
        let now = Instant::now();
        let mut changed = Vec::new();
        for (i, kind) in AppletTick::ALL.iter().enumerate() {
            let due = if matches!(kind, AppletTick::Clock) {
                let sec = Self::wall_secs();
                if sec == self.last_clock_sec {
                    false
                } else {
                    self.last_clock_sec = sec;
                    self.last[i] = now;
                    true
                }
            } else if now.duration_since(self.last[i]) >= kind.interval() {
                self.last[i] = now;
                true
            } else {
                false
            };
            if due {
                changed.extend(kind.update(tb));
            }
        }
        changed
    }

    fn time_to_next(&self) -> Duration {
        let now = Instant::now();
        let interval_min = AppletTick::ALL
            .iter()
            .enumerate()
            .filter(|(_, kind)| !matches!(kind, AppletTick::Clock))
            .map(|(i, kind)| {
                kind.interval()
                    .checked_sub(now.duration_since(self.last[i]))
                    .unwrap_or_default()
            })
            .min()
            .unwrap_or(Duration::from_secs(1));
        interval_min.min(Self::wall_to_next_second())
    }
}

struct LoopTiming {
    enabled: bool,
    last_report: Instant,
    iters: u64,
    events: u64,
    work_sum: Duration,
    work_max: Duration,
    slow: u64,
    rt_at_report: u64,
}

impl LoopTiming {
    fn new() -> LoopTiming {
        let enabled = std::env::var("ANTIBOX_LOOP_TIMING").map_or(false, |v| v != "0" && !v.is_empty());
        if enabled {
            eprintln!("loop timing on (ANTIBOX_LOOP_TIMING): ~1s summaries, warn on iterate work >8ms");
        }
        LoopTiming {
            enabled,
            last_report: Instant::now(),
            iters: 0,
            events: 0,
            work_sum: Duration::from_secs(0),
            work_max: Duration::from_secs(0),
            slow: 0,
            rt_at_report: antibox_core::metrics::round_trips(),
        }
    }

    fn record(&mut self, work: Duration, events: usize, round_trips: u64) {
        if !self.enabled {
            return;
        }
        self.iters += 1;
        self.events += events as u64;
        self.work_sum += work;
        if work > self.work_max {
            self.work_max = work;
        }
        if work >= Duration::from_millis(8) {
            self.slow += 1;
            eprintln!(
                "slow iterate: work={:.1}ms events={} round_trips={}",
                work.as_secs_f64() * 1000.0,
                events,
                round_trips
            );
        }
        let elapsed = self.last_report.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let now_rt = antibox_core::metrics::round_trips();
            let rt_delta = now_rt.saturating_sub(self.rt_at_report);
            let avg_ms = self.work_sum.as_secs_f64() * 1000.0 / self.iters.max(1) as f64;
            eprintln!(
                "loop: {:.0} iters/s, {} events, {} round-trips, work avg={:.2}ms max={:.1}ms, slow(>8ms)={}",
                self.iters as f64 / elapsed.as_secs_f64(),
                self.events,
                rt_delta,
                avg_ms,
                self.work_max.as_secs_f64() * 1000.0,
                self.slow,
            );
            self.last_report = Instant::now();
            self.iters = 0;
            self.events = 0;
            self.work_sum = Duration::from_secs(0);
            self.work_max = Duration::from_secs(0);
            self.slow = 0;
            self.rt_at_report = now_rt;
        }
    }
}

pub struct App {
    pub backend: Arc<dyn DisplayBackend>,
    pub event_loop: Box<dyn EventLoopTrait>,
    pub atom_manager: AtomManager,
    pub wm: WindowManager<dyn DisplayBackend>,
    pub switcher: SwitcherWindow,
    pub preview: PreviewWindow,
    pub winlist: WinListMenu,
    pub group_menu: Option<crate::menu::MenuView<u32>>,
    pub last_pager_sync: Instant,
    pub taskbar: Option<TaskBar>,
    pub(crate) keyboard_layouts_pref: String,
    pub running: bool,
    pub(crate) consecutive_panics: u32,
    #[cfg_attr(not(feature = "tray"), allow(dead_code))]
    pub(crate) tray_opcode_atom: u32,

    pub(crate) super_tap_armed: bool,
    pub(crate) wm_sn_atom: u32,
    pub(crate) wm_sn_owner: Option<Box<dyn WindowHandle>>,
}

fn incumbent_wm_name(b: &dyn DisplayBackend) -> Option<String> {
    let check_atom = b.intern_atom("_NET_SUPPORTING_WM_CHECK").ok()?;
    let name_atom = b.intern_atom("_NET_WM_NAME").ok()?;
    let utf8_atom = b.intern_atom("UTF8_STRING").ok()?;
    let data = b
        .get_property(b.root().read_id(), check_atom, 33, 0, 1)
        .ok()
        .and_then(|v| v)?;
    if data.len() < 4 {
        return None;
    }
    let check = u32::from_ne_bytes([data[0], data[1], data[2], data[3]]);
    if check == 0 {
        return None;
    }
    let name = b
        .get_property(check, name_atom, utf8_atom, 0, 64)
        .ok()
        .and_then(|v| v)?;
    let s = String::from_utf8_lossy(&name)
        .trim_end_matches('\0')
        .to_string();
    if !s.is_empty() { Some(s) } else { None }
}

fn claim_wm_selection(
    b: &dyn DisplayBackend,
    wm_sn: u32,
    old_owner: u32,
) -> Result<Box<dyn WindowHandle>, Box<dyn std::error::Error>> {
    let win = b.create_window(
        b.root().as_parent(),
        antibox_core::rect::Rect::new(-1, -1, 1, 1),
        WmWindowClass::InputOutput,
        true,
        EventMask::NO_EVENT,
    )?;
    let _ = b.set_selection_owner(win.id(), wm_sn, 0);
    let _ = b.flush();
    let now = b.get_selection_owner(wm_sn).unwrap_or(0);
    if now != win.id() {
        eprintln!(
            "could not acquire the WM_Sn manager selection wanted={} got={}",
            win.id(),
            now
        );
        return Err("could not acquire the WM_Sn manager selection".into());
    }
    eprintln!(
        "acquired the WM_Sn manager selection owner={} old_owner={}",
        win.id(),
        old_owner
    );
    if old_owner != 0 {
        for _ in 0..25 {
            if b.get_window_attributes(old_owner).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    Ok(win)
}

pub(crate) fn apply_xft_dpi(b: &dyn DisplayBackend, dpi: i32) {
    let rm_atom = match b.intern_atom("RESOURCE_MANAGER") {
        Ok(a) => a,
        Err(_) => return,
    };
    let str_atom = match b.intern_atom("STRING") {
        Ok(a) => a,
        Err(_) => return,
    };
    let root = b.root();
    let existing = b
        .get_property(root.read_id(), rm_atom, str_atom, 0, u32::MAX / 4)
        .ok()
        .and_then(|v| v)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .filter(|l| {
            let key = l.split(':').next().unwrap_or("").trim();
            key != "Xft.dpi"
        })
        .map(ToString::to_string)
        .collect();
    lines.push(format!("Xft.dpi:\t{}", dpi));
    let mut merged = lines.join("\n");
    merged.push('\n');
    let _ = b.change_property8(
        PropMode::Replace,
        root.read_id(),
        rm_atom,
        str_atom,
        merged.as_bytes(),
    );
}

struct TaskbarConfig {
    time_format: String,
    position: crate::taskbar::TaskBarPosition,
}

#[derive(Clone, Copy)]
pub struct LaunchOptions<'a> {
    pub display: Option<&'a str>,
}

impl App {
    pub fn new(
        backend: Arc<dyn DisplayBackend>,
        render_backend: Arc<dyn RenderBackend>,
        event_loop: Box<dyn EventLoopTrait>,
        tray: Option<&Arc<dyn TrayBackend>>,
        options: LaunchOptions<'_>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let LaunchOptions { display } = options;
        let b = backend;
        let mut event_loop = event_loop;
        if let Some(wfd) = crate::config_watch::init_watch() {
            event_loop.add_fd(wfd, Box::new(move || crate::config_watch::drain(wfd)));
        }
        let sw = b.screen_width();
        let sh = b.screen_height();
        let (mm_w, mm_h) = b.screen_size_mm();
        let dpi =
            antibox_core::scale::detect(sw as u32, sh as u32, mm_w, mm_h, b.preferred_scale());
        antibox_core::scale::set_dpi(dpi);
        apply_xft_dpi(&*b, dpi);
        eprintln!(
            "HiDPI: detected display metrics screen={}x{} dpi={} scale={} title_bar_px={}",
            sw,
            sh,
            dpi,
            antibox_core::scale::dpi() as f32 / 96.0,
            crate::frame::title_bar_height()
        );
        let composite_available = b.composite_supported();
        let mut atom_manager = AtomManager::new();
        let _ = atom_manager.intern_all(&*b);

        let prefs = wmconfig::Config::load_prefs();
        crate::fonts::apply_fonts();
        apply_font_prefs(&b, &prefs);
        crate::layout_preferences::apply();
        crate::tooltip::set_show_delay_ms(500);
        crate::tooltip::set_lifetime_ms(0);
        crate::drag::set_multi_click_ms(400);

        let (ws_count, ws_names) = wmconfig::workspaces_from(&prefs);

        let root_mask = EventMask::SUBSTRUCTURE_REDIRECT
            | EventMask::SUBSTRUCTURE_NOTIFY
            | EventMask::EXPOSURE
            | EventMask::FOCUS_CHANGE
            | EventMask::PROPERTY_CHANGE
            | EventMask::BUTTON_PRESS;

        let disp = display
            .map(str::to_string)
            .or_else(|| std::env::var("DISPLAY").ok())
            .unwrap_or_else(|| ":0".to_string());
        let wm_sn_atom = b
            .intern_atom(&format!("WM_S{}", b.default_screen()))
            .unwrap_or(0);
        let old_owner = if wm_sn_atom != 0 {
            b.get_selection_owner(wm_sn_atom).unwrap_or(0)
        } else {
            0
        };
        if old_owner != 0 {
            let who = incumbent_wm_name(&*b).map_or_else(
                || "a non-EWMH window manager".to_string(),
                |n| format!("\"{}\"", n),
            );
            return Err(format!(
                "antibox: {} already manages display {} (owns WM_S{})",
                who,
                disp,
                b.default_screen()
            )
            .into());
        }
        let wm_sn_owner = if wm_sn_atom != 0 {
            claim_wm_selection(&*b, wm_sn_atom, old_owner).ok()
        } else {
            None
        };

        let redirect = b.select_root_input_checked(root_mask);
        if redirect.is_err() {
            let who = incumbent_wm_name(&*b).map_or_else(|| {
                    "a window manager without EWMH identification (possibly the display server's built-in management)".to_string()
                }, |n| format!("\"{}\"", n));
            return Err(format!("antibox: {} already manages display {}", who, disp).into());
        }

        let mut wm = WindowManager::with_config(
            &b,
            ws_count,
            ws_names,
            true,
            false,
            &crate::render::ThemeColors::default(),
        );
        wm.render_backend = Some(render_backend);
        let _ = wm.atoms.intern_all(&*b);
        wm.set_workspace_layouts(crate::layout::Layout::parse_list(
            &prefs.workspace.layouts,
            ws_count as usize,
        ));

        logevent::init_log_events();
        {
            let rev = wm.atoms.reverse_map();
            logevent::set_atom_resolver(move |atom: u32| rev.get(&atom).cloned());
        }

        let taskbar_position = crate::taskbar::TaskBarPosition::Bottom;
        wm.cursors = crate::cursors::init_cursors(&*b);
        let (taskbar, tray_opcode_atom) = Self::create_taskbar(
            &b,
            &mut wm,
            &atom_manager,
            tray,
            composite_available,
            TaskbarConfig {
                time_format: "%H:%M:%S".to_string(),
                position: taskbar_position,
            },
        );
        let _ = wm
            .key_bindings
            .register_all(&b, &crate::keys_parser::entries_from(&prefs.keys));
        wm.monitors = b.query_monitors().unwrap_or_default();

        let _ = crate::ewmh::init_ewmh(&*b, &atom_manager, wm.config.workspace_count);
        crate::ewmh::init_xdnd(&*b, &atom_manager);
        crate::ewmh::update_desktop_names(&*b, &atom_manager, &wm.workspace_names);
        crate::handler::manage_existing_windows(&mut wm);
        if wm.focused_window.is_none() {
            let _ = b.set_input_focus(0, b.root().read_id(), 0);
        }
        crate::placement::update_workarea_from_struts(&mut wm);
        Self::run_script("startup");
        Ok(App {
            backend: b,
            event_loop,
            atom_manager,
            wm,
            switcher: SwitcherWindow::new(),
            preview: PreviewWindow::new(),
            winlist: WinListMenu::new(),
            group_menu: None,
            last_pager_sync: Instant::now(),
            taskbar,
            keyboard_layouts_pref: prefs.keyboard.layouts.clone(),
            running: true,
            consecutive_panics: 0,
            tray_opcode_atom,
            super_tap_armed: false,
            wm_sn_atom,
            wm_sn_owner,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crate::panic_guard::install_hook();
        crate::panic_guard::report_previous();
        let mut tickers = AppletTickers::new();
        let mut timing = LoopTiming::new();
        while self.running {
            self.iterate(&mut tickers, &mut timing)?;
        }
        self.release_clients();
        crate::cursors::free_cursors(&*self.backend, &self.wm.cursors);
        Self::run_script("shutdown");
        Ok(())
    }

    fn iterate(
        &mut self,
        tickers: &mut AppletTickers,
        timing: &mut LoopTiming,
    ) -> Result<(), Box<dyn std::error::Error>> {
        const MAX_IDLE: Duration = Duration::from_secs(1);

        let mut work = LoopWork::default();

        let rt_start = antibox_core::metrics::round_trips();
        let drain_start = Instant::now();
        work.events += self.drain_pending_events_count();
        let mut work_time = drain_start.elapsed();

        let timeout = if work.events > 0 {
            Duration::from_secs(0)
        } else {
            self.compute_timeout(tickers, MAX_IDLE)
        };
        let waited = self.event_loop.wait_for_one_event(timeout)?;
        let post_wait = Instant::now();
        if let Some(event) = waited {
            self.dispatch_guarded(&event);
            work.events += 1;
            work.events += self.drain_pending_events_count();
        }
        self.event_loop.fire_timers();

        if crate::config_watch::take_changed() {
            self.reload_config();
        }

        if let Some(e) = self.backend.check_for_error() {
            eprintln!("[antibox] X11 connection error: {}", e);
            self.running = false;
            return Ok(());
        }

        if !self.running {
            return Ok(());
        }

        self.tick_taskbar(tickers, &mut work);

        if let Some(ref tb) = self.taskbar {
            if work.repaint_taskbar {
                let _ = tb.paint();
            } else {
                for &wid in &work.repaint_applets {
                    let _ = tb.paint_window(wid);
                }
            }
        }
        if work.any() {
            let _ = self.backend.flush();
        }
        work_time += post_wait.elapsed();
        let round_trips = antibox_core::metrics::round_trips().saturating_sub(rt_start);
        timing.record(work_time, work.events, round_trips);
        Ok(())
    }

    fn compute_timeout(&self, tickers: &AppletTickers, max_idle: Duration) -> Duration {
        let mut timeout = max_idle;
        if let Some(t) = self.event_loop.time_to_next_timer() {
            timeout = timeout.min(t);
        }
        timeout = timeout.min(tickers.time_to_next());
        if self
            .taskbar
            .as_ref()
            .map_or(false, TaskBar::any_tooltip_pending)
        {
            timeout = timeout.min(Duration::from_millis(100));
        }
        timeout
    }

    fn tick_taskbar(&mut self, tickers: &mut AppletTickers, work: &mut LoopWork) {
        let tb = match self.taskbar {
            Some(ref mut tb) => tb,
            None => return,
        };
        let ws = self.wm.active_workspace();
        tb.set_active_workspace(ws);
        match tb.sync_task_pane(&self.wm) {
            crate::taskbar::BarRepaint::Full => work.repaint_taskbar = true,
            crate::taskbar::BarRepaint::Applet(wid) => work.repaint_applets.push(wid),
            crate::taskbar::BarRepaint::None => {}
        }
        let dragging = self.wm.drag_state.is_some();
        let now = Instant::now();
        if !dragging || now.duration_since(self.last_pager_sync) >= Duration::from_millis(120) {
            self.last_pager_sync = now;
            if let Some(wid) = tb.sync_pager(&self.wm) {
                work.repaint_applets.push(wid);
            }
        }
        let mut pane_ws: Option<u32> = None;
        for a in &tb.applets {
            if let Some(p) = a.as_any().downcast_ref::<WorkspacesPane>() {
                if p.active_workspace != ws {
                    pane_ws = Some(p.active_workspace);
                    work.repaint_taskbar = true;
                }
            }
        }
        if let Some(new_ws) = pane_ws {
            self.wm.activate_workspace(new_ws);
        }
        if let Some(ref mut tb) = self.taskbar {
            let changed = tickers.tick(tb);
            work.repaint_applets.extend(changed);
            tb.pump_tooltips();
        }
    }

    fn run_script(name: &str) {
        for d in wmconfig::Config::search_dirs() {
            let p = d.join(name);
            if p.is_file() {
                match std::fs::metadata(&p) {
                    Ok(ref m) if m.permissions().mode() & 0o111 != 0 => {
                        eprintln!("Running {} script: {}", name, p.display());
                        let _ = std::process::Command::new(&p).spawn();
                    }
                    _ => {}
                }
            }
        }
    }

    fn create_taskbar(
        conn: &Arc<dyn DisplayBackend>,
        wm: &mut WindowManager<dyn DisplayBackend>,
        atom_manager: &AtomManager,
        tray: Option<
            &Arc<dyn TrayBackend>,
        >,
        composite_available: bool,
        cfg: TaskbarConfig,
    ) -> (Option<TaskBar>, u32) {
        let TaskbarConfig {
            time_format,
            position,
        } = cfg;
        let strut_atom = atom_manager.get("_NET_WM_STRUT").unwrap_or(0);
        let mut tb = match TaskBar::new(conn, position, strut_atom) {
            Ok(tb) => tb,
            Err(_) => return (None, 0),
        };
        wm.reserved_strut = tb.strut();
        let tc = wm.theme_colours;
        tb.apply_theme_colours(&tc, wm.config.gradients);
        let wid = tb.window.id();
        wm.above_windows = vec![wid];
        use crate::layout_preferences::{taskbar_wants, Widget};
        let mut core: Vec<Box<dyn Applet>> = Vec::new();
        if taskbar_wants(Widget::Workspaces) {
            if let Ok(p) = WorkspacesPane::new(conn, wid, &wm.workspace_names, wm.theme_colours) {
                core.push(Box::new(p));
            }
        }
        if taskbar_wants(Widget::Windows) {
            if let Ok(p) = TaskPane::new(conn, wid, wm.theme_colours) {
                core.push(Box::new(p));
            }
        }
        #[cfg(feature = "tray")]
        {
            if taskbar_wants(Widget::Tray) {
                if let Ok(mut t) =
                    TrayApplet::new(conn, wid, atom_manager, tray.cloned(), composite_available)
                {
                    t.set_colours();
                    core.push(Box::new(t));
                }
            }
        }
        if taskbar_wants(Widget::Clock) {
            let clock_fmt = {
                let themed = antibox_ui::theme::clock_format();
                if themed.is_empty() {
                    time_format.clone()
                } else {
                    themed.to_string()
                }
            };
            if let Ok(mut c) = ClockApplet::new(conn, wid, Some(clock_fmt)) {
                c.set_colours(&wm.theme_colours);
                core.push(Box::new(c));
            }
        }
        let prefs = wmconfig::Config::load_prefs();
        if taskbar_wants(Widget::Cpu) {
            if let Ok(c) = crate::cpu_status_applet::CpuStatusApplet::new(conn, wid, prefs.cpu.width)
            {
                core.push(Box::new(c));
            }
        }
        if taskbar_wants(Widget::Mem) {
            if let Ok(m) = crate::mem_status_applet::MemStatusApplet::new(conn, wid, prefs.mem.width)
            {
                core.push(Box::new(m));
            }
        }
        if taskbar_wants(Widget::Net) {
            crate::net_status_applet::set_net_device(&prefs.net.device);
            if let Ok(n) = crate::net_status_applet::NetStatusApplet::new(conn, wid, prefs.net.width)
            {
                core.push(Box::new(n));
            }
        }
        if taskbar_wants(Widget::Keyboard) {
            if let Ok(Some(kb)) = crate::keyboard_applet::KeyboardApplet::new(
                conn,
                wid,
                wmconfig::split_layout_list(&prefs.keyboard.layouts),
                &wm.theme_colours,
            ) {
                core.push(Box::new(kb));
            }
        }
        for a in core {
            let _ = a.window().map();
            tb.add_applet(a);
        }
        let resize_cursor = wm
            .cursors
            .get(crate::cursors::idx::SIZE_H).cloned()
            .unwrap_or(0);
        if resize_cursor != 0 {
            for a in &tb.applets {
                if a.wants_resize_cursor() {
                    let _ = conn.define_cursor(a.window().id(), resize_cursor);
                }
            }
        }
        let tray_opcode_atom = atom_manager.get("_NET_SYSTEM_TRAY_OPCODE").unwrap_or(0);
        let _ = tb.show();
        let _ = tb.paint();
        (Some(tb), tray_opcode_atom)
    }
}

include!("../app_event.rs");
