use bevy::ecs::schedule::{IntoSystemDescriptor, StateData};
use bevy::prelude::*;

pub trait EasySystemSetCtor {
    fn on_update<Params>(self, system: impl IntoSystemDescriptor<Params>) -> SystemSet;
    fn on_enter<Params>(self, system: impl IntoSystemDescriptor<Params>) -> SystemSet;
    fn on_exit<Params>(self, system: impl IntoSystemDescriptor<Params>) -> SystemSet;
}
impl<St: StateData> EasySystemSetCtor for St {
    fn on_update<Params>(self, system: impl IntoSystemDescriptor<Params>) -> SystemSet {
        SystemSet::on_update(self).with_system(system)
    }
    fn on_exit<Params>(self, system: impl IntoSystemDescriptor<Params>) -> SystemSet {
        SystemSet::on_exit(self).with_system(system)
    }
    fn on_enter<Params>(self, system: impl IntoSystemDescriptor<Params>) -> SystemSet {
        SystemSet::on_enter(self).with_system(system)
    }
}
