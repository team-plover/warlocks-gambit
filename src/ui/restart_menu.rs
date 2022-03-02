use super::common::*;
use super::gameover::GameOverKind;
use crate::state::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_ui_build_macros::{build_ui, size, style, unit};
use bevy_ui_navigation::{Focusable, NavEvent, NavRequest};

#[derive(Component, Clone)]
enum Button {
    Restart,
    MainMenu,
    ExitApp,
}

fn init(mut commands: Commands, ui_assets: Res<UiAssets>, kind: Res<GameOverKind>) {
    let continue_text = match *kind {
        GameOverKind::PlayerWon => "New game",
        GameOverKind::PlayerLost | GameOverKind::CheatSpotted => "Restart",
    };

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

    build_ui! {
        #[cmd(commands)]
        node{ min_size: size!(100 pct, 100 pct) }[;Name::new("root node"), MenuRoot](
            node{ position_type: PositionType::Absolute }[;
                UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
                MenuCursor::default(),
                Name::new("Cursor")
            ],
            node[; Name::new("Menu columns")](
                node[ui_assets.large_text(continue_text); Focusable::default(), Button::Restart],
                node[ui_assets.large_text("Exit to main menu"); Focusable::default(), Button::MainMenu],
                node[ui_assets.large_text("Exit to desktop"); Focusable::default(), Button::ExitApp]
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
                Ok(Button::MainMenu) => state.set(GameState::MainMenu).unwrap(),
                Ok(Button::ExitApp) => app_exit.send(AppExit),
                _ => (),
            }
        }
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::RestartMenu).with_system(init));
        app.add_system_set(SystemSet::on_exit(GameState::RestartMenu).with_system(exit_menu));
        app.add_system_set(SystemSet::on_update(GameState::RestartMenu).with_system(update));
    }
}
