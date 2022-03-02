use bevy::prelude::*;

mod audio;
mod card;
mod card_effect;
mod game_ui;
mod gltf_hook;
mod oppo_hand;
mod pile;
mod player_hand;
mod scene;
mod state;
mod ui;
mod war;

mod camera {
    use bevy::prelude::Component;

    #[derive(Component)]
    pub struct PlayerCam;
}
// TODO: rename this to reflect content
mod card_spawner {
    use bevy::prelude::Component;

    /// Component attached to where the opponent draws cards from.
    #[derive(Component)]
    pub struct OppoCardSpawner;

    /// Component attached to where the player draws cards from.
    #[derive(Component)]
    pub struct PlayerCardSpawner;

    /// Position of the hand of the player
    #[derive(Component)]
    pub struct PlayerHand;

    /// Position of the hand of the opposition
    #[derive(Component)]
    pub struct OppoHand;
}

pub enum Participant {
    Player,
    Oppo,
}

use state::{GameState, TurnState};

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            #[cfg(target_os = "linux")]
            vsync: false, // workaround for https://github.com/bevyengine/bevy/issues/1908 (seems to be Mesa bug with X11 + Vulkan)
            ..Default::default()
        })
        .add_state(GameState::MainMenu)
        .add_state(TurnState::None)
        .add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());

    app.add_plugin(player_hand::Plugin(GameState::Playing))
        .add_plugin(oppo_hand::Plugin(GameState::Playing))
        .add_plugin(scene::Plugin(GameState::LoadScene))
        .add_plugin(audio::Plugin)
        .add_plugin(card::Plugin)
        .add_plugin(ui::common::Plugin)
        .add_plugin(ui::main_menu::Plugin(GameState::MainMenu))
        .add_plugin(ui::gameover::Plugin)
        .add_plugin(ui::restart_menu::Plugin)
        .add_plugin(pile::Plugin(GameState::Playing))
        .add_plugin(card_effect::Plugin(GameState::Playing))
        .add_plugin(game_ui::Plugin(GameState::Playing))
        .add_system(setup)
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(enter_game));

    app.run();
}

fn setup(mut ambiant_light: ResMut<AmbientLight>) {
    *ambiant_light = AmbientLight { color: Color::WHITE, brightness: 1.0 };
}

fn enter_game(mut turn_state: ResMut<State<TurnState>>) {
    turn_state.set(TurnState::Player).unwrap();
}
