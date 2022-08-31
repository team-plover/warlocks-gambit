//! Load the game scene and add `Component`s from all modules to entities named
//! in the scene.
use std::f32::consts::TAU;

use bevy::{
    ecs::system::EntityCommands,
    math::EulerRot::XYZ,
    pbr::wireframe::Wireframe,
    prelude::{Plugin as BevyPlugin, *},
};
use bevy_mod_raycast::{RayCastMesh, RayCastSource};
use bevy_scene_hook::{HookingSceneSpawner, HookPlugin};

use crate::{
    animate::Animated,
    card::{OppoCardSpawner, PlayerCardSpawner},
    cheat::{BirdPupil, BirdPupilRoot, PlayerSleeve},
    deck::DeckAssets,
    game_ui::{OppoScore, PlayerScore},
    numbers::Number,
    oppo_hand::OppoHand,
    pile::{Pile, PileType},
    player_hand::{CardCollisionAssets, HandDisengageArea, HandRaycast, PlayerHand, SleeveArea},
    Participant,
};

#[derive(Component)]
pub struct Graveyard;

fn hook(
    card_meshes: &CardCollisionAssets,
    decks: &DeckAssets,
    name: &str,
    cmds: &mut EntityCommands,
) {
    use Participant::{Oppo, Player};
    let participant = if name.starts_with("Oppo") { Oppo } else { Player };
    match name {
        "PlayerDeck" => cmds.insert(decks.player.clone_weak()),
        "OppoDeck" => cmds.insert(decks.oppo.clone_weak()),
        "PlayerHand" => cmds
            .insert_bundle((PlayerHand, Animated::bob(2.0, 0.05, 7.0)))
            .with_children(|cmds| {
                cmds.spawn_bundle((
                    card_meshes.circle.clone_weak(),
                    Wireframe,
                    RayCastMesh::<HandDisengageArea>::default(),
                    Visibility::default(),
                    ComputedVisibility::default(),
                    GlobalTransform::default(),
                    Transform {
                        rotation: Quat::from_rotation_y(TAU / 2.),
                        scale: Vec3::new(1.75, 1.5, 1.75),
                        translation: Vec3::ZERO,
                    },
                ));
            }),
        "PlayerSleeveStash" => cmds.insert(PlayerSleeve).with_children(|cmds| {
            cmds.spawn_bundle((
                card_meshes.circle.clone_weak(),
                Wireframe,
                RayCastMesh::<SleeveArea>::default(),
                Visibility::default(),
                ComputedVisibility::default(),
                GlobalTransform::default(),
                Transform {
                    rotation: Quat::from_rotation_y(TAU / 2.),
                    scale: Vec3::new(1., 0.7, 1.),
                    translation: Vec3::new(0., 1.7, 0.2),
                },
            ));
        }),
        "PlayerPerspective_Orientation" => cmds.insert_bundle((
            RayCastSource::<HandRaycast>::new(),
            RayCastSource::<SleeveArea>::new(),
            RayCastSource::<HandDisengageArea>::new(),
        )),
        "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
        "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
        "OppoHand" => cmds.insert_bundle((OppoHand, Animated::bob(1.0, 0.3, 6.0))),
        "Pile" => cmds.insert(Pile::new(PileType::War)),
        "OppoPile" | "PlayerPile" => {
            cmds.insert(Pile::new(participant.into()))
                .with_children(|cmds| {
                    let pile_rotation = match participant {
                        Oppo => Quat::from_euler(XYZ, TAU / 4., 0.5, 0.0),
                        Player => Quat::from_euler(XYZ, TAU / 4., -1.1, 0.0),
                    };
                    let transform = Transform {
                        translation: Vec3::Z,
                        rotation: pile_rotation,
                        scale: Vec3::splat(0.3),
                    };
                    let mut cmds = cmds.spawn_bundle((
                        Name::new(participant.name().to_owned() + " score"),
                        Number::new(0, participant.color()),
                    ));

                    match participant {
                        Oppo => cmds.insert(OppoScore),
                        Player => cmds.insert(PlayerScore),
                    };
                })
        }
        "ManBody" => cmds.insert(Animated::breath(0.0, 0.03, 6.0)),
        "ManHead" => cmds.insert(Animated::bob(6. / 4., 0.1, 6.0)),
        "Bird" => cmds.insert(Animated::breath(0.0, 0.075, 5.0)),
        "BirdPupillaSprite" => cmds.insert(BirdPupil),
        "BirdEyePupilla" => cmds.insert_bundle((BirdPupilRoot, Animated::bob(5. / 4., 0.02, 5.0))),
        _ => cmds,
    };
}
fn load_scene(
    mut cmds: Commands,
    mut scene_spawner: HookingSceneSpawner,
    card_meshes: Res<CardCollisionAssets>,
    decks: Res<DeckAssets>,
    asset_server: Res<AssetServer>,
) {
    let card_meshes = card_meshes.clone();
    let decks = decks.clone();
    let result = scene_spawner.with_comp_hook(
        asset_server.load("scene.glb#Scene0"),
        move |name: &Name, cmds| hook(&card_meshes, &decks, name.as_str(), cmds),
    );
    cmds.entity(result).insert(Graveyard);
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_scene);
    }
}
