use super::bindings::*;
use super::connection::XcbConnection;
use antibox_core::backend::{KeyboardInfo, RenderBackend};
use antibox_core::libc;
use std::os::raw::c_uint;

pub(crate) fn init(conn: *mut xcb_connection_t) -> u8 {
    let name = b"XKEYBOARD";
    let cookie = unsafe { xcb_query_extension(conn, name.len() as u16, name.as_ptr() as *const _) };
    let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
    antibox_core::metrics::bump_round_trips();
        let r = unsafe { xcb_query_extension_reply(conn, cookie, &mut e) };
    if r.is_null() {
        return 0;
    }
    let present = unsafe { (*r).present != 0 };
    let first_event = unsafe { (*r).first_event };
    unsafe { libc::free(r as *mut libc::c_void) };
    if !present || first_event == 0 {
        return 0;
    }
    let cookie = unsafe { xcb_xkb_use_extension(conn, 1, 0) };
    antibox_core::metrics::bump_round_trips();
        let r = unsafe { xcb_xkb_use_extension_reply(conn, cookie, &mut e) };
    if r.is_null() {
        return 0;
    }
    let supported = unsafe { (*r).supported != 0 };
    unsafe { libc::free(r as *mut libc::c_void) };
    if !supported {
        return 0;
    }
    unsafe {
        xcb_xkb_select_events(
            conn,
            XCB_XKB_ID_USE_CORE_KBD,
            XCB_XKB_EVENT_TYPE_STATE_NOTIFY,
            0,
            XCB_XKB_EVENT_TYPE_STATE_NOTIFY,
            0,
            0,
            std::ptr::null(),
        );
    }
    first_event
}

fn current_group(conn: &XcbConnection) -> usize {
    let cookie = unsafe { xcb_xkb_get_state(conn.raw(), XCB_XKB_ID_USE_CORE_KBD) };
    let mut e: *mut xcb_generic_event_t = std::ptr::null_mut();
    antibox_core::metrics::bump_round_trips();
        let r = unsafe { xcb_xkb_get_state_reply(conn.raw(), cookie, &mut e) };
    if r.is_null() {
        return 0;
    }
    let group = unsafe { (*r).group } as usize;
    unsafe { libc::free(r as *mut libc::c_void) };
    group
}

fn rules_names(conn: &XcbConnection) -> Option<Vec<u8>> {
    let atom = conn.intern_atom("_XKB_RULES_NAMES").ok()?;
    if atom == 0 {
        return None;
    }
    conn.get_property(conn.root().read_id(), atom, 0, 0, 1024)
        .ok()
        .and_then(|v| v)
}

pub(crate) fn group_and_rules(conn: &XcbConnection) -> Option<(usize, Vec<u8>)> {
    let value = rules_names(conn)?;
    Some((current_group(conn), value))
}

pub(crate) fn set_group(conn: &XcbConnection, group: usize) -> bool {
    if group > 3 {
        return false;
    }
    let cookie: c_uint = unsafe {
        xcb_xkb_latch_lock_state(
            conn.raw(),
            XCB_XKB_ID_USE_CORE_KBD,
            0,
            0,
            1,
            group as u8,
            0,
            0,
            0,
        )
    };
    unsafe { xcb_flush(conn.raw()) };
    cookie != 0
}

pub(crate) fn parse_xkb_layout(rules_names: &[u8], group: usize) -> Option<String> {
    let parts: Vec<&[u8]> = rules_names.split(|&b| b == 0).collect();
    let entry = parts.get(group).or_else(|| parts.first())?;
    if entry.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(entry).into_owned())
}

pub(crate) fn parse_xkb_rules(value: &[u8], group: usize) -> Option<KeyboardInfo> {
    if value.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(value);
    let mut parts = text.split('\0');
    let info = KeyboardInfo {
        rules: parts.next().unwrap_or("").to_string(),
        model: parts.next().unwrap_or("").to_string(),
        layouts: parts.next().unwrap_or("").to_string(),
        variants: parts.next().unwrap_or("").to_string(),
        options: parts.next().unwrap_or("").to_string(),
        group,
    };
    if info.layouts.is_empty() {
        None
    } else {
        Some(info)
    }
}

#[cfg(test)]
#[path = "xkb_tests.rs"]
mod tests;
