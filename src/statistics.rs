use bevy::prelude::*;

use crate::GameState;

pub struct StatisticsPlugin;

impl Plugin for StatisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Statistics>().add_systems(
            Update,
            update_statistics.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource, Default)]
pub struct Statistics {
    pub summoned_minions: usize,
    pub elapsed_seconds: f32,
}

fn update_statistics(time: Res<Time>, mut statistics: ResMut<Statistics>) {
    statistics.elapsed_seconds += time.delta_seconds();
}
