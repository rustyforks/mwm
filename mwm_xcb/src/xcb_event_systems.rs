use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use log::{trace, warn};

use crate::component::XWinId;
use crate::xcb_event_type::XcbEventType;
use crate::xconn::XConn;
use crate::{event as ev, Point, Region};

/// Polls as many XCB events as are in the queue
pub fn _poll_xcb_events(xconn: Res<XConn>) -> Vec<xcb::GenericEvent> {
    let mut buf = Vec::new();
    while let Some(ev) = xconn.poll_for_event() {
        buf.push(ev);
    }
    buf
}

/// Blocks until at least one XCB event arrives and then polls as many as are in
/// the queue
///
/// Uses `ResMut` even though it only needs shared access to force blocking the
/// bevy event loop.
pub fn wait_for_xcb_events(xconn: ResMut<XConn>) -> Vec<xcb::GenericEvent> {
    let mut buf = Vec::with_capacity(1);
    if let Some(ev) = xconn.wait_for_event() {
        buf.push(ev);
    } else {
        return buf;
    }
    while let Some(ev) = xconn.poll_for_event() {
        buf.push(ev);
    }
    buf
}

/// Blocks until all buffered XCB requests are sent
///
/// Uses `ResMut` even though it only needs shared access to force blocking the
/// bevy event loop.
pub fn flush_xcb(xconn: ResMut<XConn>) {
    let res = xconn.flush();
    if !res {
        warn!("xcb flush returned error");
    }
}

// temporary workaround to not having any output handling
//
// this system runs once at startup, ensures there is only one screen connected
// and generates one "screen added" event
pub fn add_singleton_output(xconn: Res<XConn>, mut ev_screen_added: EventWriter<ev::ScreenAdded>) {
    let mut outputs = xconn.current_outputs();
    assert!(outputs.len() == 1);
    let output = outputs.pop().unwrap();
    let e = ev::ScreenAdded { name: output.name, region: output.region };
    trace!("received event {e:?}");
    ev_screen_added.send(e);
}

/// Processes XCB events
///
/// Converts xcb::GenericEvent into concrete event types and forwards them as
/// bevy events
///
/// # Safety
/// Using this function is safe.
///
/// Internaly, this function casts (transmutes) `&xcb::GenericEvent` to
/// references to concrete event types based on the tag provided by
/// `GenericEvent::response_type`.  The safety of these transmutes
/// relies on matching the tags with the types.
///
/// Make sure to check the types thoroughly!
pub fn process_xcb_events(
    In(events): In<Vec<xcb::GenericEvent>>,
    // mut ev_client_message: EventWriter<ev::ClientMessage>,
    // mut ev_property_notify: EventWriter<ev::PropertyNotify>,
    // mut ev_randr_notify: EventWriter<ev::RandrNotify>,
    // mut ev_screen_change: EventWriter<ev::ScreenChange>,
    mut ev_button_press: EventWriter<ev::ButtonPress>,
    mut ev_button_release: EventWriter<ev::ButtonRelease>,
    mut ev_configure_notify: EventWriter<ev::ConfigureNotify>,
    mut ev_configure_request: EventWriter<ev::ConfigureRequest>,
    mut ev_create_notify: EventWriter<ev::CreateNotify>,
    mut ev_destroy_notify: EventWriter<ev::DestroyNotify>,
    mut ev_enter_notify: EventWriter<ev::EnterNotify>,
    mut ev_focus_in: EventWriter<ev::FocusIn>,
    mut ev_focus_out: EventWriter<ev::FocusOut>,
    mut ev_key_press: EventWriter<ev::KeyPress>,
    mut ev_leave_notify: EventWriter<ev::LeaveNotify>,
    mut ev_map_notify: EventWriter<ev::MapNotify>,
    mut ev_map_request: EventWriter<ev::MapRequest>,
    mut ev_motion_notify: EventWriter<ev::MotionNotify>,
    mut ev_unmap_notify: EventWriter<ev::UnmapNotify>,
) {
    for event in events.into_iter() {
        let etype = match XcbEventType::try_from(event.response_type()) {
            Ok(etype) => etype,
            Err(unknown) => {
                warn!("unrecognized event type {unknown:#x}");
                continue;
            },
        };

        match etype {
            XcbEventType::ButtonPress => {
                let e = unsafe { xcb::cast_event::<xcb::ButtonPressEvent>(&event) };
                let e = ev::ButtonPress {
                    detail: e.detail(),
                    time: e.time(),
                    root: XWinId::from_raw(e.root()),
                    event: XWinId::from_raw(e.event()),
                    child: XWinId::from_raw(e.child()),
                    root_pos: Point {
                        x: e.root_x().into(),
                        y: e.root_y().into(),
                    },
                    event_pos: Point {
                        x: e.event_x().into(),
                        y: e.event_y().into(),
                    },
                    state: e.state(),
                    same_screen: e.same_screen(),
                };
                trace!("received event {e:?}");
                ev_button_press.send(e);
            },

            XcbEventType::ButtonRelease => {
                let e = unsafe { xcb::cast_event::<xcb::ButtonPressEvent>(&event) };
                let e = ev::ButtonRelease {
                    detail: e.detail(),
                    time: e.time(),
                    root: XWinId::from_raw(e.root()),
                    event: XWinId::from_raw(e.event()),
                    child: XWinId::from_raw(e.child()),
                    root_pos: Point {
                        x: e.root_x().into(),
                        y: e.root_y().into(),
                    },
                    event_pos: Point {
                        x: e.event_x().into(),
                        y: e.event_y().into(),
                    },
                    state: e.state(),
                    same_screen: e.same_screen(),
                };
                trace!("received event {e:?}");
                ev_button_release.send(e);
            },

            XcbEventType::ConfigureNotify => {
                let e = unsafe { xcb::cast_event::<xcb::ConfigureNotifyEvent>(&event) };
                let e = ev::ConfigureNotify {
                    event: XWinId::from_raw(e.event()),
                    window: XWinId::from_raw(e.window()),
                    above_sibling: XWinId::from_raw_nullable(e.above_sibling()),
                    region: Region {
                        x: e.x().into(),
                        y: e.y().into(),
                        w: e.width().into(),
                        h: e.height().into(),
                    },
                    border_width: e.border_width(),
                    override_redirect: e.override_redirect(),
                };
                trace!("received event {e:?}");
                ev_configure_notify.send(e);
            },

            XcbEventType::ConfigureRequest => {
                let e = unsafe { xcb::cast_event::<xcb::ConfigureRequestEvent>(&event) };
                let e = ev::ConfigureRequest {
                    stack_mode: e.stack_mode(),
                    parent: XWinId::from_raw(e.parent()),
                    window: XWinId::from_raw(e.window()),
                    sibling: XWinId::from_raw_nullable(e.sibling()),
                    region: Region {
                        x: e.x().into(),
                        y: e.y().into(),
                        w: e.width().into(),
                        h: e.height().into(),
                    },
                    border_width: e.border_width(),
                    value_mask: e.value_mask(),
                };
                trace!("received event {e:?}");
                ev_configure_request.send(e);
            },

            XcbEventType::CreateNotify => {
                let e = unsafe { xcb::cast_event::<xcb::CreateNotifyEvent>(&event) };
                let e = ev::CreateNotify {
                    parent: XWinId::from_raw(e.parent()),
                    window: XWinId::from_raw(e.window()),
                    region: Region {
                        x: e.x().into(),
                        y: e.y().into(),
                        w: e.width().into(),
                        h: e.height().into(),
                    },
                    border_width: e.border_width(),
                    override_redirect: e.override_redirect(),
                };
                trace!("received event {e:?}");
                ev_create_notify.send(e);
            },

            XcbEventType::DestroyNotify => {
                let e = unsafe { xcb::cast_event::<xcb::DestroyNotifyEvent>(&event) };
                let e = ev::DestroyNotify {
                    event: XWinId::from_raw(e.event()),
                    window: XWinId::from_raw(e.window()),
                };
                trace!("received event {e:?}");
                ev_destroy_notify.send(e);
            },

            XcbEventType::EnterNotify => {
                let e = unsafe { xcb::cast_event::<xcb::EnterNotifyEvent>(&event) };
                let e = ev::EnterNotify {
                    detail: e.detail(),
                    time: e.time(),
                    root: XWinId::from_raw(e.root()),
                    event: XWinId::from_raw(e.event()),
                    child: XWinId::from_raw(e.child()),
                    root_pos: Point {
                        x: e.root_x().into(),
                        y: e.root_y().into(),
                    },
                    event_pos: Point {
                        x: e.event_x().into(),
                        y: e.event_y().into(),
                    },
                    state: e.state(),
                    mode: e.mode(),
                    same_screen_focus: e.same_screen_focus(),
                };
                trace!("received event {e:?}");
                ev_enter_notify.send(e);
            },

            XcbEventType::FocusIn => {
                let e = unsafe { xcb::cast_event::<xcb::FocusInEvent>(&event) };
                let e = ev::FocusIn {
                    detail: e.detail(),
                    event: XWinId::from_raw(e.event()),
                    mode: e.mode(),
                };
                trace!("received event {e:?}");
                ev_focus_in.send(e);
            },

            XcbEventType::FocusOut => {
                let e = unsafe { xcb::cast_event::<xcb::FocusOutEvent>(&event) };
                let e = ev::FocusOut {
                    detail: e.detail(),
                    event: XWinId::from_raw(e.event()),
                    mode: e.mode(),
                };
                trace!("received event {e:?}");
                ev_focus_out.send(e);
            },

            XcbEventType::KeyPress => {
                let e = unsafe { xcb::cast_event::<xcb::KeyPressEvent>(&event) };
                let e = ev::KeyPress {
                    detail: e.detail(),
                    time: e.time(),
                    root: XWinId::from_raw(e.root()),
                    event: XWinId::from_raw(e.event()),
                    child: XWinId::from_raw(e.child()),
                    root_pos: Point {
                        x: e.root_x().into(),
                        y: e.root_y().into(),
                    },
                    event_pos: Point {
                        x: e.event_x().into(),
                        y: e.event_y().into(),
                    },
                    state: e.state(),
                    same_screen: e.same_screen(),
                };
                trace!("received event {e:?}");
                ev_key_press.send(e);
            },

            XcbEventType::LeaveNotify => {
                let e = unsafe { xcb::cast_event::<xcb::LeaveNotifyEvent>(&event) };
                let e = ev::LeaveNotify {
                    detail: e.detail(),
                    time: e.time(),
                    root: XWinId::from_raw(e.root()),
                    event: XWinId::from_raw(e.event()),
                    child: XWinId::from_raw(e.child()),
                    root_pos: Point {
                        x: e.root_x().into(),
                        y: e.root_y().into(),
                    },
                    event_pos: Point {
                        x: e.event_x().into(),
                        y: e.event_y().into(),
                    },
                    state: e.state(),
                    mode: e.mode(),
                    same_screen_focus: e.same_screen_focus(),
                };
                trace!("received event {e:?}");
                ev_leave_notify.send(e);
            },

            XcbEventType::MapNotify => {
                let e = unsafe { xcb::cast_event::<xcb::MapNotifyEvent>(&event) };
                let e = ev::MapNotify {
                    event: XWinId::from_raw(e.event()),
                    window: XWinId::from_raw(e.window()),
                    override_redirect: e.override_redirect(),
                };
                trace!("received event {e:?}");
                ev_map_notify.send(e);
            },

            XcbEventType::MapRequest => {
                let e = unsafe { xcb::cast_event::<xcb::MapRequestEvent>(&event) };
                let e = ev::MapRequest {
                    parent: XWinId::from_raw(e.parent()),
                    window: XWinId::from_raw(e.window()),
                };
                trace!("received event {e:?}");
                ev_map_request.send(e);
            },

            XcbEventType::MotionNotify => {
                let e = unsafe { xcb::cast_event::<xcb::MotionNotifyEvent>(&event) };
                let e = ev::MotionNotify {
                    detail: e.detail(),
                    time: e.time(),
                    root: XWinId::from_raw(e.root()),
                    event: XWinId::from_raw(e.event()),
                    child: XWinId::from_raw(e.child()),
                    root_pos: Point {
                        x: e.root_x().into(),
                        y: e.root_y().into(),
                    },
                    event_pos: Point {
                        x: e.event_x().into(),
                        y: e.event_y().into(),
                    },
                    state: e.state(),
                    same_screen: e.same_screen(),
                };
                trace!("received event {e:?}");
                ev_motion_notify.send(e);
            },

            XcbEventType::UnmapNotify => {
                let e = unsafe { xcb::cast_event::<xcb::UnmapNotifyEvent>(&event) };
                let e = ev::UnmapNotify {
                    event: XWinId::from_raw(e.event()),
                    window: XWinId::from_raw(e.window()),
                    from_configure: e.from_configure(),
                };
                trace!("received event {e:?}");
                ev_unmap_notify.send(e);
            },

            // XcbEventType::ClientMessage => {
            //     let e = unsafe { xcb::cast_event::<xcb::ClientMessageEvent>(&event) };
            //     // TODO - I don't think this needs to query the X server for the atom name.
            //     //        We should be able to just use the already interned atoms.
            //     xcb::xproto::get_atom_name(&self.conn, e.type_())
            //         .get_reply()
            //         .ok()
            //         .and_then(|a| {
            //             Some(XEvent::ClientMessage(ev::ClientMessage {
            //                 id: XWinId(e.window()),
            //                 dtype: a.name().to_string(),
            //                 data: match e.format() {
            //                     // see the documentation [`xcb::xproto::ClientMessageEvent`]
            //                     8 => ClientMessageData::U8(e.data().data8().to_vec()),
            //                     16 => ClientMessageData::U16(e.data().data16().to_vec()),
            //                     32 => ClientMessageData::U32(e.data().data32().to_vec()),
            //                     _ => unreachable!(),
            //                 },
            //             }))
            //         })
            // },

            // XcbEventType::PropertyNotify => {
            //     let e = unsafe { xcb::cast_event::<xcb::PropertyNotifyEvent>(&event) };
            //     // TODO - I don't think this needs to query the X server for the atom name.
            //     //        We should be able to just use the already interned atoms.
            //     //        At least for WM_NAME and _NET_WM_NAME.
            //     xcb::xproto::get_atom_name(&self.conn, e.atom())
            //         .get_reply()
            //         .ok()
            //         .and_then(|a| {
            //             let atom = a.name().to_string();
            //             let is_root = XWinId(e.window()) == self.root;
            //             if is_root && !(atom == "WM_NAME" || atom == "_NET_WM_NAME") {
            //                 None
            //             } else {
            //                 let id = XWinId(e.window());
            //                 Some(XEvent::PropertyNotify(ev::PropertyNotify {
            //                     id,
            //                     atom,
            //                     is_root,
            //                 }))
            //             }
            //         })
            // },

            // XcbEventType::ScreenChangeNotify => Some(XEvent::ScreenChange(ev::ScreenChange)),

            // XcbEventType::RandrNotify => Some(XEvent::RandrNotify(ev::RandrNotify)),

            // ignoring other event types
            _ => trace!("ignoring received event type {etype:?}"),
        }
    }
}
