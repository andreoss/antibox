use crate::backend::{
    BackendEvent, ButtonGrabSpec, EventHandler, GrabMode, KeyboardMapping, ModifierMapping,
    MonitorInfo, PointerState, QueryTreeResult, RenderBackend, WindowAttributes,
};
use crate::point::{Dimension, Point};
use crate::rect::Rect;
use std::error::Error;
use std::os::unix::io::RawFd;

pub trait DisplayBackend: RenderBackend {
    fn open(name: Option<&str>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;

    fn fd(&self) -> RawFd;
    fn flush(&self) -> Result<(), Box<dyn Error>>;
    fn check_for_error(&self) -> Option<Box<dyn Error>> {
        None
    }
    fn poll_for_event(&self) -> Result<Option<BackendEvent>, Box<dyn Error>>;
    fn last_event_time(&self) -> u32 {
        0
    }
    fn process_events(&mut self, handler: &mut dyn EventHandler) -> Result<(), Box<dyn Error>> {
        while let Some(event) = self.poll_for_event()? {
            handler.handle_event(&event);
        }
        Ok(())
    }

    fn default_screen(&self) -> usize;

    fn screen_size_mm(&self) -> (u32, u32) {
        (0, 0)
    }

    fn preferred_scale(&self) -> Option<f64> {
        None
    }
    fn set_screen_size(&self, _width: u16, _height: u16) {}
    fn keyboard_layout(&self) -> Option<String> {
        None
    }
    fn keyboard_info(&self) -> Option<crate::backend::KeyboardInfo> {
        None
    }
    fn set_keyboard_group(&self, _group: usize) -> bool {
        false
    }

    fn root_visual(&self) -> u32;
    fn argb_visual(&self) -> Option<u32> {
        None
    }
    fn argb_depth(&self) -> Option<u8> {
        None
    }
    fn composite_supported(&self) -> bool {
        false
    }
    fn setup_min_keycode(&self) -> u8;
    fn setup_max_keycode(&self) -> u8;

    fn get_atom_name(&self, atom: u32) -> Result<String, Box<dyn Error>>;
    fn query_extension(&self, name: &str) -> Result<bool, Box<dyn Error>>;

    fn change_save_set(&self, window: u32, insert: bool) -> Result<(), Box<dyn Error>> {
        let _ = (window, insert);
        Ok(())
    }

    fn get_keyboard_mapping(
        &self,
        first_keycode: u8,
        count: u8,
    ) -> Result<KeyboardMapping, Box<dyn Error>>;

    fn get_modifier_mapping(&self) -> Result<ModifierMapping, Box<dyn Error>>;
    fn grab_key(
        &self,
        owner_events: bool,
        window: u32,
        modifiers: u16,
        keycode: u8,
        pointer_mode: GrabMode,
        keyboard_mode: GrabMode,
    ) -> Result<(), Box<dyn Error>>;
    fn ungrab_key(&self, keycode: u8, modifiers: u16, window: u32) -> Result<(), Box<dyn Error>>;
    fn grab_keyboard(
        &self,
        owner_events: bool,
        window: u32,
        time: u32,
        pointer_mode: GrabMode,
        keyboard_mode: GrabMode,
    ) -> Result<(), Box<dyn Error>>;
    fn ungrab_keyboard(&self, time: u32) -> Result<(), Box<dyn Error>>;

    fn query_pointer(&self, window: u32) -> Result<PointerState, Box<dyn Error>>;

    fn grab_button(&self, grab: ButtonGrabSpec) -> Result<(), Box<dyn Error>> {
        let _ = grab;
        Ok(())
    }
    fn ungrab_button(&self, button: u8, modifiers: u16, window: u32) -> Result<(), Box<dyn Error>> {
        let _ = (button, modifiers, window);
        Ok(())
    }
    fn allow_events(&self, mode: u8, time: u32) -> Result<(), Box<dyn Error>> {
        let _ = (mode, time);
        Ok(())
    }
    fn warp_pointer(
        &self,
        src_window: u32,
        dst_window: u32,
        src_area: Rect,
        dst: Point,
    ) -> Result<(), Box<dyn Error>>;

    fn set_input_focus(&self, revert_to: u8, window: u32, time: u32) -> Result<(), Box<dyn Error>>;

    fn map_window(&self, window: u32) -> Result<(), Box<dyn Error>>;
    fn unmap_window(&self, window: u32) -> Result<(), Box<dyn Error>>;
    fn destroy_window(&self, window: u32) -> Result<(), Box<dyn Error>>;
    fn kill_client(&self, resource: u32) -> Result<(), Box<dyn Error>>;
    fn configure_window(&self, window: u32, value_list: &[u32]) -> Result<(), Box<dyn Error>>;

    fn send_configure_notify(
        &self,
        window: u32,
        rect: Rect,
        border: u32,
    ) -> Result<(), Box<dyn Error>> {
        let _ = (window, rect, border);
        Ok(())
    }
    fn change_window_attributes(
        &self,
        window: u32,
        value_list: &[u32],
    ) -> Result<(), Box<dyn Error>>;

    fn reparent_window(&self, child: u32, parent: u32, pos: Point) -> Result<(), Box<dyn Error>>;

    fn clear_area(&self, exposures: bool, window: u32, area: Rect) -> Result<(), Box<dyn Error>>;

    fn set_selection_owner(
        &self,
        owner: u32,
        selection: u32,
        time: u32,
    ) -> Result<(), Box<dyn Error>>;
    fn get_selection_owner(&self, selection: u32) -> Result<u32, Box<dyn Error>>;

    fn generate_id(&self) -> Result<u32, Box<dyn Error>>;

    fn query_tree(&self, window: u32) -> Result<QueryTreeResult, Box<dyn Error>>;

    fn get_window_attributes(&self, window: u32) -> Result<WindowAttributes, Box<dyn Error>>;

    fn query_monitors(&self) -> Result<Vec<MonitorInfo>, Box<dyn Error>>;

    fn list_core_font_families(&self) -> Vec<String> {
        Vec::new()
    }

    fn restack_windows(&self, windows: &[u32]) -> Result<(), Box<dyn Error>>;

    fn create_render_picture(&self, _pixmap: u32, _depth: u8) -> Result<u32, Box<dyn Error>> {
        Err("create_render_picture not implemented".into())
    }

    fn free_render_picture(&self, _picture: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn render_format_for_depth(&self, _depth: u8) -> Option<u32> {
        None
    }

    fn set_window_opacity(
        &self,
        _window: u32,
        _opacity: f32,
        _opacity_atom: u32,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn grab_root_window(&self) -> Result<(u16, u16, Vec<u8>), Box<dyn Error>> {
        Err("grab_root_window not implemented".into())
    }

    fn send_selection_notify(
        &self,
        _requestor: u32,
        _selection: u32,
        _target: u32,
        _property: u32,
        _time: u32,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn set_win_gravity(&self, _window: u32, _gravity: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn define_cursor(&self, _window: u32, _cursor: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn undefine_cursor(&self, _window: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn free_cursor(&self, _cursor: u32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn create_cursor_from_pixmap(
        &self,
        _source: u32,
        _mask: u32,
        _fore: [u16; 3],
        _back: [u16; 3],
        _hotspot: Point,
    ) -> Result<u32, Box<dyn Error>> {
        Err("create_cursor_from_pixmap not implemented".into())
    }

    fn create_font_cursor(&self, _glyph: u32) -> Result<u32, Box<dyn Error>> {
        Err("create_font_cursor not implemented".into())
    }

    fn create_cursor_from_rgba(
        &self,
        _pixels: &[u8],
        _size: Dimension,
        _hotspot: Point,
        _foreground: [u8; 3],
        _background: [u8; 3],
    ) -> Result<u32, Box<dyn Error>> {
        Err("create_cursor_from_rgba not implemented".into())
    }

    fn create_named_cursor(&self, _name: &str) -> Result<u32, Box<dyn Error>> {
        Err("create_named_cursor not implemented".into())
    }
}
