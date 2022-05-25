//! Player interaction with cards in hand.
//!
//! Handle mouse pointer interactions, grabbing cards and slipping them into
//! the sleeves.
//!
//! It uses the `bevy_mod_raycast` crate to handle pointer stuff. It specifies
//! a mesh for each card in player [`HandRaycast`], a mesh for the area in
//! which dropping a dragged card will "cancel" the card selection
//! [`HandDisengageArea`] and an area where you can drop grabbed cards in the
//! sleeve [`SleeveArea`].
//!
//! * [`DrawParams`] defines how to spawn a card with all the collision meshes
//!   setup.
//! * [`CardCollisionAssets`] defines the meshes used for collision detection.
use std::f32::consts::FRAC_PI_4;

use bevy::{
    ecs::{query::QueryItem, system::SystemParam},
    math::EulerRot::XYZ,
    pbr::wireframe::Wireframe,
    prelude::{Plugin as BevyPlugin, *},
};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource};

use crate::{
    animate::DisableAnimation,
    audio::AudioRequest::{self, PlayShuffleLong, PlayShuffleShort},
    card::{CardStatus, SpawnCard},
    cheat::{CheatEvent, SleeveCard},
    deck::PlayerDeck,
    game_flow::PlayCard,
    game_ui::EffectEvent,
    state::{GameState, TurnState},
    war::Card,
    Participant,
};

/// Position of the hand of the player
#[derive(Component)]
pub struct PlayerHand;

/// Mark the card that the player is currently dragging. Used in [`crate::cheat`] for
/// the bird eye tracking player card.
#[derive(Component)]
pub struct GrabbedCard;

/// Mesh for selecting the card.
pub enum HandRaycast {}

/// Marks the mesh that represents where if we disengage the card (relese the
/// grab button), it will go back into the hand.
pub enum HandDisengageArea {}

/// Where if we disengage the card, the card will fall into the sleeve.
pub enum SleeveArea {}

#[rustfmt::skip]
const AREA_VERTICES: [[f32; 2]; 9] = [
    [0.0, 0.0],
    [0.0, 1.0], [ 0.7,  0.7],
    [1.0, 0.0], [ 0.7, -0.7],
    [0.0, -1.], [-0.7, -0.7],
    [-1., 0.0], [-0.7,  0.7],
];

#[rustfmt::skip]
const AREA_EDGES: [u16; 24] = [
    0, 1, 2,    2, 3, 0,    3, 4, 0,
    4, 5, 0,    5, 6, 0,    6, 7, 0,
    7, 8, 0,    8, 1, 0,
];

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
struct HandCard {
    index: usize,
    dragging: bool,
    underlay: Entity,
}
impl HandCard {
    fn new(index: usize, underlay: Entity) -> Self {
        Self { index, underlay, dragging: false }
    }
}

/// Meshes used for collision detection.
pub struct CardCollisionAssets {
    bounding_box: Handle<Mesh>,
    underlay: Handle<Mesh>,
    pub circle: Handle<Mesh>,
}
impl FromWorld for CardCollisionAssets {
    fn from_world(world: &mut World) -> Self {
        use bevy::render::{
            mesh::{
                Indices,
                VertexAttributeValues::{Float32x2, Float32x3},
            },
            render_resource::PrimitiveTopology,
        };
        let pos: Vec<[f32; 3]> = AREA_VERTICES.iter().map(|&[x, y]| [x, y, 0.0]).collect();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(pos));
        mesh.set_indices(Some(Indices::U16(AREA_EDGES.into())));
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Float32x2([[0., 0.]; 9].into()));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Float32x3([[0., 0., 1.]; 9].into()));
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        Self {
            bounding_box: meshes.add(shape::Quad::new(Vec2::new(2.3, 3.3)).into()),
            underlay: meshes.add(shape::Quad::new(Vec2::new(2.4, 6.3)).into()),
            circle: meshes.add(mesh),
        }
    }
}

// Underlay mesh is for preventing cards going up/down very fast when hovering over two
// of them at the same time. It acts as a screen that prevents raycasts from reaching
// under it. By enabling/disabling visibility, it's possible to enable/disable the underlay.
#[derive(Component)]
struct Underlay;

/// System parameter to spawn a card with all the collision meshes setup.
#[derive(SystemParam)]
struct DrawParams<'w, 's> {
    card_spawner: SpawnCard<'w, 's>,
    assets: Res<'w, CardCollisionAssets>,
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
            let cmds = &mut self.card_spawner.cmds;
            let underlay = cmds
                .spawn_bundle((
                    Transform::from_xyz(0.0, 0.0, -0.001),
                    GlobalTransform::default(),
                    RayCastMesh::<HandRaycast>::default(),
                    self.assets.underlay.clone(),
                    Visibility { is_visible: false },
                    Wireframe,
                    Underlay,
                    ComputedVisibility::default(),
                ))
                .id();
            self.card_spawner
                .spawn_card(card, Participant::Player)
                .add_child(underlay)
                .insert_bundle((
                    HandCard::new(i, underlay),
                    Wireframe,
                    RayCastMesh::<HandRaycast>::default(),
                    self.assets.bounding_box.clone(),
                    Visibility::default(),
                    ComputedVisibility::default(),
                ));
        }
    }
}

/// Draw cards to full hand (managing sleeved ones) and
#[allow(clippy::type_complexity)]
fn draw_hand(
    mut card_drawer: DrawParams,
    mut cmds: Commands,
    sleeve_cards: Query<Entity, With<SleeveCard>>,
    parents: Query<(Entity, &Parent), (With<Underlay>, With<RayCastMesh<HandRaycast>>)>,
) {
    let underlay_of = |e| parents.iter().find_map(|(c, p)| (p.0 == e).then(|| c));
    let unsleeved: Vec<_> = sleeve_cards.iter().collect();
    card_drawer.draw(3 - unsleeved.len());
    for entity in unsleeved.into_iter() {
        cmds.entity(entity)
            .remove::<SleeveCard>()
            .insert(HandCard::new(0, underlay_of(entity).unwrap()));
    }
}

/// Update the `bevy_mod_raycast` `RayCastSource` each frame so that it tracks
/// the cursor position.
fn update_raycast(
    mut hand: Query<&mut RayCastSource<HandRaycast>>,
    mut disengage: Query<&mut RayCastSource<HandDisengageArea>>,
    mut sleeve: Query<&mut RayCastSource<SleeveArea>>,
    mut cursor: EventReader<CursorMoved>,
) {
    if let Some(cursor) = cursor.iter().last() {
        for mut pick_source in hand.iter_mut() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor.position);
        }
        for mut pick_source in disengage.iter_mut() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor.position);
        }
        for mut pick_source in sleeve.iter_mut() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor.position);
        }
    }
}

/// Set the [`CardStatus`] of cards, un-hovering cards not under cursor and
/// hovering ones that just came under it.
fn hover_card(
    hand_raycaster: Query<&RayCastSource<HandRaycast>>,
    mouse: Res<Input<MouseButton>>,
    mut hand_cards: Query<(Entity, &Card, &mut CardStatus)>,
    mut audio: EventWriter<AudioRequest>,
    mut ui_events: EventWriter<EffectEvent>,
) {
    if mouse.pressed(MouseButton::Left) {
        return;
    }
    let query = hand_raycaster.get_single().map(|ray| ray.intersect_top());
    if let Ok(Some((card_under_cursor, _))) = query {
        // Does not have `CardStatus` component, meaning it's an underlay, so do nothing
        if hand_cards.get(card_under_cursor).is_err() {
            return;
        }
        let mut already_new_word_description = false;
        for (entity, card, mut hover) in hand_cards.iter_mut() {
            let is_under_cursor = entity == card_under_cursor;
            let is_hovering = *hover == CardStatus::Hovered;
            if is_under_cursor && !is_hovering {
                *hover = CardStatus::Hovered;
                if let Some(word) = card.word {
                    already_new_word_description = true;
                    ui_events.send(EffectEvent::Show(word));
                }
                audio.send(PlayShuffleShort);
            }
            if !is_under_cursor && is_hovering {
                if card.word.is_some() && !already_new_word_description {
                    ui_events.send(EffectEvent::Hide);
                }
                *hover = CardStatus::Normal;
            }
        }
    }
}

// TODO: remove this, move the sleeve logic from play_card to update_sleeve
enum HandEvent {
    RaiseSleeve,
    LowerSleeve,
}

/// Handle player interaction with cards in hand.
fn play_card(
    mouse: Res<Input<MouseButton>>,
    hand_raycaster: Query<&RayCastSource<HandRaycast>>,
    disengage_raycaster: Query<&RayCastSource<HandDisengageArea>>,
    sleeve_raycaster: Query<&RayCastSource<SleeveArea>>,
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
    let is_disengaging = || disengage_raycaster.single().intersect_top().is_some();
    let is_sleeving = || sleeve_raycaster.single().intersect_top().is_some();
    for (entity, mut hover_state, mut card, mut trans) in hand_cards.iter_mut() {
        match (*hover_state, card.dragging) {
            (Hovered, false) if mouse.just_pressed(MouseButton::Left) => {
                let under_cursor = if let Ok(Some((e, _))) = query { e } else { break };
                if entity == under_cursor {
                    cmds.entity(entity).insert(GrabbedCard);
                    card.dragging = true;
                    // Move toward camera so no z-fighting with other cards
                    // Not too much otherwise card offset on screen causes bug
                    // because it's not under the cursor anymore
                    trans.translation.z += 0.15;
                    break;
                }
            }
            (_, false) => {}
            (_, true) if mouse.just_released(MouseButton::Left) => {
                let cards_remaining = card_drawer.deck().remaining() != 0;
                let can_sleeve = sleeve_cards.iter().count() < 3 && cards_remaining;
                cmds.entity(entity).remove::<GrabbedCard>();
                *hover_state = CardStatus::Normal;
                if is_sleeving() && can_sleeve {
                    cmds.entity(entity).remove::<HandCard>();
                    cheat_events.send(CheatEvent::HideInSleeve(entity));
                    hand_events.send(HandEvent::LowerSleeve);
                    card_drawer.draw(1);
                } else if !is_disengaging() {
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
                if is_sleeving() && can_sleeve {
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
/// Move sleeve up/down based on whether the player is currently dragging over
/// the sleeve hot zone. Also move the card the player is dragging into it.
fn update_sleeve(
    mut cmds: Commands,
    mut hand: Query<(Entity, &mut Transform), With<PlayerHand>>,
    mut cards: Query<(&mut Transform, &HandCard), Without<PlayerHand>>,
    mut events: EventReader<HandEvent>,
    mut raised: Local<bool>,
    time: Res<Time>,
) {
    let (hand, mut trans) = hand.single_mut();
    if *raised {
        if let Some((mut trans, _)) = cards.iter_mut().find(|c| c.1.dragging) {
            let delta = time.delta_seconds();
            let (x, y, _) = trans.rotation.to_euler(XYZ);
            let target_rot = Quat::from_euler(XYZ, x, y, 0.1);
            trans.rotation = trans.rotation.lerp(target_rot, delta * 10.0);
        }
    }
    for event in events.iter() {
        match event {
            HandEvent::RaiseSleeve if !*raised => {
                cmds.entity(hand).insert(DisableAnimation);
                *raised = true;
                let offset = 1.5 * trans.up();
                trans.translation += offset;
            }
            HandEvent::LowerSleeve if *raised => {
                *raised = false;
                cmds.entity(hand).remove::<DisableAnimation>();
                let offset = 1.5 * trans.up();
                trans.translation -= offset;
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

/// Animate card movements into the player hand, skipping the dragged one.
fn update_hand(
    hand: Query<&GlobalTransform, With<PlayerHand>>,
    mut cards: Query<HoverQuery>,
    time: Res<Time>,
) {
    let card_speed = 10.0 * time.delta_seconds();
    let hand_transform = hand.single();
    let (hand_pos, hand_rot) = (hand_transform.translation, hand_transform.rotation);
    let not_dragging = |c: &QueryItem<HoverQuery>| !c.2.dragging;
    for (mut transform, hover, HandCard { index, .. }) in cards.iter_mut().filter(not_dragging) {
        let is_hovering = *hover == CardStatus::Hovered;
        let i_f32 = 0.7 * *index as f32;
        let hover_mul = if is_hovering { 2.0 } else { 1.0 };
        let y_offset = i_f32.cos() * hover_mul;
        let x_offset = i_f32.sin() * hover_mul;
        let z_offset = i_f32 * -0.01;
        let target = Vec3::new(x_offset - 0.3, y_offset, z_offset + 0.04);
        let target = hand_pos + hand_rot * target;
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

/// Add an underlay to hovered cards to prevent the once-per-frame on/off swap
/// of cards.
fn hovered_covers_previous_position(
    mut underlay_visibilities: Query<&mut Visibility, With<Underlay>>,
    statuses: Query<(&HandCard, &CardStatus), Changed<CardStatus>>,
) {
    for (HandCard { underlay, .. }, status) in statuses.iter() {
        let mut underlay_vis = underlay_visibilities.get_mut(*underlay).unwrap();
        underlay_vis.is_visible = matches!(*status, CardStatus::Hovered);
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<HandCard>();
        app.add_plugin(DefaultRaycastingPlugin::<HandRaycast>::default())
            .add_plugin(DefaultRaycastingPlugin::<SleeveArea>::default())
            .add_plugin(DefaultRaycastingPlugin::<HandDisengageArea>::default())
            .add_event::<HandEvent>()
            .init_resource::<CardCollisionAssets>()
            .add_system_set(SystemSet::on_enter(TurnState::Draw).with_system(draw_hand))
            .add_system_set(
                SystemSet::on_update(TurnState::Player)
                    .with_system(hover_card.label("select"))
                    .with_system(hovered_covers_previous_position)
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
