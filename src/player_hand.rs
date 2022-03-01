use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource};

use crate::{
    camera::PlayerCam,
    card::{Card, SpawnCard, Value, WordOfPower},
    state::GameState,
};

enum HandRaycast {}

#[derive(Component)]
struct Hand;

#[derive(Component)]
struct HoveredCard;

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
struct HandCard {
    index: usize,
}
impl HandCard {
    fn new(index: usize) -> Self {
        Self { index }
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
    mut cmds: Commands,
    mut cursor: EventReader<CursorMoved>,
    hand_raycaster: Query<&RayCastSource<HandRaycast>>,
    hovered: Query<Entity, With<HoveredCard>>,
) {
    let query = hand_raycaster.get_single().map(|ray| ray.intersect_top());
    let has_cursor_moved = cursor.iter().next().is_some();
    if let Ok(Some((hovered_card, _))) = query {
        // `hovered_card` is not the one that already exists
        if hovered.get(hovered_card).is_err() && has_cursor_moved {
            if let Ok(old_hovered) = hovered.get_single() {
                cmds.entity(old_hovered).remove::<HoveredCard>();
            }
            cmds.entity(hovered_card).insert(HoveredCard);
        }
    }
}

/// Move progressively cards from [`HandCard`] in front of player camera.
fn update_hand(
    hand: Query<&GlobalTransform, With<Hand>>,
    mut cards: Query<(&mut Transform, &HandCard, Option<&HoveredCard>)>,
) {
    const CARD_SPEED: f32 = 0.15;
    let hand_transform = hand.single();
    let hand_pos = hand_transform.translation;
    for (mut transform, HandCard { index }, hover) in cards.iter_mut() {
        let i_f32 = *index as f32;
        let vertical_offset = if hover.is_some() { -0.2 } else { -1.2 };
        let horizontal_offset = i_f32 - 1.0;
        let z_offset = if hover.is_some() { 0.01 } else { i_f32 * -0.01 };
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
                    .with_system(update_hand)
                    .with_system(select_card)
                    .with_system(update_raycast),
            );
    }
}
