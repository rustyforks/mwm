use anyhow::Context;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use log::{trace, warn};
use xcb::{randr, x};

use crate::event as ev;
use crate::xconn::XConn;

fn parse_xcb_error<T>(r: xcb::Result<T>) -> xcb::Result<Option<T>> {
    match r {
        Ok(ok) => Ok(Some(ok)),
        Err(xcb::Error::Protocol(xcb::ProtocolError::UnknownEvent)) => {
            warn!("received unknown X event");
            Ok(None)
        },
        Err(err) => Err(err),
    }
}

/// Polls as many XCB events as are in the queue
pub fn _poll_xcb_events(xconn: Res<XConn>) -> Vec<xcb::Event> {
    let mut buf = Vec::new();
    while let Some(ev) = parse_xcb_error(xconn.conn.poll_for_event()).expect("xcb error") {
        buf.extend(ev);
    }
    buf
}

/// Blocks until at least one XCB event arrives and then polls as many as are in
/// the queue
///
/// Uses `ResMut` even though it only needs shared access to force blocking the
/// bevy event loop.
pub fn wait_for_xcb_events(xconn: ResMut<XConn>) -> Vec<xcb::Event> {
    let mut buf = Vec::with_capacity(1);
    let ev = parse_xcb_error(xconn.conn.wait_for_event()).expect("xcb error");
    buf.extend(ev);
    loop {
        match xconn.conn.poll_for_queued_event() {
            Ok(None) => break,
            Ok(Some(ev)) => buf.push(ev),
            Err(xcb::ProtocolError::UnknownEvent) => {},
            Err(err) => Err(err).expect("xcb error"),
        }
    }
    buf
}

/// Blocks until all buffered XCB requests are sent
///
/// Uses `ResMut` even though it only needs shared access to force blocking the
/// bevy event loop.
pub fn flush_xcb(xconn: ResMut<XConn>) {
    xconn.conn.flush().context("flush").unwrap();
}

/// Dispatches XCB events into their individual `EventWriter`s
pub fn process_xcb_events(
    In(events): In<Vec<xcb::Event>>,
    // xcb::x events
    (
        mut ev_key_press,
        mut ev_key_release,
        mut ev_button_press,
        mut ev_button_release,
        mut ev_motion_notify,
        mut ev_enter_notify,
        mut ev_leave_notify,
        mut ev_focus_in,
        mut ev_focus_out,
        mut ev_keymap_notify,
        mut ev_expose,
        mut ev_graphics_exposure,
        mut ev_no_exposure,
        mut ev_visibility_notify,
        mut ev_create_notify,
        mut ev_destroy_notify,
    ): (
        EventWriter<ev::KeyPress>,
        EventWriter<ev::KeyRelease>,
        EventWriter<ev::ButtonPress>,
        EventWriter<ev::ButtonRelease>,
        EventWriter<ev::MotionNotify>,
        EventWriter<ev::EnterNotify>,
        EventWriter<ev::LeaveNotify>,
        EventWriter<ev::FocusIn>,
        EventWriter<ev::FocusOut>,
        EventWriter<ev::KeymapNotify>,
        EventWriter<ev::Expose>,
        EventWriter<ev::GraphicsExposure>,
        EventWriter<ev::NoExposure>,
        EventWriter<ev::VisibilityNotify>,
        EventWriter<ev::CreateNotify>,
        EventWriter<ev::DestroyNotify>,
    ),
    (
        mut ev_unmap_notify,
        mut ev_map_notify,
        mut ev_map_request,
        mut ev_reparent_notify,
        mut ev_configure_notify,
        mut ev_configure_request,
        mut ev_gravity_notify,
        mut ev_resize_request,
        mut ev_circulate_notify,
        mut ev_circulate_request,
        mut ev_property_notify,
        mut ev_selection_clear,
        mut ev_selection_request,
        mut ev_selection_notify,
        mut ev_colormap_notify,
        mut ev_client_message,
    ): (
        EventWriter<ev::UnmapNotify>,
        EventWriter<ev::MapNotify>,
        EventWriter<ev::MapRequest>,
        EventWriter<ev::ReparentNotify>,
        EventWriter<ev::ConfigureNotify>,
        EventWriter<ev::ConfigureRequest>,
        EventWriter<ev::GravityNotify>,
        EventWriter<ev::ResizeRequest>,
        EventWriter<ev::CirculateNotify>,
        EventWriter<ev::CirculateRequest>,
        EventWriter<ev::PropertyNotify>,
        EventWriter<ev::SelectionClear>,
        EventWriter<ev::SelectionRequest>,
        EventWriter<ev::SelectionNotify>,
        EventWriter<ev::ColormapNotify>,
        EventWriter<ev::ClientMessage>,
    ),
    mut ev_mapping_notify: EventWriter<ev::MappingNotify>,

    // xcb::randr events
    mut ev_screen_change_notify: EventWriter<ev::ScreenChangeNotify>,
    mut ev_notify: EventWriter<ev::Notify>,
) {
    for event in events.into_iter() {
        trace!("received event {event:?}");
        match event {
            xcb::Event::X(event) => match event {
                x::Event::KeyPress(ev) => ev_key_press.send(ev::KeyPress(ev)),
                x::Event::KeyRelease(ev) => ev_key_release.send(ev::KeyRelease(ev)),
                x::Event::ButtonPress(ev) => ev_button_press.send(ev::ButtonPress(ev)),
                x::Event::ButtonRelease(ev) => ev_button_release.send(ev::ButtonRelease(ev)),
                x::Event::MotionNotify(ev) => ev_motion_notify.send(ev::MotionNotify(ev)),
                x::Event::EnterNotify(ev) => ev_enter_notify.send(ev::EnterNotify(ev)),
                x::Event::LeaveNotify(ev) => ev_leave_notify.send(ev::LeaveNotify(ev)),
                x::Event::FocusIn(ev) => ev_focus_in.send(ev::FocusIn(ev)),
                x::Event::FocusOut(ev) => ev_focus_out.send(ev::FocusOut(ev)),
                x::Event::KeymapNotify(ev) => ev_keymap_notify.send(ev::KeymapNotify(ev)),
                x::Event::Expose(ev) => ev_expose.send(ev::Expose(ev)),
                x::Event::GraphicsExposure(ev) => {
                    ev_graphics_exposure.send(ev::GraphicsExposure(ev))
                },
                x::Event::NoExposure(ev) => ev_no_exposure.send(ev::NoExposure(ev)),
                x::Event::VisibilityNotify(ev) => {
                    ev_visibility_notify.send(ev::VisibilityNotify(ev))
                },
                x::Event::CreateNotify(ev) => ev_create_notify.send(ev::CreateNotify(ev)),
                x::Event::DestroyNotify(ev) => ev_destroy_notify.send(ev::DestroyNotify(ev)),
                x::Event::UnmapNotify(ev) => ev_unmap_notify.send(ev::UnmapNotify(ev)),
                x::Event::MapNotify(ev) => ev_map_notify.send(ev::MapNotify(ev)),
                x::Event::MapRequest(ev) => ev_map_request.send(ev::MapRequest(ev)),
                x::Event::ReparentNotify(ev) => ev_reparent_notify.send(ev::ReparentNotify(ev)),
                x::Event::ConfigureNotify(ev) => ev_configure_notify.send(ev::ConfigureNotify(ev)),
                x::Event::ConfigureRequest(ev) => {
                    ev_configure_request.send(ev::ConfigureRequest(ev))
                },
                x::Event::GravityNotify(ev) => ev_gravity_notify.send(ev::GravityNotify(ev)),
                x::Event::ResizeRequest(ev) => ev_resize_request.send(ev::ResizeRequest(ev)),
                x::Event::CirculateNotify(ev) => ev_circulate_notify.send(ev::CirculateNotify(ev)),
                x::Event::CirculateRequest(ev) => {
                    ev_circulate_request.send(ev::CirculateRequest(ev))
                },
                x::Event::PropertyNotify(ev) => ev_property_notify.send(ev::PropertyNotify(ev)),
                x::Event::SelectionClear(ev) => ev_selection_clear.send(ev::SelectionClear(ev)),
                x::Event::SelectionRequest(ev) => {
                    ev_selection_request.send(ev::SelectionRequest(ev))
                },
                x::Event::SelectionNotify(ev) => ev_selection_notify.send(ev::SelectionNotify(ev)),
                x::Event::ColormapNotify(ev) => ev_colormap_notify.send(ev::ColormapNotify(ev)),
                x::Event::ClientMessage(ev) => ev_client_message.send(ev::ClientMessage(ev)),
                x::Event::MappingNotify(ev) => ev_mapping_notify.send(ev::MappingNotify(ev)),
            },
            xcb::Event::RandR(event) => match event {
                randr::Event::ScreenChangeNotify(ev) => {
                    ev_screen_change_notify.send(ev::ScreenChangeNotify(ev))
                },
                randr::Event::Notify(ev) => ev_notify.send(ev::Notify(ev)),
            },
        }
    }
}
