use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{
    lens::{SpriteColorLens, TransformPositionLens},
    Animator, EaseFunction, RepeatCount, RepeatStrategy, Tween,
};

use crate::{GameScreen, GameState};

use super::{EnemyAttackEvent, MinionAttackEvent};

const ATTACK_DURATION: f32 = 0.2;
const MINION_ATTACK_OFFSET: Vec2 = Vec2::new(30., 0.);
const ENEMY_ATTACK_OFFSET: Vec2 = Vec2::new(-60., 0.);

const HURT_DURATION: f32 = 0.2;
const HURT_COLOR: Color = Color::rgb(2., 0., 0.);

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_minion_attack_effect,
                handle_minion_hurt_effect,
                handle_enemy_attack_effect,
                handle_enemy_hurt_effect,
            )
                .run_if(in_state(GameScreen::Battle).and_then(in_state(GameState::Playing))),
        );
    }
}

fn handle_minion_attack_effect(
    mut commands: Commands,
    mut minion_attack_event: EventReader<MinionAttackEvent>,
    query: Query<&Transform>,
) {
    for event in minion_attack_event.read() {
        let Ok(transform) = query.get(event.attacker) else {
            continue;
        };
        let position = transform.translation;

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(ATTACK_DURATION / 2.),
            TransformPositionLens {
                start: position,
                end: position + MINION_ATTACK_OFFSET.extend(position.z),
            },
        )
        .with_repeat_count(RepeatCount::Finite(2))
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.entity(event.attacker).insert(Animator::new(tween));
    }
}

fn handle_enemy_attack_effect(
    mut commands: Commands,
    mut enemy_attack_event: EventReader<EnemyAttackEvent>,
    query: Query<&Transform>,
) {
    for event in enemy_attack_event.read() {
        let Ok(transform) = query.get(event.attacker) else {
            continue;
        };
        let position = transform.translation;

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(ATTACK_DURATION / 2.),
            TransformPositionLens {
                start: position,
                end: position + ENEMY_ATTACK_OFFSET.extend(position.z),
            },
        )
        .with_repeat_count(RepeatCount::Finite(2))
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.entity(event.attacker).insert(Animator::new(tween));
    }
}

fn handle_minion_hurt_effect(
    mut commands: Commands,
    mut enemy_attack_event: EventReader<EnemyAttackEvent>,
    query: Query<()>,
) {
    for event in enemy_attack_event.read() {
        if query.get(event.target).is_err() {
            continue;
        }

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(HURT_DURATION / 2.),
            SpriteColorLens {
                start: Color::WHITE,
                end: HURT_COLOR,
            },
        )
        .with_repeat_count(RepeatCount::Finite(2))
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.entity(event.target).insert(Animator::new(tween));
    }
}

fn handle_enemy_hurt_effect(
    mut commands: Commands,
    mut minion_attack_event: EventReader<MinionAttackEvent>,
    query: Query<()>,
) {
    for event in minion_attack_event.read() {
        if query.get(event.target).is_err() {
            continue;
        }

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(HURT_DURATION / 2.),
            SpriteColorLens {
                start: Color::WHITE,
                end: HURT_COLOR,
            },
        )
        .with_repeat_count(RepeatCount::Finite(2))
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.entity(event.target).insert(Animator::new(tween));
    }
}
