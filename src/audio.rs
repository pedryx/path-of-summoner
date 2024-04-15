use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

const GLOBAL_VOLUME: f64 = 1.0;

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .init_resource::<AudioInit>()
            .init_resource::<Soundtrack>()
            .add_systems(OnEnter(GameState::Menu), start_audio);
    }
}

#[derive(Resource, Default)]
pub struct Soundtrack {
    pub basic: Handle<AudioInstance>,
    pub battle: Handle<AudioInstance>,
    pub game_over: Handle<AudioInstance>,
}

#[derive(Resource, Default)]
struct AudioInit(bool);

fn start_audio(
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
    mut audio_init: ResMut<AudioInit>,
    mut soundtrack: ResMut<Soundtrack>,
) {
    if audio_init.0 {
        return;
    }
    audio_init.0 = true;

    // ambient
    audio
        .play(audio_assets.ambient.clone())
        .looped()
        .with_volume(GLOBAL_VOLUME * 0.01);

    // basic soundtrack
    soundtrack.basic = audio
        .play(audio_assets.soundtrack.clone())
        .looped()
        .with_volume(GLOBAL_VOLUME * 0.07)
        .handle();

    // battle soundtrack
    soundtrack.battle = audio
        .play(audio_assets.battle_soundtrack.clone())
        .looped()
        .with_volume(GLOBAL_VOLUME * 0.07)
        .paused()
        .handle();

    // game over soundtrack
    soundtrack.game_over = audio
        .play(audio_assets.game_over_soundtrack.clone())
        .looped()
        .with_volume(GLOBAL_VOLUME * 0.07)
        .paused()
        .handle();
}
