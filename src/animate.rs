//! Animations.
use std::f64::consts::PI;

use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

#[derive(Component)]
pub struct DisableAnimation;

/// Modify the transform of entities it's attached to.
#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub enum Animated {
    /// Change `scale` to give a feeling of breathing.
    Breath {
        offset: f64,
        strength: f32,
        period: f64,
    },
    /// Bob up and down, offset by `f32` seconds.
    Bob {
        offset: f64,
        strength: f32,
        period: f64,
    },
    /// Go in a cirlce on provided axis.
    Circle {
        offset: f64,
        radius: f32,
        period: f64,
    },
    Static,
}
impl Animated {
    pub fn bob(offset: f64, strength: f32, period: f64) -> Self {
        Animated::Bob { offset, strength, period }
    }
    pub fn breath(offset: f64, strength: f32, period: f64) -> Self {
        Animated::Breath { offset, strength, period }
    }
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
struct InitialTransform(Transform);

fn enable_animation(animated: Query<(Entity, &Transform), Added<Animated>>, mut cmds: Commands) {
    let mut cmd_buffer = Vec::new();
    for (entity, transform) in animated.iter() {
        cmd_buffer.push((entity, (InitialTransform(*transform),)));
    }
    cmds.insert_or_spawn_batch(cmd_buffer);
}

fn reset_static(
    mut animated: Query<(&mut Transform, &InitialTransform, &Animated), Changed<Animated>>,
) {
    for (mut trans, init, anim) in animated.iter_mut() {
        if matches!(anim, Animated::Static) {
            *trans = init.0;
        }
    }
}

fn run_animation(
    time: Res<Time>,
    mut animated: Query<(&mut Transform, &InitialTransform, &Animated), Without<DisableAnimation>>,
) {
    let time = time.seconds_since_startup();
    for (mut trans, init, anim) in animated.iter_mut() {
        match *anim {
            Animated::Static => {}
            Animated::Bob { offset, strength, period } => {
                let anim_offset = (time + offset) % period / period * PI * 2.0;
                // ao = 0 → 0; ao = 1 → 0.2; ao = 2 → 0
                let with_strength = (anim_offset as f32).sin() * strength;
                let space_offset = Vec3::Y * with_strength;
                trans.translation = init.0.translation + space_offset;
            }
            Animated::Breath { offset, strength, period } => {
                let anim_offset = (time + offset) % period / period * PI * 2.0;
                // ao = 0 → (0, 0.1); ao = 1 → (0.1, 0.0); ao = 2 → (0, 0.1)
                let scale_offset = Vec3::new(
                    (anim_offset as f32).sin() * strength,
                    0.0,
                    (anim_offset as f32).cos() * strength,
                );
                trans.scale = init.0.scale + scale_offset;
            }
            Animated::Circle { offset, period, radius } => {
                let anim_offset = ((time + offset) % period / period * PI * 2.0) as f32;
                let trans_offset = Vec3::new(anim_offset.sin(), anim_offset.cos(), 0.0) * radius;
                trans.translation = init.0.translation + trans_offset;
            }
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<Animated>()
            .register_inspectable::<InitialTransform>();

        app.add_system(enable_animation)
            .add_system(reset_static)
            .add_system(run_animation.label("animation"));
    }
}
