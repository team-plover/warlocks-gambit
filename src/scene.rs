use bevy::{
    ecs::system::EntityCommands,
    prelude::{Plugin as BevyPlugin, *},
};

use crate::{
    camera::PlayerCam,
    card_spawner::{OppoCardSpawner, OppoHand, PlayerCardSpawner, PlayerHand},
    gltf_hook::{GltfHook, GltfInstance},
    pile::Pile,
    state::GameState,
};

pub enum Scene {}
impl GltfHook for Scene {
    fn hook_named_node(name: &Name, cmds: &mut EntityCommands) {
        match name.as_str() {
            "PlayerPerspective_Orientation" => cmds.insert(PlayerCam),
            "PlayerCardSpawn" => cmds.insert(PlayerCardSpawner),
            "OppoCardSpawn" => cmds.insert(OppoCardSpawner),
            "OppoHand" => cmds.insert(OppoHand),
            "PlayerHand" => cmds.insert(PlayerHand),
            "Pile" => cmds.insert(Pile::default()),
            _ => cmds,
        };
    }
}

fn setup_scene(
    mut cmds: Commands,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
) {
    let scene = if cfg!(feature = "debug") {
        "scene_debug.glb#Scene0"
    } else {
        "scene.glb#Scene0"
    };
    let gltf = scene_spawner.spawn(asset_server.load(scene));
    cmds.spawn().insert(GltfInstance::<Scene>::new(gltf));
}
fn exit_load_state(mut state: ResMut<State<GameState>>) {
    if state.current() == &GameState::LoadScene {
        state.set(GameState::Playing).unwrap();
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(self.0).with_system(setup_scene))
            .add_system(exit_load_state.with_run_criteria(Scene::when_spawned))
            .add_system(Scene::hook.with_run_criteria(Scene::when_not_spawned));
    }
}
