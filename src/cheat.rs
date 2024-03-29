//! Let the player put cards in his sleeve, and control when they are spotted
//! and distracting the watchers.
//!
//! Mostly responsible for moving into the sleeve, moving the bird eye.
//!
//! But it also defines the [`CheatEvent`] events, they are read in the
//! [`execute_cheat`] system, it controls the game over condition when player
//! forgot to distract the bird before cheating.
use bevy::input::keyboard::KeyCode;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_debug_text_overlay::screen_print;

use crate::{
    animate::Animated, game_flow::SeedCount, game_ui::EffectEvent, player_hand::GrabbedCard,
    state::GameState, EndReason, GameOver,
};

#[derive(Component)]
pub struct BirdPupilRoot;

#[derive(Component)]
pub struct BirdPupil;

/// Where to stash cards added to sleeve
#[derive(Component)]
pub struct PlayerSleeve;

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

fn cleanup(
    mut bird_eye: ResMut<BirdEye>,
    mut bird_eye_anim: Query<&mut Animated, With<BirdPupilRoot>>,
) {
    *bird_eye = BirdEye::default();
    if let Ok(mut bird_eye_anim) = bird_eye_anim.get_single_mut() {
        *bird_eye_anim = Animated::Static;
    }
}

fn use_seed(
    mut seed: ResMut<SeedCount>,
    mut cheats: EventWriter<CheatEvent>,
    mut ui: EventWriter<EffectEvent>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) && seed.consume() {
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

// HACK: fix the transform of the child mesh used for detecting we are hovering
// the sleeve not updating correctly on being loaded.
// This is because bavy doesn't properly update the transform of children that
// were just added if the parent transform is never updated after it being added.
fn update_sleeve_transform(
    mut transform: Query<&mut Transform, (With<PlayerSleeve>, Changed<Children>)>,
) {
    if let Ok(transform) = transform.get_single_mut() {
        let _ = transform.into_inner();
    }
}

fn execute_cheat(
    mut bird_eye: Query<&mut Animated, With<BirdPupilRoot>>,
    mut gameover_events: EventWriter<GameOver>,
    mut ui: EventWriter<EffectEvent>,
    mut watch: ResMut<BirdEye>,
    mut cmds: Commands,
    mut events: EventReader<CheatEvent>,
) {
    for event in events.iter() {
        match event {
            CheatEvent::ConfuseBird => {
                watch.is_watching = false;
                if let Ok(mut anim) = bird_eye.get_single_mut() {
                    *anim = Animated::Circle { radius: 0.1, period: 1.0, offset: 0.0 };
                }
            }
            CheatEvent::HideInSleeve(_) if watch.is_watching => {
                screen_print!("caught cheating");
                gameover_events.send(GameOver(EndReason::CaughtCheating));
            }
            CheatEvent::HideInSleeve(entity) => {
                if let Ok(mut anim) = bird_eye.get_single_mut() {
                    *anim = Animated::Static;
                }
                watch.is_watching = true;
                ui.send(EffectEvent::EndCheat);
                cmds.entity(*entity).insert(SleeveCard);
            }
        }
    }
}

fn follow_sleeve(
    mut cards: Query<&mut Transform, With<SleeveCard>>,
    sleeve: Query<&GlobalTransform, With<PlayerSleeve>>,
    time: Res<Time>,
) {
    let card_speed = 10.0 * time.delta_seconds();
    for mut transform in cards.iter_mut() {
        let sleeve_pos = sleeve.single().compute_transform();
        let target = sleeve_pos.translation;
        let origin = transform.translation;
        transform.translation += (target - origin) * card_speed;

        let target = sleeve_pos.rotation;
        let origin = transform.rotation;
        transform.rotation = origin.lerp(target, card_speed);
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CheatEvent>()
            .init_resource::<BirdEye>()
            .add_system_set(SystemSet::on_exit(self.0).with_system(cleanup))
            .add_system_set(SystemSet::on_update(self.0).with_system(update_sleeve_transform))
            .add_system(use_seed)
            .add_system(follow_sleeve)
            .add_system(control_bird_pupil)
            .add_system(execute_cheat);
    }
}
