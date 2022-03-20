use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use fastrand::usize as randusize;

use crate::{
    card::{Card, SpawnCard, WordOfPower},
    deck::OppoDeck,
    game_flow::{PlayCard, PlayedCard},
    state::{GameState, TurnState},
    war::{BattleOutcome, Value},
    Participant,
};

/// Position of the hand of the opposition
#[derive(Component)]
pub struct OppoHand;

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
struct OppoCard {
    index: usize,
}
impl OppoCard {
    fn new(index: usize) -> Self {
        Self { index }
    }
}

fn draw_hand(mut card_spawner: SpawnCard, mut deck: Query<&mut OppoDeck>) {
    for (i, card) in deck.single_mut().draw(3).into_iter().enumerate() {
        card_spawner
            .spawn_card(card, Participant::Oppo)
            .insert(OppoCard::new(i));
    }
}

fn update_oppo_hand(
    oppo_hand: Query<&GlobalTransform, With<OppoHand>>,
    mut cards: Query<(&mut Transform, &OppoCard)>,
    time: Res<Time>,
) {
    let card_speed = 10.0 * time.delta_seconds();
    let hand_transform = oppo_hand.single();
    let hand_pos = hand_transform.translation;
    for (mut transform, OppoCard { index }) in cards.iter_mut() {
        let i_f32 = *index as f32;
        let target = hand_pos + Vec3::new(i_f32 * 1.2 - 1.0, 0.0, 0.0);
        let origin = transform.translation;
        transform.translation += (target - origin) * card_speed;

        let target = hand_transform.rotation;
        let origin = transform.rotation;
        transform.rotation = origin.lerp(target, card_speed);
    }
}

fn play_card(
    mut cmds: Commands,
    mut card_events: EventWriter<PlayCard>,
    mut card_transform: Query<&mut Transform, With<OppoCard>>,
    cards: Query<(Entity, &Card), With<OppoCard>>,
    war_card: Query<&Card, With<PlayedCard>>,
) {
    let (entities, cards): (Vec<_>, Vec<_>) = cards.iter().map(|(e, c)| (e, c.clone())).unzip();
    assert!(!cards.is_empty(), "Oppo must have a least a card on play");
    let selected_index = chose_card(war_card.get_single().ok(), &cards);
    let selected = entities[selected_index];

    // Offset up the card so that it doesn't go through the already-played one
    let mut trans = card_transform.get_mut(selected).unwrap();
    trans.translation.y += 1.0;
    cmds.entity(selected).remove::<OppoCard>();
    card_events.send(PlayCard::new(selected, Participant::Oppo));
}

fn index_of<T: PartialEq>(t: &T, slice: &[T]) -> usize {
    slice
        .iter()
        .enumerate()
        .find_map(|(i, elem)| (elem == t).then(|| i))
        .unwrap()
}

/// Chose from cards in hand which one to play.
fn chose_card(played: Option<&Card>, in_hand: &[Card]) -> usize {
    use BattleOutcome::{Tie, Win};
    use Value::Zero;
    use WordOfPower::Geh;

    let played = if let Some(played) = played {
        played
    } else {
        // Actual random card otherwise it's too easy
        return randusize(..in_hand.len());
    };
    let wins = |this: &&Card| this.beats(played) == Win;
    let zero12 = |card: &&Card| {
        let bonus = played.word == Some(Geh) || card.word == Some(Geh);
        (card.value == Zero && bonus) as i32 * 12
    };
    let card_value = |card: &&Card| card.value as i32 + zero12(card);
    let lowest_value = || in_hand.iter().min_by_key(card_value);
    let a_tie = || in_hand.iter().find(|this| this.beats(played) == Tie);
    let winning = in_hand.iter().filter(wins).min();

    let chosen = winning.or_else(a_tie).or_else(lowest_value).unwrap();
    index_of(chosen, in_hand)
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use crate::system_helper::EasySystemSetCtor;
        #[cfg(feature = "debug")]
        app.register_inspectable::<OppoCard>();
        app.add_system_set(TurnState::Draw.on_enter(draw_hand))
            .add_system_set(TurnState::Oppo.on_enter(play_card))
            .add_system_set(self.0.on_update(update_oppo_hand));
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! cards {
        (hand: $( $values:tt $words:tt),+) => (
            (None, [ $(cards!(@card $values $words),)+ ])
        );
        (war $value:tt $word:tt ; hand: $( $values:tt $words:tt),+) => (
            (Some(cards!(@card $value $word)), [ $(cards!(@card $values $words),)+ ])
        );
        (@card $value:tt $word:tt) => (
             Card::new(cards!(@word $word), cards!(@val $value))
        );
        (@val 0) => (Value::Zero);
        (@val 1) => (Value::One);
        (@val 2) => (Value::Two);
        (@val 3) => (Value::Three);
        (@val 4) => (Value::Four);
        (@val 5) => (Value::Five);
        (@val 6) => (Value::Six);
        (@val 7) => (Value::Seven);
        (@val 8) => (Value::Eight);
        (@val 9) => (Value::Nine);
        (@word _) => (None);
        (@word s) => (Some(WordOfPower::Egeq)); // Seed
        (@word d) => (Some(WordOfPower::Qube)); // Double
        (@word w) => (Some(WordOfPower::Zihbm)); // Swap
        (@word z) => (Some(WordOfPower::Geh)); // 0 -> 12
        ($value:tt $word:tt) => (
            cards!(@card $value $word)
        );
    }
    #[test]
    fn chose_card_test() {
        macro_rules! test_hand {
            ([$($state:tt)*] is: $is_value:tt $is_word:tt $(, $msg:expr)?) => ({
                let (pile, hand) = cards!($($state)*);
                let actual = chose_card(pile.as_ref(), &hand);
                assert_eq!(hand[actual], cards!(@card $is_value $is_word) $(, $msg)?);
            })
        }
        test_hand!([war 1 _; hand: 2 _, 3 _, 5 _] is: 2 _, "lowest winning");
        // Note: it could be wishable that oppo prioritizes playing hight value swap
        test_hand!([war 1 w; hand: 2 w, 3 _, 5 _] is: 2 w, "lowest winning with potential swap");
        test_hand!([war 1 w; hand: 2 _, 3 _, 5 _] is: 2 _, "lowest loosing with swap");
        test_hand!([war 5 w; hand: 1 w, 3 _, 6 _] is: 3 _, "do not pick lowest swap if already swap");
        test_hand!([war 5 w; hand: 6 _, 3 _] is: 3 _, "select lower when oppo is swap");
        test_hand!([war 5 w; hand: 5 _, 6 _] is: 5 _, "prefer tie to loosing when swap");
        test_hand!([war 5 w; hand: 5 _, 6 w] is: 6 w, "select swap if wins");
        test_hand!([war 9 _; hand: 0 _, 9 _] is: 0 _, "chose 0 when oppo has 9");
        test_hand!([war 9 z; hand: 0 w, 3 _] is: 3 _, "prefer 3 to 0 when +12 and losing");
        test_hand!([war 9 z; hand: 0 _, 3 _] is: 0 _, "chose 0 even if +12 if wins");
        test_hand!([war 9 w; hand: 0 _, 1 _] is: 1 _, "do not lose when swap 0/9w");
        test_hand!([war 9 _; hand: 0 w, 1 _] is: 0 w, "chose lowest even in losing 0/9");
        test_hand!([war 5 _; hand: 5 _, 3 _] is: 5 _, "prefer tie to loss");
    }
}
