use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

use crate::{
    card::{Card, CardStatus, SpawnCard, WordOfPower},
    card_effect::ActivateCard,
    card_spawner::OppoHand,
    // pile::{Pile, PileCard, PileType},
    state::{GameState, TurnState},
    war::Value,
    Participant,
};

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

fn spawn_hand(mut card_spawner: SpawnCard) {
    use Value::{One, Three, Two};
    for (i, value) in [One, Two, Three].iter().enumerate() {
        card_spawner
            .spawn_card(Card::new(WordOfPower::Egeq, *value), Participant::Oppo)
            .insert(OppoCard::new(i));
    }
}

fn update_oppo_hand(
    oppo_hand: Query<&GlobalTransform, With<OppoHand>>,
    mut cards: Query<(&mut Transform, &OppoCard)>,
) {
    // TODO: subtile go up/down hover effect
    const CARD_SPEED: f32 = 0.15;
    let hand_transform = oppo_hand.single();
    let hand_pos = hand_transform.translation;
    for (mut transform, OppoCard { index }) in cards.iter_mut() {
        let i_f32 = *index as f32;
        let target = hand_pos + Vec3::new(i_f32 - 1.0, 0.0, i_f32 * -0.01);
        let origin = transform.translation;
        transform.translation += (target - origin) * CARD_SPEED;

        let target = hand_transform.rotation;
        let origin = transform.rotation;
        transform.rotation = origin.lerp(target, CARD_SPEED);
    }
}

fn chose_card(
    mut cmds: Commands,
    mut card_events: EventWriter<ActivateCard>,
    mut cards: Query<(Entity, &mut Card), With<OppoCard>>,
    // pile_cards: Query<&PileCard>,
    // pile: Query<&Pile>,
) {
    // use PileType::War;
    // let pile = pile.iter().find(|p| p.which == War).expect("War pile exists");
    // TODO: use an actual heuristic instead of picking first at all time
    if let Some((selected, mut card)) = cards.iter_mut().next() {
        // TODO: migrate setting that status to card_effect::handle_activated
        card.set_status(CardStatus::Activated);
        cmds.entity(selected).remove::<OppoCard>();
        card_events.send(ActivateCard::new(selected, Participant::Oppo));
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<OppoCard>();
        app.add_system_set(SystemSet::on_enter(self.0).with_system(spawn_hand))
            .add_system_set(SystemSet::on_enter(TurnState::Oppo).with_system(chose_card))
            .add_system_set(SystemSet::on_update(self.0).with_system(update_oppo_hand));
    }
}
