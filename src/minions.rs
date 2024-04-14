use crate::GameScreen;
use bevy::prelude::*;

pub const MAX_MINION_COUNT: usize = 4;
const NDC_SPAWN_AREA_SIZE: f32 = 1.6;
const NDC_SPAWN_X: f32 = -0.2;

pub const MINION_SIZE: f32 = 128.;

pub struct MinionsPlugin;

impl Plugin for MinionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::Battle), reposition_minions);
    }
}

#[derive(Component)]
pub struct Minion;

fn reposition_minions(
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut query: Query<&mut Transform, With<Minion>>,
) {
    let (camera, camera_transform) = camera.single();

    for (i, mut transform) in query.iter_mut().enumerate() {
        let ndc_spawn_pos_y =
            (NDC_SPAWN_AREA_SIZE / (MAX_MINION_COUNT + 2) as f32) * (i + 1) as f32 - 1.;
        let spawn_pos = camera
            .ndc_to_world(
                camera_transform,
                Vec3::new(NDC_SPAWN_X, ndc_spawn_pos_y, 0.),
            )
            .unwrap();

        transform.translation = spawn_pos;
    }
}
