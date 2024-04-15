use std::time::Duration;

mod effects;

use self::effects::EffectsPlugin;
use crate::{
    audio::Soundtrack,
    enemy::{DropRewards, Enemy},
    health_bar::HealthBar,
    loading::TextureAssets,
    minions::Minion,
    stats::Stats,
    summoning::{InventoryItems, MAX_ITEM_COUNT},
    BattleCount, GameScreen, GameState,
};
use bevy::prelude::*;
use bevy_kira_audio::{AudioInstance, AudioTween};
use rand::{rngs::StdRng, Rng, SeedableRng};

const VOLUME_TRANSITION: f32 = 0.5;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectsPlugin)
            .add_event::<MinionAttackEvent>()
            .add_event::<EnemyAttackEvent>()
            .add_event::<MinionDiedEvent>()
            .add_event::<EnemyDiedEvent>()
            .add_systems(
                OnEnter(GameScreen::Battle),
                (prepare_battle, prepare_battle_screen),
            )
            .add_systems(OnExit(GameScreen::Battle), clean_up_battle_screen)
            .add_systems(
                Update,
                (update_battle, handle_enemy_dead, handle_minion_dead)
                    .chain()
                    .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Battle))),
            )
            .insert_resource(BattleRng(StdRng::from_entropy()))
            .insert_resource(MinionCount(0));
    }
}

#[derive(Component, Default)]
pub struct BattleParticipant {
    pub turn_accumulator: f32,
}

#[derive(Component)]
pub struct BattleScreenEntity;

#[derive(Resource)]
pub struct BattleRng(StdRng);

#[derive(Resource)]
pub struct MinionCount(usize);

#[derive(Event)]
pub struct MinionAttackEvent {
    attacker: Entity,
    target: Entity,
}

#[derive(Event)]
pub struct EnemyAttackEvent {
    attacker: Entity,
    target: Entity,
}

#[derive(Event)]
pub struct EnemyDiedEvent;

#[derive(Event)]
pub struct MinionDiedEvent;

fn prepare_battle(
    mut commands: Commands,
    mut minion_count: ResMut<MinionCount>,
    minion_query: Query<Entity, With<Minion>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for entity in minion_query.iter() {
        commands.entity(entity).insert(BattleParticipant::default());
    }

    commands.entity(enemy_query.single()).insert((
        HealthBar {
            width: 256.,
            offset: Vec2::new(0., 380.),
            ..Default::default()
        },
        BattleParticipant::default(),
    ));

    minion_count.0 = minion_query.iter().count();
}

pub fn update_battle(
    time: Res<Time>,
    mut battle_rng: ResMut<BattleRng>,
    minion_count: Res<MinionCount>,
    mut minion_attack_event: EventWriter<MinionAttackEvent>,
    mut enemy_attack_event: EventWriter<EnemyAttackEvent>,
    mut minion_query: Query<
        (Entity, &mut BattleParticipant, &mut Stats),
        (With<Minion>, Without<Enemy>),
    >,
    mut enemy_query: Query<
        (Entity, &mut BattleParticipant, &mut Stats),
        (With<Enemy>, Without<Minion>),
    >,
) {
    if let Ok((enemy_entity, mut enemy_battle_participant, mut enemy_stats)) =
        enemy_query.get_single_mut()
    {
        for (entity, mut battle_participant, stats) in minion_query.iter_mut() {
            battle_participant.turn_accumulator += time.delta_seconds();

            if battle_participant.turn_accumulator >= 1. / stats.speed {
                battle_participant.turn_accumulator -= 1. / stats.speed;

                enemy_stats.current_hp -= stats.damage;
                println!(
                    "minion attacking for {}, enemy has {} hp",
                    stats.damage, enemy_stats.current_hp
                );
                minion_attack_event.send(MinionAttackEvent {
                    attacker: entity,
                    target: enemy_entity,
                });
            }
        }

        enemy_battle_participant.turn_accumulator += time.delta_seconds();

        if enemy_battle_participant.turn_accumulator >= 1. / enemy_stats.speed {
            enemy_battle_participant.turn_accumulator -= 1. / enemy_stats.speed;

            let target = minion_query
                .iter_mut()
                .nth(battle_rng.0.gen_range(0..minion_count.0));
            if let Some((entity, _, mut stats)) = target {
                stats.current_hp -= enemy_stats.damage;
                println!(
                    "enemy attacking for {}, minion has {} hp",
                    enemy_stats.damage, stats.current_hp
                );
                enemy_attack_event.send(EnemyAttackEvent {
                    attacker: enemy_entity,
                    target: entity,
                });
            }
        }
    }
}

fn handle_enemy_dead(
    mut commands: Commands,
    mut enemy_died_event: EventWriter<EnemyDiedEvent>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut inventory_items: ResMut<InventoryItems>,
    mut battle_count: ResMut<BattleCount>,
    enemy_query: Query<(Entity, &Stats, &DropRewards), With<Enemy>>,
) {
    let (entity, stats, drop_rewards) = enemy_query.single();

    if stats.current_hp > 0. {
        return;
    }

    for reward_item in drop_rewards.0.iter() {
        if let Some(item) = inventory_items
            .0
            .iter_mut()
            .find(|item| item.item_type == reward_item.item_type && item.tier == reward_item.tier)
        {
            item.quantity += reward_item.quantity;
        } else if inventory_items.0.len() < MAX_ITEM_COUNT {
            inventory_items.0.push(reward_item.clone());
        }
    }

    enemy_died_event.send(EnemyDiedEvent);

    // battle win
    battle_count.0 += 1;
    commands.entity(entity).despawn_recursive();
    next_screen.set(GameScreen::Summoning);
}

fn handle_minion_dead(
    mut commands: Commands,
    mut minion_died_event: EventWriter<MinionDiedEvent>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut minion_count: ResMut<MinionCount>,
    minion_query: Query<(Entity, &Stats), With<Minion>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for (entity, stats) in minion_query.iter() {
        if stats.current_hp > 0. {
            continue;
        }

        commands.entity(entity).despawn_recursive();
        minion_count.0 -= 1;

        minion_died_event.send(MinionDiedEvent);

        // game over
        if minion_count.0 == 0 {
            if enemy_query.iter().next().is_some() {
                commands.entity(enemy_query.single()).despawn_recursive();
            }
            next_screen.set(GameScreen::Other);
            next_state.set(GameState::GameOver);
        }
    }
}

fn prepare_battle_screen(
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    textures: Res<TextureAssets>,
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
        .get_mut(&soundtrack.battle)
        .unwrap()
        .resume(AudioTween::linear(Duration::from_secs_f32(
            VOLUME_TRANSITION,
        )));

    // background
    commands.spawn((
        SpriteBundle {
            texture: textures.battleground_background.clone(),
            transform: Transform::from_xyz(0., 0., -2.),
            ..Default::default()
        },
        BattleScreenEntity,
    ));
    commands.spawn((
        SpriteBundle {
            texture: textures.square.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1920., 1080.)),
                color: Color::BLACK.with_a(0.7),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., -1.),
            ..Default::default()
        },
        BattleScreenEntity,
    ));
}

fn clean_up_battle_screen(
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    soundtrack: Res<Soundtrack>,
    query: Query<Entity, With<BattleScreenEntity>>,
) {
    // audio
    audio_instances
        .get_mut(&soundtrack.basic)
        .unwrap()
        .resume(AudioTween::linear(Duration::from_secs_f32(
            VOLUME_TRANSITION,
        )));
    audio_instances
        .get_mut(&soundtrack.battle)
        .unwrap()
        .pause(AudioTween::linear(Duration::from_secs_f32(
            VOLUME_TRANSITION,
        )));

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
