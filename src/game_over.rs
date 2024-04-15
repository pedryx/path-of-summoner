use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::{AudioInstance, AudioTween};

use crate::{
    audio::Soundtrack,
    loading::{FontAssets, TextureAssets},
    mouse_control::Clickable,
    statistics::Statistics,
    BattleCount, GameState,
};

const TITLE_SIZE: f32 = 128.;
const TITLE_Y: f32 = 400.;

const LABELS_SIZE: f32 = 96.;
const BATTLE_COUNT_Y: f32 = 150.;
const TIME_Y: f32 = 0.;
const MINIONS_Y: f32 = -150.;

const MENU_BUTTON_SIZE: Vec2 = Vec2::new(256., 96.);
const MENU_BUTTON_Y: f32 = -300.;
const MENU_BUTTON_TEXT_SIZE: f32 = 64.;

const UI_X: f32 = -1920. / 4.;
const MONSTER_X: f32 = 1920. / 4.;

const VOLUME_TRANSITION: f32 = 0.5;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), spawn_entities)
            .add_systems(OnExit(GameState::GameOver), despawn_entities)
            .add_systems(
                Update,
                handle_menu_button.run_if(in_state(GameState::GameOver)),
            );
    }
}

#[derive(Component)]
struct GameOverEntity;

#[derive(Component)]
struct MenuButton;

fn spawn_entities(
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    statistics: Res<Statistics>,
    battle_count: Res<BattleCount>,
    soundtrack: Res<Soundtrack>,
) {
    // audio
    audio_instances
        .get_mut(&soundtrack.basic)
        .unwrap()
        .pause(AudioTween::linear(Duration::from_secs_f32(
            VOLUME_TRANSITION,
        )));
    audio_instances
        .get_mut(&soundtrack.game_over)
        .unwrap()
        .resume(AudioTween::linear(Duration::from_secs_f32(
            VOLUME_TRANSITION,
        )));

    // background
    commands.spawn((
        SpriteBundle {
            texture: textures.battleground_background.clone(),
            transform: Transform::from_xyz(0., 0., -3.),
            ..Default::default()
        },
        GameOverEntity,
    ));
    commands.spawn((
        SpriteBundle {
            texture: textures.square.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1920., 1080.)),
                color: Color::BLACK.with_a(0.7),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., -2.),
            ..Default::default()
        },
        GameOverEntity,
    ));

    // monster
    commands.spawn((
        SpriteBundle {
            texture: textures.game_over_enemy.clone(),
            transform: Transform::from_xyz(MONSTER_X, 0., -1.),
            ..Default::default()
        },
        GameOverEntity,
    ));

    // title
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "GAME OVER",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: TITLE_SIZE,
                    },
                )],
                ..Default::default()
            },
            transform: Transform::from_xyz(0., TITLE_Y, 0.),
            ..Default::default()
        },
        GameOverEntity,
    ));

    // battle count label
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    format!("BATTLE COUNT: {}", battle_count.0),
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: LABELS_SIZE,
                    },
                )],
                ..Default::default()
            },
            transform: Transform::from_xyz(UI_X, BATTLE_COUNT_Y, 0.),
            ..Default::default()
        },
        GameOverEntity,
    ));

    // time label
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    format!(
                        "PLAY TIME: {}:{:02}",
                        (statistics.elapsed_seconds / 60.) as u16,
                        statistics.elapsed_seconds as u16 % 60
                    ),
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: LABELS_SIZE,
                    },
                )],
                ..Default::default()
            },
            transform: Transform::from_xyz(UI_X, TIME_Y, 0.),
            ..Default::default()
        },
        GameOverEntity,
    ));

    // minions label
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    format!("SUMMONED MINIONS: {}", statistics.summoned_minions),
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: LABELS_SIZE,
                    },
                )],
                ..Default::default()
            },
            transform: Transform::from_xyz(UI_X, MINIONS_Y, 0.),
            ..Default::default()
        },
        GameOverEntity,
    ));

    // menu button
    commands
        .spawn((
            SpriteBundle {
                texture: textures.square.clone(),
                sprite: Sprite {
                    color: Color::DARK_GRAY,
                    custom_size: Some(MENU_BUTTON_SIZE),
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., MENU_BUTTON_Y, 0.),
                ..Default::default()
            },
            GameOverEntity,
            Clickable::default(),
            MenuButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            "Menu",
                            TextStyle {
                                font: fonts.texts.clone(),
                                color: Color::WHITE,
                                font_size: MENU_BUTTON_TEXT_SIZE,
                            },
                        )],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                GameOverEntity,
            ));
        });
}

fn despawn_entities(
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    soundtrack: Res<Soundtrack>,
    query: Query<Entity, With<GameOverEntity>>,
) {
    // audio
    audio_instances
        .get_mut(&soundtrack.game_over)
        .unwrap()
        .pause(AudioTween::linear(Duration::from_secs_f32(
            VOLUME_TRANSITION,
        )));
    audio_instances
        .get_mut(&soundtrack.basic)
        .unwrap()
        .resume(AudioTween::linear(Duration::from_secs_f32(
            VOLUME_TRANSITION,
        )));

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_menu_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut battle_count: ResMut<BattleCount>,
    mut statistics: ResMut<Statistics>,
    query: Query<&Clickable, With<MenuButton>>,
) {
    let clickable = query.single();

    if !clickable.just_left_clicked {
        return;
    }

    battle_count.0 = 1;
    statistics.elapsed_seconds = 0.;
    statistics.summoned_minions = 0;

    next_state.set(GameState::Menu);
}
