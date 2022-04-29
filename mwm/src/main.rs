use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use mwm_xcb::component::{IsManaged, Window};
use mwm_xcb::event as ev;
use mwm_xcb::request::RequestMap;

fn main() {
    pretty_env_logger::init();

    App::new()
        .add_plugin(mwm_xcb::XcbPlugin::default())
        .add_system(map_all_windows)
        .set_runner(|mut app| loop {
            app.update();
        })
        .run()
}

fn map_all_windows(
    mut events: EventReader<ev::MapRequest>,
    query: Query<(Entity, &Window), With<IsManaged>>,
    mut commands: Commands,
) {
    for e in events.iter() {
        for (entity, &window) in query.iter() {
            if window == e.window() {
                commands.entity(entity).insert(RequestMap::Map);
            }
        }
    }
}
