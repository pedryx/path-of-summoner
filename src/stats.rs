use crate::{GameScreen, GameState};
use bevy::prelude::*;

pub const MINION_HP_BASE: f32 = 80.0;
pub const MINION_HP_REGEN_BASE: f32 = 0.0;
pub const MINION_DMG_BASE: f32 = 19.0;
pub const MINION_SPEED_BASE: f32 = 0.5;

pub const MINION_HP_INC: f32 = 15.0;
pub const MINION_HP_REGEN_INC: f32 = 1.5;
pub const MINION_DMG_INC: f32 = 7.0;
pub const MINION_SPEED_INC: f32 = 0.1;

pub const ENEMY_HP_BASE: f32 = 160.0;
pub const ENEMY_HP_REGEN_BASE: f32 = 0.0;
pub const ENEMY_DMG_BASE: f32 = 30.0;
pub const ENEMY_SPEED_BASE: f32 = 0.6;

pub const ENEMY_HP_INC: f32 = 18.0;
pub const ENEMY_HP_REGEN_INC: f32 = 4.0;
pub const ENEMY_DMG_INC: f32 = 7.0;
pub const ENEMY_SPEED_INC: f32 = 0.15;

pub const BATTLES_TO_ITEM_TIER_INC: usize = 2;
pub const BATTLES_TO_ENEMY_TIER_INC: usize = 2;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (regenerate_hp_and_mana,)
                .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Battle))),
        );
    }
}

#[derive(Component, Clone)]
pub struct Stats {
    pub current_hp: f32,
    pub max_hp: f32,
    pub hp_regeneration: f32,
    pub speed: f32,
    pub damage: f32,
    pub current_mana: f32,
    pub max_mana: f32,
    pub mana_regeneration: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            current_hp: 10.,
            max_hp: 10.,
            hp_regeneration: 0.,
            speed: 1.,
            damage: 1.,
            current_mana: 0.,
            max_mana: 0.,
            mana_regeneration: 0.,
        }
    }
}

fn regenerate_hp_and_mana(time: Res<Time>, mut query: Query<&mut Stats>) {
    for mut stats in query.iter_mut() {
        stats.current_hp += stats.hp_regeneration * time.delta_seconds();
        if stats.current_hp > stats.max_hp {
            stats.current_hp = stats.max_hp;
        }

        stats.current_mana += stats.mana_regeneration * time.delta_seconds();
        if stats.current_mana > stats.max_mana {
            stats.current_mana = stats.max_mana;
        }
    }
}
