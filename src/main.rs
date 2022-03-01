use bevy::prelude::*;

mod audio;
mod ui;

#[cfg(debug_assertions)] // only include if compiling in debug mode
mod debug_overlay;
#[cfg(not(debug_assertions))] // add a dummy to make sure code doesn't break
#[macro_export]
macro_rules! add_dbg_text {
    ($($whatever:tt)*) => {};
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            #[cfg(target_os = "linux")]
            vsync: false, // workaround for https://github.com/bevyengine/bevy/issues/1908 (seems to be Mesa bug with X11 + Vulkan)
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(audio::Plugin)
        .add_plugin(ui::Plugin);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new())
        .add_plugin(debug_overlay::Plugin);

    app.run();
}
