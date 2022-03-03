//! Ui showing game state to player during gameplay
use std::fmt::Write;

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_build_macros::{build_ui, size, style, unit};

use crate::card_effect::TurnCount;
use crate::state::{GameState, TurnState};

#[derive(Component, Clone)]
struct UiRoot;

#[derive(Component, Clone)]
enum UiInfo {
    Playing,
    Turns,
    PlayerScore,
    OppoScore,
    CardsLeft,
}

struct UiAssets {
    font: Handle<Font>,
}
impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self { font: assets.load("Boogaloo-Regular.otf") }
    }
}

fn spawn_game_ui(mut cmds: Commands, ui_assets: Res<UiAssets>) {
    use FlexDirection as FD;

    let text = |content: &str| {
        let color = Color::ANTIQUE_WHITE;
        let horizontal = HorizontalAlign::Left;
        let style = TextStyle {
            color,
            font: ui_assets.font.clone(),
            font_size: 60.0,
        };
        let align = TextAlignment { horizontal, ..Default::default() };
        let text = Text::with_section(content, style, align);
        TextBundle { text, ..Default::default() }
    };
    let node = NodeBundle {
        color: Color::NONE.into(),
        style: style! {
            display: Display::Flex,
            flex_direction: FD::ColumnReverse,
            align_items: AlignItems::Center,
        },
        ..Default::default()
    };
    build_ui! {
        #[cmd(cmds)]
        node{
            min_size: size!(100 pct, 100 pct),
            display: Display::Flex,
            align_items: AlignItems::FlexEnd
        }[;Name::new("game ui root node"), UiRoot](
            node{ flex_direction: FD::Row }[; Name::new("Playing")](
                node[text("Turn: ");],
                node[text("Player"); UiInfo::Playing]
            ),
            node{ flex_direction: FD::Row }[; Name::new("Turn Count")](
                node[text("Turn: ");],
                node[text("1"); UiInfo::Turns]
            ),
            node{ flex_direction: FD::Row }[; Name::new("Player score")](
                node[text("Player: ");],
                node[text("0"); UiInfo::PlayerScore]
            ),
            node{ flex_direction: FD::Row }[; Name::new("Oppo score")](
                node[text("Oppo: ");],
                node[text("0"); UiInfo::OppoScore]
            ),
            node{ flex_direction: FD::Row }[; Name::new("Cards remaining")](
                node[text("Cards remaining: ");],
                node[text("60"); UiInfo::CardsLeft]
            )
        )
    };
}

fn despawn_game_ui(mut cmds: Commands, query: Query<Entity, With<UiRoot>>) {
    cmds.entity(query.single()).despawn_recursive();
}

fn update_game_ui(
    mut ui_infos: Query<(&mut Text, &UiInfo)>,
    turn_state: Res<State<TurnState>>,
    turn_counter: Res<TurnCount>,
) {
    for (mut text, ui_info) in ui_infos.iter_mut() {
        let txt = &mut text.sections[0].value;
        txt.clear();
        match ui_info {
            UiInfo::Playing => {
                use TurnState::*;
                let turn = match turn_state.current() {
                    Player => "Player",
                    New => "Waiting",
                    Oppo => "Oppo",
                    PlayerActivated | OppoActivated => "Playing card",
                };
                write!(txt, "{turn}").unwrap();
            }
            UiInfo::OppoScore => {
                write!(txt, "0").unwrap();
            }
            UiInfo::PlayerScore => {
                write!(txt, "0").unwrap();
            }
            UiInfo::CardsLeft => {
                write!(txt, "60").unwrap();
            }
            UiInfo::Turns => {
                write!(txt, "{}", turn_counter.0).unwrap();
            }
        }
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAssets>()
            .add_system_set(SystemSet::on_enter(self.0).with_system(spawn_game_ui))
            .add_system_set(SystemSet::on_update(self.0).with_system(update_game_ui))
            .add_system_set(SystemSet::on_exit(self.0).with_system(despawn_game_ui));
    }
}
