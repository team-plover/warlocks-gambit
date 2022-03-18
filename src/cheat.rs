use bevy::input::keyboard::KeyCode;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_debug_text_overlay::screen_print;

use crate::{
    animate::Animated,
    card_effect::SeedCount,
    card_spawner::{BirdPupil, BirdPupilRoot, GameStarts, GrabbedCard, PlayerSleeve},
    game_ui::EffectEvent,
    state::GameState,
    ui::gameover::GameOverKind,
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

fn control_bird_pupil(
    eye_status: Res<BirdEye>,
    mut eye: Query<&mut Transform, With<BirdPupil>>,
    grabbed_card: Query<&Transform, (With<GrabbedCard>, Without<BirdPupil>)>,
) {
    if eye_status.is_watching {
        match (grabbed_card.get_single(), eye.get_single_mut()) {
            (Ok(look_at), Ok(mut eye)) => {
                screen_print!("Tracking player card");
                let hand = look_at.translation;
                let new_trans = Vec3::new(hand.x / 2.7, (hand.y - 6.05) / 1.65, 0.0) * 0.1;
                eye.translation = new_trans;
            }
            (Err(_), Ok(mut eye)) => {
                screen_print!("Not tracking player card");
                eye.translation = Vec3::ZERO;
            }
            _ => {}
        }
    }
}

fn execute_cheat(
    sleeve: Query<&GlobalTransform, With<PlayerSleeve>>,
    game_starts: Res<GameStarts>,
    mut bird_eye: Query<&mut Animated, With<BirdPupilRoot>>,
    mut gameover_events: EventWriter<GameOverKind>,
    mut ui: EventWriter<EffectEvent>,
    mut watch: ResMut<BirdEye>,
    mut cmds: Commands,
    mut events: EventReader<CheatEvent>,
    mut tuto_shown: Local<bool>,
) {
    for event in events.iter() {
        match event {
            CheatEvent::ConfuseBird => {
                watch.is_watching = false;
                if let Ok(mut anim) = bird_eye.get_single_mut() {
                    *anim = Animated::Circle { radius: 0.1, period: 1.0, offset: 0.0 };
                }
                if game_starts.0 == 2 && !*tuto_shown {
                    *tuto_shown = true;
                    ui.send(EffectEvent::TutoSleeve);
                }
            }
            CheatEvent::HideInSleeve(_) if watch.is_watching => {
                screen_print!("you got caught cheating!");
                gameover_events.send(GameOverKind::CheatSpotted);
            }
            CheatEvent::HideInSleeve(entity) => {
                let mut target: Transform = (*sleeve.single()).into();
                target.translation -= Vec3::Y * 1.5;
                if let Ok(mut anim) = bird_eye.get_single_mut() {
                    *anim = Animated::Static;
                }
                watch.is_watching = true;
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
        app.add_event::<CheatEvent>()
            .init_resource::<BirdEye>()
            .add_system_set(SystemSet::on_exit(self.0).with_system(cleanup))
            .add_system(use_seed)
            .add_system(control_bird_pupil)
            .add_system(execute_cheat);
    }
}
