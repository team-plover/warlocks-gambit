//! What happens after activating a card

use bevy::ecs::system::SystemParam;
use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::{
    card::Card,
    card_spawner::CardOrigin,
    pile::{Pile, PileCard, PileType},
    state::{GameState, TurnState},
    war::BattleOutcome,
    Participant,
};

pub struct Initiative(Participant);
impl Initiative {
    fn swap(&mut self) {
        match self.0 {
            Participant::Oppo => self.0 = Participant::Player,
            Participant::Player => self.0 = Participant::Oppo,
        }
    }
}

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

fn handle_turn_end(
    cards: Query<(&PileCard, &CardOrigin, &Card, Entity)>,
    mut piles: Query<&mut Pile>,
    mut turn: ResMut<State<TurnState>>,
    mut cmds: Commands,
) {
    use PileType::{Oppo, Player, War};
    let war_pile: Vec<_> = cards.iter().filter(|c| c.0.which == War).collect();
    macro_rules! add_card_to_pile {
        ($entity:expr, $pile_type:expr) => {
            let mut pile = piles.iter_mut().find(|p| p.which == $pile_type).unwrap();
            cmds.entity($entity).insert(pile.additional_card());
        };
    }
    match &war_pile[..] {
        [card1, card2] => {
            let (player_card, oppo_card) = if card1.1 .0 == Participant::Player {
                (card1, card2)
            } else {
                (card2, card1)
            };
            match player_card.2.value.beats(&oppo_card.2.value) {
                BattleOutcome::Tie => {
                    add_card_to_pile!(player_card.3, Player);
                    add_card_to_pile!(oppo_card.3, Oppo);
                }
                BattleOutcome::Loss => {
                    add_card_to_pile!(player_card.3, Oppo);
                    add_card_to_pile!(oppo_card.3, Oppo);
                }
                BattleOutcome::Win => {
                    add_card_to_pile!(player_card.3, Player);
                    add_card_to_pile!(oppo_card.3, Player);
                }
            }
            let err_msg = "handle_turn_end only activated when in '*Activated' state";
            turn.set(TurnState::New).expect(err_msg);
        }
        [] | [_] => {}
        _ => {
            unreachable!("There should be no more than 2 cards on the war pile");
        }
    }
}

fn handle_new_turn(
    mut initative: ResMut<Initiative>,
    mut turn: ResMut<State<TurnState>>,
    mut turn_count: ResMut<TurnCount>,
    hands: Query<(), (With<CardOrigin>, Without<PileCard>)>,
) {
    if hands.iter().count() == 0 {
        turn.set(TurnState::Draw).unwrap();
    } else {
        match initative.0 {
            Participant::Oppo => turn.set(TurnState::Oppo).unwrap(),
            Participant::Player => turn.set(TurnState::Player).unwrap(),
        };
    }
    turn_count.0 += 1;
    initative.swap();
}

fn complete_draw(
    initative: Res<Initiative>,
    mut turn: ResMut<State<TurnState>>,
    // Sets of cards that are not in piles (aka: in hand)
    hands: Query<(), (With<CardOrigin>, Without<PileCard>)>,
) {
    if hands.iter().count() >= 6 {
        match initative.0 {
            Participant::Oppo => turn.set(TurnState::Oppo).unwrap(),
            Participant::Player => turn.set(TurnState::Player).unwrap(),
        };
    }
}

#[derive(SystemParam)]
struct ActiveParams<'w, 's> {
    turn: ResMut<'w, State<TurnState>>,
    time: Res<'w, Time>,
    timeout: Local<'s, Option<f64>>,
}

// TODO: handle Word effects
fn handle_generic_active(goes_into: TurnState, mut params: ActiveParams) {
    const TURN_INTERLUDE: f64 = 0.5;
    match *params.timeout {
        Some(some_timeout) if some_timeout < params.time.seconds_since_startup() => {
            params.turn.set(goes_into).unwrap();
            *params.timeout = None;
        }
        None => {
            *params.timeout = Some(params.time.seconds_since_startup() + TURN_INTERLUDE);
        }
        _ => {}
    };
}

fn handle_player_active(params: ActiveParams) {
    handle_generic_active(TurnState::Oppo, params);
}

fn handle_oppo_active(params: ActiveParams) {
    handle_generic_active(TurnState::Player, params);
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use TurnState::{OppoActivated, PlayerActivated};
        app.add_event::<ActivateCard>()
            .init_resource::<TurnCount>()
            .insert_resource(Initiative(Participant::Player))
            .add_system_set(SystemSet::on_update(self.0).with_system(handle_activated))
            .add_system_set(SystemSet::on_update(TurnState::New).with_system(handle_new_turn))
            .add_system_set(SystemSet::on_update(TurnState::Draw).with_system(complete_draw))
            .add_system_set(SystemSet::on_exit(PlayerActivated).with_system(handle_turn_end))
            .add_system_set(SystemSet::on_exit(OppoActivated).with_system(handle_turn_end))
            .add_system_set(SystemSet::on_update(PlayerActivated).with_system(handle_player_active))
            .add_system_set(SystemSet::on_update(OppoActivated).with_system(handle_oppo_active));
    }
}
