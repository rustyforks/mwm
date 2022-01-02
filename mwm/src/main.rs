use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use mwm_xcb::component::{IsManaged, XWinId};
use mwm_xcb::event as ev;
use mwm_xcb::request::RequestMap;

fn main() {
    pretty_env_logger::init();

    App::build()
        .add_plugin(mwm_xcb::XcbPlugin::default())
        .add_system(map_all_windows.system())
        .set_runner(|mut app| loop {
            app.update();
        })
        .run()
}

fn map_all_windows(
    mut events: EventReader<ev::MapRequest>,
    query: Query<(Entity, &XWinId), With<IsManaged>>,
    mut commands: Commands,
) {
    for &ev::MapRequest { window, .. } in events.iter() {
        for (entity, &id) in query.iter() {
            if window == id {
                commands.entity(entity).insert(RequestMap::Map);
            }
        }
    }
}
