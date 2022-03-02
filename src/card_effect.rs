//! What happens after activating a card

use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::{
    pile::{Pile, PileType},
    state::{GameState, TurnState},
    Participant,
};

pub struct ActivateCard {
    pub card: Entity,
    pub who: Participant,
}
impl ActivateCard {
    pub fn new(card: Entity, who: Participant) -> Self {
        Self { card, who }
    }
}

#[derive(Default)]
pub struct TurnCount(pub usize);

fn handle_activated(
    mut events: EventReader<ActivateCard>,
    mut cmds: Commands,
    mut pile: Query<&mut Pile>,
    mut turn: ResMut<State<TurnState>>,
) {
    use PileType::War;
    for ActivateCard { card, who } in events.iter() {
        let mut pile = pile
            .iter_mut()
            .find(|p| p.which == War)
            .expect("War pile exists");
        cmds.entity(*card).insert(pile.additional_card());

        let new_state = match who {
            Participant::Oppo => TurnState::OppoActivated,
            Participant::Player => TurnState::PlayerActivated,
        };
        turn.set(new_state).unwrap();
    }
}

fn handle_player_active(mut turn: ResMut<State<TurnState>>, mut turn_count: ResMut<TurnCount>) {
    turn.set(TurnState::Oppo).unwrap();
    turn_count.0 += 1;
}

fn handle_oppo_active(mut turn: ResMut<State<TurnState>>, mut turn_count: ResMut<TurnCount>) {
    turn.set(TurnState::Player).unwrap();
    turn_count.0 += 1;
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActivateCard>()
            .init_resource::<TurnCount>()
            .add_system_set(SystemSet::on_update(self.0).with_system(handle_activated))
            .add_system_set(
                SystemSet::on_update(TurnState::PlayerActivated).with_system(handle_player_active),
            )
            .add_system_set(
                SystemSet::on_update(TurnState::OppoActivated).with_system(handle_oppo_active),
            );
    }
}
