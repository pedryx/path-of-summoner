#![allow(clippy::type_complexity)]

mod audio;
mod battle;
mod enemy;
mod health_bar;
mod loading;
mod menu;
mod minions;
mod mouse_control;
mod planning_screen;
mod stats;
mod summoning;
mod utils;
mod statistics;

use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use crate::battle::BattlePlugin;
use crate::enemy::EnemyPlugin;
use crate::health_bar::HealthBarPlugin;
use crate::minions::MinionsPlugin;
use crate::mouse_control::MouseControlPlugin;
use crate::planning_screen::PlanningScreenPlugin;
use crate::stats::StatsPlugin;
use crate::summoning::SummoningPlugin;
use crate::statistics::StatisticsPlugin;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::{app::App, window::close_on_esc};

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameScreen {
    #[default]
    Other,
    Battle,
    Summoning,
    Planning,
}

#[derive(Resource, Default)]
pub struct BattleCount(usize);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<GameScreen>()
            .insert_resource(BattleCount(1))
            .add_systems(OnExit(GameState::Loading), spawn_camera)
            .add_systems(Update, close_on_esc)
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                InternalAudioPlugin,
                EnemyPlugin,
                HealthBarPlugin,
                StatsPlugin,
                MinionsPlugin,
                BattlePlugin,
                SummoningPlugin,
                MouseControlPlugin,
                PlanningScreenPlugin,
                StatisticsPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
