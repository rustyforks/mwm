use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use log::debug;

use crate::component::*;
use crate::event as ev;
use crate::request::*;
use crate::xcb_event_systems::*;
use crate::xcb_request_systems::*;
use crate::xconn::XConn;

#[derive(Default)]
pub struct XcbPlugin {}

impl Plugin for XcbPlugin {
    fn build(&self, builder: &mut bevy_app::AppBuilder) {
        builder
            .add_event::<ev::ButtonPress>()
            .add_event::<ev::ButtonRelease>()
            .add_event::<ev::ConfigureNotify>()
            .add_event::<ev::ConfigureRequest>()
            .add_event::<ev::CreateNotify>()
            .add_event::<ev::DestroyNotify>()
            .add_event::<ev::EnterNotify>()
            .add_event::<ev::FocusIn>()
            .add_event::<ev::FocusOut>()
            .add_event::<ev::KeyPress>()
            .add_event::<ev::LeaveNotify>()
            .add_event::<ev::MapNotify>()
            .add_event::<ev::MapRequest>()
            .add_event::<ev::MotionNotify>()
            .add_event::<ev::ScreenAdded>()
            .add_event::<ev::UnmapNotify>()
            // .add_event::<ev::ClientMessage>()
            // .add_event::<ev::PropertyNotify>()
            // .add_event::<ev::RandrNotify>()
            // .add_event::<ev::ScreenChange>()
            .init_resource::<XConn>()
            .add_plugin(crate::diagnostic::UpdateTimePlugin)
            .add_startup_system(add_singleton_output.system())
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new().with_system(
                    wait_for_xcb_events
                        .system()
                        .chain(process_xcb_events.system()),
                ),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(spawn_windows.system())
                    .with_system(despawn_windows.system())
                    .with_system(map_unmanaged_windows.system())
                    .with_system(mark_mapped_windows.system())
                    .with_system(mark_unmapped_windows.system())
                    .with_system(mark_preffered_size_windows.system())
                    .with_system(mark_size_windows.system()),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_system(process_request_map.system())
                    .with_system(process_request_resize.system()),
            )
            .add_system_set_to_stage(
                CoreStage::Last,
                SystemSet::new().with_system(flush_xcb.system()),
            );
    }
}

impl FromWorld for XConn {
    /// Initializes XConn resource
    ///
    /// Creates an XCB connection and attempts to register self as a
    /// substructure_redirect client (a WindowManager)
    fn from_world(_world: &mut World) -> Self {
        let conn = XConn::init().expect("create X server connection");
        conn.substructure_redirect().expect("register as a WM");
        conn
    }
}

/// Reacts to [`ev::CreateNotify`] events and spawns new window entities
fn spawn_windows(mut events: EventReader<ev::CreateNotify>, mut commands: Commands) {
    for &ev::CreateNotify { window, region, override_redirect, .. } in events.iter() {
        let mut entity = commands.spawn();
        debug!("spawn window {window:?}");
        entity.insert_bundle((window, PrefferedSize(region)));
        if !override_redirect {
            entity.insert(IsManaged);
        }
    }
}

/// Reacts to [`ev::DestroyNotify`] events and despawns window entities with
/// matching [`XWinId`]
fn despawn_windows(
    mut events: EventReader<ev::DestroyNotify>,
    query: Query<(Entity, &XWinId)>,
    mut commands: Commands,
) {
    for &ev::DestroyNotify { window, .. } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                debug!("destroy window {id:?}");
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Reacts to [`ev::MapRequest`] for unmanaged windows only and maps them
/// unconditionally as WMs are supposed to
fn map_unmanaged_windows(
    mut events: EventReader<ev::MapRequest>,
    query: Query<(Entity, &XWinId), (Without<IsMapped>, Without<IsManaged>)>,
    mut commands: Commands,
) {
    for &ev::MapRequest { window, .. } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                debug!("map unmanaged window {id:?}");
                commands.entity(entity).insert(RequestMap::Map);
            }
        }
    }
}

/// Reacts to [`ev::MapNotify`], adds [`IsMapped`] marker and clears
/// [`RequestMap`] if present
fn mark_mapped_windows(
    mut events: EventReader<ev::MapNotify>,
    query: Query<(Entity, &XWinId)>,
    mut commands: Commands,
) {
    for &ev::MapNotify { window, .. } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                commands
                    .entity(entity)
                    .remove::<RequestMap>()
                    .insert(IsMapped);
            }
        }
    }
}

/// Reacts to [`ev::UnmapNotify`], removes [`IsMapped`] marker and clears
/// [`RequestMap`] if present
fn mark_unmapped_windows(
    mut events: EventReader<ev::UnmapNotify>,
    query: Query<(Entity, &XWinId)>,
    mut commands: Commands,
) {
    for &ev::UnmapNotify { window, .. } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                commands.entity(entity).remove::<(RequestMap, IsMapped)>();
            }
        }
    }
}

/// Reacts to [`ev::ConfigureRequest`], updates window's preferred
/// size. If the window is not marked [`IsManaged`] it'll also add
/// [`RequestConfigure`]
fn mark_preffered_size_windows(
    mut events: EventReader<ev::ConfigureRequest>,
    query: Query<(Entity, &XWinId, &Option<IsManaged>)>,
    mut commands: Commands,
) {
    for &ev::ConfigureRequest { window, region, .. } in events.iter() {
        for (entity, &id, is_managed) in query.iter() {
            if window == id {
                debug!("configure window {id:?} {region:?}");
                let mut entity = commands.entity(entity);
                entity.insert(PrefferedSize(region));
                if is_managed.is_none() {
                    entity.insert(RequestConfigure(region));
                }
            }
        }
    }
}

/// Reacts to [`ev::ConfigureNotify`] events and updates window's actual size
/// [`Size`]
fn mark_size_windows(
    mut events: EventReader<ev::ConfigureNotify>,
    query: Query<(Entity, &XWinId)>,
    mut commands: Commands,
) {
    for &ev::ConfigureNotify { window, region, .. } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                commands.entity(entity).insert(Size(region));
            }
        }
    }
}
