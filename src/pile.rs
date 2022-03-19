use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use fastrand::f32 as randf32;

use crate::{state::GameState, Participant};

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(PartialEq, Clone, Copy)]
pub enum PileType {
    War,
    Player,
    Oppo,
}
impl From<Participant> for PileType {
    fn from(who: Participant) -> Self {
        match who {
            Participant::Oppo => Self::Oppo,
            Participant::Player => Self::Player,
        }
    }
}

/// Where to drop played cards
#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct Pile {
    stack_size: usize,
    pub which: PileType,
}
impl Pile {
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
    pub which: PileType,
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
}

fn move_to_pile(
    pile: Query<(&GlobalTransform, &Pile)>,
    mut cards: Query<(&mut Transform, &PileCard)>,
    time: Res<Time>,
) {
    let card_speed = 10.0 * time.delta_seconds();
    for (mut transform, PileCard { offset, stack_pos, which }) in cards.iter_mut() {
        let msg = "Pile exists";
        let (pile_transform, _) = pile.iter().find(|p| &p.1.which == which).expect(msg);
        let pile_pos = pile_transform.translation;
        let target = pile_pos + offset.translation + Vec3::Y * 0.012 * *stack_pos as f32;
        let origin = transform.translation;
        transform.translation += (target - origin) * card_speed;

        let target = pile_transform.rotation * offset.rotation;
        let origin = transform.rotation;
        transform.rotation = origin.lerp(target, card_speed);
    }
}

// TODO: add system to readjust Pile stack_size and PileCard stack_pos in
// PostUpdate

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<PileCard>()
            .register_inspectable::<Pile>();
        app.add_system_set(SystemSet::on_update(self.0).with_system(move_to_pile));
    }
}
