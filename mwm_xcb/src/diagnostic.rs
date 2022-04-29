use std::time::Instant;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use log::debug;

pub struct UpdateTimePlugin;

impl Plugin for UpdateTimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpdateStart>()
            .add_system_to_stage(CoreStage::PreUpdate, update_time_start)
            .add_system_to_stage(CoreStage::PostUpdate, update_time_end);
    }
}


struct UpdateStart(Instant);

impl Default for UpdateStart {
    fn default() -> Self {
        UpdateStart(Instant::now())
    }
}

fn update_time_start(mut update_start: ResMut<UpdateStart>) {
    update_start.0 = Instant::now();
}

fn update_time_end(update_start: Res<UpdateStart>) {
    let time = Instant::now().duration_since(update_start.0);
    debug!("update time {time:?}");
}
