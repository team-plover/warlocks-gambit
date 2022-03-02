use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use fastrand::f32 as randf32;

use crate::state::GameState;

/// Where to drop played cards
#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component, Default)]
pub struct Pile {
    stack_size: usize,
}
impl Pile {
    pub fn additional_card(&mut self) -> PileCard {
        let stack_pos = self.stack_size;
        self.stack_size += 1;
        PileCard::new(stack_pos)
    }
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct PileCard {
    offset: Transform,
    stack_pos: usize,
}

impl PileCard {
    fn new(stack_pos: usize) -> Self {
        let translation = Vec3::new(randf32() * 0.6 - 0.3, 0.0, randf32() * 0.6 - 0.3);
        let rotation = Quat::from_rotation_z(randf32() - 0.5);
        let scale = Vec3::ONE;
        let offset = Transform { translation, rotation, scale };
        Self { offset, stack_pos }
    }
}

fn move_to_pile(
    pile: Query<&GlobalTransform, With<Pile>>,
    mut cards: Query<(&mut Transform, &PileCard)>,
) {
    const CARD_SPEED: f32 = 0.15;
    let pile_transform = pile.single();
    let pile_pos = pile_transform.translation;
    for (mut transform, PileCard { offset, stack_pos }) in cards.iter_mut() {
        let target = pile_pos + offset.translation + Vec3::Y * 0.012 * *stack_pos as f32;
        let origin = transform.translation;
        transform.translation += (target - origin) * CARD_SPEED;

        let target = pile_transform.rotation * offset.rotation;
        let origin = transform.rotation;
        transform.rotation = origin.lerp(target, CARD_SPEED);
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<PileCard>()
            .register_inspectable::<Pile>();
        app.add_system_set(SystemSet::on_update(self.0).with_system(move_to_pile));
    }
}
