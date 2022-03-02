use crate::state::GameState;
use bevy::prelude::*;

pub fn init() {
    todo!()
}

pub fn cleanup() {
    todo!()
}

pub fn update() {
    todo!()
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::RestartMenu).with_system(init));
        app.add_system_set(SystemSet::on_exit(GameState::RestartMenu).with_system(cleanup));
        app.add_system_set(SystemSet::on_update(GameState::RestartMenu).with_system(update));
    }
}
