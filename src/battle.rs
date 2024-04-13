use crate::{enemy::Enemy, minions::{Minion, MAX_MINION_COUNT}, stats::Stats, GameState};
use bevy::prelude::*;
use rand::{rngs::ThreadRng, Rng};

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_battle,
        ).run_if(in_state(GameState::Playing)))
            .insert_non_send_resource(BattleRng(rand::thread_rng()))
            .insert_resource(MinionCount(MAX_MINION_COUNT));
    }
}

#[derive(Component, Default)]
pub struct BattleParticipant {
    pub turn_accumulator: f32,
}

struct BattleRng(ThreadRng);

#[derive(Resource)]
struct MinionCount(usize);

fn update_battle(
    mut commands: Commands,
    time: Res<Time>,
    mut battle_rng: NonSendMut<BattleRng>,
    mut minion_count: ResMut<MinionCount>,
    mut minion_query: Query<(Entity, &mut BattleParticipant, &mut Stats), (With<Minion>, Without<Enemy>)>,
    mut enemy_query: Query<(Entity, &mut BattleParticipant, &mut Stats), (With<Enemy>, Without<Minion>)>,
) {
    if let Ok((enemy_entity, mut enemy_battle_participant, mut enemy_stats)) = enemy_query.get_single_mut() {
        for (_, mut battle_participant, stats) in minion_query.iter_mut() {
            battle_participant.turn_accumulator +=  time.delta_seconds();
    
            if battle_participant.turn_accumulator >= 1. / stats.speed {
                battle_participant.turn_accumulator -= 1. / stats.speed;
    
                enemy_stats.current_hp -= stats.damage;
                if enemy_stats.current_hp <= 0. {
                    commands.entity(enemy_entity).despawn_recursive();
                }
            }
        }

        enemy_battle_participant.turn_accumulator += time.delta_seconds();

        if enemy_battle_participant.turn_accumulator >= 1. / enemy_stats.speed {
            enemy_battle_participant.turn_accumulator -= 1. / enemy_stats.speed;

            let target = minion_query.iter_mut().nth(battle_rng.0.gen_range(0..minion_count.0));
            if let Some((entity, _, mut stats)) = target {
                stats.current_hp -= enemy_stats.damage;
                if stats.current_hp <= 0. {
                    commands.entity(entity).despawn_recursive();
                    minion_count.0 -= 1;
                }
            }
        }
    }
}
