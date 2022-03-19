use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use fastrand::usize as randusize;

use crate::{
    card::{Card, SpawnCard, WordOfPower},
    card_effect::{ActivateCard, PlayedCard},
    deck::OppoDeckRes,
    // pile::{Pile, PileCard, PileType},
    state::{GameState, TurnState},
    war::BattleOutcome,
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

fn draw_hand(mut card_spawner: SpawnCard, mut deck: ResMut<OppoDeckRes>) {
    for (i, card) in deck.draw(3).into_iter().enumerate() {
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
    // TODO: subtile go up/down hover effect
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

fn chose_card(
    mut cmds: Commands,
    mut card_events: EventWriter<ActivateCard>,
    cards: Query<(Entity, &Card), With<OppoCard>>,
    war_card: Query<&Card, With<PlayedCard>>,
    // pile_cards: Query<&PileCard>,
    // pile: Query<&Pile>,
) {
    use BattleOutcome::{Loss, Win};
    use WordOfPower::Zihbm;
    let in_hand: Vec<_> = cards.iter().collect();
    let selected = match war_card.get_single() {
        Ok(war_card) => {
            let wins_over_played = |(_, card): &&(_, &Card)| {
                if war_card.word == Some(Zihbm) {
                    card.value.beats(&war_card.value) != Win
                } else {
                    card.value.beats(&war_card.value) != Loss
                }
            };
            let best_lowest_value = |(_, card): &&(_, &Card)| {
                if war_card.value == card.value {
                    u32::MAX
                } else {
                    card.value as u32
                }
            };
            let fallback_when_losing =
                || in_hand.iter().min_by_key(|(_, c)| c.value as u32).unwrap();
            in_hand
                .iter()
                .filter(wins_over_played)
                .min_by_key(best_lowest_value)
                .unwrap_or_else(fallback_when_losing)
                .0
        }
        Err(_) => {
            // Actual random card otherwise it's too easy
            let selected_index = randusize(..in_hand.len());
            in_hand[selected_index].0
        }
    };
    cmds.entity(selected).remove::<OppoCard>();
    card_events.send(ActivateCard::new(selected, Participant::Oppo));
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<OppoCard>();
        app.add_system_set(SystemSet::on_enter(TurnState::Draw).with_system(draw_hand))
            .add_system_set(SystemSet::on_enter(TurnState::Oppo).with_system(chose_card))
            .add_system_set(SystemSet::on_update(self.0).with_system(update_oppo_hand));
    }
}
