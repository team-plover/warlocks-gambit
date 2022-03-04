use bevy::input::keyboard::KeyCode;
use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::add_dbg_text;
use crate::{
    animate::Animated, card_effect::SeedCount, card_spawner::PlayerSleeve, game_ui::EffectEvent,
    state::GameState, ui::gameover::GameOverKind,
};

#[derive(Debug)]
pub enum CheatEvent {
    HideInSleeve(Entity),
    ConfuseBird,
}

#[derive(Component)]
pub struct SleeveCard;

pub struct BirdEye {
    pub is_watching: bool,
}
impl Default for BirdEye {
    fn default() -> Self {
        Self { is_watching: true }
    }
}

fn cleanup(mut bird_eye: ResMut<BirdEye>) {
    *bird_eye = BirdEye::default();
}

fn use_seed(
    mut seed: ResMut<SeedCount>,
    mut cheats: EventWriter<CheatEvent>,
    mut ui: EventWriter<EffectEvent>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) && seed.count() != 0 {
        assert!(seed.consume());
        cheats.send(CheatEvent::ConfuseBird);
        ui.send(EffectEvent::UseSeed);
    }
}

fn execute_cheat(
    sleeve: Query<&GlobalTransform, With<PlayerSleeve>>,
    mut gameover_events: EventWriter<GameOverKind>,
    mut ui: EventWriter<EffectEvent>,
    mut watch: ResMut<BirdEye>,
    mut cmds: Commands,
    mut events: EventReader<CheatEvent>,
) {
    for event in events.iter() {
        match event {
            CheatEvent::ConfuseBird => {
                watch.is_watching = false;
            }
            CheatEvent::HideInSleeve(_) if watch.is_watching => {
                add_dbg_text!("you got caught cheating!");
                gameover_events.send(GameOverKind::CheatSpotted);
            }
            CheatEvent::HideInSleeve(entity) => {
                let mut target: Transform = (*sleeve.single()).into();
                target.translation -= Vec3::Y * 1.5;
                ui.send(EffectEvent::EndCheat);
                cmds.entity(*entity)
                    .insert(SleeveCard)
                    .insert(Animated::MoveInto { target, speed: 1.0 });
            }
        }
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BirdEye>()
            .add_system_set(SystemSet::on_exit(self.0).with_system(cleanup));
        app.add_event::<CheatEvent>()
            .add_system(use_seed)
            .add_system(execute_cheat);
    }
}
