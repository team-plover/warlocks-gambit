//! Game flow driver, manages how cards are played, when it is possible to
//! play them and who can play them. Also manages score and card effects.
//!
//! # Architecture
//!
//! This module does:
//! * Manage game state transitions (see [#Transitions] section).
//! * Compute points obtained in a turn and distribute cards from the war pile
//!   into their corresponding piles in accordance to the game rules, see
//!   [#Scores] section.
//!   * This includes keeping track of the effects active for this turn.
//!   * This includes providing an API to let other modules access the party
//!     scores, and therefore keeping track of the points.
//!
//! ## Transitions
//!
//! Note that the game enters [`TurnState::New`] whenever who is playing a card
//! changes.
//!
//! * [`handle_new_turn`]: end game if one of the players cannot win
//! * [`complete_draw`]: Set who's turn it is to play after drawing cards
//! * [`handle_played`]: Handle effects based on played card and enter
//!   `CardPlayed` state.
//! * [`wait_active`]: Wait a little time after a card is played
//! * [`handle_turn_end`]: Start new turn after swapping initiative,
//!   if two cards are played, update scores and distribute cards to
//!   the winner's pile.
//!
//! Following is the flowchart of the game logic, states are from the
//! [`TurnState`] `enum`.
//!
//! ### Flowchart
//! ```text
//!                   init
//!                    ↓
//!               -----------
//!               |New State|←-------------------------------←
//!               -----------                                |
//! -------------------|--------------------------------     |
//! |handle_new_turn   |                               |     |
//! |                  ↓                               |     |
//! |      has one of the players won?-→ no            |     |
//! |         ↓                          ↓             |     |
//! |        yes           are the players hand empty? |     |
//! |         |                ↓                 |     |     |
//! |         |               yes                no    |     |
//! ----------|----------------|-----------------|------     |
//!           ↓                ↓                 |           |
//!   ----------------    ------------           |           |
//!   |GameOver state|    |Draw state|           |           |
//!   ----------------    ------------           |           |
//!                            ↓                 |           |
//!             -----------------------------    |           |
//!             |complete_draw              |    |           |
//!             | wait until all cards drawn|    |           |
//!             -----------------------------    ↓           |
//!                            |    -----------------------  |
//!                            →---→|(Player | Oppo) State|  |
//!                                 -----------------------  |
//!                                            ↓             |
//!   ---------------------------------------------------    |
//!   | oppo_hand or player_hand send an PlayCard event |    |
//!   ---------------------------------------------------    |
//!                           ↓                              |
//!                      handle_played                       |
//!                           ↓                              |
//!                  --------------------                    |
//!                  | CardPlayed State |                    |
//!                  --------------------                    |
//!                           ↓                              |
//!                      wait_active--→ handle_turn_end→-----↑
//! ```
//!
//! ## Scores
//!
//! Player and opposition scores are tracked in this module. The
//! [`handle_turn_end`] system computes the points at the end of each "Battle"
//! according to specification in [crate::war] module and hands out point
//! bonuses based on played card [`WordOfPower`]s. Currently only four words
//! are handled. See [`handle_turn_end`] docs for specifics.
//!
//! The module provides the [`CardStats`] system parameter for other modules
//! to query the game scores.
//!
//! ## Effects
//!
//! The [`handle_played`] system adds card effects to the [`TurnEffects`] or
//! directly updates the [`SeedCount`] resource when an [`PlayCard`] event
//! is received, it then enters [`TurnState::CardPlayed`].

use bevy::ecs::{query::QueryItem, system::SystemParam};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_debug_text_overlay::screen_print;

use crate::{
    audio::AudioRequest,
    cheat::SleeveCard,
    deck::{OppoDeck, PlayerDeck},
    game_ui::EffectEvent,
    pile::{Pile, PileCard, PileType},
    state::{GameState, TurnState},
    war::{BattleOutcome, Card, Value, WordOfPower},
    CardOrigin, EndReason, GameOver, GameStarts, Participant,
};

/// Cards in the War pile
#[derive(Component)]
#[non_exhaustive]
pub struct PlayedCard;

/// Who is playing a card currently
pub struct Initiative(Participant);
impl Initiative {
    fn swap(&mut self) {
        match self.0 {
            Participant::Oppo => self.0 = Participant::Player,
            Participant::Player => self.0 = Participant::Oppo,
        }
    }
}

/// Play a card.
///
/// Used by sending an `PlayCard` event to an `EventWriter<PlayCard>`.
/// See [`handle_played`] for what happens when a card is played.
pub struct PlayCard {
    pub card: Entity,
    pub who: Participant,
}
impl PlayCard {
    pub fn new(card: Entity, who: Participant) -> Self {
        Self { card, who }
    }
}

/// Active effects.
///
/// It is updated in [`handle_played`] when a card is played. It is read and
/// reset in [`handle_turn_end`] when each player has played a card.
struct TurnEffects {
    /// Bonus multiplier of card values.
    multiplier: i32,
    /// The value of card of [`Value::Zero`] is 12.
    zero_bonus: bool,
}
impl Default for TurnEffects {
    fn default() -> Self {
        Self { multiplier: 1, zero_bonus: false }
    }
}
impl TurnEffects {
    /// Add the effect corresponding to the given [`WordOfPower`] to effects
    /// this turn.
    fn add(&mut self, word: WordOfPower) {
        use WordOfPower::*;
        match word {
            Qube => self.multiplier *= 2,
            Geh => self.zero_bonus = true,
            _ => {}
        }
    }
}

/// Keep track of extra points obtained from card effects. The "regular"
/// points are kept track of in the player and oppo [`Pile`]s.
#[derive(Default)]
pub struct ScoreBonuses {
    player: i32,
    oppo: i32,
}
impl ScoreBonuses {
    fn add_to_owner(&mut self, who: Participant, value: i32) {
        match who {
            Participant::Oppo => self.oppo += value,
            Participant::Player => self.player += value,
        }
    }
}

/// How many seeds the player has.
#[derive(Default)]
pub struct SeedCount(usize);
impl SeedCount {
    pub fn count(&self) -> usize {
        self.0
    }
    /// True if can use a seed (consuming it)
    pub fn consume(&mut self) -> bool {
        if self.0 != 0 {
            self.0 -= 1;
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct TurnCount(pub usize);

/// Handle [`PlayCard`] events.
///
/// Adds card effects from the [`PlayCard::card`] to the [`TurnEffects`] or
/// directly updates the [`SeedCount`] resource when an [`PlayCard`] event
/// is received, move the card to the war [`Pile`], and then enter the active
/// [`TurnState`] corresponding to [`PlayCard::who`] played the card.
///
/// ## Card effects
///
/// * `Egeq`: Give an extra seed to the player.
/// * `Qube`, `Zihbm` and `Geh`: See [`TurnEffects::add`].
fn handle_played(
    mut events: EventReader<PlayCard>,
    mut ui_events: EventWriter<EffectEvent>,
    mut cmds: Commands,
    mut pile: Query<&mut Pile>,
    mut turn: ResMut<State<TurnState>>,
    mut turn_effects: ResMut<TurnEffects>,
    mut seed_count: ResMut<SeedCount>,
    mut audio_events: EventWriter<AudioRequest>,
    mut tuto_shown: Local<bool>,
    game_starts: Res<GameStarts>,
    cards: Query<&Card>,
) {
    use PileType::War;
    use WordOfPower::*;
    for PlayCard { card, .. } in events.iter() {
        let msg = "War pile exists";
        let mut pile = pile.iter_mut().find(|p| p.which == War).expect(msg);
        cmds.entity(*card)
            .insert_bundle((pile.add_existing(*card), PlayedCard));
        let card_word = cards.get(*card).map(|c| c.word);
        audio_events.send(AudioRequest::PlayShuffleLong);
        if let Ok(Some(word)) = card_word {
            // TODO: spawn clouds of smoke
            ui_events.send(EffectEvent::Show(word));
            audio_events.send(AudioRequest::PlayWord(word));
        }
        match card_word {
            Ok(Some(Egeq)) => {
                if game_starts.0 == 2 && !*tuto_shown {
                    *tuto_shown = true;
                    ui_events.send(EffectEvent::TutoUseSeed);
                }
                seed_count.0 += 1;
            }
            Ok(Some(Qube)) => turn_effects.add(Qube),
            Ok(Some(Zihbm)) => turn_effects.add(Zihbm),
            Ok(Some(Geh)) => turn_effects.add(Geh),
            _ => {}
        }
        turn.set(TurnState::CardPlayed).unwrap();
    }
}

type CardsQuery = (
    &'static PileCard,
    &'static CardOrigin,
    &'static Card,
    Entity,
);

/// Handle what happens after a card is played
///
/// If there is exactly two cards in the war pile, compute results, move cards
/// to the winner pile(s) and add any bonus points to [`ScoreBonuses`] if
/// any card effects were in play this turn. Then enter new turn.
fn handle_turn_end(
    played_cards: Query<CardsQuery, With<PlayedCard>>,
    mut piles: Query<&mut Pile>,
    mut cmds: Commands,
    mut turn_effects: ResMut<TurnEffects>,
    mut score_bonuses: ResMut<ScoreBonuses>,
) {
    use Participant::{Oppo, Player};
    use PileType::War;

    let war_pile: Vec<_> = played_cards.iter().collect();

    // TODO: move this to war.rs and add tests
    let mut add_card_to_pile = |(_, _, card, entity): QueryItem<CardsQuery>, who: Participant| {
        let mut pile = piles.iter_mut().find(|p| p.which == who.into()).unwrap();
        cmds.entity(entity)
            .insert(pile.add_existing(entity))
            .remove::<PlayedCard>();
        let mut old_pile = piles.iter_mut().find(|p| p.which == War).unwrap();
        old_pile.remove(entity);
        let multi = turn_effects.multiplier - 1;
        // TODO: this is broken with the SWAP modifier I think?
        let zero_bonus = card.value == Value::Zero && turn_effects.zero_bonus;
        let zero_bonus = if zero_bonus { 12 } else { 0 };
        let total_bonus = card.value as i32 * multi + zero_bonus * (multi + 1);
        score_bonuses.add_to_owner(who, total_bonus);
    };
    match war_pile[..] {
        [card1, card2] => {
            let player_is_1 = card1.1 .0 == Participant::Player;
            let (player, oppo) = if player_is_1 { (card1, card2) } else { (card2, card1) };
            match player.2.beats(oppo.2) {
                BattleOutcome::Tie => {
                    add_card_to_pile(player, Player);
                    add_card_to_pile(oppo, Oppo);
                }
                BattleOutcome::Loss => {
                    add_card_to_pile(player, Oppo);
                    add_card_to_pile(oppo, Oppo);
                }
                BattleOutcome::Win => {
                    add_card_to_pile(player, Player);
                    add_card_to_pile(oppo, Player);
                }
            }
            *turn_effects = TurnEffects::default();
        }
        [] | [_] => {}
        _ => {
            unreachable!("There should be no more than 2 cards on the war pile");
        }
    }
}

/// Sets of cards that are not in piles (aka: in hand)
type HandFilter = (With<CardOrigin>, Without<PileCard>, Without<SleeveCard>);

/// Query scores.
///
/// A [`Participant`]'s score is exactly the [`Value`] of cards in their
/// [`Pile`] plus any bonus points earned with [`WordOfPower`]s. Since it is
/// not trivial to compute it, this `SystemParam` let you query the scores
/// through its methods.
#[derive(SystemParam)]
pub struct CardStats<'w, 's> {
    piles: Query<'w, 's, (&'static PileCard, &'static Card)>,
    hands: Query<'w, 's, &'static Card, HandFilter>,
    sleeve: Query<'w, 's, &'static Card, With<SleeveCard>>,
    player_deck: Query<'w, 's, &'static PlayerDeck>,
    oppo_deck: Query<'w, 's, &'static OppoDeck>,
    score_bonuses: Res<'w, ScoreBonuses>,
}
impl<'w, 's> CardStats<'w, 's> {
    pub fn remaining_score(&self) -> i32 {
        let hands_score: i32 = self.hands.iter().map(Card::max_value).sum();
        let sleeve_score: i32 = self.sleeve.iter().map(Card::max_value).sum();
        let player_score = self.player_deck.single().score();
        let oppo_score = self.oppo_deck.single().score();
        player_score + oppo_score + sleeve_score + hands_score
    }
    pub fn player_score(&self) -> i32 {
        use PileType::Player;
        let player_score: i32 = self
            .piles
            .iter()
            .filter_map(|(p, c)| matches!(p.which, Player).then(|| c.value as i32))
            .sum();
        player_score + self.score_bonuses.player
    }
    pub fn oppo_score(&self) -> i32 {
        use PileType::Oppo;
        let oppo_score: i32 = self
            .piles
            .iter()
            .filter_map(|(p, c)| matches!(p.which, Oppo).then(|| c.value as i32))
            .sum();
        oppo_score + self.score_bonuses.oppo
    }
}

/// Check for score-based lose/win conditions and enter selection state.
fn handle_new_turn(
    mut initative: ResMut<Initiative>,
    mut turn: ResMut<State<TurnState>>,
    mut turn_count: ResMut<TurnCount>,
    mut gameover_events: EventWriter<GameOver>,
    hands: Query<(), HandFilter>,
    card_stats: CardStats,
) {
    screen_print!(sec: 1.0, "handle turn n*{}", turn_count.0);
    let player_score = card_stats.player_score();
    let oppo_score = card_stats.oppo_score();
    let remaining_scores = card_stats.remaining_score();
    if player_score - oppo_score > remaining_scores {
        gameover_events.send(GameOver(EndReason::Victory));
        return;
    } else if oppo_score - player_score > remaining_scores {
        gameover_events.send(GameOver(EndReason::Loss));
        return;
    }
    turn_count.0 += 1;
    initative.swap();
    // TODO: use size_hint once bevy#4244 is merged (https://github.com/bevyengine/bevy/pull/4244)
    match initative.0 {
        _ if hands.iter().count() == 0 => turn.set(TurnState::Draw).unwrap(),
        Participant::Oppo => turn.set(TurnState::Oppo).unwrap(),
        Participant::Player => turn.set(TurnState::Player).unwrap(),
    };
}

/// Wait until all cards are drawn by the two participants and then enter the
/// card selection state.
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

fn wait_active(
    mut turn: ResMut<State<TurnState>>,
    mut timeout: Local<Option<f64>>,
    time: Res<Time>,
) {
    const TURN_INTERLUDE: f64 = 0.5;
    match *timeout {
        Some(some_timeout) if some_timeout < time.seconds_since_startup() => {
            turn.set(TurnState::New).unwrap();
            *timeout = None;
        }
        None => {
            *timeout = Some(time.seconds_since_startup() + TURN_INTERLUDE);
        }
        _ => {}
    };
}

/// Remove all entities related to the game and resets resource values.
fn cleanup(
    mut cmds: Commands,
    all_cards: Query<Entity, With<Card>>,
    mut turn_count: ResMut<TurnCount>,
    mut initative: ResMut<Initiative>,
    mut score_bonuses: ResMut<ScoreBonuses>,
    mut turn_effects: ResMut<TurnEffects>,
    mut seed_count: ResMut<SeedCount>,
) {
    turn_count.0 = 0;
    initative.0 = Participant::Player;
    *score_bonuses = ScoreBonuses::default();
    *turn_effects = TurnEffects::default();
    *seed_count = SeedCount::default();
    for entity in all_cards.iter() {
        cmds.entity(entity).despawn_recursive();
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use crate::system_helper::EasySystemSetCtor;
        app.add_event::<PlayCard>()
            .init_resource::<TurnCount>()
            .init_resource::<ScoreBonuses>()
            .init_resource::<TurnEffects>()
            .init_resource::<SeedCount>()
            .insert_resource(Initiative(Participant::Player))
            .add_system_set(self.0.on_update(handle_played))
            .add_system_set(self.0.on_exit(cleanup))
            .add_system_set(TurnState::New.on_enter(handle_new_turn))
            .add_system_set(TurnState::Draw.on_update(complete_draw))
            .add_system_set(TurnState::CardPlayed.on_update(wait_active))
            .add_system_set(TurnState::CardPlayed.on_exit(handle_turn_end));
    }
}
