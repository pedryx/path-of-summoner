use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    enemy::{DropRewards, Enemy},
    loading::{FontAssets, TextureAssets},
    mouse_control::Clickable,
    stats::Stats,
    summoning::{SummoningItem, SummoningItemType},
    utils::num_to_roman,
    BattleCount, GameScreen, GameState,
};

const MAX_CARD_COUNT: usize = 3;
const MAX_REWARD_COUNT: usize = 3;

const CARD_SIZE: Vec2 = Vec2::new(400., 600.);
const CARDS_Y: f32 = 30.;
const ENEMY_STAT_ICON_SIZE: f32 = 64.;
const CARD_STAT_ICON_OFFSET_Y: f32 = 32.;
const CARD_STAT_TIER_OFFSET_X: f32 = 16.;

const REWARD_CARD_SIZE: Vec2 = Vec2::new(CARD_SIZE.x, ENEMY_STAT_ICON_SIZE);

pub struct PlanningScreenPlugin;

impl Plugin for PlanningScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlanningRng(StdRng::from_entropy()))
            .add_systems(
                OnEnter(GameScreen::Planning),
                (spawn_title, spawn_enemy_cards),
            )
            .add_systems(OnExit(GameScreen::Planning), clean_planning_screen)
            .add_systems(
                Update,
                (handle_enemy_selection)
                    .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Planning))),
            );
    }
}

#[derive(Component)]
struct EnemyCard;

#[derive(Resource)]
struct PlanningRng(StdRng);

#[derive(Component)]
struct PlanningScreenEntity;

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
                    font_size: ENEMY_STAT_ICON_SIZE,
                },
            )],
            ..Default::default()
        },
        text_anchor: bevy::sprite::Anchor::TopRight,
        transform: Transform::from_translation(
            position - Vec3::new(CARD_STAT_TIER_OFFSET_X, 0., 0.),
        ),
        ..Default::default()
    });

    // icon
    parent.spawn(SpriteBundle {
        texture,
        sprite: Sprite {
            anchor: bevy::sprite::Anchor::TopLeft,
            custom_size: Some(Vec2::splat(ENEMY_STAT_ICON_SIZE * 0.7)),
            color: Color::CYAN,
            ..Default::default()
        },
        transform: Transform::from_translation(position),
        ..Default::default()
    });
}

fn spawn_reward_card(
    parent: &mut ChildBuilder,
    fonts: &Res<FontAssets>,
    textures: &Res<TextureAssets>,
    index: usize,
    item: &SummoningItem,
) {
    // reward card
    parent
        .spawn((SpriteBundle {
            texture: textures.square.clone(),
            sprite: Sprite {
                color: Color::GRAY,
                custom_size: Some(REWARD_CARD_SIZE),
                ..Default::default()
            },
            transform: Transform::from_xyz(
                0.,
                REWARD_CARD_SIZE.y * (index) as f32 - CARD_SIZE.y / 2. + REWARD_CARD_SIZE.y / 2.,
                1.,
            )
            .with_scale(Vec3::new(0.99, 0.9, 1.)),
            ..Default::default()
        },))
        .with_children(|card| {
            // reward category icon
            card.spawn(SpriteBundle {
                texture: textures.circle.clone(),
                sprite: Sprite {
                    color: Color::YELLOW,
                    custom_size: Some(Vec2::splat(REWARD_CARD_SIZE.y * 0.5)),
                    anchor: bevy::sprite::Anchor::CenterRight,
                    ..Default::default()
                },
                transform: Transform::from_xyz(REWARD_CARD_SIZE.x / 2. - 16., 0., 3.),
                ..Default::default()
            });

            // tier number
            card.spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        num_to_roman(item.tier),
                        TextStyle {
                            color: Color::WHITE,
                            font: fonts.tier_numbers.clone(),
                            font_size: REWARD_CARD_SIZE.y,
                        },
                    )],
                    justify: JustifyText::Center,
                    ..Default::default()
                },
                transform: Transform::from_xyz(-32., 0., 3.),
                ..Default::default()
            });

            // item type icon
            card.spawn(SpriteBundle {
                texture: match item.item_type {
                    SummoningItemType::Damage => textures.sword_icon.clone(),
                    SummoningItemType::Speed => textures.boot_icon.clone(),
                    SummoningItemType::MaxHP => textures.hearth_icon.clone(),
                    SummoningItemType::HPRegeneration => textures.hp_regeneration_icon.clone(),
                },
                sprite: Sprite {
                    color: Color::CYAN,
                    custom_size: Some(Vec2::splat(REWARD_CARD_SIZE.y * 0.7)),
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 0., 3.),
                ..Default::default()
            });
        });
}

fn spawn_title(mut commands: Commands, fonts: Res<FontAssets>) {
    commands.spawn((
        Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "SELECT AN OPONENT:",
                    TextStyle {
                        color: Color::WHITE,
                        font: fonts.texts.clone(),
                        font_size: 96.,
                    },
                )],
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 1080. / 2. - 128., 0.),
            ..Default::default()
        },
        PlanningScreenEntity,
    ));
}

fn spawn_enemy_cards(
    mut commands: Commands,
    mut planning_rng: ResMut<PlanningRng>,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    battle_count: Res<BattleCount>,
) {
    let card_count = planning_rng.0.gen_range(1..=MAX_CARD_COUNT);

    for i in 0..card_count {
        let damage_tier = planning_rng
            .0
            .gen_range(1..(battle_count.0 / 2 + 2))
            .min(10) as u8;
        let speed_tier = planning_rng
            .0
            .gen_range(1..(battle_count.0 / 2 + 2))
            .min(10) as u8;
        let hp_tier = planning_rng
            .0
            .gen_range(1..(battle_count.0 / 2 + 2))
            .min(10) as u8;
        let hp_regeneration_tier = planning_rng
            .0
            .gen_range(0..(battle_count.0 / 2 + 1))
            .min(10) as u8;

        let stats = Stats {
            current_hp: hp_tier as f32 * 10.,
            max_hp: hp_tier as f32 * 10.,
            hp_regeneration: hp_regeneration_tier as f32 * 1.,
            damage: damage_tier as f32 * 4.,
            speed: speed_tier as f32 * 1.,
            ..Default::default()
        };

        let reward_count = planning_rng.0.gen_range(1..=MAX_REWARD_COUNT);
        let mut rewards = DropRewards(Vec::new());
        for _ in 0..reward_count {
            rewards.0.push(SummoningItem {
                item_type: match planning_rng.0.gen_range(0..4) {
                    0 => SummoningItemType::Damage,
                    1 => SummoningItemType::Speed,
                    2 => SummoningItemType::MaxHP,
                    3 => SummoningItemType::HPRegeneration,
                    _ => panic!("Invalid item type!"),
                },
                tier: planning_rng
                    .0
                    .gen_range(1..(battle_count.0 / 2 + 2))
                    .min(10) as u8,
                quantity: 1,
            });
        }

        let x_pos = (1920. / (card_count as f32 + 1.)) * (i as f32 + 1.) - 1920. / 2.;

        // enemy card
        commands
            .spawn((
                SpriteBundle {
                    texture: textures.square.clone(),
                    sprite: Sprite {
                        color: Color::DARK_GRAY,
                        custom_size: Some(CARD_SIZE),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(x_pos, CARDS_Y, 0.),
                    ..Default::default()
                },
                Clickable::default(),
                EnemyCard,
                stats,
                rewards.clone(),
                PlanningScreenEntity,
            ))
            .with_children(|parent| {
                let first_row_y = CARD_SIZE.y / 2. - CARD_STAT_ICON_OFFSET_Y;
                let second_row_y =
                    CARD_SIZE.y / 2. - 2. * CARD_STAT_ICON_OFFSET_Y - ENEMY_STAT_ICON_SIZE;
                let left_x = -CARD_SIZE.x / 4.;
                let right_x = CARD_SIZE.x / 4.;

                // damage icon
                let position = Vec3::new(left_x, first_row_y, 1.);
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.sword_icon.clone(),
                    position,
                    damage_tier,
                );

                // speed icon
                let position = Vec3::new(right_x, first_row_y, 1.);
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.boot_icon.clone(),
                    position,
                    speed_tier,
                );

                // hp icon
                let position = Vec3::new(left_x, second_row_y, 1.);
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.hearth_icon.clone(),
                    position,
                    hp_tier,
                );

                // hp regeneration icon
                let position = Vec3::new(right_x, second_row_y, 1.);
                spawn_icon(
                    parent,
                    fonts.tier_numbers.clone(),
                    textures.hp_regeneration_icon.clone(),
                    position,
                    hp_regeneration_tier,
                );

                // reward cards
                for (index, item) in rewards.0.iter().enumerate() {
                    spawn_reward_card(parent, &fonts, &textures, index, item);
                }
            });
    }
}

fn handle_enemy_selection(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    query: Query<(&Clickable, &Stats, &DropRewards), With<EnemyCard>>,
) {
    for (clickable, stats, drop_rewards) in query.iter() {
        if !clickable.just_clicked {
            continue;
        }

        commands.spawn((
            SpriteBundle {
                texture: textures.enemy1.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            stats.clone(),
            drop_rewards.clone(),
            Enemy,
        ));

        next_screen.set(GameScreen::Battle);
    }
}

fn clean_planning_screen(mut commands: Commands, query: Query<Entity, With<PlanningScreenEntity>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
