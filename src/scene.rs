//! Load the game scene and add `Component`s from all modules to entities named
//! in the scene.
use std::f32::consts::TAU;

use bevy::{
    math::EulerRot::XYZ,
    pbr::wireframe::Wireframe,
    prelude::{Plugin as BevyPlugin, *},
};
use bevy_mod_raycast::{RayCastMesh, RayCastSource};
use bevy_scene_hook::{world::SceneHook as WorldSceneHook, SceneInstance};

use crate::{
    animate::Animated,
    card::{OppoCardSpawner, PlayerCardSpawner},
    cheat::{BirdPupil, BirdPupilRoot, PlayerSleeve},
    deck::{Deck, DeckAssets, OppoDeck, PlayerDeck},
    game_ui::{OppoScore, PlayerScore},
    numbers::Number,
    oppo_hand::OppoHand,
    pile::{Pile, PileType},
    player_hand::{CardCollisionAssets, HandDisengageArea, HandRaycast, PlayerHand, SleeveArea},
    Participant,
};

pub enum Scene {}
impl WorldSceneHook for Scene {
    fn hook_named_node(name: Name, world: &mut World, entity: Entity) {
        match name.as_str() {
            "PlayerDeck" => {
                let handle = world.get_resource::<DeckAssets>().unwrap().player.clone();
                let assets = world.get_resource::<Assets<Deck>>().unwrap();
                let deck = assets.get(handle).unwrap().clone();
                world.entity_mut(entity).insert(PlayerDeck::new(deck));
            }
            "OppoDeck" => {
                let handle = world.get_resource::<DeckAssets>().unwrap().oppo.clone();
                let assets = world.get_resource::<Assets<Deck>>().unwrap();
                let deck = assets.get(handle).unwrap().clone();
                world.entity_mut(entity).insert(OppoDeck::new(deck));
            }
            "PlayerHand" => {
                let mesh = world.get_resource::<CardCollisionAssets>().unwrap();
                let mesh = mesh.circle.clone();
                world.spawn().insert_bundle((
                    mesh,
                    Wireframe,
                    RayCastMesh::<HandDisengageArea>::default(),
                    Visibility::default(),
                    ComputedVisibility::default(),
                    Parent(entity),
                    GlobalTransform::default(),
                    Transform {
                        rotation: Quat::from_rotation_y(TAU / 2.),
                        scale: Vec3::new(1.75, 1.5, 1.75),
                        translation: Vec3::ZERO,
                    },
                ));
            }
            "PlayerSleeveStash" => {
                let mesh = world.get_resource::<CardCollisionAssets>().unwrap();
                let mesh = mesh.circle.clone();
                world.spawn().insert_bundle((
                    mesh,
                    Wireframe,
                    RayCastMesh::<SleeveArea>::default(),
                    Visibility::default(),
                    ComputedVisibility::default(),
                    Parent(entity),
                    GlobalTransform::default(),
                    Transform {
                        rotation: Quat::from_rotation_y(TAU / 2.),
                        scale: Vec3::new(1., 0.7, 1.),
                        translation: Vec3::new(0., 1.7, 0.2),
                    },
                ));
            }
            _ => {}
        }
        let mut cmds = world.entity_mut(entity);
        match name.as_str() {
            "PlayerPerspective_Orientation" => cmds.insert_bundle((
                RayCastSource::<HandRaycast>::new(),
                RayCastSource::<SleeveArea>::new(),
                RayCastSource::<HandDisengageArea>::new(),
            )),
            "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
            "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
            "OppoHand" => cmds.insert_bundle((OppoHand, Animated::bob(1.0, 0.3, 6.0))),
            "PlayerHand" => cmds.insert_bundle((PlayerHand, Animated::bob(2.0, 0.05, 7.0))),
            "Pile" => cmds.insert(Pile::new(PileType::War)),
            "OppoPile" => cmds
                .insert(Pile::new(PileType::Oppo))
                .with_children(|cmds| {
                    cmds.spawn_bundle((
                        Name::new("Oppo score"),
                        OppoScore,
                        Number::new(0, Participant::Oppo.color()),
                        Transform {
                            translation: Vec3::Z,
                            rotation: Quat::from_euler(XYZ, TAU / 4., 0.5, 0.0),
                            scale: Vec3::splat(0.3),
                        },
                        GlobalTransform::default(),
                    ));
                }),
            "PlayerPile" => cmds
                .insert(Pile::new(PileType::Player))
                .with_children(|cmds| {
                    cmds.spawn_bundle((
                        Name::new("Player score"),
                        PlayerScore,
                        Number::new(0, Participant::Player.color()),
                        Transform {
                            translation: Vec3::Z,
                            rotation: Quat::from_euler(XYZ, TAU / 4., -1.1, 0.0),
                            scale: Vec3::splat(0.3),
                        },
                        GlobalTransform::default(),
                    ));
                }),
            "ManBody" => cmds.insert(Animated::breath(0.0, 0.03, 6.0)),
            "ManHead" => cmds.insert(Animated::bob(6. / 4., 0.1, 6.0)),
            "Bird" => cmds.insert(Animated::breath(0.0, 0.075, 5.0)),
            "BirdPupillaSprite" => cmds.insert(BirdPupil),
            "BirdEyePupilla" => {
                cmds.insert_bundle((BirdPupilRoot, Animated::bob(5. / 4., 0.02, 5.0)))
            }
            "PlayerSleeveStash" => cmds.insert(PlayerSleeve),
            _ => &mut cmds,
        };
    }
}

fn load_scene(
    mut cmds: Commands,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
) {
    let scene = scene_spawner.spawn(asset_server.load("scene.glb#Scene0"));
    cmds.spawn().insert(SceneInstance::<Scene>::new(scene));
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            Scene::hook
                .exclusive_system()
                .with_run_criteria(Scene::when_not_spawned),
        )
        .add_startup_system(load_scene);
    }
}
