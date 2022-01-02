use bevy_app::prelude::*;

fn main() {
    pretty_env_logger::init();

    App::build()
        .add_plugin(mwm_xcb::XcbPlugin)
        .set_runner(|mut app| loop {
            app.update();
        })
        .run()
}
