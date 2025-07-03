use super::*;
use antibox_core::backend::ButtonGrabSpec;
use antibox_core::point::{Dimension, Point};

impl DisplayBackend for WaylandCompositor {
    fn open(_name: Option<&str>) -> R<Self>
    where
        Self: Sized,
    {
        Err("WaylandCompositor is constructed via the event loop, not open()".into())
    }

    fn fd(&self) -> RawFd {
        self.shared.lock().display_fd
    }

    fn preferred_scale(&self) -> Option<f64> {
        let s = self.shared.lock().screen_scale;
        Some(s.max(1.0))
    }

    fn flush(&self) -> R {
        self.shared.request_redraw();
        Ok(())
    }

    fn poll_for_event(&self) -> R<Option<BackendEvent>> {
        Ok(self.shared.lock().events.pop())
    }

    fn last_event_time(&self) -> u32 {
        self.shared.lock().last_time
    }

    fn default_screen(&self) -> usize {
        0
    }

    fn set_screen_size(&self, width: u16, height: u16) {
        let mut s = self.shared.lock();
        s.screen_w = width;
        s.screen_h = height;
    }

    fn root_visual(&self) -> u32 {
        0
    }
    fn argb_visual(&self) -> Option<u32> {
        Some(0)
    }
    fn argb_depth(&self) -> Option<u8> {
        Some(32)
    }
    fn composite_supported(&self) -> bool {
        true
    }
    fn setup_min_keycode(&self) -> u8 {
        MIN_KEYCODE
    }
    fn setup_max_keycode(&self) -> u8 {
        MAX_KEYCODE
    }

    fn get_atom_name(&self, atom: u32) -> R<String> {
        self.shared
            .lock()
            .atoms
            .name(atom)
            .ok_or_else(|| "unknown atom".into())
    }
    fn query_extension(&self, _name: &str) -> R<bool> {
        Ok(false)
    }

    fn get_keyboard_mapping(&self, first_keycode: u8, count: u8) -> R<KeyboardMapping> {
        let s = self.shared.lock();
        let kpc = s.keysyms_per_keycode as usize;
        if kpc == 0 || s.keymap.is_empty() {
            let keysyms = (0..count as u32)
                .map(|i| first_keycode as u32 + i)
                .collect();
            return Ok(KeyboardMapping {
                keysyms_per_keycode: 1,
                keysyms,
            });
        }
        let mut keysyms = Vec::with_capacity(count as usize * kpc);
        for i in 0..count as u32 {
            let kc = first_keycode as u32 + i;
            let idx = kc.checked_sub(MIN_KEYCODE as u32).map(|d| d as usize * kpc);
            match idx.and_then(|o| s.keymap.get(o..o + kpc)) {
                Some(chunk) => keysyms.extend_from_slice(chunk),
                None => keysyms.extend(std::iter::repeat(0).take(kpc)),
            }
        }
        Ok(KeyboardMapping {
            keysyms_per_keycode: kpc as u8,
            keysyms,
        })
    }
    fn get_modifier_mapping(&self) -> R<ModifierMapping> {
        Ok(ModifierMapping {
            keycodes_per_modifier: Default::default(),
        })
    }

    fn grab_key(
        &self,
        _owner_events: bool,
        window: u32,
        modifiers: u16,
        keycode: u8,
        _pointer_mode: GrabMode,
        _keyboard_mode: GrabMode,
    ) -> R {
        self.shared.lock().key_grabs.push(KeyGrab {
            keycode,
            modifiers,
            window,
        });
        Ok(())
    }
    fn ungrab_key(&self, keycode: u8, modifiers: u16, window: u32) -> R {
        self.shared
            .lock()
            .key_grabs
            .retain(|g| !(g.keycode == keycode && g.modifiers == modifiers && g.window == window));
        Ok(())
    }
    fn grab_keyboard(
        &self,
        _owner_events: bool,
        window: u32,
        _time: u32,
        _pointer_mode: GrabMode,
        _keyboard_mode: GrabMode,
    ) -> R {
        self.shared.lock().keyboard_grab = Some(window);
        Ok(())
    }
    fn ungrab_keyboard(&self, _time: u32) -> R {
        self.shared.lock().keyboard_grab = None;
        Ok(())
    }

    fn query_pointer(&self, _window: u32) -> R<PointerState> {
        let s = self.shared.lock();
        Ok(PointerState {
            root_x: s.pointer_x,
            root_y: s.pointer_y,
            mask: KeyButMask::new(s.pointer_mask | s.key_mods),
        })
    }

    fn grab_button(&self, grab: ButtonGrabSpec) -> R {
        self.shared.lock().button_grabs.push(ButtonGrab {
            button: grab.button,
            modifiers: grab.modifiers,
            window: grab.window,
        });
        Ok(())
    }
    fn ungrab_button(&self, button: u8, modifiers: u16, window: u32) -> R {
        self.shared
            .lock()
            .button_grabs
            .retain(|g| !(g.button == button && g.modifiers == modifiers && g.window == window));
        Ok(())
    }

    fn warp_pointer(&self, _src_window: u32, _dst_window: u32, _src_area: Rect, dst: Point) -> R {
        self.push_intent(Intent::Warp {
            x: dst.x as i16,
            y: dst.y as i16,
        });
        Ok(())
    }

    fn set_input_focus(&self, _revert_to: u8, window: u32, _time: u32) -> R {
        self.shared.lock().focus = window;
        self.push_intent(Intent::Focus(window));
        Ok(())
    }

    fn map_window(&self, window: u32) -> R {
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&window) {
                rec.mapped = true;
                if rec.kind == WinKind::Server {
                    let rect = rec.rect;
                    s.events.push(BackendEvent::Expose {
                        window,
                        rect: Rect::new(0, 0, rect.w, rect.h),
                    });
                }
            }
        }
        self.push_intent(Intent::Map(window));
        Ok(())
    }
    fn unmap_window(&self, window: u32) -> R {
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&window) {
                let was = rec.mapped;
                rec.mapped = false;
                if was && rec.kind == WinKind::Client {
                    s.events.push(BackendEvent::UnmapNotify { window });
                }
            }
        }
        self.push_intent(Intent::Unmap(window));
        Ok(())
    }
    fn destroy_window(&self, window: u32) -> R {
        let ids = {
            let mut s = self.shared.lock();
            let ids = s.subtree(window);
            for id in &ids {
                s.windows.remove(id);
            }
            ids
        };
        for id in ids {
            self.buffers.remove(id);
            self.push_intent(Intent::Destroy(id));
        }
        Ok(())
    }
    fn kill_client(&self, resource: u32) -> R {
        self.push_intent(Intent::Kill(resource));
        Ok(())
    }

    fn configure_window(&self, window: u32, value_list: &[u32]) -> R {
        let (x, y, w, h) = match value_list {
            [x, y, w, h] => (
                Some(*x as i32),
                Some(*y as i32),
                Some(*w as u16),
                Some(*h as u16),
            ),
            [x, y] => (Some(*x as i32), Some(*y as i32), None, None),
            [x] => (Some(*x as i32), None, None, None),
            _ => (None, None, None, None),
        };
        {
            let mut s = self.shared.lock();
            if let Some(rec) = s.windows.get_mut(&window) {
                if let Some(x) = x {
                    rec.rect.x = x;
                }
                if let Some(y) = y {
                    rec.rect.y = y;
                }
                if let Some(w) = w {
                    rec.rect.w = w as i32;
                }
                if let Some(h) = h {
                    rec.rect.h = h as i32;
                }
            }
        }
        if let (Some(w), Some(h)) = (w, h) {
            let same = self
                .buffers
                .with(window, |c| c.width() == w.max(1) && c.height() == h.max(1))
                .unwrap_or(false);
            if !same && self.buffers.contains(window) {
                self.buffers
                    .insert(window, SoftCanvas::new(w.max(1), h.max(1)));
                let mut s = self.shared.lock();
                if s.windows.get(&window).map(|r| (r.kind, r.mapped))
                    == Some((WinKind::Server, true))
                {
                    s.events.push(BackendEvent::Expose {
                        window,
                        rect: Rect::new(0, 0, w as i32, h as i32),
                    });
                }
            }
        }
        self.push_intent(Intent::Configure {
            win: window,
            x,
            y,
            w,
            h,
        });
        Ok(())
    }
    fn change_window_attributes(&self, window: u32, value_list: &[u32]) -> R {
        if window == ROOT_WINDOW && value_list.len() >= 2 && value_list[0] & (1 << 1) != 0 {
            self.shared.lock().set_root_background(value_list[1]);
        }
        Ok(())
    }

    fn reparent_window(&self, child: u32, parent: u32, pos: Point) -> R {
        let mut s = self.shared.lock();
        if let Some(rec) = s.windows.get_mut(&child) {
            rec.parent = parent;
            rec.rect.x = pos.x;
            rec.rect.y = pos.y;
            if rec.mapped && rec.kind == WinKind::Client {
                s.events.push(BackendEvent::UnmapNotify { window: child });
            }
        }
        Ok(())
    }

    fn clear_area(&self, _exposures: bool, _window: u32, _area: Rect) -> R {
        Ok(())
    }

    fn set_selection_owner(&self, owner: u32, selection: u32, _time: u32) -> R {
        self.shared.lock().selections.insert(selection, owner);
        Ok(())
    }
    fn get_selection_owner(&self, selection: u32) -> R<u32> {
        Ok(self
            .shared
            .lock()
            .selections
            .get(&selection)
            .copied()
            .unwrap_or(0))
    }

    fn generate_id(&self) -> R<u32> {
        Ok(self.shared.lock().alloc_id())
    }

    fn query_tree(&self, _window: u32) -> R<QueryTreeResult> {
        Ok(QueryTreeResult {
            root: ROOT_WINDOW,
            parent: 0,
            children: Vec::new(),
        })
    }

    fn get_window_attributes(&self, window: u32) -> R<WindowAttributes> {
        let s = self.shared.lock();
        let rec = s.windows.get(&window);
        Ok(WindowAttributes {
            override_redirect: rec.is_some_and(|r| r.override_redirect),
            map_state: rec.map_or(antibox_core::backend::MapState::Unmapped, WinRec::map_state),
            depth: rec.map_or(32, |r| r.depth),
        })
    }

    fn query_monitors(&self) -> R<Vec<MonitorInfo>> {
        let s = self.shared.lock();
        Ok(vec![MonitorInfo {
            x: 0,
            y: 0,
            width: s.screen_w,
            height: s.screen_h,
        }])
    }

    fn restack_windows(&self, windows: &[u32]) -> R {
        self.push_intent(Intent::Restack(windows.to_vec()));
        Ok(())
    }

    fn create_font_cursor(&self, _glyph: u32) -> R<u32> {
        Ok(self.shared.lock().alloc_id())
    }
    fn create_named_cursor(&self, _name: &str) -> R<u32> {
        Ok(self.shared.lock().alloc_id())
    }
    fn create_cursor_from_rgba(
        &self,
        _pixels: &[u8],
        _size: Dimension,
        _hotspot: Point,
        _foreground: [u8; 3],
        _background: [u8; 3],
    ) -> R<u32> {
        Ok(self.shared.lock().alloc_id())
    }

    fn list_core_font_families(&self) -> Vec<String> {
        Vec::new()
    }
}
