use bevy::prelude::*;

mod audio;
mod camera;
mod card;
mod player_hand;
mod state;
mod ui;

use state::GameState;

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            #[cfg(target_os = "linux")]
            vsync: false, // workaround for https://github.com/bevyengine/bevy/issues/1908 (seems to be Mesa bug with X11 + Vulkan)
            ..Default::default()
        })
        .add_state(GameState::MainMenu)
        .add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());

    app.add_plugin(camera::Plugin)
        .add_plugin(player_hand::Plugin(GameState::Playing))
        .add_plugin(audio::Plugin)
        .add_plugin(card::Plugin)
        .add_plugin(ui::Plugin(GameState::MainMenu))
        .add_system(setup);

    app.run();
}

fn setup(mut ambiant_light: ResMut<AmbientLight>) {
    *ambiant_light = AmbientLight { color: Color::ALICE_BLUE, brightness: 0.9 };
}
