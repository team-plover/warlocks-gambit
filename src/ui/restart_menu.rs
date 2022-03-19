use super::common::{MenuCursor, UiAssets};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_ui_build_macros::{build_ui, size, style, unit};
use bevy_ui_navigation::{Focusable, NavEvent, NavRequest};

use crate::{cleanup_marked, state::GameState, EndReason, GameOver};

struct RestartAssets {
    defeat: Handle<Image>,
    victory: Handle<Image>,
}

impl FromWorld for RestartAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            defeat: assets.load("menu/ending_Defeat.png"),
            victory: assets.load("menu/ending_Victory.png"),
        }
    }
}

#[derive(Component, Clone)]
enum Button {
    Restart,
    ExitApp,
}

#[derive(Clone, Component)]
struct RestartMenuRoot;

fn handle_gameover_event(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    assets: Res<RestartAssets>,
    mut state: ResMut<State<GameState>>,
    mut events: EventReader<GameOver>,
) {
    use self::Button::{ExitApp, Restart};
    use EndReason::{CaughtCheating, Loss, Victory};
    if let Some(GameOver(reason)) = events.iter().next() {
        state.set(GameState::RestartMenu).unwrap();
        let continue_text = match *reason {
            Victory => "Congratulation! Replay?",
            Loss => "You couldn't make up the point difference! Try again?",
            CaughtCheating => "You got caught cheating! Try again?",
        };
        let won = matches!(*reason, Victory);
        let image = if won { &assets.victory } else { &assets.defeat };
        let image = ImageBundle { image: image.clone().into(), ..Default::default() };

        let node = NodeBundle {
            color: Color::NONE.into(),
            style: style! {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            },
            ..Default::default()
        };

        let focusable = Focusable::default();
        let cursor = MenuCursor::spawn_ui_element(&mut commands);
        build_ui! {
            #[cmd(commands)]
            node{ size: size!(100 pct, 100 pct) }[;Name::new("Restart Menu root"), RestartMenuRoot](
                id(cursor),
                node[;
                    UiColor(Color::rgba(0., 0., 0., 0.7)),
                    Name::new("Shadow"),
                    style! { position_type: PositionType::Absolute, size: size!(100 pct, 100 pct), }
                ],
                node[; Name::new("Menu columns")](
                    entity[image; style! { size: size!(auto, 45 pct), }],
                    entity[ui_assets.large_text(continue_text);],
                    if (cfg!(target_arch = "wasm32")) {
                        entity[ui_assets.large_text("(Press space to restart)");]
                    } else {
                        entity[ui_assets.large_text("Restart"); focusable, Restart],
                        entity[ui_assets.large_text("Exit to desktop"); focusable, ExitApp],
                    }
                )
            )
        };
    }
}

fn update(
    mut nav_events: EventReader<NavEvent>,
    mut state: ResMut<State<GameState>>,
    mut app_exit: EventWriter<AppExit>,
    buttons: Query<&Button>,
) {
    for event in nav_events.iter() {
        if let NavEvent::NoChanges { from, request: NavRequest::Action } = event {
            match buttons.get(*from.first()) {
                Ok(Button::Restart) => state.set(GameState::Playing).unwrap(),
                Ok(Button::ExitApp) => app_exit.send(AppExit),
                _ => (),
            }
        }
    }
}

fn continue_on_space(mut keys: ResMut<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if keys.just_pressed(KeyCode::Space) {
        state.set(GameState::Playing).unwrap();
        keys.reset(KeyCode::Space);
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RestartAssets>().add_event::<GameOver>();
        app.add_system(handle_gameover_event);
        app.add_system_set(
            SystemSet::on_exit(GameState::RestartMenu)
                .with_system(cleanup_marked::<RestartMenuRoot>),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::RestartMenu)
                .with_system(update)
                .with_system(continue_on_space),
        );
    }
}
