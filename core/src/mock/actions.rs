 use antibox_gfx::error::Result;
use crate::backend::{
    BackendEvent, DisplayBackend, GrabMode, KeyButMask, KeyboardMapping, MapState, ModifierMapping,
    MonitorInfo, PointerState, QueryTreeResult, RenderBackend, WindowAttributes,
};
use antibox_gfx::mock::MockDisplay;
use antibox_gfx::point::Point;
use antibox_gfx::rect::Rect;
use std::os::unix::io::RawFd;

impl DisplayBackend for MockDisplay {
    fn open(_name: Option<&str>) -> Result<Self> {
        Ok(Self::new(1920, 1080, 24))
    }
    fn fd(&self) -> RawFd {
        -1
    }
    fn flush(&self) -> Result<()> {
        Ok(())
    }
    fn poll_for_event(&self) -> Result<Option<BackendEvent>> {
        Ok(self.events.lock().unwrap().pop())
    }

    fn default_screen(&self) -> usize {
        self.screen_num
    }

    fn set_screen_size(&self, width: u16, height: u16) {
        self.width
            .store(width, std::sync::atomic::Ordering::Relaxed);
        self.height
            .store(height, std::sync::atomic::Ordering::Relaxed);
    }

    fn root_visual(&self) -> u32 {
        self.visual
    }
    fn setup_min_keycode(&self) -> u8 {
        8
    }
    fn setup_max_keycode(&self) -> u8 {
        255
    }

    fn get_atom_name(&self, atom: u32) -> Result<String> {
        let atoms = self.atoms.lock().unwrap();
        for (name, &id) in atoms.iter() {
            if id == atom {
                return Ok(name.clone());
            }
        }
        Err(format!("unknown atom: {atom}").into())
    }
    fn query_extension(&self, _name: &str) -> Result<bool> {
        Ok(false)
    }

    fn get_keyboard_mapping(
        &self,
        _first_keycode: u8,
        count: u8,
    ) -> Result<KeyboardMapping> {
        Ok(KeyboardMapping {
            keysyms_per_keycode: 1,
            keysyms: vec![0; count as usize],
        })
    }
    fn get_modifier_mapping(&self) -> Result<ModifierMapping> {
        Ok(ModifierMapping {
            keycodes_per_modifier: [
                vec![],
                vec![],
                vec![],
                vec![64],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
        })
    }
    fn grab_key(
        &self,
        _owner_events: bool,
        _window: u32,
        _modifiers: u16,
        _keycode: u8,
        _pointer_mode: GrabMode,
        _keyboard_mode: GrabMode,
    ) -> Result<()> {
        Ok(())
    }
    fn ungrab_key(
        &self,
        _keycode: u8,
        _modifiers: u16,
        _window: u32,
    ) -> Result<()> {
        Ok(())
    }
    fn grab_keyboard(
        &self,
        _owner_events: bool,
        _window: u32,
        _time: u32,
        _pointer_mode: GrabMode,
        _keyboard_mode: GrabMode,
    ) -> Result<()> {
        Ok(())
    }
    fn ungrab_keyboard(&self, _time: u32) -> Result<()> {
        Ok(())
    }
    fn query_pointer(&self, _window: u32) -> Result<PointerState> {
        Ok(PointerState {
            root_x: 0,
            root_y: 0,
            mask: KeyButMask(0),
        })
    }

    fn warp_pointer(
        &self,
        _src_window: u32,
        dst_window: u32,
        _src_area: Rect,
        dst: Point,
    ) -> Result<()> {
        self.warps
            .lock()
            .unwrap()
            .push((dst_window, dst.x as i16, dst.y as i16));
        Ok(())
    }
    fn set_input_focus(
        &self,
        _revert_to: u8,
        _window: u32,
        _time: u32,
    ) -> Result<()> {
        Ok(())
    }
    fn map_window(&self, window: u32) -> Result<()> {
        self.lifecycle
            .lock()
            .unwrap()
            .push((window, antibox_gfx::mock::Lifecycle::Map));
        Ok(())
    }
    fn unmap_window(&self, window: u32) -> Result<()> {
        self.lifecycle
            .lock()
            .unwrap()
            .push((window, antibox_gfx::mock::Lifecycle::Unmap));
        Ok(())
    }
    fn destroy_window(&self, window: u32) -> Result<()> {
        self.lifecycle
            .lock()
            .unwrap()
            .push((window, antibox_gfx::mock::Lifecycle::Destroy));
        Ok(())
    }
    fn kill_client(&self, _resource: u32) -> Result<()> {
        Ok(())
    }
    fn configure_window(&self, _window: u32, _value_list: &[u32]) -> Result<()> {
        Ok(())
    }
    fn change_window_attributes(
        &self,
        _window: u32,
        _value_list: &[u32],
    ) -> Result<()> {
        Ok(())
    }
    fn clear_area(&self, _exposures: bool, _window: u32, _area: Rect) -> Result<()> {
        Ok(())
    }

    fn set_selection_owner(
        &self,
        _owner: u32,
        _selection: u32,
        _time: u32,
    ) -> Result<()> {
        Ok(())
    }
    fn get_selection_owner(&self, _selection: u32) -> Result<u32> {
        Ok(0)
    }
    fn reparent_window(&self, _child: u32, _parent: u32, _pos: Point) -> Result<()> {
        Ok(())
    }
    fn generate_id(&self) -> Result<u32> {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        Ok(id)
    }
    fn query_tree(&self, _window: u32) -> Result<QueryTreeResult> {
        Ok(QueryTreeResult {
            root: self.root_window,
            parent: 0,
            children: vec![],
        })
    }
    fn get_window_attributes(&self, _window: u32) -> Result<WindowAttributes> {
        Ok(WindowAttributes {
            override_redirect: false,
            map_state: MapState::Viewable,
            depth: 24,
        })
    }
    fn query_monitors(&self) -> Result<Vec<MonitorInfo>> {
        Ok(vec![MonitorInfo {
            x: 0,
            y: 0,
            width: self.screen_width(),
            height: self.screen_height(),
        }])
    }
    fn restack_windows(&self, _windows: &[u32]) -> Result<()> {
        Ok(())
    }

    fn render_format_for_depth(&self, _depth: u8) -> Option<u32> {
        Some(1)
    }

    fn create_render_picture(&self, _pixmap: u32, _depth: u8) -> Result<u32> {
        Ok(42)
    }

    fn free_render_picture(&self, _picture: u32) -> Result<()> {
        Ok(())
    }

    fn set_window_opacity(
        &self,
        _window: u32,
        _opacity: f32,
        _opacity_atom: u32,
    ) -> Result<()> {
        Ok(())
    }

    fn grab_root_window(&self) -> Result<(u16, u16, Vec<u8>)> {
        Ok((800, 600, vec![0u8; 800 * 600 * 4]))
    }

    fn send_selection_notify(
        &self,
        _requestor: u32,
        _selection: u32,
        _target: u32,
        _property: u32,
        _time: u32,
    ) -> Result<()> {
        Ok(())
    }

    fn set_win_gravity(&self, _window: u32, _gravity: u32) -> Result<()> {
        Ok(())
    }

    fn define_cursor(&self, _window: u32, _cursor: u32) -> Result<()> {
        Ok(())
    }

    fn undefine_cursor(&self, _window: u32) -> Result<()> {
        Ok(())
    }

    fn free_cursor(&self, _cursor: u32) -> Result<()> {
        Ok(())
    }

    fn create_cursor_from_pixmap(
        &self,
        _source: u32,
        _mask: u32,
        _fore: [u16; 3],
        _back: [u16; 3],
        _hotspot: Point,
    ) -> Result<u32> {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        Ok(id)
    }

    fn create_font_cursor(&self, _glyph: u32) -> Result<u32> {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        Ok(id)
    }

    fn create_cursor_from_rgba(
        &self,
        _pixels: &[u8],
        _size: crate::point::Dimension,
        _hotspot: Point,
        _foreground: [u8; 3],
        _background: [u8; 3],
    ) -> Result<u32> {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        Ok(id)
    }

    fn create_named_cursor(&self, _name: &str) -> Result<u32> {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        Ok(id)
    }
}
