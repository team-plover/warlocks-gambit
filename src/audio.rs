//! Play music and sound effects.
//!
//! Defines an [`AudioRequest`] event, reads them in [`play_audio`] system
//! using the kira backend for mixing and loudness controls.
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_kira_audio::prelude::{AudioChannel as KiraChannel, *};
use enum_map::{enum_map, EnumMap};

use crate::war::WordOfPower;

#[derive(SystemLabel, Debug, Clone, Hash, PartialEq, Eq)]
pub struct AudioRequestSystem;

#[derive(Clone, Copy, PartialEq)]
pub enum AudioChannel {
    Master,
    Sfx,
    Music,
}
struct ChannelVolumes {
    master: f64,
    sfx: f64,
    music: f64,
}
impl Default for ChannelVolumes {
    fn default() -> Self {
        Self { master: 1.0, sfx: 0.5, music: 0.5 }
    }
}

struct AudioAssets {
    wood_clink: Handle<AudioSource>,
    shuffle_long: Handle<AudioSource>,
    shuffle_short: Handle<AudioSource>,
    music: Handle<AudioSource>,
    words: EnumMap<WordOfPower, Handle<AudioSource>>,
}
impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            music: assets.load("sfx/music.ogg"),
            shuffle_long: assets.load("sfx/shuffle_long.ogg"),
            shuffle_short: assets.load("sfx/shuffle_short.ogg"),
            wood_clink: assets.load("wood_clink.ogg"),
            words: enum_map! { word => assets.load(&format!("word_audio/{word:?}.ogg")) },
        }
    }
}

enum Music {}
enum Sfx {}

pub enum SfxParam {
    StartLoop,
    PlayOnce,
}
pub enum AudioRequest {
    StopSfxLoop,
    PlayWoodClink(SfxParam),
    PlayWord(WordOfPower),
    PlayShuffleLong,
    PlayShuffleShort,
    StartMusic,
    SetVolume(AudioChannel, f64),
}
fn play_audio(
    assets: Res<AudioAssets>,
    music: Res<KiraChannel<Music>>,
    sfx: Res<KiraChannel<Sfx>>,
    mut volumes: ResMut<ChannelVolumes>,
    mut events: EventReader<AudioRequest>,
) {
    for event in events.iter() {
        match event {
            AudioRequest::StartMusic => {
                music.play(assets.music.clone_weak()).looped();
            }
            AudioRequest::SetVolume(AudioChannel::Sfx, volume) if *volume != volumes.sfx => {
                volumes.sfx = *volume;
                sfx.set_volume(volume * volumes.master);
            }
            AudioRequest::SetVolume(AudioChannel::Music, volume) if *volume != volumes.music => {
                volumes.music = *volume;
                music.set_volume(volume * volumes.master);
            }
            AudioRequest::SetVolume(AudioChannel::Master, volume) if *volume != volumes.master => {
                volumes.master = *volume;
                music.set_volume(volume * volumes.music);
                sfx.set_volume(volume * volumes.sfx);
            }
            // Volume is equal to what it is requested to be changed to
            AudioRequest::SetVolume(_, _) => {}
            AudioRequest::StopSfxLoop => {
                sfx.stop();
            }
            AudioRequest::PlayWoodClink(SfxParam::StartLoop) => {
                sfx.play(assets.wood_clink.clone_weak()).looped();
            }
            AudioRequest::PlayWoodClink(SfxParam::PlayOnce) => {
                sfx.play(assets.wood_clink.clone_weak());
            }
            AudioRequest::PlayWord(word) => {
                sfx.play(assets.words[*word].clone_weak());
            }
            AudioRequest::PlayShuffleShort => {
                sfx.play(assets.shuffle_short.clone_weak());
            }
            AudioRequest::PlayShuffleLong => {
                sfx.play(assets.shuffle_long.clone_weak());
            }
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .init_resource::<ChannelVolumes>()
            .init_resource::<AudioAssets>()
            .add_event::<AudioRequest>()
            .add_audio_channel::<Music>()
            .add_audio_channel::<Sfx>()
            .add_system(play_audio.label(AudioRequestSystem));
    }
}
