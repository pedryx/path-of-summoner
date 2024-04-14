use bevy::prelude::*;

use crate::{
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
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    statistics: Res<Statistics>,
    battle_count: Res<BattleCount>,
) {
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
            transform: Transform::from_xyz(0., BATTLE_COUNT_Y, 0.),
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
            transform: Transform::from_xyz(0., TIME_Y, 0.),
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
            transform: Transform::from_xyz(0., MINIONS_Y, 0.),
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

fn despawn_entities(mut commands: Commands, query: Query<Entity, With<GameOverEntity>>) {
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
