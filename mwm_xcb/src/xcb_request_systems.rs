use bevy_ecs::prelude::*;
use log::debug;

use crate::component::*;
use crate::request::*;
use crate::xconn::XConn;

/// Turn [`RequestMap`] markers into XCB requests
pub fn process_request_map(
    xconn: Res<XConn>,
    query: Query<(Entity, &Window, &RequestMap, Option<&IsMapped>), Added<RequestMap>>,
    mut commands: Commands,
) {
    for (entity, &window, request, is_mapped) in query.iter() {
        match (is_mapped.is_some(), request) {
            (false, RequestMap::Map) => {
                // TODO error handling
                debug!("mapping window {window:?}");
                xconn.conn.send_request(&xcb::x::MapWindow { window });
            },
            (true, RequestMap::Unmap) => {
                // TODO error handling
                debug!("unmapping window {window:?}");
                xconn.conn.send_request(&xcb::x::UnmapWindow { window });
            },
            _ => {
                // skip windows which are already in the requested state
            },
        }
        commands.entity(entity).remove::<RequestMap>();
    }
}

/// Turn [`RequestConfigure`] markers into XCB requests
// TODO also handle window borders, sibling and stackmode (if/when we need those
// in the future) in the same system as the xcb configure request can handle all
// at once
pub fn process_request_resize(
    xconn: Res<XConn>,
    query: Query<(&xcb::x::Window, &RequestConfigure, &Size), Added<RequestConfigure>>,
) {
    for (&window, RequestConfigure(request), Size(size)) in query.iter() {
        let mut cmd = Vec::new();

        if request.x != size.x {
            cmd.push(xcb::x::ConfigWindow::X(request.x));
        }
        if request.y != size.y {
            cmd.push(xcb::x::ConfigWindow::Y(request.y));
        }
        if request.w != size.w {
            cmd.push(xcb::x::ConfigWindow::Width(request.w));
        }
        if request.h != size.h {
            cmd.push(xcb::x::ConfigWindow::Height(request.h));
        }

        if !cmd.is_empty() {
            // TODO error handling
            debug!("configuring window {window:?} with {request:?}");
            xconn
                .conn
                .send_request(&xcb::x::ConfigureWindow { window, value_list: cmd.as_slice() });
        }
    }
}
