use bevy::prelude::*;
use heron::prelude::*;

mod audio;
mod ui;

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(audio::Plugin)
        .add_plugin(ui::Plugin);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());

    app.run();
}
