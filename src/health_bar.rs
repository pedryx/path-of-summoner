use crate::{stats::Stats, GameScreen, GameState};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

const HEALTH_BAR_BACKGROUND_Z: f32 = 100.;
const HEALTH_BAR_FOREGROUND_Z: f32 = 101.;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_health_bars, update_health_bars)
                .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Battle))),
        );
    }
}

#[derive(Component)]
pub struct HealthBar {
    pub health_bar_background_entity: Option<Entity>,
    pub health_bar_foreground_entity: Option<Entity>,
    pub width: f32,
    pub height: f32,
    pub offset: Vec2,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self {
            health_bar_background_entity: None,
            health_bar_foreground_entity: None,
            width: 96.,
            height: 16.,
            offset: Vec2::new(0., 96.),
        }
    }
}

fn spawn_health_bars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &mut HealthBar), With<Stats>>,
) {
    for (entity, mut health_bar) in query.iter_mut() {
        if health_bar.health_bar_background_entity.is_some() {
            continue;
        }

        let health_bar_background_entity = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(health_bar.width, health_bar.height))),
                material: materials.add(Color::BLACK),
                transform: Transform::from_xyz(
                    health_bar.offset.x,
                    health_bar.offset.y,
                    HEALTH_BAR_BACKGROUND_Z,
                ),
                ..default()
            })
            .id();

        let health_bar_foreground_entity = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(health_bar.width, health_bar.height))),
                material: materials.add(Color::RED),
                transform: Transform::from_xyz(0., 0., HEALTH_BAR_FOREGROUND_Z),
                ..default()
            })
            .id();

        commands
            .entity(entity)
            .push_children(&[health_bar_background_entity]);
        commands
            .entity(health_bar_background_entity)
            .push_children(&[health_bar_foreground_entity]);

        health_bar.health_bar_background_entity = Some(health_bar_background_entity);
        health_bar.health_bar_foreground_entity = Some(health_bar_foreground_entity);
    }
}

fn update_health_bars(
    query: Query<(&HealthBar, &Stats)>,
    mut transform_query: Query<&mut Transform>,
) {
    for (health_bar, hp) in query.iter() {
        if health_bar.health_bar_background_entity.is_none() {
            continue;
        }

        let value = hp.current_hp / hp.max_hp;
        if let Ok(mut transform) =
            transform_query.get_mut(health_bar.health_bar_foreground_entity.unwrap())
        {
            transform.scale.x = value;
            transform.translation.x = -(1. - value) * (health_bar.width / 2.);
        }
    }
}
