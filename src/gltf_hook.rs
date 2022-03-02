//! Systems to insert components on loaded gltf scenes
use std::marker::PhantomData;

use bevy::{
    ecs::{
        schedule::ShouldRun::{self, No, Yes},
        system::EntityCommands,
    },
    prelude::*,
    scene::InstanceId,
};

/// Add this as a component to any entity to trigger
/// [`<T as GltfHook>::hook`](GltfHook::hook)
#[derive(Component)]
pub struct GltfInstance<T: ?Sized> {
    instance: InstanceId,
    loaded: bool,
    _marker: PhantomData<T>,
}
impl<T> GltfInstance<T> {
    pub fn new(instance: InstanceId) -> Self {
        GltfInstance { instance, loaded: false, _marker: PhantomData }
    }
}

/// Define systems to handle adding components to entites named in a loaded
/// gltf model.
///
/// Note that you _should_ (but don't need to) use an uninhabited type to
/// `impl` this trait.
///
/// ## Example
///
/// First you need to define your model type:
/// ```rust
/// const FINGER_COUNT: usize = 5;
/// #[derive(Component)]
/// struct Finger(usize);
/// // Uninhabited type (there are no values of this type and therefore cannot
/// // be instantiated, since we don't intend to instantiate it, might as well
/// // prevent from doing so)
/// enum HandModel {}
/// impl GltfHook for HandModel {
///     fn hook_named_node(name: &Name, cmds: &mut EntityCommands) {
///         const FINGER_NODES_NAMES: [&str; FINGER_COUNT] = [
///             "thumb", "index", "major", "ring", "pinky"
///         ];
///         fn finger_node_index(name: &Name) -> Option<usize> {
///             NECK_JOINT_NAMES
///                 .iter()
///                 .enumerate()
///                 .find(|(_, n)| **n == name.as_str())
///                 .map(|(i, _)| i)
///         }
///         if let Some(index) = finger_node_index(name) {
///             cmds.insert_bundle((
///                 Finger(index),
///                 RigidBody::Dynamic,
///             ));
///         }
///     }
/// }
/// ```
///
/// Then, you should add the `HandModel::hook` system to your bevy ecs, and can
/// add the `HandModel::when_spawned` run criteria to the systems that rely on
/// the presence of the `Finger` component.
/// ```rust
/// fn main {
///     let mut app = App::new();
///     app.add_system_set_to_stage(
///         CoreStage::Update,
///         SystemSet::new()
///             .with_system(play_piano)
///             .with_system(move_finger)
///             // Systems that use a `Finger` component can be made to run
///             // only when the model is spawned with this run criteria
///             .with_run_criteria(HandModel::when_spawned),
///     );
///     // You need to add the `HandModel::hook` system with the
///     // `when_not_spawned` run criteria
///     app.add_system(HandModel::hook.with_run_criteria(HandModel::when_not_spawned));
/// }
/// ```
///
/// If you have multiple of the same models, you _probably want to use another
/// method_ (and take inspiration from the implementation of this trait). But
/// if you have a known-at-compile-time count of the model (typically for
/// player models) you can use a const generic. In the previous example, it is
/// question of replacing the two lines:
/// ```rust
/// // From:
/// enum HandModel {}
/// impl GltfHook for HandModel {
/// // To:
/// enum HandModel<const N: usize> {}
/// impl<const N: usize> GltfHook for HandModel<N> {
/// ```
#[allow(unused_parens)]
pub trait GltfHook: Send + Sync + 'static {
    /// Add [`Component`]s or do anything with `commands`, the
    /// [`EntityCommands`] for the joint node entity in the
    /// `GltfInstance<Self>` gltf file.
    fn hook_named_node(name: &Name, commands: &mut EntityCommands);

    /// `RunCriteria` to add to systems that only run after the joint nodes
    /// were "hooked"
    fn when_spawned(instance: Query<&GltfInstance<Self>>) -> ShouldRun {
        let is_loaded = instance.get_single().map_or(false, |inst| inst.loaded);
        (if is_loaded { Yes } else { No })
    }
    /// `RunCriteria` to add to systems that only run before the joint nodes
    /// were "hooked"
    fn when_not_spawned(instance: Query<&GltfInstance<Self>>) -> ShouldRun {
        let is_loaded = instance.get_single().map_or(false, |inst| inst.loaded);
        (if !is_loaded { Yes } else { No })
    }
    /// Calls [`hook_named_node`] for each named node in the Gltf scene
    /// specified in [`GltfInstance<Self>`](GltfInstance)
    fn hook(
        mut instance: Query<&mut GltfInstance<Self>>,
        mut cmds: Commands,
        names: Query<&Name>,
        scene_manager: Res<SceneSpawner>,
    ) {
        if let Ok(mut gltf_instance) = instance.get_single_mut() {
            if let Some(entities) = scene_manager.iter_instance_entities(gltf_instance.instance) {
                for entity in entities {
                    if let Ok(name) = names.get(entity) {
                        Self::hook_named_node(name, &mut cmds.entity(entity));
                    }
                }
                gltf_instance.loaded = true;
            }
        }
    }
}
