use bevy::prelude::*;

mod audio;
mod debug_overlay;
mod ui;

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            #[cfg(target_os = "linux")]
            vsync: false, // workaround for https://github.com/bevyengine/bevy/issues/1908 (seems to be Mesa bug with X11 + Vulkan)
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(debug_overlay::Plugin)
        .add_plugin(audio::Plugin)
        .add_plugin(ui::Plugin);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());

    app.run();
}
