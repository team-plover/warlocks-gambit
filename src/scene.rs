use bevy::{
    ecs::system::EntityCommands,
    prelude::{Plugin as BevyPlugin, *},
};

use crate::{
    animate::Animated,
    camera::PlayerCam,
    card_spawner::{OppoCardSpawner, OppoHand, PlayerCardSpawner, PlayerHand, PlayerSleeve},
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
            "BirdEyePupilla" => cmds.insert(Animated::bob(5. / 4., 0.02, 5.0)),
            "PlayerSleeveStash" => cmds.insert(PlayerSleeve),
            "ManSkull" => cmds.insert(GameOverAnimation::Skull),
            "DemonArm" => cmds.insert(GameOverAnimation::DemonArmOppo),
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
            .add_system(gameover_prepare_scene.with_run_criteria(Scene::when_spawned))
            .add_system(Scene::hook.with_run_criteria(Scene::when_not_spawned));
    }
}
