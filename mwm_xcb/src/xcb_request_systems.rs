use bevy_ecs::prelude::*;

use crate::component::*;
use crate::request::*;
use crate::xconn::XConn;

/// Turn [`RequestMap`] markers into XCB requests
pub fn process_request_map(
    xconn: Res<XConn>,
    query: Query<(Entity, &XWinId, &RequestMap, &Option<IsMapped>), Added<RequestMap>>,
    mut commands: Commands,
) {
    for (entity, &window, request, is_mapped) in query.iter() {
        match (is_mapped.is_some(), request) {
            (false, RequestMap::Map) => {
                // TODO error handling
                xcb::map_window(&xconn.conn, window.as_raw());
            },
            (true, RequestMap::Unmap) => {
                // TODO error handling
                xcb::unmap_window(&xconn.conn, window.as_raw());
            },
            _ => {
                // skip windows which are already in the requested state
            },
        }
        commands.entity(entity).remove::<RequestMap>();
    }
}

/// Turn [`RequestConfigure`] markers into XCB requests
// TODO also handle window borders in the same system as the xcb configure
// request can handle all at once
pub fn process_request_resize(
    xconn: Res<XConn>,
    query: Query<(&XWinId, &RequestConfigure, &Size), Added<RequestConfigure>>,
) {
    for (&window, RequestConfigure(request), Size(size)) in query.iter() {
        let mut cmd = Vec::new();

        if request.x != size.x {
            cmd.push((xcb::CONFIG_WINDOW_X as u16, request.x.try_into().unwrap()));
        }
        if request.y != size.y {
            cmd.push((xcb::CONFIG_WINDOW_Y as u16, request.y.try_into().unwrap()));
        }
        if request.w != size.w {
            cmd.push((xcb::CONFIG_WINDOW_WIDTH as u16, request.w));
        }
        if request.h != size.h {
            cmd.push((xcb::CONFIG_WINDOW_HEIGHT as u16, request.h));
        }

        if !cmd.is_empty() {
            // TODO error handling
            xcb::configure_window(&xconn.conn, window.as_raw(), cmd.as_slice());
        }
    }
}
