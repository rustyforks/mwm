use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use log::{trace, warn};

use crate::event_type::XcbEventType;
use crate::xconn::XConn;
use crate::{events as ev, MouseButton, Region, XWinId};

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
    xconn: ResMut<XConn>,
    mut ev_button_press: EventWriter<ev::ButtonPress>,
    mut ev_button_release: EventWriter<ev::ButtonRelease>,
    // mut ev_client_message: EventWriter<ev::ClientMessage>,
    // mut ev_configure_notify: EventWriter<ev::ConfigureNotify>,
    mut ev_configure_request: EventWriter<ev::ConfigureRequest>,
    mut ev_create_notify: EventWriter<ev::CreateNotify>,
    mut ev_destroy_notify: EventWriter<ev::DestroyNotify>,
    // mut ev_enter: EventWriter<ev::Enter>,
    // mut ev_focus_in: EventWriter<ev::FocusIn>,
    // mut ev_focus_out: EventWriter<ev::FocusOut>,
    // mut ev_key_press: EventWriter<ev::KeyPress>,
    // mut ev_leave: EventWriter<ev::Leave>,
    mut ev_map_request: EventWriter<ev::MapRequest>,
    // mut ev_motion_notify: EventWriter<ev::MotionNotify>,
    // mut ev_property_notify: EventWriter<ev::PropertyNotify>,
    // mut ev_randr_notify: EventWriter<ev::RandrNotify>,
    // mut ev_screen_change: EventWriter<ev::ScreenChange>,
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
                    id: XWinId::from_raw(e.event()),
                    btn: match MouseButton::from_detail(e.detail()) {
                        Some(btn) => btn,
                        None => {
                            warn!("unrecognized mouse button {}", e.detail());
                            continue;
                        },
                    },
                };
                trace!("received event {e:?}");
                ev_button_press.send(e);
            },

            XcbEventType::ButtonRelease => {
                let e = unsafe { xcb::cast_event::<xcb::ButtonPressEvent>(&event) };
                let e = ev::ButtonRelease {
                    btn: match MouseButton::from_detail(e.detail()) {
                        Some(btn) => btn,
                        None => {
                            warn!("unrecognized mouse button {}", e.detail());
                            continue;
                        },
                    },
                };
                trace!("received event {e:?}");
                ev_button_release.send(e);
            },

            // XcbEventType::ConfigureNotify => {
            //     let e = unsafe { xcb::cast_event::<xcb::ConfigureNotifyEvent>(&event) };
            //     let window = XWinId::from_raw(e.window());
            //     let above_sibling = e.above_sibling();
            //     let e = ev::ConfigureNotify {
            //         event: XWinId::from_raw(e.event()),
            //         window,
            //         above_sibling: XWinId::from_raw_nullable(above_sibling),
            //         region: Region {
            //             x: e.x().into(),
            //             y: e.y().into(),
            //             w: e.width().into(),
            //             h: e.height().into(),
            //         },
            //         border_width: e.border_width(),
            //         override_redirect: e.override_redirect(),
            //         is_root: window == xconn.root(),
            //     };
            //     trace!("received event {e:?}");
            //     ev_configure_notify.send(e);
            // },
            XcbEventType::ConfigureRequest => {
                let e = unsafe { xcb::cast_event::<xcb::ConfigureRequestEvent>(&event) };
                let window = XWinId::from_raw(e.window());
                let e = ev::ConfigureRequest {
                    stack_mode: e.stack_mode(),
                    parent: XWinId::from_raw(e.parent()),
                    window,
                    sibling: XWinId::from_raw_nullable(e.sibling()),
                    region: Region {
                        x: e.x().into(),
                        y: e.y().into(),
                        w: e.width().into(),
                        h: e.height().into(),
                    },
                    border_width: e.border_width(),
                    value_mask: e.value_mask(),
                    is_root: window == xconn.root(),
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

            XcbEventType::MapRequest => {
                let e = unsafe { xcb::cast_event::<xcb::MapRequestEvent>(&event) };
                // let id = XWinId(e.window());
                // let override_redirect = match xconn.get_window_attributes(id) {
                //     Ok(reply) => reply.override_redirect(),
                //     Err(err) => {
                //         warn!("unable to get override_redirect information {err}");
                //         false
                //     },
                // };
                let e = ev::MapRequest {
                    parent: XWinId::from_raw(e.parent()),
                    window: XWinId::from_raw(e.window()),
                };
                trace!("received event {e:?}");
                ev_map_request.send(e);
            },

            // XcbEventType::MotionNotify => {
            //     let e = unsafe { xcb::cast_event::<xcb::MotionNotifyEvent>(&event) };
            //     Some(XEvent::MotionNotify(ev::MotionNotify {
            //         id: XWinId(e.event()),
            //         rpt: Point {
            //             x: e.root_x() as u32,
            //             y: e.root_y() as u32,
            //         },
            //         wpt: Point {
            //             x: e.event_x() as u32,
            //             y: e.event_y() as u32,
            //         },
            //     }))
            // },

            // XcbEventType::KeyPress => {
            //     let e = unsafe { xcb::cast_event::<xcb::KeyPressEvent>(&event) };
            //     Some(XEvent::KeyPress(ev::KeyPress {
            //         key: KeyCode::from_event(e),
            //     }))
            // },

            // XcbEventType::EnterNotify => {
            //     let e = unsafe { xcb::cast_event::<xcb::EnterNotifyEvent>(&event) };
            //     Some(XEvent::Enter(ev::Enter { id: XWinId(e.event()) }))
            // },

            // XcbEventType::LeaveNotify => {
            //     let e = unsafe { xcb::cast_event::<xcb::LeaveNotifyEvent>(&event) };
            //     Some(XEvent::Leave(ev::Leave { id: XWinId(e.event()) }))
            // },

            // XcbEventType::FocusIn => {
            //     let e = unsafe { xcb::cast_event::<xcb::FocusInEvent>(&event) };
            //     Some(XEvent::FocusIn(ev::FocusIn { id: XWinId(e.event()) }))
            // },

            // XcbEventType::FocusOut => {
            //     let e = unsafe { xcb::cast_event::<xcb::FocusOutEvent>(&event) };
            //     Some(XEvent::FocusOut(ev::FocusOut { id: XWinId(e.event()) }))
            // },

            // XcbEventType::DestroyNotify => {
            //     let e = unsafe { xcb::cast_event::<xcb::MapNotifyEvent>(&event) };
            //     Some(XEvent::Destroy(ev::Destroy { id: XWinId(e.window()) }))
            // },

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
