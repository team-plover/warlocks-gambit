//! What happens after activating a card

use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::{pile::Pile, state::GameState};

pub struct ActivateCard(pub Entity);

fn handle_activated(
    mut events: EventReader<ActivateCard>,
    mut cmds: Commands,
    mut pile: Query<&mut Pile>,
) {
    for ActivateCard(card) in events.iter() {
        let mut pile = pile.single_mut();
        cmds.entity(*card).insert(pile.additional_card());
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(self.0).with_system(handle_activated))
            .add_event::<ActivateCard>();
    }
}
