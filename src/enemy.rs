use crate::summoning::SummoningItem;
use crate::GameScreen;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::Battle), reposition_enemy);
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Clone)]
pub struct DropRewards(pub Vec<SummoningItem>);

fn reposition_enemy(
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
    let (camera, camera_transform) = camera.single();
    let spawn_pos = camera
        .ndc_to_world(camera_transform, Vec3::new(0.5, 0., 0.))
        .unwrap();

    let mut transform = query.single_mut();

    transform.translation = spawn_pos;
}
