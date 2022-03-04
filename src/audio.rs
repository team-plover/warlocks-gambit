use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_kira_audio::{Audio, AudioChannel as KiraChannel, AudioPlugin, AudioSource};
use enum_map::{enum_map, EnumMap};

use crate::card::WordOfPower;

#[derive(Clone, Copy, PartialEq)]
pub enum AudioChannel {
    Master,
    Sfx,
    Music,
}
struct ChannelVolumes {
    master: f32,
    sfx: f32,
    music: f32,
}
struct AudioChannels {
    sfx: KiraChannel,
    music: KiraChannel,
    volumes: ChannelVolumes,
}
impl Default for AudioChannels {
    fn default() -> Self {
        Self {
            sfx: KiraChannel::new("sfx".to_owned()),
            music: KiraChannel::new("music".to_owned()),
            volumes: ChannelVolumes { master: 1.0, sfx: 0.5, music: 0.5 },
        }
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
    SetChannelVolume(AudioChannel, f32),
}
fn play_audio(
    assets: Res<AudioAssets>,
    audio: Res<Audio>,
    mut channels: ResMut<AudioChannels>,
    mut events: EventReader<AudioRequest>,
) {
    for event in events.iter() {
        match event {
            AudioRequest::StartMusic => {
                audio.play_looped_in_channel(assets.music.clone(), &channels.music);
            }
            AudioRequest::SetChannelVolume(AudioChannel::Sfx, volume) => {
                channels.volumes.sfx = *volume;
                let master = channels.volumes.master;
                audio.set_volume_in_channel(volume * master, &channels.sfx);
            }
            AudioRequest::SetChannelVolume(AudioChannel::Music, volume) => {
                channels.volumes.music = *volume;
                let master = channels.volumes.master;
                audio.set_volume_in_channel(volume * master, &channels.music);
            }
            AudioRequest::SetChannelVolume(AudioChannel::Master, volume) => {
                channels.volumes.master = *volume;
                let music_volume = volume * channels.volumes.music;
                let sfx_volume = volume * channels.volumes.sfx;
                audio.set_volume_in_channel(music_volume, &channels.music);
                audio.set_volume_in_channel(sfx_volume, &channels.sfx);
            }
            AudioRequest::StopSfxLoop => {
                audio.stop_channel(&channels.sfx);
            }
            AudioRequest::PlayWoodClink(SfxParam::StartLoop) => {
                audio.play_looped_in_channel(assets.wood_clink.clone(), &channels.sfx);
            }
            AudioRequest::PlayWoodClink(SfxParam::PlayOnce) => {
                audio.play_in_channel(assets.wood_clink.clone(), &channels.sfx);
            }
            AudioRequest::PlayWord(word) => {
                audio.play_in_channel(assets.words[*word].clone(), &channels.sfx);
            }
            AudioRequest::PlayShuffleShort => {
                audio.play_in_channel(assets.shuffle_short.clone(), &channels.sfx);
            }
            AudioRequest::PlayShuffleLong => {
                audio.play_in_channel(assets.shuffle_long.clone(), &channels.sfx);
            }
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .init_resource::<AudioChannels>()
            .init_resource::<AudioAssets>()
            .add_event::<AudioRequest>()
            .add_system(play_audio);
    }
}
