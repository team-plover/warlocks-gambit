use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource};

use crate::{
    camera::PlayerCam,
    card::{Card, CardStatus, SpawnCard},
    card_effect::ActivateCard,
    card_spawner::PlayerHand,
    deck::PlayerDeck,
    state::{GameState, TurnState},
    Participant,
};

enum HandRaycast {}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(PartialEq)]
enum HoverStatus {
    Hovered,
    Dragging,
    None,
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
struct HandCard {
    index: usize,
    hover: HoverStatus,
}
impl HandCard {
    fn new(index: usize) -> Self {
        let hover = HoverStatus::None;
        Self { index, hover }
    }
}

fn draw_hand(
    mut cmds: Commands,
    mut card_spawner: SpawnCard,
    mut meshes: ResMut<Assets<Mesh>>,
    mut deck: ResMut<PlayerDeck>,
    cam: Query<Entity, With<PlayerCam>>,
) {
    cmds.entity(cam.single())
        .insert(RayCastSource::<HandRaycast>::new());
    for (i, card) in deck.draw(3).into_iter().enumerate() {
        card_spawner
            .spawn_card(card, Participant::Player)
            .insert_bundle((
                HandCard::new(i),
                RayCastMesh::<HandRaycast>::default(),
                meshes.add(shape::Quad::new(Vec2::new(2.3, 3.3)).into()),
                Visibility::default(),
                ComputedVisibility::default(),
            ));
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

/// Set the [`HoveredCard`] as the last one on which the cursor hovered.
fn select_card(
    mut cursor: EventReader<CursorMoved>,
    hand_raycaster: Query<&RayCastSource<HandRaycast>>,
    mut hand_cards: Query<(Entity, &mut Card, &mut HandCard)>,
) {
    use HoverStatus::{Dragging, Hovered};
    let query = hand_raycaster.get_single().map(|ray| ray.intersect_top());
    let has_cursor_moved = cursor.iter().next().is_some();
    if let Ok(Some((hovered_card, _))) = query {
        if !has_cursor_moved {
            return;
        }
        for (entity, mut card, mut hand_card) in hand_cards.iter_mut() {
            if entity == hovered_card && hand_card.hover != Dragging {
                card.set_status(CardStatus::Hovered);
                hand_card.hover = Hovered;
            } else if hand_card.hover != Dragging {
                card.set_status(CardStatus::Normal);
                hand_card.hover = HoverStatus::None;
            }
        }
    }
}

fn play_card(
    mouse: Res<Input<MouseButton>>,
    hand_raycaster: Query<&RayCastSource<HandRaycast>>,
    mut card_events: EventWriter<ActivateCard>,
    mut cmds: Commands,
    mut hand_cards: Query<(Entity, &mut Card, &mut HandCard, &mut Transform)>,
) {
    use HoverStatus::{Dragging, Hovered};
    let query = hand_raycaster.get_single().map(|ray| ray.intersect_top());
    for (entity, mut card, mut hand_card, mut trans) in hand_cards.iter_mut() {
        match hand_card.hover {
            Hovered if mouse.just_pressed(MouseButton::Left) => {
                let hovered_card = if let Ok(Some((e, _))) = query { e } else { break };
                if hovered_card == entity {
                    hand_card.hover = Dragging;
                    break;
                }
            }
            // TODO: Test where the card was released (if in sleeve, then sleeve cheat
            // else if far from hand then activate else return to hand)
            Dragging if mouse.just_released(MouseButton::Left) => {
                // TODO: migrate setting that status to card_effect::handle_activated
                card.set_status(CardStatus::Activated);
                cmds.entity(entity).remove::<HandCard>();
                card_events.send(ActivateCard::new(entity, Participant::Player));
                break;
            }
            Dragging => {
                let intersection = if let Ok(Some((_, i))) = query { i } else { break };
                let cursor_pos = intersection.position();
                trans.translation = cursor_pos;
                break;
            }
            _ => {}
        }
    }
}

/// Move progressively cards from [`HandCard`] in front of player camera.
fn update_hand(
    hand: Query<&GlobalTransform, With<PlayerHand>>,
    mut cards: Query<(&mut Transform, &HandCard)>,
) {
    use HoverStatus::Hovered;
    const CARD_SPEED: f32 = 0.15;
    let hand_transform = hand.single();
    let hand_pos = hand_transform.translation;
    for (mut transform, HandCard { index, hover }) in cards.iter_mut() {
        let i_f32 = *index as f32;
        let vertical_offset = if *hover == Hovered { 2.0 } else { 0.9 };
        let horizontal_offset = i_f32 - 1.0;
        let z_offset = if *hover == Hovered { 0.01 } else { i_f32 * -0.01 };
        let target = hand_pos + Vec3::new(horizontal_offset, vertical_offset, z_offset + 0.05);
        let origin = transform.translation;
        transform.translation += (target - origin) * CARD_SPEED;

        let target = hand_transform.rotation;
        let origin = transform.rotation;
        transform.rotation = origin.lerp(target, CARD_SPEED);
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<HandCard>();
        app.add_plugin(DefaultRaycastingPlugin::<HandRaycast>::default())
            .add_system_set(SystemSet::on_enter(TurnState::Draw).with_system(draw_hand))
            .add_system_set(
                SystemSet::on_update(TurnState::Player)
                    .with_system(select_card.label("select"))
                    .with_system(play_card.label("play").after("select"))
                    .with_system(update_raycast),
            )
            .add_system_set(SystemSet::on_update(self.0).with_system(update_hand.after("play")));
    }
}