use super::common::*;
use crate::state::GameState;
use bevy::prelude::*;

// To enter game over screen send GameOverKind via EventWriter.
// Upon entering screen animation is played, when it goes to RestartMenu.

// TODO: actual, non-placeholder animation
// TODO: normal game screen should be drawn when GameOver state is active

#[derive(Clone, Copy, Debug)]
#[allow(unused)]
pub enum GameOverKind {
    PlayerWon,
    PlayerLost,
    CheatSpotted,
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum GameOverAnimation {
    Head,
    Skull,
    DemonArmOppo,
}

//

pub struct GameoverAssets {
    pub defeat: Handle<Image>,
    pub victory: Handle<Image>,
}

impl FromWorld for GameoverAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            defeat: assets.load("menu/ending_Defeat.png"),
            victory: assets.load("menu/ending_Victory.png"),
        }
    }
}

/// This inits gameover-specific parts of the scene
pub fn gameover_prepare_scene(
    gameover: Query<(&Children, &GameOverAnimation)>,
    mut visibility: Query<&mut Visibility>,
) {
    for (children, animation) in gameover.iter() {
        match animation {
            GameOverAnimation::Head => (),
            _ => {
                for child in children.iter() {
                    if let Ok(mut v) = visibility.get_mut(*child) {
                        v.is_visible = false;
                    }
                }
            }
        }
    }
}

//

/// Cleanup marker
#[derive(Component, Clone)]
struct SceneRoot;

/// Marker for 'Press ESC to exit' text
#[derive(Component)]
struct EscapeMessage;

fn init(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    mut start_time: ResMut<StartTime>,
    time: Res<Time>,
) {
    let skip_message = "Press ESCAPE to skip";
    commands
        .spawn_bundle(NodeBundle { color: Color::NONE.into(), ..Default::default() })
        .insert(SceneRoot)
        .with_children(|parent| {
            parent
                .spawn_bundle(ui_assets.large_text(skip_message))
                .insert(Visibility { is_visible: false })
                .insert(EscapeMessage);
        });

    start_time.0 = time.seconds_since_startup();
}

fn interrupt_animation(
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut state: ResMut<State<GameState>>,
    mut message: Query<&mut Visibility, With<EscapeMessage>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        state.set(GameState::RestartMenu).unwrap()
    }
    if keys.get_just_pressed().len() + buttons.get_just_pressed().len() > 0 {
        message.iter_mut().for_each(|mut vis| vis.is_visible = true)
    }
}

fn cleanup(root: Query<Entity, With<SceneRoot>>, mut commands: Commands) {
    for entity in root.iter() {
        commands.entity(entity).despawn_recursive()
    }
}

//

#[derive(Default)]
struct StartTime(f64); // seconds since startup

fn animation(
    kind: Res<GameOverKind>,
    start_time: Res<StartTime>,
    time: Res<Time>,
    anim_parents: Query<(&Children, &GameOverAnimation)>,
    mut animatable: Query<(&mut Visibility, &mut Transform)>,
    mut state: ResMut<State<GameState>>,
) {
    let get_animatables = |which: GameOverAnimation| {
        anim_parents
            .iter()
            .find(|(_, anim)| **anim == which)
            .map(|(children, _)| children.iter().cloned())
    };
    let seconds_passed = (time.seconds_since_startup() - start_time.0) as f32;

    match *kind {
        GameOverKind::PlayerWon => {
            let arm_raise_time = 2.;
            let arm_hold_time = 1.;
            let arm_lower_time = 1.;

            if seconds_passed < arm_raise_time {
                let t = seconds_passed / arm_raise_time;

                if let Some(entities) = get_animatables(GameOverAnimation::DemonArmOppo) {
                    for entity in entities {
                        if let Ok((mut visible, mut transform)) = animatable.get_mut(entity) {
                            visible.is_visible = true;
                            //transform.translation.y = t * -200.;
                        }
                    }
                }
            } else if seconds_passed < arm_raise_time + arm_hold_time {
                let _t = (seconds_passed - arm_raise_time) / arm_hold_time;

                // replace head with skull
                if let Some(entities) = get_animatables(GameOverAnimation::Head) {
                    for entity in entities {
                        if let Ok((mut visible, _)) = animatable.get_mut(entity) {
                            visible.is_visible = false;
                        }
                    }
                }
                if let Some(entities) = get_animatables(GameOverAnimation::Skull) {
                    for entity in entities {
                        if let Ok((mut visible, _)) = animatable.get_mut(entity) {
                            visible.is_visible = true;
                        }
                    }
                }
            } else if seconds_passed < arm_raise_time + arm_hold_time + arm_lower_time {
                let t = (seconds_passed - arm_raise_time - arm_hold_time) / arm_lower_time;

                if let Some(entities) = get_animatables(GameOverAnimation::DemonArmOppo) {
                    for entity in entities {
                        if let Ok((mut _visible, mut transform)) = animatable.get_mut(entity) {
                            transform.translation.y = (1. - t) * -200.;
                        }
                    }
                }
            } else {
                state.set(GameState::RestartMenu).unwrap()
            }
        }
        GameOverKind::PlayerLost => todo!(),
        GameOverKind::CheatSpotted => todo!(),
    }
}

//

fn enter_state(
    mut events: EventReader<GameOverKind>,
    mut state: ResMut<State<GameState>>,
    mut kind: ResMut<GameOverKind>,
) {
    if let Some(new_kind) = events.iter().next() {
        *kind = *new_kind;
        state.set(GameState::GameOver).unwrap();
    }
}

#[cfg(feature = "debug")]
fn debug_buttons(
    mut ctx: ResMut<bevy_inspector_egui::bevy_egui::EguiContext>,
    mut events: EventWriter<GameOverKind>,
) {
    use bevy_inspector_egui::egui::*;
    Area::new("gameover::debug_buttons")
        .anchor(Align2::RIGHT_BOTTOM, vec2(0., 0.))
        .show(ctx.ctx_mut(), |ui| {
            if ui.button("PlayerWon").clicked() {
                events.send(GameOverKind::PlayerWon);
            }
            if ui.button("PlayerLost").clicked() {
                events.send(GameOverKind::PlayerLost);
            }
            if ui.button("CheatSpotted").clicked() {
                events.send(GameOverKind::CheatSpotted);
            }
        });
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameOverKind::PlayerWon);
        app.insert_resource(StartTime::default());
        app.init_resource::<GameoverAssets>();
        app.add_system(enter_state);

        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(init));
        app.add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(cleanup));
        app.add_system_set(
            SystemSet::on_update(GameState::GameOver)
                .with_system(interrupt_animation)
                .with_system(animation),
        );

        app.add_event::<GameOverKind>();

        #[cfg(feature = "debug")]
        app.add_system(debug_buttons);
    }
}
