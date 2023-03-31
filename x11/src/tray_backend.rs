use crate::xcb::connection::XcbConnection;
use antibox_core::backend::{DisplayBackend, RenderBackend, TrayBackend};
use std::sync::Arc;

const XEMBED_MAPPED: u32 = 1 << 0;
const ANY_PROPERTY_TYPE: u32 = 0;
const NO_EVENT_MASK: u32 = 0;

pub struct XcbTray {
    conn: Arc<XcbConnection>,
}

impl XcbTray {
    pub fn new(conn: Arc<XcbConnection>) -> XcbTray {
        XcbTray { conn }
    }
}

fn word(data: &[u8], index: usize) -> Option<u32> {
    let start = index * 4;
    let end = start.checked_add(4)?;
    if end > data.len() {
        return None;
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&data[start..end]);
    Some(u32::from_ne_bytes(buf))
}

impl TrayBackend for XcbTray {
    fn composite_redirect(&self, _client: u32) {}

    fn composite_unredirect(&self, _client: u32) {}

    fn create_damage(&self, _client: u32) -> Option<u32> {
        None
    }

    fn destroy_damage(&self, _damage: u32) {}

    fn get_xembed_info(&self, client: u32, xembed_info_atom: u32) -> Option<(u32, bool)> {
        if xembed_info_atom == 0 {
            return None;
        }
        let data = RenderBackend::get_property(
            &*self.conn,
            client,
            xembed_info_atom,
            ANY_PROPERTY_TYPE,
            0,
            2,
        )
        .ok()
        .flatten()?;
        let version = word(&data, 0)?;
        let flags = word(&data, 1)?;
        Some((version, flags & XEMBED_MAPPED != 0))
    }

    fn get_text_property(&self, window: u32, atom: u32) -> Option<String> {
        if atom == 0 {
            return None;
        }
        let data =
            RenderBackend::get_property(&*self.conn, window, atom, ANY_PROPERTY_TYPE, 0, 1024)
                .ok()
                .flatten()?;
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        if end == 0 {
            return None;
        }
        Some(String::from_utf8_lossy(&data[..end]).into_owned())
    }

    fn send_xembed(&self, target: u32, xembed_atom: u32, data: [u32; 5]) {
        if xembed_atom == 0 {
            return;
        }
        let _ = RenderBackend::send_event(
            &*self.conn,
            false,
            target,
            NO_EVENT_MASK,
            xembed_atom,
            &data,
        );
        self.flush();
    }

    fn flush(&self) {
        let _ = DisplayBackend::flush(&*self.conn);
    }
}
