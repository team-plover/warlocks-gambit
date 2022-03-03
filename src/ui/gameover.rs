use super::common::*;
use crate::state::GameState;
use bevy::prelude::*;

// To enter game over screen send GameOverKind via EventWriter.
// Upon entering screen animation is played, when it goes to RestartMenu.

// TODO: actual, non-placeholder animation
// TODO: normal game screen should be drawn when GameOver state is active

#[derive(Clone, Copy, Debug)]
pub enum GameOverKind {
    PlayerWon,
    PlayerLost,
    CheatSpotted,
}

//

/// Cleanup marker
#[derive(Component, Clone)]
struct SceneRoot;

/// Marker for 'Press ESC to exit' text
#[derive(Component)]
struct EscapeMessage;

/// Placeholder
#[derive(Component)]
struct Animation;

fn init(mut commands: Commands, kind: Res<GameOverKind>, ui_assets: Res<UiAssets>) {
    let message = match *kind {
        GameOverKind::PlayerWon => "You won",
        GameOverKind::PlayerLost => "You lost",
        GameOverKind::CheatSpotted => "You were caught cheating",
    };
    let skip_message = "Press ESCAPE to skip";

    //

    commands
        .spawn_bundle(NodeBundle {
            color: Color::NONE.into(),
            style: Style {
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,

                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,

                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SceneRoot)
        .with_children(|parent| {
            parent
                .spawn_bundle(ui_assets.large_text(message))
                .insert(Animation);
        });

    commands
        .spawn_bundle(NodeBundle { color: Color::NONE.into(), ..Default::default() })
        .insert(SceneRoot)
        .with_children(|parent| {
            parent
                .spawn_bundle(ui_assets.large_text(skip_message))
                .insert(Visibility { is_visible: false })
                .insert(EscapeMessage);
        });
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

fn placeholder_animation(mut entities: Query<&mut Text, With<Animation>>, time: Res<Time>) {
    fn sine_animation(v0: f32, v1: f32, period_seconds: f32, time: &Time) -> f32 {
        let t = time.seconds_since_startup() as f32 / period_seconds;
        let t = (t * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        v0 * (1. - t) + v1 * t
    }

    for mut text in entities.iter_mut() {
        let t = sine_animation(0.2, 1., 2.5, &time);
        text.sections.get_mut(0).unwrap().style.color.set_r(t);
    }
}

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
        app.add_system(enter_state.label("check_gameover"));

        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(init));
        app.add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(cleanup));
        app.add_system_set(
            SystemSet::on_update(GameState::GameOver)
                .with_system(interrupt_animation)
                .with_system(placeholder_animation),
        );

        app.add_event::<GameOverKind>();

        #[cfg(feature = "debug")]
        app.add_system(debug_buttons);
    }
}
