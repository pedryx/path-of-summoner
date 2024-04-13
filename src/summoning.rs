use crate::{
    loading::{FontAssets, TextureAssets},
    GameScreen,
};
use bevy::prelude::*;

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
        .add_systems(OnEnter(GameScreen::Summoning), spawn_entities);
    }
}

pub enum SummoningItemType {
    Damage,
    Speed,
}

pub struct SummoningItem {
    item_type: SummoningItemType,
    tier: u8,
    quantity: usize,
}

#[derive(Component)]
struct ItemInventory;

#[derive(Component)]
struct IngredientsInventory;

#[derive(Component)]
struct ItemCard;

#[derive(Resource, Default)]
struct InventoryItems(Vec<SummoningItem>);

fn spawn_item_card(
    commands: &mut Commands,
    textures: &Res<TextureAssets>,
    fonts: &Res<FontAssets>,
    index: usize,
    item: &SummoningItem,
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
            ItemCard,
        ))
        .id();

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

    let base_stat_icon_entity = commands
        .spawn(SpriteBundle {
            texture: textures.base_stat_icon.clone(),
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
        quantity_entity,
        base_stat_icon_entity,
        tier_number_entity,
        effect_icon_entity,
    ]);

    card_entity
}

fn spawn_entities(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    items: Res<InventoryItems>,
) {
    let inventory_entity = commands
        .spawn((
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
        ))
        .id();

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
        IngredientsInventory,
    ));

    commands.spawn(SpriteBundle {
        texture: textures.summoning_circle.clone(),
        transform: Transform::from_translation(SUMMONING_CURCLE_POS),
        sprite: Sprite {
            color: Color::PURPLE,
            ..Default::default()
        },
        ..Default::default()
    });

    for (index, item) in items.0.iter().enumerate() {
        let card_entity = spawn_item_card(&mut commands, &textures, &fonts, index, item);
        commands
            .entity(inventory_entity)
            .push_children(&[card_entity]);
    }
}
