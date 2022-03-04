use bevy::{
    ecs::system::EntityCommands,
    prelude::{Plugin as BevyPlugin, *},
};

use crate::{
    animate::Animated,
    camera::PlayerCam,
    card_spawner::{
        BirdPupil, BirdPupilRoot, OppoCardSpawner, OppoHand, PlayerCardSpawner, PlayerHand,
        PlayerSleeve,
    },
    gltf_hook::{GltfHook, GltfInstance},
    pile::{Pile, PileType},
    state::GameState,
    ui::gameover::{gameover_prepare_scene, GameOverAnimation},
};

pub enum Scene {}
impl GltfHook for Scene {
    fn hook_named_node(name: &Name, cmds: &mut EntityCommands) {
        match name.as_str() {
            "PlayerPerspective_Orientation" => cmds.insert(PlayerCam),
            "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
            "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
            "OppoHand" => cmds.insert_bundle((OppoHand, Animated::bob(1.0, 0.3, 6.0))),
            "PlayerHand" => cmds.insert_bundle((PlayerHand, Animated::bob(2.0, 0.05, 7.0))),
            "Pile" => cmds.insert(Pile::new(PileType::War)),
            "OppoPile" => cmds.insert(Pile::new(PileType::Oppo)),
            "PlayerPile" => cmds.insert(Pile::new(PileType::Player)),
            "ManBody" => cmds.insert(Animated::breath(0.0, 0.03, 6.0)),
            "ManHead" => cmds
                .insert(Animated::bob(6. / 4., 0.1, 6.0))
                .insert(GameOverAnimation::Head),
            "Bird" => cmds.insert(Animated::breath(0.0, 0.075, 5.0)),
            "BirdPupillaSprite" => cmds.insert(BirdPupil),
            "BirdEyePupilla" => {
                cmds.insert_bundle((BirdPupilRoot, Animated::bob(5. / 4., 0.02, 5.0)))
            }
            "PlayerSleeveStash" => cmds.insert(PlayerSleeve),
            "ManSkull" => cmds.insert(GameOverAnimation::Skull),
            "DemonArm" => cmds.insert(GameOverAnimation::DemonArmOppo),
            _ => cmds,
        };
    }
}

#[derive(Default)]
pub struct ScenePreload {
    pub game: Handle<bevy::prelude::Scene>,
    pub main_menu: Handle<bevy::prelude::Scene>,
}

fn load_scene(asset_server: Res<AssetServer>, mut scene: ResMut<ScenePreload>) {
    scene.game = asset_server.load(if cfg!(feature = "debug") {
        "scene_debug.glb#Scene0"
    } else {
        "scene.glb#Scene0"
    });
    scene.main_menu = asset_server.load("scene_mainmenu.glb#Scene0");
}

fn wait_load_scene(
    scene: Res<ScenePreload>,
    mut state: ResMut<State<GameState>>,
    server: Res<AssetServer>,
) {
    if server.get_group_load_state([scene.game.id, scene.main_menu.id])
        == bevy::asset::LoadState::Loaded
    {
        state.set(GameState::MainMenu).unwrap();
    }
}

fn setup_scene(
    mut cmds: Commands,
    mut scene_spawner: ResMut<SceneSpawner>,
    scene: Res<ScenePreload>,
    mut state: ResMut<State<GameState>>,
    mut ugly_hack: Local<u8>, // avoid panic on initing oppo hand
) {
    if *ugly_hack == 0 {
        let gltf = scene_spawner.spawn(scene.game.clone());
        cmds.spawn().insert(GltfInstance::<Scene>::new(gltf));

        *ugly_hack = 1;
    } else if *ugly_hack == 1 {
        state.set(GameState::Playing).unwrap();
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(self.0).with_system(setup_scene))
            .add_system(gameover_prepare_scene.with_run_criteria(Scene::when_spawned))
            .add_system(Scene::hook.with_run_criteria(Scene::when_not_spawned))
            .insert_resource(ScenePreload::default())
            .add_startup_system(load_scene)
            .add_system_set(
                SystemSet::on_update(GameState::ScenePreload).with_system(wait_load_scene),
            );
    }
}
