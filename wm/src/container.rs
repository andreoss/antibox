use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
pub fn handle_click<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, window: u32) {
    if wm.frames.values().any(|fw| fw.frame().id() == window) {
        return;
    }
    let Some(client_id) = wm.cid_for_xid(window) else { return };
    if wm.frames.contains_key(&client_id) {
        crate::focus::focus_window(wm, client_id);
    }
}
#[cfg(test)]
#[path = "container_tests.rs"]
mod tests;
