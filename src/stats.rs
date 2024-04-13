use bevy::prelude::*;

// pub struct StatsPlugin;

// impl Plugin for StatsPlugin {
//     fn build(&self, app: &mut App) {
        
//     }
// }

#[derive(Component)]
pub struct HP {
    pub value: f32,
    pub max: f32,
}

impl Default for HP {
    fn default() -> Self {
        Self { value: 100., max: 100. }
    }
}