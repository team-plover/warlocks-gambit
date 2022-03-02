use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use fastrand::f32 as randf32;

use crate::state::GameState;

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(PartialEq, Clone, Copy)]
pub enum PileType {
    War,
    Player,
    Oppo,
}

/// Where to drop played cards
#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct Pile {
    stack_size: usize,
    pub which: PileType,
}
impl Pile {
    // TODO: account for Participant when throwing into the war pile
    pub fn additional_card(&mut self) -> PileCard {
        let Self { stack_size, which } = *self;
        self.stack_size += 1;
        PileCard::new(stack_size, which)
    }
    pub fn new(which: PileType) -> Self {
        Self { stack_size: 0, which }
    }
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct PileCard {
    offset: Transform,
    stack_pos: usize,
    which: PileType,
}

impl PileCard {
    fn new(stack_pos: usize, which: PileType) -> Self {
        let offset = Transform {
            translation: Vec3::new(randf32() * 0.6 - 0.3, 0.0, randf32() * 0.6 - 0.3),
            rotation: Quat::from_rotation_z(randf32() - 0.5),
            scale: Vec3::ONE,
        };
        Self { offset, stack_pos, which }
    }
    // pub fn last_in_pile(&self, pile: &Pile) -> bool {
    //     self.stack_pos == pile.stack_size - 1 && self.which == pile.which
    // }
}

fn move_to_pile(
    pile: Query<(&GlobalTransform, &Pile)>,
    mut cards: Query<(&mut Transform, &PileCard)>,
) {
    const CARD_SPEED: f32 = 0.15;
    for (mut transform, PileCard { offset, stack_pos, which }) in cards.iter_mut() {
        let (pile_transform, _) = pile
            .iter()
            .find(|p| &p.1.which == which)
            .expect("Pile exists");
        let pile_pos = pile_transform.translation;
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
