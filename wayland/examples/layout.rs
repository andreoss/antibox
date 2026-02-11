use antibox_wayland::ffi::buffer::wlr_buffer;
use antibox_wayland::ffi::wlr::*;
use std::mem::{offset_of, size_of};

fn main() {
    let mut out = String::new();
    macro_rules! p {
        ($s:ident, $f:ident) => {
            out.push_str(&format!(
                "{}.{} {}\n",
                stringify!($s),
                stringify!($f),
                offset_of!($s, $f)
            ));
        };
    }
    macro_rules! s {
        ($s:ident) => {
            out.push_str(&format!("{} SIZE {}\n", stringify!($s), size_of::<$s>()));
        };
    }
    p!(wlr_backend, events);
    p!(wlr_output, name);
    p!(wlr_output, width);
    p!(wlr_output, height);
    p!(wlr_output, scale);
    p!(wlr_output, events);
    p!(wlr_input_device, name);
    p!(wlr_input_device, data);
    p!(wlr_keyboard, modifiers);
    p!(wlr_keyboard, events);
    p!(wlr_keyboard, keycodes);
    p!(wlr_keyboard, num_keycodes);
    p!(wlr_keyboard, xkb_state);
    p!(wlr_keyboard_key_event, keycode);
    p!(wlr_keyboard_key_event, state);
    p!(wlr_pointer_motion_event, delta_x);
    p!(wlr_pointer_motion_event, unaccel_dx);
    p!(wlr_pointer_motion_absolute_event, x);
    p!(wlr_pointer_button_event, button);
    p!(wlr_pointer_button_event, state);
    p!(wlr_pointer_axis_event, delta);
    p!(wlr_pointer_axis_event, delta_discrete);
    p!(wlr_xdg_shell, events);
    p!(wlr_xdg_surface, surface);
    p!(wlr_xdg_surface, role);
    p!(wlr_xdg_surface, geometry);
    p!(wlr_xdg_surface, events);
    p!(wlr_xdg_toplevel, base);
    p!(wlr_xdg_toplevel, title);
    p!(wlr_xdg_toplevel, app_id);
    p!(wlr_xdg_toplevel, current);
    p!(wlr_xdg_toplevel, events);
    p!(wlr_scene_node, enabled);
    p!(wlr_scene_node, x);
    p!(wlr_scene_node, data);
    p!(wlr_scene_tree, children);
    p!(wlr_scene_output, output);
    p!(wlr_scene_buffer, buffer);
    p!(wlr_scene_surface, surface);
    p!(wlr_scene_rect, width);
    p!(wlr_surface_full, mapped);
    p!(wlr_surface_full, events);
    p!(wlr_surface_full, current);
    p!(wlr_buffer, width);
    p!(wlr_buffer, height);
    p!(wlr_buffer, events);
    s!(wlr_buffer);
    s!(wlr_surface_state);
    s!(wlr_scene_node);
    print!("{out}");
}
