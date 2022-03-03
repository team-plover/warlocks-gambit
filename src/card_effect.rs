//! What happens after activating a card (including win/loss conditions and
//! score tracking, this module should be renamed into "game_state" or
//! something)

use bevy::ecs::system::SystemParam;
use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::{
    card::Card,
    card_spawner::CardOrigin,
    cheat::SleeveCard,
    deck::{OppoDeck, PlayerDeck},
    pile::{Pile, PileCard, PileType},
    state::{GameState, TurnState},
    ui::gameover::GameOverKind,
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

// Sets of cards that are not in piles (aka: in hand)
type HandFilter = (With<CardOrigin>, Without<PileCard>, Without<SleeveCard>);

#[allow(clippy::type_complexity)]
fn handle_new_turn(
    mut initative: ResMut<Initiative>,
    mut turn: ResMut<State<TurnState>>,
    mut turn_count: ResMut<TurnCount>,
    mut gameover_events: EventWriter<GameOverKind>,
    hands: Query<(), HandFilter>,
    player_deck: Res<PlayerDeck>,
    oppo_deck: Res<OppoDeck>,
    piles: Query<(&PileCard, &Card)>,
) {
    let remaining_scores = player_deck.score() + oppo_deck.score();
    let scores = |(pile, card): (&PileCard, &Card)| match pile.which {
        PileType::Player => (card.value as i32, 0),
        PileType::Oppo => (0, card.value as i32),
        PileType::War => (0, 0),
    };
    let add_tuples = |(t1_1, t1_2), (t2_1, t2_2)| (t1_1 + t2_1, t1_2 + t2_2);
    let (player_score, oppo_score) = piles.iter().map(scores).fold((0, 0), add_tuples);
    if player_score - oppo_score > remaining_scores as i32 {
        gameover_events.send(GameOverKind::PlayerWon);
        return;
    } else if oppo_score - player_score > remaining_scores as i32 {
        gameover_events.send(GameOverKind::PlayerLost);
        return;
    }
    turn_count.0 += 1;
    initative.swap();
    if hands.iter().count() == 0 {
        turn.set(TurnState::Draw).unwrap();
    } else {
        match initative.0 {
            Participant::Oppo => turn.set(TurnState::Oppo).unwrap(),
            Participant::Player => turn.set(TurnState::Player).unwrap(),
        };
    }
}

fn complete_draw(
    initative: Res<Initiative>,
    mut turn: ResMut<State<TurnState>>,
    hands: Query<(), HandFilter>,
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

fn cleanup(
    mut cmds: Commands,
    all_cards: Query<Entity, With<Card>>,
    mut turn_count: ResMut<TurnCount>,
    mut initative: ResMut<Initiative>,
) {
    turn_count.0 = 0;
    initative.0 = Participant::Player;
    for entity in all_cards.iter() {
        cmds.entity(entity).despawn_recursive();
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use TurnState::{OppoActivated, PlayerActivated};
        let handle_new_turn = handle_new_turn.before("check_gameover");
        app.add_event::<ActivateCard>()
            .init_resource::<TurnCount>()
            .insert_resource(Initiative(Participant::Player))
            .add_system_set(SystemSet::on_update(self.0).with_system(handle_activated))
            .add_system_set(SystemSet::on_enter(TurnState::New).with_system(handle_new_turn))
            .add_system_set(SystemSet::on_update(TurnState::Draw).with_system(complete_draw))
            .add_system_set(SystemSet::on_exit(PlayerActivated).with_system(handle_turn_end))
            .add_system_set(SystemSet::on_exit(OppoActivated).with_system(handle_turn_end))
            .add_system_set(SystemSet::on_exit(self.0).with_system(cleanup))
            .add_system_set(SystemSet::on_update(PlayerActivated).with_system(handle_player_active))
            .add_system_set(SystemSet::on_update(OppoActivated).with_system(handle_oppo_active));
    }
}
