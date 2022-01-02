use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use log::debug;

use crate::plugin::xcb_event_systems::*;
use crate::xconn::XConn;
use crate::{events as ev, Region, XWinId};

mod xcb_event_systems;

pub struct XcbPlugin;

impl Plugin for XcbPlugin {
    fn build(&self, builder: &mut bevy_app::AppBuilder) {
        builder
            .add_event::<ev::ButtonPress>()
            .add_event::<ev::ButtonRelease>()
            // .add_event::<ev::ClientMessage>()
            // .add_event::<ev::ConfigureNotify>()
            .add_event::<ev::ConfigureRequest>()
            .add_event::<ev::CreateNotify>()
            .add_event::<ev::DestroyNotify>()
            // .add_event::<ev::Enter>()
            // .add_event::<ev::FocusIn>()
            // .add_event::<ev::FocusOut>()
            // .add_event::<ev::KeyPress>()
            // .add_event::<ev::Leave>()
            .add_event::<ev::MapRequest>()
            // .add_event::<ev::MotionNotify>()
            // .add_event::<ev::PropertyNotify>()
            // .add_event::<ev::RandrNotify>()
            // .add_event::<ev::ScreenChange>()
            .init_resource::<XConn>()
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
                    .with_system(configure_windows.system())
                    .with_system(map_unmanaged_windows.system()),
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

/// Marks windows without the `override_redirect` flag - windows that should be
/// managed by the window manager
pub struct IsManaged;

/// Holds Region the window last reported as it's preffered dimensions, gets
/// inserted by CreateNotify events and updated by ConfigureRequest events
pub struct PrefferedDimensions(Region);

/// Keeps track of window's parent window
pub struct Parent(XWinId);

/// Responds to CreateNotify events and spawns window entities
fn spawn_windows(mut events: EventReader<ev::CreateNotify>, mut commands: Commands) {
    for &ev::CreateNotify {
        parent,
        window,
        region,
        border_width: _,
        override_redirect,
    } in events.iter()
    {
        let mut entity = commands.spawn();
        debug!("spawn window {window:?}");
        entity.insert_bundle((window, Parent(parent), PrefferedDimensions(region)));
        if !override_redirect {
            entity.insert(IsManaged);
        }
    }
}

/// Responds to DestroyNotify events and despawns window entities
fn despawn_windows(
    mut events: EventReader<ev::DestroyNotify>,
    query: Query<(Entity, &XWinId)>,
    mut commands: Commands,
) {
    for &ev::DestroyNotify { event: _, window } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                debug!("destroy window {id:?}");
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Responds to ConfigureRequest events and updates window's preferred
/// dimensions
fn configure_windows(
    mut events: EventReader<ev::ConfigureRequest>,
    query: Query<(Entity, &XWinId)>,
    mut commands: Commands,
) {
    for &ev::ConfigureRequest {
        stack_mode: _,
        parent: _,
        window,
        sibling: _,
        region,
        border_width: _,
        value_mask: _,
        is_root: _,
    } in events.iter()
    {
        for (entity, &id) in query.iter() {
            if window == id {
                debug!("configure window {id:?} {region:?}");
                commands.entity(entity).insert(PrefferedDimensions(region));
            }
        }
    }
}

/// Marks windows which have been mapped.
pub struct IsMapped;

/// Responds to MapRequest events for unmapped windows only and maps them
/// unconditionally as WMs are supposed to
fn map_unmanaged_windows(
    conn: Res<XConn>,
    mut events: EventReader<ev::MapRequest>,
    query: Query<(Entity, &XWinId), (Without<IsMapped>, Without<IsManaged>)>,
    mut commands: Commands,
) {
    for &ev::MapRequest { parent: _, window } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                debug!("map unmanaged window {id:?}");
                // FIXME handle errors
                conn.map_window(id);
                // FIXME only mark when MapNotify event is received
                commands.entity(entity).insert(IsMapped);
            }
        }
    }
}
