use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

const MAX_MINION_COUNT: usize = 4;
const NDC_SPAWN_AREA_SIZE: f32 = 1.6;
const NDC_SPAWN_X: f32 = -0.05;

pub struct MinionsPlugin;

impl Plugin for MinionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_minions);
    }
}

fn spawn_minions(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>
) {
    let (camera, camera_transform) = camera.single();

    for i in 0..MAX_MINION_COUNT {
        let ndc_spawn_pos_y = (NDC_SPAWN_AREA_SIZE / (MAX_MINION_COUNT + 2) as f32) * (i + 1) as f32 - 1.;
        let spawn_pos = camera.ndc_to_world(camera_transform, Vec3::new(NDC_SPAWN_X, ndc_spawn_pos_y, 0.)).unwrap();

        commands.spawn(SpriteBundle {
            texture: textures.bevy.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(64.)),
                ..default()
            },
            transform: Transform::from_translation(spawn_pos),
            ..default()
        });
    }
}