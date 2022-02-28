use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

use crate::{
    camera::PlayerCam,
    card::{Card, SpawnCard, Value, WordOfMagic},
    state::GameState,
};

#[derive(Component)]
struct Hand;

#[derive(Component)]
struct HoveredCard;

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
struct HandCard {
    index: usize,
}
impl HandCard {
    fn new(index: usize) -> Self {
        Self { index }
    }
}

fn spawn_hand(
    mut cmds: Commands,
    cam: Query<Entity, With<PlayerCam>>,
    mut card_spawner: SpawnCard,
) {
    use Value::{Eight, Seven, Two, Zero};
    let cam = cam.single();
    let hand = cmds
        .spawn_bundle((
            GlobalTransform::default(),
            Transform::from_xyz(0.0, -3.6, -10.0),
            Name::new("Player hand"),
            Hand,
            Parent(cam),
        ))
        .id();
    for (i, value) in [Zero, Two, Seven, Eight].iter().enumerate() {
        card_spawner
            .spawn_card(Card::new(WordOfMagic::Wealth, *value))
            .insert_bundle((
                HandCard::new(i),
                Parent(hand),
                GlobalTransform::default(),
                Transform::default(),
            ));
    }
}

// Workaround the Hand (and children) GlobalTransform not being set correctly
// when spawned
fn update_hand_transform(
    query: Query<Entity, Added<Hand>>,
    mut cam: Query<&mut Transform, (Without<Hand>, With<PlayerCam>)>,
) {
    if query.get_single().is_ok() {
        cam.single_mut().set_changed();
    }
}

fn update_hand(mut hand: Query<(&mut Transform, &HandCard, Option<&HoveredCard>)>) {
    for (mut transform, HandCard { index }, hover) in hand.iter_mut() {
        let i_f32 = *index as f32;
        let horizontal_offset = if hover.is_some() { 0.0 } else { 2.0 };
        let z_offset = i_f32 * 0.1;
        // TODO: full transform lerp
        let target = Vec3::new(i_f32 - horizontal_offset, -0.0, z_offset);
        let origin = transform.translation;
        transform.translation += (target - origin) * 0.5;
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<HandCard>();
        app.add_system_set(SystemSet::on_enter(self.0).with_system(spawn_hand))
            .add_system_to_stage(CoreStage::PreUpdate, update_hand_transform)
            .add_system_set(SystemSet::on_update(self.0).with_system(update_hand));
    }
}
