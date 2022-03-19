//! What happens after activating a card (including win/loss conditions and
//! score tracking, this module should be renamed into "game_state" or
//! something)

use bevy::ecs::system::SystemParam;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_debug_text_overlay::screen_print;

use crate::{
    audio::AudioRequest,
    card::{Card, WordOfPower},
    cheat::SleeveCard,
    deck::{OppoDeckRes, PlayerDeckRes},
    game_ui::EffectEvent,
    pile::{Pile, PileCard, PileType},
    state::{GameState, TurnState},
    war::{BattleOutcome, Value},
    CardOrigin, EndReason, GameOver, GameStarts, Participant,
};

/// Card in the War pile played by the player
#[derive(Component)]
pub struct PlayedCard;

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

struct TurnEffects {
    swap: bool,
    multiplier: i32,
    zero_bonus: bool,
}
impl Default for TurnEffects {
    fn default() -> Self {
        Self { swap: false, multiplier: 1, zero_bonus: false }
    }
}
impl TurnEffects {
    fn add(&mut self, word: WordOfPower) {
        use WordOfPower::*;
        match word {
            Qube => self.multiplier *= 2,
            Geh => self.zero_bonus = true,
            Zihbm => self.swap = !self.swap,
            _ => {}
        }
    }
}

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

fn handle_activated(
    mut events: EventReader<ActivateCard>,
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
    for ActivateCard { card, who } in events.iter() {
        let mut pile = pile
            .iter_mut()
            .find(|p| p.which == War)
            .expect("War pile exists");
        cmds.entity(*card)
            .insert_bundle((pile.additional_card(), PlayedCard));
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
    mut turn_effects: ResMut<TurnEffects>,
    mut score_bonuses: ResMut<ScoreBonuses>,
) {
    use Participant::{Oppo, Player};
    use PileType::War;
    let war_pile: Vec<_> = cards.iter().filter(|c| c.0.which == War).collect();
    macro_rules! add_card_to_pile {
        ($entry:expr, $who:expr) => {
            let (_, _, card, entity) = $entry;
            let mut pile = piles.iter_mut().find(|p| p.which == $who.into()).unwrap();
            cmds.entity(*entity)
                .insert(pile.additional_card())
                .remove::<PlayedCard>();
            let multi = turn_effects.multiplier - 1;
            let zero_bonus = if card.value == Value::Zero && turn_effects.zero_bonus {
                12
            } else {
                0
            };
            score_bonuses.add_to_owner($who, (card.value as i32) * multi);
            score_bonuses.add_to_owner($who, zero_bonus * (multi + 1));
        };
    }
    match &war_pile[..] {
        [card1, card2] => {
            let (player_card, oppo_card) = if card1.1 .0 == Participant::Player {
                (card1, card2)
            } else {
                (card2, card1)
            };
            let mut turn_outcome = player_card.2.value.beats(&oppo_card.2.value);
            if turn_effects.swap {
                turn_outcome = turn_outcome.invert();
            };
            match turn_outcome {
                BattleOutcome::Tie => {
                    add_card_to_pile!(player_card, Player);
                    add_card_to_pile!(oppo_card, Oppo);
                }
                BattleOutcome::Loss => {
                    add_card_to_pile!(player_card, Oppo);
                    add_card_to_pile!(oppo_card, Oppo);
                }
                BattleOutcome::Win => {
                    add_card_to_pile!(player_card, Player);
                    add_card_to_pile!(oppo_card, Player);
                }
            }
            *turn_effects = TurnEffects::default();
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

#[derive(SystemParam)]
pub struct CardStats<'w, 's> {
    piles: Query<'w, 's, (&'static PileCard, &'static Card)>,
    hands: Query<'w, 's, &'static Card, HandFilter>,
    sleeve: Query<'w, 's, &'static Card, With<SleeveCard>>,
    player_deck: Res<'w, PlayerDeckRes>,
    oppo_deck: Res<'w, OppoDeckRes>,
    score_bonuses: Res<'w, ScoreBonuses>,
}
impl<'w, 's> CardStats<'w, 's> {
    pub fn remaining_score(&self) -> i32 {
        let hands_score: i32 = self.hands.iter().map(Card::max_value).sum();
        let sleeve_score: i32 = self.sleeve.iter().map(Card::max_value).sum();
        self.player_deck.score() + self.oppo_deck.score() + sleeve_score + hands_score
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

fn handle_new_turn(
    mut initative: ResMut<Initiative>,
    mut turn: ResMut<State<TurnState>>,
    mut turn_count: ResMut<TurnCount>,
    mut gameover_events: EventWriter<GameOver>,
    hands: Query<(), HandFilter>,
    card_stats: CardStats,
) {
    screen_print!(sec: 1.0, "handle new turn");
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
        use TurnState::{OppoActivated, PlayerActivated};
        app.add_event::<ActivateCard>()
            .init_resource::<TurnCount>()
            .init_resource::<ScoreBonuses>()
            .init_resource::<TurnEffects>()
            .init_resource::<SeedCount>()
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
