use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource};

use crate::{
    camera::PlayerCam,
    card::{Card, SpawnCard, Value, WordOfMagic},
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

    let hand = cmds
        .spawn_bundle((
            GlobalTransform::default(),
            Transform::from_xyz(0.0, -3.6, -10.0),
            Name::new("Player hand"),
            Hand,
            Parent(cam),
        ))
        .id();
    for (i, value) in [Zero, Two, Seven, Eight].iter().enumerate() {
        card_spawner
            .spawn_card(Card::new(WordOfMagic::Meb, *value))
            .insert_bundle((
                HandCard::new(i),
                Parent(hand),
                GlobalTransform::default(),
                Transform::default(),
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

fn update_hand(mut hand: Query<(&mut Transform, &HandCard, Option<&HoveredCard>)>) {
    for (mut transform, HandCard { index }, hover) in hand.iter_mut() {
        let i_f32 = *index as f32;
        let vertical_offset = if hover.is_some() { 2.0 } else { 0.0 };
        let z_offset = if hover.is_some() { 0.1 } else { i_f32 * -0.1 };
        // TODO: full transform lerp
        let target = Vec3::new(i_f32 * 1.7 - 2.0, vertical_offset, z_offset);
        let origin = transform.translation;
        transform.translation += (target - origin) * 0.2;
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
