use bevy::prelude::{Plugin as BevyPlugin, *};

#[derive(Component)]
pub struct PlayerCam;

fn spawn_camera(mut cmds: Commands) {
    cmds.spawn_bundle(PerspectiveCameraBundle::new_3d())
        .insert(PlayerCam);
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera);
    }
}
