
use super::bindings::*;
use super::connection::XcbConnection;
use antibox_core::backend::BackendEvent;
use antibox_core::point::Point;
use antibox_core::rect::Rect;

const XCB_KEY_PRESS: u8 = 2;
const XCB_KEY_RELEASE: u8 = 3;
const XCB_BUTTON_PRESS: u8 = 4;
const XCB_BUTTON_RELEASE: u8 = 5;
const XCB_MOTION_NOTIFY: u8 = 6;
const XCB_ENTER_NOTIFY: u8 = 7;
const XCB_LEAVE_NOTIFY: u8 = 8;
const XCB_FOCUS_IN: u8 = 9;
const XCB_FOCUS_OUT: u8 = 10;
const XCB_EXPOSE: u8 = 12;
const XCB_CREATE_NOTIFY: u8 = 16;
const XCB_DESTROY_NOTIFY: u8 = 17;
const XCB_UNMAP_NOTIFY: u8 = 18;
const XCB_MAP_NOTIFY: u8 = 19;
const XCB_MAP_REQUEST: u8 = 20;
const XCB_REPARENT_NOTIFY: u8 = 21;
const XCB_CONFIGURE_NOTIFY: u8 = 22;
const XCB_CONFIGURE_REQUEST: u8 = 23;
const XCB_PROPERTY_NOTIFY: u8 = 28;
const XCB_SELECTION_CLEAR: u8 = 29;
const XCB_SELECTION_REQUEST: u8 = 30;
const XCB_SELECTION_NOTIFY: u8 = 31;
const XCB_CLIENT_MESSAGE: u8 = 33;
const XCB_MAPPING_NOTIFY: u8 = 34;

pub fn convert(ev: &xcb_generic_event_t, conn: &XcbConnection) -> Option<BackendEvent> {
    let _ = conn;
    let code = ev.response_type & 0x7f;
    match code {
        XCB_MAP_REQUEST => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_map_request_event_t) };
            Some(BackendEvent::MapRequest { window: e.window })
        }
        XCB_CONFIGURE_REQUEST => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_configure_request_event_t) };
            Some(BackendEvent::ConfigureRequest {
                window: e.window,
                parent: e.parent,
                rect: Rect::new(e.x as i32, e.y as i32, e.width as i32, e.height as i32),
                border_width: e.border_width as u32,
                value_mask: e.value_mask,
            })
        }
        XCB_DESTROY_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_destroy_notify_event_t) };
            Some(BackendEvent::DestroyNotify { window: e.window })
        }
        XCB_UNMAP_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_unmap_notify_event_t) };
            Some(BackendEvent::UnmapNotify { window: e.window })
        }
        XCB_MAP_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_map_notify_event_t) };
            Some(BackendEvent::MapNotify { window: e.window })
        }
        XCB_EXPOSE => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_expose_event_t) };
            Some(BackendEvent::Expose {
                window: e.window,
                rect: Rect::new(e.x as i32, e.y as i32, e.width as i32, e.height as i32),
            })
        }
        XCB_PROPERTY_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_property_notify_event_t) };
            Some(BackendEvent::PropertyNotify {
                window: e.window,
                atom: e.atom,
                state: e.state,
            })
        }
        XCB_CLIENT_MESSAGE => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_client_message_event_t) };
            Some(BackendEvent::ClientMessage {
                window: e.window,
                message_type: e.type_,
                format: e.format,
                data: e.data,
            })
        }
        XCB_BUTTON_PRESS => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_button_press_event_t) };
            Some(BackendEvent::ButtonPress {
                window: e.event,
                event: e.event,
                point: Point::new(e.event_x as i32, e.event_y as i32),
                root: Point::new(e.root_x as i32, e.root_y as i32),
                button: e.detail,
                state: e.state,
            })
        }
        XCB_BUTTON_RELEASE => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_button_press_event_t) };
            Some(BackendEvent::ButtonRelease {
                window: e.event,
                point: Point::new(e.event_x as i32, e.event_y as i32),
                button: e.detail,
            })
        }
        XCB_MOTION_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_motion_notify_event_t) };
            Some(BackendEvent::MotionNotify {
                window: e.event,
                point: Point::new(e.event_x as i32, e.event_y as i32),
                root: Point::new(e.root_x as i32, e.root_y as i32),
                state: e.state,
            })
        }
        XCB_KEY_PRESS => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_key_press_event_t) };
            Some(BackendEvent::KeyPress {
                window: e.event,
                event: e.event,
                keycode: e.detail as u32,
                state: e.state,
            })
        }
        XCB_KEY_RELEASE => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_key_press_event_t) };
            Some(BackendEvent::KeyRelease {
                window: e.event,
                keycode: e.detail as u32,
                state: e.state,
            })
        }
        XCB_ENTER_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_enter_notify_event_t) };
            Some(BackendEvent::EnterNotify { window: e.event, mode: e.mode })
        }
        XCB_LEAVE_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_leave_notify_event_t) };
            Some(BackendEvent::LeaveNotify { window: e.event, mode: e.mode })
        }
        XCB_FOCUS_IN => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_focus_in_event_t) };
            Some(BackendEvent::FocusIn { window: e.event })
        }
        XCB_FOCUS_OUT => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_focus_out_event_t) };
            Some(BackendEvent::FocusOut { window: e.event })
        }
        XCB_CREATE_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_create_notify_event_t) };
            Some(BackendEvent::CreateNotify {
                window: e.window,
                parent: e.parent,
                rect: Rect::new(e.x as i32, e.y as i32, e.width as i32, e.height as i32),
                border_width: e.border_width as u32,
            })
        }
        XCB_REPARENT_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_reparent_notify_event_t) };
            Some(BackendEvent::ReparentNotify {
                window: e.window,
                parent: e.parent,
                x: e.x as i32,
                y: e.y as i32,
            })
        }
        XCB_CONFIGURE_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_configure_notify_event_t) };
            Some(BackendEvent::ConfigureNotify {
                window: e.window,
                rect: Rect::new(e.x as i32, e.y as i32, e.width as i32, e.height as i32),
            })
        }
        XCB_MAPPING_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_mapping_notify_event_t) };
            Some(BackendEvent::MappingNotify {
                request: e.request,
                first_keycode: e.first_keycode,
                count: e.count,
            })
        }
        XCB_SELECTION_NOTIFY => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_selection_notify_event_t) };
            Some(BackendEvent::SelectionNotify {
                requestor: e.requestor,
                selection: e.selection,
                target: e.target,
                property: e.property,
                time: e.time,
            })
        }
        XCB_SELECTION_REQUEST => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_selection_request_event_t) };
            Some(BackendEvent::SelectionRequest {
                owner: e.owner,
                requestor: e.requestor,
                selection: e.selection,
                target: e.target,
                property: e.property,
                time: e.time,
            })
        }
        XCB_SELECTION_CLEAR => {
            let e = unsafe { &*(ev as *const xcb_generic_event_t as *const xcb_selection_clear_event_t) };
            Some(BackendEvent::SelectionClear {
                owner: e.owner,
                selection: e.selection,
                time: e.time,
            })
        }
        _ => None,
    }
}
