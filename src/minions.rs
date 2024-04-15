use crate::{
    loading::{FontAssets, TextureAssets},
    mouse_control::Clickable,
    stats::{
        Stats, MINION_DMG_BASE, MINION_DMG_INC, MINION_HP_BASE, MINION_HP_INC,
        MINION_HP_REGEN_BASE, MINION_HP_REGEN_INC, MINION_SPEED_BASE, MINION_SPEED_INC,
    },
    utils::num_to_roman,
    GameScreen, GameState,
};
use bevy::prelude::*;

const NDC_SPAWN_AREA_SIZE: f32 = 2.5;
const NDC_SPAWN_X: f32 = -0.2;

pub const MAX_MINION_COUNT: usize = 4;
pub const MINION_SIZE: f32 = 128.;

const HOVER_WINDOW_SIZE: Vec2 = Vec2::new(192., 192.);
const HOVER_WINDOW_OFFSET: Vec2 = Vec2::new(0., 160.);
const HOVER_WINDOW_Z: f32 = 100.;
const ICON_SIZE: f32 = 32.;

pub struct MinionsPlugin;

impl Plugin for MinionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::Battle), reposition_minions)
            .add_systems(
                Update,
                handle_minion_stats_hover.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct HoverWindow;

#[derive(Component)]
pub struct Minion;

fn reposition_minions(
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut query: Query<&mut Transform, With<Minion>>,
) {
    let (camera, camera_transform) = camera.single();

    for (i, mut transform) in query.iter_mut().enumerate() {
        let ndc_spawn_pos_y =
            (NDC_SPAWN_AREA_SIZE / (MAX_MINION_COUNT + 2) as f32) * (i + 1) as f32 - 1. - 0.1;
        let spawn_pos = camera
            .ndc_to_world(
                camera_transform,
                Vec3::new(NDC_SPAWN_X, ndc_spawn_pos_y, 0.),
            )
            .unwrap();

        transform.translation = spawn_pos;
        transform.translation.z = 0.;
    }
}

fn spawn_icon(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    texture: Handle<Image>,
    position: Vec3,
    tier: u8,
) {
    // tier number
    parent.spawn(Text2dBundle {
        text: Text {
            sections: vec![TextSection::new(
                num_to_roman(tier),
                TextStyle {
                    color: Color::WHITE,
                    font,
                    font_size: ICON_SIZE * 1.2,
                },
            )],
            ..Default::default()
        },
        text_anchor: bevy::sprite::Anchor::CenterRight,
        transform: Transform::from_translation(position - Vec3::new(8., 0., 0.)),
        ..Default::default()
    });

    // icon
    parent.spawn(SpriteBundle {
        texture,
        sprite: Sprite {
            anchor: bevy::sprite::Anchor::CenterLeft,
            custom_size: Some(Vec2::splat(ICON_SIZE)),
            color: Color::CYAN,
            ..Default::default()
        },
        transform: Transform::from_translation(position),
        ..Default::default()
    });
}

fn handle_minion_stats_hover(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    minion_query: Query<(Entity, &Clickable, &Stats), With<Minion>>,
    hover_window_query: Query<Entity, With<HoverWindow>>,
) {
    for (entity, clickable, stats) in minion_query.iter() {
        if clickable.hover_ended {
            for entity in hover_window_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            continue;
        }

        if !clickable.hover_started {
            continue;
        }

        // spawn hover window
        let window_entity = commands
            .spawn((
                SpriteBundle {
                    texture: textures.square.clone(),
                    sprite: Sprite {
                        color: Color::BLACK.with_a(0.95),
                        custom_size: Some(HOVER_WINDOW_SIZE),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(
                        HOVER_WINDOW_OFFSET.extend(HOVER_WINDOW_Z),
                    ),
                    ..Default::default()
                },
                HoverWindow,
            ))
            .with_children(|parent| {
                let x = HOVER_WINDOW_SIZE.x / 4.;
                let y = HOVER_WINDOW_SIZE.y / 4.;

                let tier = ((stats.damage - MINION_DMG_BASE) / MINION_DMG_INC) as u8;
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.sword_icon.clone(),
                    Vec3::new(-x, y, HOVER_WINDOW_Z + 1.),
                    tier,
                );

                let tier = ((stats.speed - MINION_SPEED_BASE) / MINION_SPEED_INC) as u8;
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.boot_icon.clone(),
                    Vec3::new(x, y, HOVER_WINDOW_Z + 1.),
                    tier,
                );

                let tier = ((stats.max_hp - MINION_HP_BASE) / MINION_HP_INC) as u8;
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.hearth_icon.clone(),
                    Vec3::new(-x, -y, HOVER_WINDOW_Z + 1.),
                    tier,
                );

                let tier =
                    ((stats.hp_regeneration - MINION_HP_REGEN_BASE) / MINION_HP_REGEN_INC) as u8;
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.hp_regeneration_icon.clone(),
                    Vec3::new(x, -y, HOVER_WINDOW_Z + 1.),
                    tier,
                );
            })
            .id();

        commands.entity(entity).add_child(window_entity);
    }
}
