use crate::manager::WindowManager;
use antibox_core::backend::DisplayBackend;
pub fn handle_click<H: DisplayBackend + 'static + ?Sized>(wm: &mut WindowManager<H>, window: u32) {
    if wm.frames.values().any(|fw| fw.frame().id() == window) {
        return;
    }
    let client_id = match wm.cid_for_xid(window) {
        Some(cid) => cid,
        None => return,
    };
    if wm.frames.contains_key(&client_id) {
        crate::focus::focus_window(wm, client_id);
    }
}
#[cfg(test)]
#[path = "container_tests.rs"]
mod tests;
