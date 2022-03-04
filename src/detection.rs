use bevy::prelude::{Plugin as BevyPlugin, *};

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {}
}
