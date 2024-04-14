use crate::{
    loading::{FontAssets, TextureAssets},
    GameScreen, GameState,
};
use bevy::{prelude::*, transform::TransformSystem, window::PrimaryWindow};

const INVENTORY_POS: Vec3 = Vec3::new(-1920. / 4. - 128., 1080. / 2. - 64., 0.);
const INVENTORY_SIZE: Vec2 = Vec2::new(600., 800.);
const INGREDIENTS_POS: Vec3 = Vec3::new(1920. / 4. + 128., 1080. / 2. - 64., 0.);
const SUMMONING_CURCLE_POS: Vec3 = Vec3::new(0., 64., 0.);

const MAX_ITEM_COUNT: usize = 10;
const ITEM_CARD_SIZE: Vec2 = Vec2::new(INVENTORY_SIZE.x, INVENTORY_SIZE.y / MAX_ITEM_COUNT as f32);

pub struct SummoningPlugin;

impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InventoryItems(vec![
            SummoningItem {
                item_type: SummoningItemType::Damage,
                tier: 1,
                quantity: 4,
            },
            SummoningItem {
                item_type: SummoningItemType::Speed,
                tier: 4,
                quantity: 1,
            },
        ]))
        .init_resource::<IngredientItems>()
        .insert_resource(ShouldRecreateItemCards {
            should_recreate_ingredient_items: true,
            should_recreate_inventory_items: true,
        })
        .init_resource::<DragActive>()
        .init_resource::<MouseInfo>()
        .add_systems(OnEnter(GameScreen::Summoning), spawn_inventories_and_circle)
        .add_systems(
            Update,
            update_mouse_info.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                spawn_inventory_cards,
                spawn_ingredient_cards,
                handle_drag_move,
                handle_drag_end,
            )
                .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Summoning))),
        )
        .add_systems(
            PostUpdate,
            handle_drag_start
                .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Summoning)))
                .after(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SummoningItemType {
    Damage,
    Speed,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SummoningItem {
    item_type: SummoningItemType,
    tier: u8,
    quantity: usize,
}

#[derive(Component)]
struct ItemInventory;

#[derive(Component)]
struct IngredientInventory;

#[derive(Component)]
struct ItemCard(usize);

#[derive(Component)]
struct DragItem(usize);

#[derive(Component)]
struct Draggable;

#[derive(Resource, Default)]
struct InventoryItems(Vec<SummoningItem>);

#[derive(Resource, Default)]
struct IngredientItems(Vec<SummoningItem>);

#[derive(Resource, Default)]
struct DragActive(bool);

#[derive(Resource, Default)]
struct ShouldRecreateItemCards {
    should_recreate_inventory_items: bool,
    should_recreate_ingredient_items: bool,
}

#[derive(Resource, Default)]
struct MouseInfo {
    position: Vec2,
    pressed: bool,
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
}

fn spawn_item_card(
    commands: &mut Commands,
    textures: &Res<TextureAssets>,
    fonts: &Res<FontAssets>,
    index: usize,
    item: &SummoningItem,
    is_ingredient: bool,
) -> Entity {
    let card_entity = commands
        .spawn((
            SpriteBundle {
                texture: textures.square.clone(),
                sprite: Sprite {
                    color: Color::GRAY.with_a(0.5),
                    custom_size: Some(ITEM_CARD_SIZE),
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    0.,
                    -ITEM_CARD_SIZE.y * index as f32 - ITEM_CARD_SIZE.y / 2.,
                    1.,
                )
                .with_scale(Vec3::new(0.99, 0.9, 1.)),
                ..Default::default()
            },
            ItemCard(index),
        ))
        .id();

    if !is_ingredient {
        commands.entity(card_entity).insert(Draggable);

        let quantity_entity = commands
            .spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        item.quantity.to_string() + "x",
                        TextStyle {
                            color: Color::WHITE,
                            font: fonts.quantity_numbers.clone(),
                            font_size: ITEM_CARD_SIZE.y,
                        },
                    )],
                    justify: JustifyText::Left,
                    ..Default::default()
                },
                text_anchor: bevy::sprite::Anchor::CenterLeft,
                transform: Transform::from_xyz(-ITEM_CARD_SIZE.x / 2. + 16., 0., 3.),
                ..Default::default()
            })
            .id();

        commands
            .entity(card_entity)
            .push_children(&[quantity_entity]);
    }

    let base_stat_icon_entity = commands
        .spawn(SpriteBundle {
            texture: textures.circle.clone(),
            sprite: Sprite {
                color: Color::YELLOW,
                custom_size: Some(Vec2::splat(ITEM_CARD_SIZE.y * 0.5)),
                anchor: bevy::sprite::Anchor::CenterRight,
                ..Default::default()
            },
            transform: Transform::from_xyz(ITEM_CARD_SIZE.x / 2. - 16., 0., 3.),
            ..Default::default()
        })
        .id();

    let tier_number_entity = commands
        .spawn(Text2dBundle {
            text: Text {
                sections: vec![TextSection::new(
                    match item.tier {
                        0 => "0",
                        1 => "I",
                        2 => "II",
                        3 => "III",
                        4 => "IV",
                        5 => "V",
                        6 => "VI",
                        7 => "VII",
                        8 => "VIII",
                        9 => "IX",
                        10 => "X",
                        _ => panic!("Tier not supported!"),
                    },
                    TextStyle {
                        color: Color::WHITE,
                        font: fonts.tier_numbers.clone(),
                        font_size: ITEM_CARD_SIZE.y,
                    },
                )],
                justify: JustifyText::Center,
                ..Default::default()
            },
            transform: Transform::from_xyz(-32., 0., 3.),
            ..Default::default()
        })
        .id();

    let effect_icon_entity = commands
        .spawn(SpriteBundle {
            texture: match item.item_type {
                SummoningItemType::Damage => textures.sword_icon.clone(),
                SummoningItemType::Speed => textures.boot_icon.clone(),
            },
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::splat(ITEM_CARD_SIZE.y * 0.7)),
                anchor: bevy::sprite::Anchor::CenterLeft,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., 3.),
            ..Default::default()
        })
        .id();

    commands.entity(card_entity).push_children(&[
        base_stat_icon_entity,
        tier_number_entity,
        effect_icon_entity,
    ]);

    card_entity
}

fn spawn_inventory_cards(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    items: Res<InventoryItems>,
    mut recreate_items: ResMut<ShouldRecreateItemCards>,
    inventory: Query<(Entity, Option<&Children>), With<ItemInventory>>,
) {
    if !recreate_items.should_recreate_inventory_items {
        return;
    }

    let (inventory_entity, children) = inventory.single();

    if children.is_some() {
        for &child in children.unwrap().iter() {
            commands.entity(child).despawn_recursive();
        }
        commands.entity(inventory_entity).clear_children();
    }

    for (index, item) in items.0.iter().enumerate() {
        let card_entity = spawn_item_card(&mut commands, &textures, &fonts, index, item, false);
        commands
            .entity(inventory_entity)
            .push_children(&[card_entity]);
    }

    recreate_items.should_recreate_inventory_items = false;
}

fn spawn_ingredient_cards(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    items: Res<IngredientItems>,
    mut recreate_items: ResMut<ShouldRecreateItemCards>,
    inventory: Query<(Entity, Option<&Children>), With<IngredientInventory>>,
) {
    if !recreate_items.should_recreate_ingredient_items {
        return;
    }

    let (inventory_entity, children) = inventory.single();

    if children.is_some() {
        for &child in children.unwrap().iter() {
            commands.entity(child).despawn_recursive();
        }
        commands.entity(inventory_entity).clear_children();
    }

    for (index, item) in items.0.iter().enumerate() {
        let card_entity = spawn_item_card(&mut commands, &textures, &fonts, index, item, true);
        commands
            .entity(inventory_entity)
            .push_children(&[card_entity]);
    }

    recreate_items.should_recreate_ingredient_items = false;
}

fn spawn_inventories_and_circle(mut commands: Commands, textures: Res<TextureAssets>) {
    // item inventory
    commands.spawn((
        SpriteBundle {
            texture: textures.square.clone(),
            sprite: Sprite {
                color: Color::DARK_GRAY,
                custom_size: Some(INVENTORY_SIZE),
                anchor: bevy::sprite::Anchor::TopCenter,
                ..Default::default()
            },
            transform: Transform::from_translation(INVENTORY_POS),
            ..Default::default()
        },
        ItemInventory,
    ));

    // ingredient inventory
    commands.spawn((
        SpriteBundle {
            texture: textures.square.clone(),
            sprite: Sprite {
                color: Color::DARK_GRAY,
                custom_size: Some(INVENTORY_SIZE),
                anchor: bevy::sprite::Anchor::TopCenter,
                ..Default::default()
            },
            transform: Transform::from_translation(INGREDIENTS_POS),
            ..Default::default()
        },
        IngredientInventory,
    ));

    // Summoning circle
    commands.spawn(SpriteBundle {
        texture: textures.summoning_circle.clone(),
        transform: Transform::from_translation(SUMMONING_CURCLE_POS),
        sprite: Sprite {
            color: Color::PURPLE,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn handle_drag_start(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mouse_info: Res<MouseInfo>,
    mut drag_active: ResMut<DragActive>,
    item_card_query: Query<(&GlobalTransform, &ItemCard), With<Draggable>>,
) {
    if drag_active.0 || !mouse_info.pressed {
        return;
    }

    for (transform, item_card) in item_card_query.iter() {
        let rect = Rect {
            min: transform.translation().xy() - ITEM_CARD_SIZE / 2.,
            max: transform.translation().xy() + ITEM_CARD_SIZE / 2.,
        };

        if !rect.contains(mouse_info.position) {
            continue;
        }

        drag_active.0 = true;
        commands.spawn((
            SpriteBundle {
                texture: textures.circle.clone(),
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::splat(32.)),
                    ..Default::default()
                },
                transform: Transform::from_translation(mouse_info.position.extend(4.)),
                ..Default::default()
            },
            DragItem(item_card.0),
        ));
    }
}

fn handle_drag_move(
    mouse_info: Res<MouseInfo>,
    drag_active: ResMut<DragActive>,
    mut dragged_item: Query<&mut Transform, With<DragItem>>,
) {
    if !drag_active.0 {
        return;
    }

    let mut transform = dragged_item.single_mut();

    transform.translation.x = mouse_info.position.x;
    transform.translation.y = mouse_info.position.y;
}

fn handle_drag_end(
    mut commands: Commands,
    mouse_info: Res<MouseInfo>,
    mut drag_active: ResMut<DragActive>,
    mut inventory_items: ResMut<InventoryItems>,
    mut ingredient_items: ResMut<IngredientItems>,
    mut should_recreate_item_cards: ResMut<ShouldRecreateItemCards>,
    mut drag_item: Query<(Entity, &DragItem)>,
) {
    if !drag_active.0 {
        return;
    }

    let (entity, drag_item) = drag_item.single_mut();

    if mouse_info.pressed {
        return;
    }

    commands.entity(entity).despawn_recursive();
    drag_active.0 = false;

    let rect = Rect {
        min: SUMMONING_CURCLE_POS.xy() - Vec2::splat(256.),
        max: SUMMONING_CURCLE_POS.xy() + Vec2::splat(256.),
    };
    if !rect.contains(mouse_info.position) {
        return;
    }

    let item = &mut inventory_items.0[drag_item.0];
    let is_duplicate = ingredient_items
        .0
        .iter()
        .any(|ingredient| ingredient.item_type == item.item_type);

    if !is_duplicate {
        ingredient_items.0.push(item.clone());

        item.quantity -= 1;
        if item.quantity == 0 {
            inventory_items.0.remove(drag_item.0);
        }
    }

    should_recreate_item_cards.should_recreate_inventory_items = true;
    should_recreate_item_cards.should_recreate_ingredient_items = true;
}
