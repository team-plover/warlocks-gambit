//! Animations.
use std::f64::consts::PI;

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_debug_text_overlay::screen_print;
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
        period: f64,
        strength: f32,
    },
    /// Bob up and down, offset by `f32` seconds.
    Bob {
        offset: f64,
        period: f64,
        strength: f32,
    },
    /// Go in a cirlce on provided axis.
    Circle {
        offset: f64,
        period: f64,
        radius: f32,
    },
    /// Go in direction for seconds and progressively become tinny.
    RiseAndFade {
        duration: f32,
        direction: Vec3,
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
struct AnimationState {
    transform: Transform,
    time: f64,
}

fn enable_animation(
    time: Res<Time>,
    animated: Query<(Entity, &Transform), Added<Animated>>,
    mut cmds: Commands,
) {
    let mut cmd_buffer = Vec::new();
    for (entity, &transform) in animated.iter() {
        let state = AnimationState { transform, time: time.seconds_since_startup() };
        cmd_buffer.push((entity, (state,)));
    }
    cmds.insert_or_spawn_batch(cmd_buffer);
}

fn reset_static(
    mut animated: Query<(&mut Transform, &AnimationState, &Animated), Changed<Animated>>,
) {
    for (mut trans, init, anim) in animated.iter_mut() {
        if matches!(anim, Animated::Static) {
            *trans = init.transform;
        }
    }
}

fn run_animation(
    time: Res<Time>,
    mut cmds: Commands,
    mut animated: Query<
        (Entity, &mut Transform, &AnimationState, &Animated),
        Without<DisableAnimation>,
    >,
) {
    let time = time.seconds_since_startup();
    for (entity, mut trans, init, anim) in animated.iter_mut() {
        match *anim {
            Animated::Static => {}
            Animated::Bob { offset, strength, period } => {
                let anim_offset = (time + offset) % period / period * PI * 2.0;
                // ao = 0 → 0; ao = 1 → 0.2; ao = 2 → 0
                let with_strength = (anim_offset as f32).sin() * strength;
                let space_offset = Vec3::Y * with_strength;
                trans.translation = init.transform.translation + space_offset;
            }
            Animated::Breath { offset, strength, period } => {
                let anim_offset = (time + offset) % period / period * PI * 2.0;
                // ao = 0 → (0, 0.1); ao = 1 → (0.1, 0.0); ao = 2 → (0, 0.1)
                let scale_offset = Vec3::new(
                    (anim_offset as f32).sin() * strength,
                    0.0,
                    (anim_offset as f32).cos() * strength,
                );
                trans.scale = init.transform.scale + scale_offset;
            }
            Animated::RiseAndFade { duration, direction } => {
                let delta = (time - init.time) as f32;
                let expiring = delta - duration > 0.0;
                let extra_delta = if expiring { delta - duration } else { 0.0 };
                let scale = 1.0 - extra_delta;
                if scale <= 0.0 {
                    screen_print!("Despawning a RiseAndFade animation");
                    cmds.entity(entity).despawn_recursive();
                } else {
                    let offset = delta.min(duration) + extra_delta * 0.7;
                    trans.translation = init.transform.translation + direction * offset;
                    trans.scale = Vec3::splat(scale);
                }
            }
            Animated::Circle { offset, period, radius } => {
                let anim_offset = ((time + offset) % period / period * PI * 2.0) as f32;
                let trans_offset = Vec3::new(anim_offset.sin(), anim_offset.cos(), 0.0) * radius;
                trans.translation = init.transform.translation + trans_offset;
            }
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<Animated>()
            .register_inspectable::<AnimationState>();

        app.add_system(enable_animation)
            .add_system(reset_static)
            .add_system(run_animation.label("animation"));
    }
}
