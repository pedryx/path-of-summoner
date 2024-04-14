use crate::{
    loading::{FontAssets, TextureAssets},
    minions::{Minion, MAX_MINION_COUNT},
    mouse_control::{Clickable, MouseInfo},
    stats::Stats,
    utils::num_to_roman,
    GameScreen, GameState,
};
use bevy::{prelude::*, transform::TransformSystem};

const INVENTORY_POS: Vec3 = Vec3::new(-1920. / 4. - 128., 1080. / 2. - 64., 0.);
const INVENTORY_SIZE: Vec2 = Vec2::new(600., 800.);
const INGREDIENTS_POS: Vec3 = Vec3::new(1920. / 4. + 128., 1080. / 2. - 64., 0.);
const SUMMONING_CIRCLE_POS: Vec3 = Vec3::new(0., 220., 0.);

const MAX_ITEM_COUNT: usize = 10;
const ITEM_CARD_SIZE: Vec2 = Vec2::new(INVENTORY_SIZE.x, INVENTORY_SIZE.y / MAX_ITEM_COUNT as f32);

const MINIONS_Y: f32 = -400.;

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
            SummoningItem {
                item_type: SummoningItemType::HPRegeneration,
                tier: 8,
                quantity: 6,
            },
            SummoningItem {
                item_type: SummoningItemType::MaxHP,
                tier: 10,
                quantity: 9,
            },
        ]))
        .init_resource::<IngredientItems>()
        .insert_resource(ShouldRecreateItemCards {
            should_recreate_ingredient_items: true,
            should_recreate_inventory_items: true,
        })
        .init_resource::<DragActive>()
        .add_systems(
            OnEnter(GameScreen::Summoning),
            (spawn_inventories_and_circle, reposition_minions),
        )
        .add_systems(OnExit(GameScreen::Summoning), clean_summoning_screen)
        .add_systems(
            Update,
            (
                spawn_inventory_cards,
                spawn_ingredient_cards,
                handle_drag_move,
                handle_drag_end,
                summon_minion,
                move_to_preparation_screen,
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
    MaxHP,
    HPRegeneration,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SummoningItem {
    pub item_type: SummoningItemType,
    pub tier: u8,
    pub quantity: usize,
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

#[derive(Component)]
struct SummoningCircle;

#[derive(Component)]
struct ReadyButton;

#[derive(Resource, Default)]
struct InventoryItems(Vec<SummoningItem>);

#[derive(Resource, Default)]
struct IngredientItems(Vec<SummoningItem>);

#[derive(Resource, Default)]
struct DragActive(bool);

#[derive(Component)]
struct SummoningScreenEntity;

#[derive(Resource, Default)]
struct ShouldRecreateItemCards {
    should_recreate_inventory_items: bool,
    should_recreate_ingredient_items: bool,
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
                    color: Color::GRAY,
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
                    num_to_roman(item.tier),
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
                SummoningItemType::MaxHP => textures.hearth_icon.clone(),
                SummoningItemType::HPRegeneration => textures.hp_regeneration_icon.clone(),
            },
            sprite: Sprite {
                color: Color::CYAN,
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

fn spawn_inventories_and_circle(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
) {
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
        SummoningScreenEntity,
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
        SummoningScreenEntity,
    ));

    // Summoning circle
    commands.spawn((
        SpriteBundle {
            texture: textures.summoning_circle.clone(),
            transform: Transform::from_translation(SUMMONING_CIRCLE_POS),
            sprite: Sprite {
                color: Color::PURPLE,
                ..Default::default()
            },
            ..Default::default()
        },
        Clickable::default(),
        SummoningCircle,
        SummoningScreenEntity,
    ));

    // ready button
    commands
        .spawn((
            SpriteBundle {
                texture: textures.square.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(384., 128.)),
                    color: Color::DARK_GRAY,
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., -255., 0.),
                ..Default::default()
            },
            Clickable::default(),
            ReadyButton,
            SummoningScreenEntity,
        ))
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Ready".to_string(),
                        style: TextStyle {
                            font: fonts.texts.clone(),
                            font_size: 96.,
                            color: Color::WHITE,
                        },
                    }],
                    justify: JustifyText::Center,
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 0., 1.),
                ..Default::default()
            });
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
        min: SUMMONING_CIRCLE_POS.xy() - Vec2::splat(256.),
        max: SUMMONING_CIRCLE_POS.xy() + Vec2::splat(256.),
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

fn summon_minion(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut ingredient_items: ResMut<IngredientItems>,
    mut should_recreate_item_cards: ResMut<ShouldRecreateItemCards>,
    summoning_circle_query: Query<&Clickable, With<SummoningCircle>>,
    minion_query: Query<(), With<Minion>>,
) {
    let clickable = summoning_circle_query.single();
    let minion_count = minion_query.iter().count();

    let is_clicked = clickable.just_clicked;
    let free_slot_exist = minion_count < MAX_MINION_COUNT;
    let at_least_one_ingredient_used = ingredient_items.0.first().is_some();
    if !is_clicked || !free_slot_exist || !at_least_one_ingredient_used {
        return;
    }

    let mut stats = Stats {
        current_hp: 10.,
        max_hp: 10.,
        speed: 1.,
        damage: 1.,
        ..Default::default()
    };
    for item in ingredient_items.0.iter() {
        match item.item_type {
            SummoningItemType::MaxHP => stats.max_hp += 10. * item.tier as f32,
            SummoningItemType::HPRegeneration => stats.hp_regeneration += 0.5 * item.tier as f32,
            SummoningItemType::Speed => stats.speed += 1. * item.tier as f32,
            SummoningItemType::Damage => stats.damage += 2. * item.tier as f32,
        }
    }
    let stats = stats;

    commands.spawn((
        SpriteBundle {
            texture: textures.bevy.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(64.)),
                ..Default::default()
            },
            transform: Transform::from_xyz(
                1920. / (MAX_MINION_COUNT as f32 + 2.) * (minion_count as f32 + 1.) - 1920. / 2.,
                MINIONS_Y,
                0.,
            ),
            ..Default::default()
        },
        Minion,
        stats,
    ));

    ingredient_items.0.clear();
    should_recreate_item_cards.should_recreate_ingredient_items = true;
}

fn move_to_preparation_screen(
    mut next_screen: ResMut<NextState<GameScreen>>,
    button_query: Query<&Clickable, With<ReadyButton>>,
    minion_query: Query<(), With<Minion>>,
) {
    let clickable = button_query.single();

    if !clickable.just_clicked || minion_query.iter().next().is_none() {
        return;
    }

    next_screen.set(GameScreen::Planning);
}

fn clean_summoning_screen(
    mut commands: Commands,
    query: Query<Entity, With<SummoningScreenEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn reposition_minions(mut query: Query<&mut Transform, With<Minion>>) {
    for (index, mut transform) in query.iter_mut().enumerate() {
        transform.translation = Vec3::new(
            1920. / (MAX_MINION_COUNT as f32 + 2.) * (index as f32 + 1.) - 1920. / 2.,
            MINIONS_Y,
            0.,
        );
    }
}
