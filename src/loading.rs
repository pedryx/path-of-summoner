use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<FontAssets>()
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>(),
        );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/Magician.ttf")]
    pub quantity_numbers: Handle<Font>,
    #[asset(path = "fonts/SCARY OF HORROR.otf")]
    pub tier_numbers: Handle<Font>,
    #[asset(path = "fonts/MQSMagic.ttf")]
    pub texts: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/eerie_ambience.ogg")]
    pub ambient: Handle<AudioSource>,
    #[asset(path = "audio/spooky_halloween_soundtrack.ogg")]
    pub soundtrack: Handle<AudioSource>,
    #[asset(path = "audio/battle_cinematic_soundtrack.ogg")]
    pub battle_soundtrack: Handle<AudioSource>,
    #[asset(path = "audio/visible_sadness.ogg")]
    pub game_over_soundtrack: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,

    #[asset(path = "textures/summoner_background.png")]
    pub summoner_background: Handle<Image>,
    #[asset(path = "textures/game_over_enemy.png")]
    pub game_over_enemy: Handle<Image>,

    #[asset(path = "textures/dungeon_floor_background.png")]
    pub dungeon_floor_background: Handle<Image>,
    #[asset(path = "textures/battleground_background.png")]
    pub battleground_background: Handle<Image>,

    #[asset(path = "textures/summoning_circle.png")]
    pub summoning_circle: Handle<Image>,
    #[asset(path = "textures/minion.png")]
    pub minion: Handle<Image>,
    #[asset(path = "textures/enemy1.png")]
    pub enemy1: Handle<Image>,

    #[asset(path = "textures/square.png")]
    pub square: Handle<Image>,
    #[asset(path = "textures/circle.png")]
    pub circle: Handle<Image>,

    #[asset(path = "textures/icons/sword_icon.png")]
    pub sword_icon: Handle<Image>,
    #[asset(path = "textures/icons/boot_icon.png")]
    pub boot_icon: Handle<Image>,
    #[asset(path = "textures/icons/hearth_icon.png")]
    pub hearth_icon: Handle<Image>,
    #[asset(path = "textures/icons/hp_regeneration_icon.png")]
    pub hp_regeneration_icon: Handle<Image>,
}
