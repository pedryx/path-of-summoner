use crate::{
    enemy::{DropRewards, Enemy},
    health_bar::HealthBar,
    minions::Minion,
    stats::Stats,
    summoning::InventoryItems,
    BattleCount, GameScreen, GameState,
};
use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::Battle), prepare_battle)
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

#[derive(Resource)]
pub struct BattleRng(StdRng);

#[derive(Resource)]
pub struct MinionCount(usize);

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
    mut minion_query: Query<(&mut BattleParticipant, &mut Stats), (With<Minion>, Without<Enemy>)>,
    mut enemy_query: Query<(&mut BattleParticipant, &mut Stats), (With<Enemy>, Without<Minion>)>,
) {
    if let Ok((mut enemy_battle_participant, mut enemy_stats)) = enemy_query.get_single_mut() {
        for (mut battle_participant, stats) in minion_query.iter_mut() {
            battle_participant.turn_accumulator += time.delta_seconds();

            if battle_participant.turn_accumulator >= 1. / stats.speed {
                battle_participant.turn_accumulator -= 1. / stats.speed;

                enemy_stats.current_hp -= stats.damage;
                println!(
                    "minion attacking for {}, enemy has {} hp",
                    stats.damage, enemy_stats.current_hp
                );
            }
        }

        enemy_battle_participant.turn_accumulator += time.delta_seconds();

        if enemy_battle_participant.turn_accumulator >= 1. / enemy_stats.speed {
            enemy_battle_participant.turn_accumulator -= 1. / enemy_stats.speed;

            let target = minion_query
                .iter_mut()
                .nth(battle_rng.0.gen_range(0..minion_count.0));
            if let Some((_, mut stats)) = target {
                stats.current_hp -= enemy_stats.damage;
                println!(
                    "enemy attacking for {}, minion has {} hp",
                    enemy_stats.damage, stats.current_hp
                );
            }
        }
    }
}

fn handle_enemy_dead(
    mut commands: Commands,
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
        } else {
            inventory_items.0.push(reward_item.clone());
        }
    }

    // battle win
    battle_count.0 += 1;
    commands.entity(entity).despawn_recursive();
    next_screen.set(GameScreen::Summoning);
}

fn handle_minion_dead(
    mut commands: Commands,
    mut next_screen: ResMut<NextState<GameScreen>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut minion_count: ResMut<MinionCount>,
    mut inventory_items: ResMut<InventoryItems>,
    minion_query: Query<(Entity, &Stats), With<Minion>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for (entity, stats) in minion_query.iter() {
        if stats.current_hp > 0. {
            continue;
        }

        commands.entity(entity).despawn_recursive();
        minion_count.0 -= 1;

        // game over
        if minion_count.0 == 0 {
            inventory_items.0.clear();
            if enemy_query.iter().next().is_some() {
                commands.entity(enemy_query.single()).despawn_recursive();
            }
            next_screen.set(GameScreen::Other);
            next_state.set(GameState::Menu);
        }
    }
}
