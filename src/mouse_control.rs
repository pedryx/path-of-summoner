use bevy::{prelude::*, window::PrimaryWindow};

use crate::GameState;

pub struct MouseControlPlugin;

impl Plugin for MouseControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseInfo>().add_systems(
            Update,
            (update_mouse_info, update_clickables).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Default)]
pub struct Clickable {
    pub just_clicked: bool,
}

#[derive(Resource, Default)]
pub struct MouseInfo {
    pub position: Vec2,
    pub pressed: bool,
    pub just_pressed: bool,
}

fn update_mouse_info(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut mouse_info: ResMut<MouseInfo>,
) {
    let (camera, camera_transform) = camera.single();
    let Some(position) = window.single().cursor_position() else {
        return;
    };
    let position = camera
        .viewport_to_world_2d(camera_transform, position)
        .unwrap();

    mouse_info.position = position;
    mouse_info.pressed = buttons.pressed(MouseButton::Left);
    mouse_info.just_pressed = buttons.just_pressed(MouseButton::Left);
}

fn update_clickables(
    assets: Res<Assets<Image>>,
    mouse_info: Res<MouseInfo>,
    mut query: Query<(&mut Clickable, &GlobalTransform, &Handle<Image>, &Sprite)>,
) {
    for (mut clickable, _, _, _) in query.iter_mut() {
        clickable.just_clicked = false;
    }

    if !mouse_info.just_pressed {
        return;
    }

    for (mut clickable, transform, image_handle, sprite) in query.iter_mut() {
        let transform = transform.compute_transform();

        let size = assets.get(image_handle).unwrap().texture_descriptor.size;
        let mut size = Vec2::new(size.width as f32, size.height as f32);
        if let Some(custom_size) = sprite.custom_size {
            size = custom_size;
        }
        size *= transform.scale.xy();
        let size = size;

        let rect = Rect {
            min: transform.translation.xy() - size / 2.,
            max: transform.translation.xy() + size / 2.,
        };
        if rect.contains(mouse_info.position) {
            clickable.just_clicked = true;
        }
    }
}
