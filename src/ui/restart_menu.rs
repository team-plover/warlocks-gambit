use super::gameover::GameOverKind;
use super::{common::*, gameover::GameoverAssets};
use crate::state::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_ui_build_macros::{build_ui, size, style, unit};
use bevy_ui_navigation::{Focusable, NavEvent, NavRequest};

#[derive(Component, Clone)]
enum Button {
    Restart,
    ExitApp,
}

fn init(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    kind: Res<GameOverKind>,
    images: Res<GameoverAssets>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    let continue_text = match *kind {
        GameOverKind::PlayerWon => "New game",
        GameOverKind::PlayerLost | GameOverKind::CheatSpotted => "Restart",
    };
    #[cfg(target_arch = "wasm32")]
    let continue_text = match *kind {
        GameOverKind::PlayerWon => "Press SPACE to start new game",
        GameOverKind::PlayerLost | GameOverKind::CheatSpotted => "Press SPACE to restart",
    };
    let image = match *kind {
        GameOverKind::PlayerWon => images.victory.clone(),
        GameOverKind::PlayerLost | GameOverKind::CheatSpotted => images.defeat.clone(),
    }
    .into();

    //

    let node = NodeBundle {
        color: Color::NONE.into(),
        style: style! {
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,

            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
        },
        ..Default::default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    build_ui! {
        #[cmd(commands)]
        node{ min_size: size!(100 pct, 100 pct) }[;Name::new("root node"), MenuRoot](
            node{ position_type: PositionType::Absolute, size: Size::new(Val::Percent(0.), Val::Percent(0.)) }[;
                UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
                MenuCursor::default(),
                Name::new("Cursor")
            ],
            node{ position_type: PositionType::Absolute }[;
                UiColor(Color::rgba(0., 0., 0., 0.7)),
                Name::new("'Shadow'"),
                style! { size: size!(100 pct, 100 pct), }
            ],
            node[; Name::new("Menu columns")](
                node[
                    ImageBundle { image, ..Default::default() };
                    style! { size: size!(auto, 30 pct), }
                ],
                node[ui_assets.large_text(continue_text); Focusable::default(), Button::Restart],
                node[ui_assets.large_text("Exit to desktop"); Focusable::default(), Button::ExitApp]
            )
        )
    };

    // TODO: this is copied from code above with few minor changes
    #[cfg(target_arch = "wasm32")]
    build_ui! {
        #[cmd(commands)]
        node{ min_size: size!(100 pct, 100 pct) }[;Name::new("root node"), MenuRoot](
            node{ position_type: PositionType::Absolute, size: Size::new(Val::Percent(0.), Val::Percent(0.)) }[;
                UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
                MenuCursor::default(),
                Name::new("Cursor")
            ],
            node{ position_type: PositionType::Absolute }[;
                UiColor(Color::rgba(0., 0., 0., 0.7)),
                Name::new("'Shadow'"),
                style! { size: size!(100 pct, 100 pct), }
            ],
            node[; Name::new("Menu columns")](
                node[
                    ImageBundle { image, ..Default::default() };
                    style! { size: size!(auto, 30 pct), }
                ],
                node[ui_assets.large_text(continue_text);]
            )
        )
    };
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
        app.add_system_set(SystemSet::on_enter(GameState::RestartMenu).with_system(init));
        app.add_system_set(SystemSet::on_exit(GameState::RestartMenu).with_system(exit_menu));
        app.add_system_set(
            SystemSet::on_update(GameState::RestartMenu)
                .with_system(update)
                .with_system(continue_on_space),
        );
    }
}
