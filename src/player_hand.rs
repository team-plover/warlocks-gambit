use std::f32::consts::FRAC_PI_4;

use bevy::{
    ecs::{query::QueryItem, system::SystemParam},
    prelude::{Plugin as BevyPlugin, *},
};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource};

use crate::{
    animate::{Animated, DisableAnimation},
    audio::AudioRequest::{self, PlayShuffleLong, PlayShuffleShort},
    camera::PlayerCam,
    card::{CardStatus, SpawnCard},
    cheat::{CheatEvent, SleeveCard},
    deck::PlayerDeck,
    game_flow::PlayCard,
    state::{GameState, TurnState},
    Participant,
};

/// Position of the hand of the player
#[derive(Component)]
pub struct PlayerHand;

#[derive(Component)]
pub struct GrabbedCard;

enum HandRaycast {}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
struct HandCard {
    index: usize,
    dragging: bool,
}
impl HandCard {
    fn new(index: usize) -> Self {
        Self { index, dragging: false }
    }
}

// TODO: do not re-highlight the previously highlighted card until cursor has
// been away from it for at least 0.3 seconds
#[derive(SystemParam)]
struct DrawParams<'w, 's> {
    card_spawner: SpawnCard<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    deck: Query<'w, 's, &'static mut PlayerDeck>,
    audio: EventWriter<'w, 's, AudioRequest>,
}
impl<'w, 's> DrawParams<'w, 's> {
    fn deck(&mut self) -> Mut<PlayerDeck> {
        self.deck.single_mut()
    }
    fn draw(&mut self, count: usize) {
        self.audio.send(PlayShuffleLong);
        for (i, card) in self.deck().draw(count).into_iter().enumerate() {
            self.card_spawner
                .spawn_card(card, Participant::Player)
                .insert_bundle((
                    HandCard::new(i),
                    RayCastMesh::<HandRaycast>::default(),
                    self.meshes
                        .add(shape::Quad::new(Vec2::new(2.3, 3.3)).into()),
                    Visibility::default(),
                    ComputedVisibility::default(),
                ));
        }
    }
}

fn draw_hand(
    mut card_drawer: DrawParams,
    mut cmds: Commands,
    sleeve_cards: Query<Entity, With<SleeveCard>>,
    cam: Query<Entity, With<PlayerCam>>,
) {
    // NOTE: could have been done in scene.rs, but I prefer to keep the
    // HandRaycast type private
    let raycast_source = RayCastSource::<HandRaycast>::new();
    cmds.entity(cam.single()).insert(raycast_source);

    let unsleeved: Vec<_> = sleeve_cards.iter().collect();
    card_drawer.draw(3 - unsleeved.len());
    for entity in unsleeved.into_iter() {
        cmds.entity(entity)
            .remove::<SleeveCard>()
            .remove::<Animated>()
            .insert(HandCard::new(0));
    }
}

fn update_raycast(
    mut query: Query<&mut RayCastSource<HandRaycast>>,
    mut cursor: EventReader<CursorMoved>,
) {
    if let Some(cursor) = cursor.iter().last() {
        for mut pick_source in query.iter_mut() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor.position);
        }
    }
}

/// Set the [`CardStatus`] of cards, un-hovering cards not under cursor and
/// hovering ones that just came under it.
fn hover_card(
    hand_raycaster: Query<&RayCastSource<HandRaycast>>,
    mouse: Res<Input<MouseButton>>,
    mut hand_cards: Query<(Entity, &mut CardStatus)>,
    mut audio: EventWriter<AudioRequest>,
) {
    if mouse.pressed(MouseButton::Left) {
        return;
    }
    let query = hand_raycaster.get_single().map(|ray| ray.intersect_top());
    if let Ok(Some((card_under_cursor, _))) = query {
        for (entity, mut hover) in hand_cards.iter_mut() {
            let is_under_cursor = entity == card_under_cursor;
            let is_hovering = *hover == CardStatus::Hovered;
            if is_under_cursor && !is_hovering {
                *hover = CardStatus::Hovered;
                audio.send(PlayShuffleShort);
            }
            if !is_under_cursor && is_hovering {
                *hover = CardStatus::Normal;
            }
        }
    }
}

enum HandEvent {
    RaiseSleeve,
    LowerSleeve,
}

fn play_card(
    mouse: Res<Input<MouseButton>>,
    hand_raycaster: Query<&RayCastSource<HandRaycast>>,
    mut card_events: EventWriter<PlayCard>,
    mut cmds: Commands,
    mut hand_cards: Query<(Entity, &mut CardStatus, &mut HandCard, &mut Transform)>,
    mut hand_events: EventWriter<HandEvent>,
    mut cheat_events: EventWriter<CheatEvent>,
    mut card_drawer: DrawParams,
    sleeve_cards: Query<(), With<SleeveCard>>,
) {
    use CardStatus::Hovered;
    let query = hand_raycaster.get_single().map(|ray| ray.intersect_top());
    for (entity, mut hover_state, mut card, mut trans) in hand_cards.iter_mut() {
        match (*hover_state, card.dragging) {
            (Hovered, false) if mouse.just_pressed(MouseButton::Left) => {
                let under_cursor = if let Ok(Some((e, _))) = query { e } else { break };
                if entity == under_cursor {
                    cmds.entity(entity).insert(GrabbedCard);
                    card.dragging = true;
                    break;
                }
            }
            (_, false) => {}
            (_, true) if mouse.just_released(MouseButton::Left) => {
                let word_cursor = if let Ok(Some((_, i))) = query { i } else { break };
                let cursor_pos = word_cursor.position();
                let cards_remaining = card_drawer.deck().remaining() != 0;
                let can_sleeve = sleeve_cards.iter().count() < 3 && cards_remaining;
                cmds.entity(entity).remove::<GrabbedCard>();
                *hover_state = CardStatus::Normal;
                if cursor_pos.x < -1.0 && cursor_pos.y < 4.7 && can_sleeve {
                    cmds.entity(entity).remove::<HandCard>();
                    cheat_events.send(CheatEvent::HideInSleeve(entity));
                    hand_events.send(HandEvent::LowerSleeve);
                    card_drawer.draw(1);
                } else if cursor_pos.x > 0.2 || cursor_pos.y > 6.0 {
                    cmds.entity(entity).remove::<HandCard>();
                    cmds.entity(entity).remove::<RayCastMesh<HandRaycast>>();
                    card_events.send(PlayCard::new(entity, Participant::Player));
                } else {
                    card.dragging = false;
                }
                break;
            }
            (_, true) => {
                let word_cursor = if let Ok(Some((_, i))) = query { i } else { break };
                let cursor_pos = word_cursor.position();
                let cards_remaining = card_drawer.deck().remaining() != 0;
                // FIXME: use size_hint().0 when bevy#4244 pr is merged
                let can_sleeve = sleeve_cards.iter().count() < 3 && cards_remaining;
                trans.translation = cursor_pos;
                if cursor_pos.x < -1.0 && cursor_pos.y < 4.7 && can_sleeve {
                    hand_events.send(HandEvent::RaiseSleeve);
                } else {
                    hand_events.send(HandEvent::LowerSleeve);
                }
                break;
            }
        }
    }
}

// TODO: tilt hand backward when enemy is playing so that it's more explicitly
// the player's turn
// TODO: animate sleeve movement
fn update_sleeve(
    mut cmds: Commands,
    mut hand: Query<(Entity, &mut Transform), With<PlayerHand>>,
    mut cards: Query<(&mut Transform, &HandCard), Without<PlayerHand>>,
    mut events: EventReader<HandEvent>,
    mut raised: Local<bool>,
    time: Res<Time>,
) {
    let (hand, mut trans) = hand.single_mut();
    let sleeve_move = Vec3::Y * 1.5;
    if *raised {
        if let Some((mut trans, _)) = cards.iter_mut().find(|c| c.1.dragging) {
            let delta = time.delta_seconds();
            trans.rotation = trans.rotation.lerp(Quat::IDENTITY, delta * 10.0);
        }
    }
    for event in events.iter() {
        match event {
            HandEvent::RaiseSleeve if !*raised => {
                cmds.entity(hand).insert(DisableAnimation);
                *raised = true;
                trans.translation += sleeve_move;
            }
            HandEvent::LowerSleeve if *raised => {
                *raised = false;
                cmds.entity(hand).remove::<DisableAnimation>();
                trans.translation -= sleeve_move;
            }
            _ => {}
        }
    }
}

type HoverQuery = (
    &'static mut Transform,
    &'static CardStatus,
    &'static HandCard,
);
/// Animate card movements into the player hand
///
/// (skip card if we are currently dragging it)
fn update_hand(
    hand: Query<&GlobalTransform, With<PlayerHand>>,
    mut cards: Query<HoverQuery>,
    time: Res<Time>,
) {
    let card_speed = 10.0 * time.delta_seconds();
    let hand_transform = hand.single();
    let hand_pos = hand_transform.translation;
    let not_dragging = |c: &QueryItem<HoverQuery>| !c.2.dragging;
    for (mut transform, hover, HandCard { index, .. }) in cards.iter_mut().filter(not_dragging) {
        let is_hovering = *hover == CardStatus::Hovered;
        let i_f32 = 0.7 * *index as f32;
        let hover_mul = if is_hovering { 2.0 } else { 1.0 };
        let y_offset = i_f32.cos() * hover_mul;
        let x_offset = i_f32.sin() * hover_mul;
        let z_offset = if is_hovering { 0.01 } else { i_f32 * -0.01 };
        let target = hand_pos + Vec3::new(x_offset - 0.3, y_offset, z_offset + 0.05);
        let origin = transform.translation;
        transform.translation += (target - origin) * card_speed;

        let rot_offset = Quat::from_rotation_z(FRAC_PI_4 * -i_f32);
        let target = hand_transform.rotation * rot_offset;
        let origin = transform.rotation;
        transform.rotation = origin.lerp(target, card_speed);
    }
}

/// Reorder cards in hand.
///
/// So that they are held like a human would, even after using one.
fn update_hand_indexes(mut cards: Query<&mut HandCard>) {
    let mut cards: Vec<_> = cards.iter_mut().collect();
    cards.sort_by_key(|c| c.index);
    for (index, card) in cards.iter_mut().enumerate() {
        card.index = index;
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<HandCard>();
        app.add_plugin(DefaultRaycastingPlugin::<HandRaycast>::default())
            .add_event::<HandEvent>()
            .add_system_set(SystemSet::on_enter(TurnState::Draw).with_system(draw_hand))
            .add_system_set(
                SystemSet::on_update(TurnState::Player)
                    .with_system(hover_card.label("select"))
                    .with_system(play_card.label("play").after("select"))
                    .with_system(update_raycast),
            )
            .add_system_set(
                SystemSet::on_update(self.0)
                    .with_system(update_sleeve.after("animation"))
                    .with_system(update_hand.after("play"))
                    .with_system(update_hand_indexes),
            );
    }
}
