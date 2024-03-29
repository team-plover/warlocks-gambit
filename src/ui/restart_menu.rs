use super::common::{MenuCursor, UiAssets};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_ui_build_macros::{build_ui, rect, size, style, unit};
use bevy_ui_navigation::prelude::*;

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
    MainMenu,
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
    use self::Button::{ExitApp, MainMenu, Restart};
    use EndReason::{CaughtCheating, Loss, Victory};
    if let Some(GameOver(reason)) = events.iter().next() {
        state.set(GameState::RestartMenu).unwrap();
        let continue_text = match *reason {
            Victory => "Congratulation! You won!",
            Loss => "You couldn't make up the point difference!",
            CaughtCheating => "The BIRD saw you cheating!",
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
        let defeat_hint = "Having difficulties? The game rules are in the main menu.";
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
                    entity[
                        ui_assets.large_text(continue_text);
                        style! { margin: rect!(0 px, 0 px, 0 px, 60 px,), }
                    ],
                    if (matches!(*reason, Loss | CaughtCheating)) {
                        entity[
                            ui_assets.text_bundle(defeat_hint, 30.0);
                            style! { margin: rect!(0 px, 0 px, 0 px, 30 px,), }
                        ]
                    },
                    entity[ui_assets.large_text("Main menu"); focusable, MainMenu],
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
    buttons: Query<&Button>,
    mut state: ResMut<State<GameState>>,
    mut app_exit: EventWriter<AppExit>,
) {
    match nav_events.nav_iter().activated_in_query(&buttons).next() {
        Some(Button::ExitApp) => app_exit.send(AppExit),
        Some(Button::Restart) => state.set(GameState::Playing).unwrap(),
        Some(Button::MainMenu) => state.set(GameState::MainMenu).unwrap(),
        None => {}
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
        use crate::system_helper::EasySystemSetCtor;
        app.init_resource::<RestartAssets>().add_event::<GameOver>();
        app.add_system(handle_gameover_event);
        app.add_system_set(GameState::RestartMenu.on_exit(cleanup_marked::<RestartMenuRoot>));
        app.add_system_set(
            SystemSet::on_update(GameState::RestartMenu)
                .with_system(update)
                .with_system(continue_on_space),
        );
    }
}
