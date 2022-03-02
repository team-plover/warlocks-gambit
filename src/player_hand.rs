use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource};

use crate::{
    camera::PlayerCam,
    card::{Card, CardStatus, SpawnCard, Value, WordOfPower},
    state::GameState,
};

enum HandRaycast {}

#[derive(Component)]
struct Hand;

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

fn spawn_hand(
    mut cmds: Commands,
    mut card_spawner: SpawnCard,
    mut meshes: ResMut<Assets<Mesh>>,
    cam: Query<Entity, With<PlayerCam>>,
) {
    use Value::{Eight, Seven, Two, Zero};
    let cam = cam.single();
    cmds.entity(cam).insert(RayCastSource::<HandRaycast>::new());

    cmds.spawn_bundle((
        GlobalTransform::default(),
        Transform::from_xyz(0.0, -0.2, -5.0),
        Name::new("Player hand"),
        Hand,
        Parent(cam),
    ));
    for (i, value) in [Zero, Two, Seven, Eight].iter().enumerate() {
        card_spawner
            .spawn_card(Card::new(WordOfPower::Meb, *value))
            .insert_bundle((
                HandCard::new(i),
                RayCastMesh::<HandRaycast>::default(),
                meshes.add(shape::Quad::new(Vec2::new(2.3, 3.3)).into()),
                Visibility::default(),
                ComputedVisibility::default(),
            ));
    }
}

// Workaround the Hand (and children) GlobalTransform not being set correctly
// when spawned
fn update_hand_transform(
    query: Query<Entity, Added<Hand>>,
    mut cam: Query<&mut Transform, (Without<Hand>, With<PlayerCam>)>,
) {
    if query.get_single().is_ok() {
        cam.single_mut().set_changed();
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
    mut cmds: Commands,
    mut hand_cards: Query<(Entity, &mut Card, &mut HandCard, &mut Transform)>,
) {
    use HoverStatus::{Dragging, Hovered};
    let query = hand_raycaster.get_single().map(|ray| ray.intersect_top());
    for (entity, mut card, mut hand_card, mut trans) in hand_cards.iter_mut() {
        match hand_card.hover {
            Hovered if mouse.just_pressed(MouseButton::Left) => {
                hand_card.hover = Dragging;
                break;
            }
            // TODO: Test where the card was released (if in sleeve, then sleeve cheat
            // else if far from hand then activate else return to hand)
            Dragging if mouse.just_released(MouseButton::Left) => {
                card.set_status(CardStatus::Activated);
                cmds.entity(entity).remove::<HandCard>();
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
    hand: Query<&GlobalTransform, With<Hand>>,
    mut cards: Query<(&mut Transform, &HandCard)>,
) {
    use HoverStatus::Hovered;
    const CARD_SPEED: f32 = 0.15;
    let hand_transform = hand.single();
    let hand_pos = hand_transform.translation;
    for (mut transform, HandCard { index, hover }) in cards.iter_mut() {
        let i_f32 = *index as f32;
        let vertical_offset = if *hover == Hovered { -0.2 } else { -1.2 };
        let horizontal_offset = i_f32 - 1.0;
        let z_offset = if *hover == Hovered { 0.01 } else { i_f32 * -0.01 };
        let target = hand_pos + Vec3::new(horizontal_offset, vertical_offset, z_offset);
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
            .add_system_set(SystemSet::on_enter(self.0).with_system(spawn_hand))
            .add_system_to_stage(CoreStage::PreUpdate, update_hand_transform)
            .add_system_set(
                SystemSet::on_update(self.0)
                    .with_system(select_card.label("select"))
                    .with_system(play_card.label("play").after("select"))
                    .with_system(update_hand.after("play"))
                    .with_system(update_raycast),
            );
    }
}
