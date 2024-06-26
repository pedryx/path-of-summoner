use crate::{
    health_bar::HealthBar,
    loading::{FontAssets, TextureAssets},
    minions::{Minion, MAX_MINION_COUNT, MINION_SIZE},
    mouse_control::{update_clickables, Clickable},
    statistics::Statistics,
    stats::{
        Stats, MINION_DMG_BASE, MINION_DMG_INC, MINION_HP_BASE, MINION_HP_INC,
        MINION_HP_REGEN_BASE, MINION_HP_REGEN_INC, MINION_SPEED_BASE, MINION_SPEED_INC,
    },
    utils::num_to_roman,
    BattleCount, GameScreen, GameState,
};
use bevy::prelude::*;

const INVENTORY_POS: Vec3 = Vec3::new(-1920. / 4. - 128., 1080. / 2. - 64., 0.);
const INVENTORY_SIZE: Vec2 = Vec2::new(600., 800.);
const INGREDIENTS_POS: Vec3 = Vec3::new(1920. / 4. + 128., 1080. / 2. - 64., 0.);
const SUMMONING_CIRCLE_POS: Vec3 = Vec3::new(0., 220., 0.);

pub const MAX_ITEM_COUNT: usize = 10;
const ITEM_CARD_SIZE: Vec2 = Vec2::new(INVENTORY_SIZE.x, INVENTORY_SIZE.y / MAX_ITEM_COUNT as f32);

const MINIONS_Y: f32 = -460.;

pub struct SummoningPlugin;

impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryItems>()
            .init_resource::<IngredientItems>()
            .insert_resource(ShouldRecreateItemCards {
                should_recreate_ingredient_items: false,
                should_recreate_inventory_items: false,
            })
            //.init_resource::<DragActive>()
            .add_systems(
                OnEnter(GameScreen::Summoning),
                (
                    spawn_items,
                    spawn_inventories_and_circle,
                    reposition_minions,
                    make_minions_clickable,
                ),
            )
            .add_systems(
                OnExit(GameScreen::Summoning),
                (clean_summoning_screen, unmake_minions_clickable),
            )
            .add_systems(
                Update,
                (
                    summon_minion,
                    move_to_preparation_screen,
                    handle_remove_minion,
                    handle_delete_item,
                    handle_move_item,
                )
                    .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Summoning))),
            )
            .add_systems(
                PreUpdate,
                (spawn_inventory_cards, spawn_ingredient_cards)
                    .after(update_clickables)
                    .run_if(in_state(GameState::Playing).and_then(in_state(GameScreen::Summoning))),
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
struct SummoningCircle;

#[derive(Component)]
struct ReadyButton;

#[derive(Resource, Default)]
pub struct InventoryItems(pub Vec<SummoningItem>);

#[derive(Resource, Default)]
struct IngredientItems(Vec<SummoningItem>);

#[derive(Component)]
struct SummoningScreenEntity;

#[derive(Component)]
struct InInventoryItem;

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
                    color: Color::BLACK.with_a(0.95),
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
            Clickable::default(),
        ))
        .id();

    if !is_ingredient {
        commands.entity(card_entity).insert(InInventoryItem);

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
    // background
    commands.spawn((
        SpriteBundle {
            texture: textures.dungeon_floor_background.clone(),
            transform: Transform::from_xyz(0., 0., -2.),
            ..Default::default()
        },
        SummoningScreenEntity,
    ));
    commands.spawn((
        SpriteBundle {
            texture: textures.square.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1920., 1080.)),
                color: Color::BLACK.with_a(0.7),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., -1.),
            ..Default::default()
        },
        SummoningScreenEntity,
    ));

    // item inventory
    commands.spawn((
        SpriteBundle {
            texture: textures.square.clone(),
            sprite: Sprite {
                color: Color::DARK_GRAY.with_a(0.7),
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
                color: Color::DARK_GRAY.with_a(0.7),
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

fn summon_minion(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    mut ingredient_items: ResMut<IngredientItems>,
    mut should_recreate_item_cards: ResMut<ShouldRecreateItemCards>,
    mut statistics: ResMut<Statistics>,
    summoning_circle_query: Query<&Clickable, With<SummoningCircle>>,
    minion_query: Query<(), With<Minion>>,
) {
    let clickable = summoning_circle_query.single();
    let minion_count = minion_query.iter().count();

    let is_clicked = clickable.just_left_clicked;
    let free_slot_exist = minion_count < MAX_MINION_COUNT;
    let at_least_one_ingredient_used = ingredient_items.0.first().is_some();
    if !is_clicked || !free_slot_exist || !at_least_one_ingredient_used {
        return;
    }

    let mut stats = Stats {
        current_hp: MINION_HP_BASE,
        max_hp: MINION_HP_BASE,
        speed: MINION_SPEED_BASE,
        damage: MINION_DMG_BASE,
        hp_regeneration: MINION_HP_REGEN_BASE,
        ..Default::default()
    };
    for item in ingredient_items.0.iter() {
        match item.item_type {
            SummoningItemType::MaxHP => {
                stats.max_hp += MINION_HP_INC * item.tier as f32;
                stats.current_hp += MINION_HP_INC * item.tier as f32;
            }
            SummoningItemType::HPRegeneration => {
                stats.hp_regeneration += MINION_HP_REGEN_INC * item.tier as f32
            }
            SummoningItemType::Speed => stats.speed += MINION_SPEED_INC * item.tier as f32,
            SummoningItemType::Damage => stats.damage += MINION_DMG_INC * item.tier as f32,
        }
    }
    let stats = stats;

    commands.spawn((
        SpriteBundle {
            texture: textures.minion.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(MINION_SIZE)),
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
        HealthBar::default(),
        Clickable::default(),
    ));

    statistics.summoned_minions += 1;
    ingredient_items.0.clear();
    should_recreate_item_cards.should_recreate_ingredient_items = true;
}

fn move_to_preparation_screen(
    mut inventory_items: ResMut<InventoryItems>,
    mut ingredient_items: ResMut<IngredientItems>,
    mut next_screen: ResMut<NextState<GameScreen>>,
    button_query: Query<&Clickable, With<ReadyButton>>,
    minion_query: Query<(), With<Minion>>,
) {
    let clickable = button_query.single();

    if !clickable.just_left_clicked || minion_query.iter().next().is_none() {
        return;
    }

    next_screen.set(GameScreen::Planning);

    for ingredient_item in ingredient_items.0.iter() {
        if let Some(item) = inventory_items.0.iter_mut().find(|item| {
            item.item_type == ingredient_item.item_type && item.tier == ingredient_item.tier
        }) {
            item.quantity += ingredient_item.quantity;
        } else {
            inventory_items.0.push(ingredient_item.clone());
        }
    }
    ingredient_items.0.clear();
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

fn spawn_items(
    mut inventory_items: ResMut<InventoryItems>,
    mut recreate_items: ResMut<ShouldRecreateItemCards>,
    battle_count: Res<BattleCount>,
) {
    if battle_count.0 == 1 {
        inventory_items.0.push(SummoningItem {
            item_type: SummoningItemType::Damage,
            tier: 1,
            quantity: 1,
        });
        inventory_items.0.push(SummoningItem {
            item_type: SummoningItemType::Speed,
            tier: 1,
            quantity: 1,
        });
        inventory_items.0.push(SummoningItem {
            item_type: SummoningItemType::MaxHP,
            tier: 1,
            quantity: 1,
        });
    }

    recreate_items.should_recreate_inventory_items = true;
}

fn make_minions_clickable(mut commands: Commands, query: Query<Entity, With<Minion>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Clickable::default());
    }
}

fn unmake_minions_clickable(mut commands: Commands, query: Query<Entity, With<Minion>>) {
    for entity in query.iter() {
        commands.entity(entity).remove::<Clickable>();
    }
}

fn handle_remove_minion(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Clickable), With<Minion>>,
) {
    let mut removed = None;

    for (entity, _, clickable) in query.iter() {
        if !clickable.just_right_clicked {
            continue;
        }

        commands.entity(entity).despawn_recursive();
        removed = Some(entity);
    }

    if removed.is_none() {
        return;
    }

    for (index, (_, mut transform, _)) in query
        .iter_mut()
        .filter(|&(e, _, _)| e != removed.unwrap())
        .enumerate()
    {
        transform.translation = Vec3::new(
            1920. / (MAX_MINION_COUNT as f32 + 2.) * (index as f32 + 1.) - 1920. / 2.,
            MINIONS_Y,
            0.,
        );
    }
}

fn handle_delete_item(
    mut commands: Commands,
    mut inventory_items: ResMut<InventoryItems>,
    mut recreate_items: ResMut<ShouldRecreateItemCards>,
    query: Query<(Entity, &Clickable, &ItemCard), With<InInventoryItem>>,
) {
    for (entity, clickable, &ItemCard(index)) in query.iter() {
        if !clickable.just_right_clicked {
            continue;
        }

        inventory_items.0.remove(index);
        commands.entity(entity).despawn_recursive();
        recreate_items.should_recreate_inventory_items = true;
    }
}

fn handle_move_item(
    mut recreate_items: ResMut<ShouldRecreateItemCards>,
    mut inventory_items: ResMut<InventoryItems>,
    mut ingredient_items: ResMut<IngredientItems>,
    inventory_query: Query<(&Clickable, &ItemCard), With<InInventoryItem>>,
    ingredient_query: Query<(&Clickable, &ItemCard), Without<InInventoryItem>>,
) {
    // handle move of inventory items
    for (clickable, &ItemCard(index)) in inventory_query.iter() {
        if !clickable.just_left_clicked {
            continue;
        }

        let item = &mut inventory_items.0[index];
        let is_duplicate = ingredient_items
            .0
            .iter()
            .any(|ingredient| ingredient.item_type == item.item_type);
        if !is_duplicate {
            ingredient_items.0.push(item.clone());

            item.quantity -= 1;
            if item.quantity == 0 {
                inventory_items.0.remove(index);
            }

            recreate_items.should_recreate_inventory_items = true;
            recreate_items.should_recreate_ingredient_items = true;
        }
    }

    // handle move of ingredient items
    for (clickable, &ItemCard(index)) in ingredient_query.iter() {
        if !clickable.just_left_clicked {
            continue;
        }

        let ingredient = ingredient_items.0.remove(index);
        if let Some(item) = inventory_items
            .0
            .iter_mut()
            .find(|item| item.item_type == ingredient.item_type && item.tier == ingredient.tier)
        {
            item.quantity += ingredient.quantity;
        } else if inventory_items.0.len() < MAX_ITEM_COUNT {
            inventory_items.0.push(ingredient);
        }

        recreate_items.should_recreate_inventory_items = true;
        recreate_items.should_recreate_ingredient_items = true;
    }
}
