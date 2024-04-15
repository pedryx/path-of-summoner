use crate::{
    loading::{FontAssets, TextureAssets},
    GameScreen, GameState,
};

use bevy::prelude::*;

const TEXT_SIZE: f32 = 40.;
const HIGHLIGHT_Z: f32 = 500.;
const TEXT_Z: f32 = 501.;
const HIGHLIGHT_TRANSPARENCY: f32 = 0.2;

const ITEMS_TEXT_POS: Vec2 = Vec2::new(-615., 60.);
const INGREDIENTS_TEXT_POS: Vec2 = Vec2::new(615., 60.);
const SUMMONING_TEXT_POS: Vec2 = Vec2::new(0., 210.);
const MINIONS_TEXT_POS: Vec2 = Vec2::new(0., -440.);

const ENEMY_CARDS_TEXT_POS: Vec2 = Vec2::ZERO;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TutorialState>()
            .init_resource::<FirstTutorial>()
            .add_systems(
                Update,
                handle_tutorial_control.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(TutorialState::None), clean_up_tutorial)
            .add_systems(
                OnEnter(TutorialState::Summoning),
                show_summoning_tutorial,
            )
            .add_systems(
                OnEnter(GameScreen::Summoning),
                show_summoning_tutorial.run_if(resource_equals(FirstTutorial(true))),
            )
            .add_systems(
                OnEnter(TutorialState::Planning),
                show_planning_tutorial,
            )
            .add_systems(
                OnEnter(GameScreen::Planning),
                show_planning_tutorial.run_if(resource_equals(FirstTutorial(true))),
            );
    }
}

#[derive(States, PartialEq, PartialOrd, Eq, Debug, Hash, Clone, Copy, Default)]
pub enum TutorialState {
    #[default]
    None,
    Summoning,
    Planning,
}

#[derive(Component)]
struct TutorialEntity;

#[derive(Resource, PartialEq, Eq)]
struct FirstTutorial(bool);

impl Default for FirstTutorial {
    fn default() -> Self {
        Self(true)
    }
}

fn handle_tutorial_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    game_screen: Res<State<GameScreen>>,
    mut next_tutorial_state: ResMut<NextState<TutorialState>>,
    mut first_tutorial: ResMut<FirstTutorial>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        match game_screen.get() {
            GameScreen::Summoning => next_tutorial_state.set(TutorialState::Summoning),
            GameScreen::Planning => next_tutorial_state.set(TutorialState::Planning),
            _ => (),
        }
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        if *game_screen.get() == GameScreen::Planning {
            first_tutorial.0 = false;
        }

        next_tutorial_state.set(TutorialState::None);
    }
}

fn clean_up_tutorial(mut commands: Commands, query: Query<Entity, With<TutorialEntity>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn show_summoning_tutorial(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    mut next_tutorial_state: ResMut<NextState<TutorialState>>,
) {
    next_tutorial_state.set(TutorialState::Summoning);

    // highlight
    commands.spawn((
        SpriteBundle {
            texture: textures.tutorial_highlight_summoning.clone(),
            sprite: Sprite {
                color: Color::WHITE.with_a(HIGHLIGHT_TRANSPARENCY),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., HIGHLIGHT_Z),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // items text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "INVENTORY\nLMB - ADD ITEM TO SUMMONING CIRCLE\nRMB - DELETE ITEM",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::BLACK,
                        font_size: TEXT_SIZE,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            transform: Transform::from_translation(ITEMS_TEXT_POS.extend(TEXT_Z)),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // ingredients text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "ITEMS ADDED TO SUMMONING CIRCLE\nLMB - PUT ITEM BACK TO INVENTORY",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::BLACK,
                        font_size: TEXT_SIZE,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            transform: Transform::from_translation(INGREDIENTS_TEXT_POS.extend(TEXT_Z)),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // summon text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "SUMMONING CIRCLE\nLMB - SUMMON MINION",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::BLACK,
                        font_size: TEXT_SIZE,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            transform: Transform::from_translation(SUMMONING_TEXT_POS.extend(TEXT_Z)),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // minions text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "SUMMONED MINIONS\nRMB - DESTROY MINION",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::BLACK,
                        font_size: TEXT_SIZE,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            transform: Transform::from_translation(MINIONS_TEXT_POS.extend(TEXT_Z)),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // show help text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "F1 - SHOW THIS HELP",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: TEXT_SIZE,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            text_anchor: bevy::sprite::Anchor::TopLeft,
            transform: Transform::from_xyz(-950., 530., TEXT_Z),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // continue text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "(CLICK ANYWHERE TO CONTINUE)",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: TEXT_SIZE * 0.7,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            text_anchor: bevy::sprite::Anchor::TopCenter,
            transform: Transform::from_translation(Vec2::new(0., 530.).extend(TEXT_Z)),
            ..Default::default()
        },
        TutorialEntity,
    ));
}

fn show_planning_tutorial(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    mut next_tutorial_state: ResMut<NextState<TutorialState>>,
) {
    next_tutorial_state.set(TutorialState::Planning);

    // highlight
    commands.spawn((
        SpriteBundle {
            texture: textures.tutorial_highlight_planning.clone(),
            sprite: Sprite {
                color: Color::WHITE.with_a(HIGHLIGHT_TRANSPARENCY),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., HIGHLIGHT_Z),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // minions text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "ENEMY CARDS\nEACH SHOW ENEMY STATS AND REWARDS\nLMB - SELECT ENEMY",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::BLACK,
                        font_size: TEXT_SIZE,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            transform: Transform::from_translation(ENEMY_CARDS_TEXT_POS.extend(TEXT_Z)),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // show help text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "F1 - SHOW THIS HELP",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: TEXT_SIZE,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            text_anchor: bevy::sprite::Anchor::TopLeft,
            transform: Transform::from_xyz(-950., 530., TEXT_Z),
            ..Default::default()
        },
        TutorialEntity,
    ));

    // continue text
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "(CLICK ANYWHERE TO CONTINUE)",
                    TextStyle {
                        font: fonts.texts.clone(),
                        color: Color::WHITE,
                        font_size: TEXT_SIZE * 0.7,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            text_anchor: bevy::sprite::Anchor::TopCenter,
            transform: Transform::from_translation(Vec2::new(0., 530.).extend(TEXT_Z)),
            ..Default::default()
        },
        TutorialEntity,
    ));
}
