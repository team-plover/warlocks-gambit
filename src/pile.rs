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
    #[cfg_attr(feature = "debug", inspectable(ignore))]
    stack: Vec<Entity>,
    pub which: PileType,
}
impl Pile {
    pub fn add_existing(&mut self, entity: Entity) -> PileCard {
        self.stack.push(entity);
        let which = self.which;
        PileCard::new(which)
    }
    pub fn remove(&mut self, entity: Entity) {
        if let Some((old_index, _)) = self.stack.iter().enumerate().find(|(_, e)| **e == entity) {
            self.stack.remove(old_index);
        }
    }
    pub fn new(which: PileType) -> Self {
        Self { which, stack: Vec::new() }
    }
    pub fn cards(&self) -> &[Entity] {
        &self.stack
    }
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct PileCard {
    offset: Transform,
    pub which: PileType,
}

impl PileCard {
    fn new(which: PileType) -> Self {
        let offset = Transform {
            translation: Vec3::new(randf32() * 0.6 - 0.3, 0.0, randf32() * 0.6 - 0.3),
            rotation: Quat::from_rotation_z(randf32() - 0.5),
            scale: Vec3::ONE,
        };
        Self { offset, which }
    }
}

fn move_to_pile(
    piles: Query<(&GlobalTransform, &Pile)>,
    mut cards: Query<(&mut Transform, &PileCard)>,
    time: Res<Time>,
) {
    let card_speed = 10.0 * time.delta_seconds();
    for (pile_transform, Pile { stack, .. }) in piles.iter() {
        let mut stack_pos = 0_f32;
        for &entity in stack.iter() {
            if let Ok((mut transform, PileCard { offset, .. })) = cards.get_mut(entity) {
                let pile_pos = pile_transform.translation;
                let target = pile_pos + offset.translation + Vec3::Y * stack_pos;
                let origin = transform.translation;
                // give cool effect of falling
                let trans_speed = Vec3::new(1., 0.7, 1.) * card_speed;
                transform.translation += (target - origin) * trans_speed;

                let target = pile_transform.rotation * offset.rotation;
                let origin = transform.rotation;
                transform.rotation = origin.lerp(target, card_speed);
                stack_pos += 0.008;
            }
        }
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use crate::system_helper::EasySystemSetCtor;
        #[cfg(feature = "debug")]
        app.register_inspectable::<PileCard>()
            .register_inspectable::<Pile>();

        app.add_system_set(self.0.on_update(move_to_pile));
    }
}
