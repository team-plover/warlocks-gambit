use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_mod_raycast::RayCastSource;
use bevy_scene_hook::{world::SceneHook as WorldSceneHook, SceneInstance};

use crate::{
    animate::Animated,
    card::{OppoCardSpawner, PlayerCardSpawner},
    cheat::{BirdPupil, BirdPupilRoot, PlayerSleeve},
    deck::{Deck, DeckAssets, OppoDeck, PlayerDeck},
    oppo_hand::OppoHand,
    pile::{Pile, PileType},
    player_hand::{HandDisengageArea, HandRaycast, PlayerHand, SleeveArea},
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
            "OppoPile" => cmds.insert(Pile::new(PileType::Oppo)),
            "PlayerPile" => cmds.insert(Pile::new(PileType::Player)),
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
    let scene_name = if cfg!(feature = "debug") {
        "scene_debug.glb#Scene0"
    } else {
        "scene.glb#Scene0"
    };
    let scene = scene_spawner.spawn(asset_server.load(scene_name));
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
