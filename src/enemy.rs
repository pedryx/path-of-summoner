use crate::battle::BattleParticipant;
use crate::health_bar::HealthBar;
use crate::loading::TextureAssets;
use crate::stats::Stats;
use crate::GameScreen;
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::Battle), spawn_enemy);
    }
}

#[derive(Component)]
pub struct Enemy;

fn spawn_enemy(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let (camera, camera_transform) = camera.single();
    let spawn_pos = camera
        .ndc_to_world(camera_transform, Vec3::new(0.5, 0., 0.))
        .unwrap();

    commands.spawn((
        SpriteBundle {
            texture: textures.bevy.clone(),
            transform: Transform::from_translation(spawn_pos),
            ..default()
        },
        Stats {
            damage: 5.,
            speed: 1.,
            hp_regeneration: 1.,
            ..default()
        },
        HealthBar { ..default() },
        Enemy,
        BattleParticipant::default(),
    ));
}
