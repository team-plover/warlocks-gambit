//! Ui showing game state to player during gameplay
use std::fmt::Write;

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_build_macros::{build_ui, size, style, unit};
use enum_map::{enum_map, EnumMap};

use crate::{
    card::WordOfPower,
    card_effect::{CardStats, SeedCount, TurnCount},
    state::{GameState, TurnState},
};

#[derive(Component, Clone)]
struct UiRoot;

#[derive(Component, Clone)]
struct CardEffectDescription;

#[derive(Component, Clone)]
struct CardEffectImage;

#[derive(Component, Clone)]
enum UiInfo {
    Seeds,
    Playing,
    Turns,
    PlayerScore,
    OppoScore,
    CardsLeft,
}

struct UiAssets {
    font: Handle<Font>,
    words: EnumMap<WordOfPower, Handle<Image>>,
}
impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            font: assets.load("Boogaloo-Regular.otf"),
            words: enum_map! { word => assets.load(&format!("cards/Word{word:?}.png")) },
        }
    }
}

fn spawn_game_ui(mut cmds: Commands, ui_assets: Res<UiAssets>) {
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
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        },
        ..Default::default()
    };
    build_ui! {
        #[cmd(cmds)]
        node{ size: size!(100 pct, 100 pct) }[; UiRoot](
            node{ size: size!(20 pct, 100 pct) },
            node{
                size: size!(60 pct, 100 pct),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center
            }[; Name::new("game ui effect display")](
                    node[
                        ImageBundle::default();
                        style! { max_size: size!(470 px, 200 px), },
                        Visibility { is_visible: false },
                        CardEffectImage
                    ],
                    entity[text(""); CardEffectDescription]
            ),
            node{
                size: size!(20 pct, 100 pct),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexEnd
            }[;Name::new("game ui right column")](
                node[; Name::new("Seeds")](
                    node[text("Seeds: ");],
                    node[text("0"); UiInfo::Seeds]
                ),
                node[; Name::new("Playing")](
                    node[text("Turn: ");],
                    node[text("Player"); UiInfo::Playing]
                ),
                node[; Name::new("Turn Count")](
                    node[text("Turn: ");],
                    node[text("1"); UiInfo::Turns]
                ),
                node[; Name::new("Player score")](
                    node[text("Player: ");],
                    node[text("0"); UiInfo::PlayerScore]
                ),
                node[; Name::new("Oppo score")](
                    node[text("Oppo: ");],
                    node[text("0"); UiInfo::OppoScore]
                ),
                node[; Name::new("Cards remaining")](
                    node[text("Cards remaining: ");],
                    node[text("60"); UiInfo::CardsLeft]
                )
            )
        )
    };
}

fn despawn_game_ui(mut cmds: Commands, query: Query<Entity, With<UiRoot>>) {
    cmds.entity(query.single()).despawn_recursive();
}

#[derive(PartialEq)]
pub enum EffectEvent {
    Show(WordOfPower),
    UseSeed,
    EndCheat,
}

/// Show effect description on screen
///
/// NOTE: to remove it, you need to set the `timeout` to lower than current time
#[derive(Default)]
struct EffectDisplay {
    showing: bool,
    timeout: f64,
}

fn hide_effects(
    time: Res<Time>,
    mut display: ResMut<EffectDisplay>,
    mut image: Query<&mut Visibility, With<CardEffectImage>>,
    mut description: Query<&mut Text, With<CardEffectDescription>>,
) {
    if display.showing && display.timeout <= time.seconds_since_startup() {
        if let Ok(mut img) = image.get_single_mut() {
            img.is_visible = false;
        }
        if let Ok(mut txt) = description.get_single_mut() {
            txt.sections[0].value.clear();
        }
        display.showing = false;
    }
}

fn handle_effect_events(
    mut events: EventReader<EffectEvent>,
    mut display: ResMut<EffectDisplay>,
    mut image: Query<(&mut UiImage, &mut Visibility), With<CardEffectImage>>,
    mut description: Query<&mut Text, With<CardEffectDescription>>,
    ui_assets: Res<UiAssets>,
    time: Res<Time>,
) {
    for event in events.iter() {
        match event {
            EffectEvent::UseSeed | EffectEvent::EndCheat => {
                display.showing = true;
                display.timeout = time.seconds_since_startup() + 1.5;
                let txt_box = &mut description.single_mut().sections[0];
                txt_box.style.color = Color::ANTIQUE_WHITE;
                txt_box.style.font_size = 50.0;
                txt_box.value.clear();
                let text = match event {
                    EffectEvent::UseSeed => "Used seed, now is the time to cheat!",
                    EffectEvent::EndCheat => "The bird is watching again!",
                    EffectEvent::Show(_) => "BUGBUGBUG D:",
                };
                write!(txt_box.value, "{}", text).unwrap();
            }
            EffectEvent::Show(word) => {
                display.showing = true;
                display.timeout = time.seconds_since_startup() + 1.5;
                let txt_box = &mut description.single_mut().sections[0];
                txt_box.style.color = word.color();
                txt_box.style.font_size = 60.0;
                txt_box.value.clear();
                write!(txt_box.value, "{}", word.flavor_text()).unwrap();
                let (mut img, mut visibility) = image.single_mut();
                img.0 = ui_assets.words[*word].clone();
                visibility.is_visible = true;
            }
        }
    }
}

fn update_game_ui(
    mut ui_infos: Query<(&mut Text, &UiInfo)>,
    turn_state: Res<State<TurnState>>,
    turn_counter: Res<TurnCount>,
    player_seeds: Res<SeedCount>,
    stats: CardStats,
) {
    let player_score = stats.player_score();
    let oppo_score = stats.oppo_score();
    let total_cards = stats.cards_remaining();
    for (mut text, ui_info) in ui_infos.iter_mut() {
        let txt = &mut text.sections[0].value;
        txt.clear();
        match ui_info {
            UiInfo::Seeds => {
                let seeds = player_seeds.count();
                write!(txt, "{seeds}").unwrap();
            }
            UiInfo::Playing => {
                let turn = turn_state.current();
                write!(txt, "{turn:?}").unwrap();
            }
            UiInfo::OppoScore => {
                write!(txt, "{oppo_score}").unwrap();
            }
            UiInfo::PlayerScore => {
                write!(txt, "{player_score}").unwrap();
            }
            UiInfo::CardsLeft => {
                write!(txt, "{total_cards}").unwrap();
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
            .add_event::<EffectEvent>()
            .init_resource::<EffectDisplay>()
            .add_system_set(SystemSet::on_enter(self.0).with_system(spawn_game_ui))
            .add_system(hide_effects)
            .add_system_set(
                SystemSet::on_update(self.0)
                    .with_system(update_game_ui)
                    .with_system(handle_effect_events),
            )
            .add_system_set(SystemSet::on_exit(self.0).with_system(despawn_game_ui));
    }
}
