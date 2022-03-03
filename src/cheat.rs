use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::{animate::Animated, card_spawner::PlayerSleeve};

#[derive(Debug)]
pub enum CheatEvent {
    HideInSleeve(Entity),
}

#[derive(Component)]
pub struct SleeveCard;

fn execute_cheat(
    sleeve: Query<&GlobalTransform, With<PlayerSleeve>>,
    mut cmds: Commands,
    mut events: EventReader<CheatEvent>,
) {
    for event in events.iter() {
        match event {
            CheatEvent::HideInSleeve(entity) => {
                let mut target: Transform = (*sleeve.single()).into();
                target.translation -= Vec3::Y * 1.5;
                cmds.entity(*entity)
                    .insert(SleeveCard)
                    .insert(Animated::MoveInto { target, speed: 1.0 });
            }
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CheatEvent>().add_system(execute_cheat);
    }
}
