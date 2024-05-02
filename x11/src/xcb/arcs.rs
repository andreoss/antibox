
use super::bindings::xcb_connection_t;
use super::connection::XcbConnection;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    static REGISTRY: RefCell<HashMap<usize, Arc<XcbConnection>>> = RefCell::new(HashMap::new());
}

pub fn register(conn: &Arc<XcbConnection>) {
    let key = conn.raw() as usize;
    REGISTRY.with(|r| {
        r.borrow_mut().insert(key, Arc::clone(conn));
    });
}

pub fn get_arc(conn: *mut xcb_connection_t) -> Arc<XcbConnection> {
    let key = conn as usize;
    REGISTRY.with(|r| {
        r.borrow()
            .get(&key)
            .map_or_else(|| panic!("xcb connection {key:#x} not registered"), Arc::clone)
    })
}
