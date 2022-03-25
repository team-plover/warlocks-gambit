//! Ui showing game state to player during gameplay
use std::fmt::Write;

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_debug_text_overlay::screen_print;
use bevy_ui_build_macros::{build_ui, rect, size, style, unit};

use crate::{
    animate::Animated,
    game_flow::{CardStats, SeedCount},
    numbers::Number,
    state::GameState,
    war::WordOfPower,
    Participant,
};

#[derive(Component, Clone)]
struct UiRoot;

#[derive(Component)]
pub struct PlayerScore;

#[derive(Component)]
pub struct OppoScore;

#[derive(Component, Clone)]
struct CardEffectDescription;

pub enum ScoreEvent {
    Add(Participant, i32),
    Reset,
}
#[derive(Component, Clone)]
enum UiInfo {
    Seeds,
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
    let text = |content: &str| {
        let color = Color::NAVY;
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
                padding: rect!(auto, 7 pct),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center
            }[; Name::new("game ui effect display")](
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
    Hide,
    UseSeed,
    EndCheat,
}

fn handle_effect_events(
    mut events: EventReader<EffectEvent>,
    mut description: Query<&mut Text, With<CardEffectDescription>>,
) {
    use EffectEvent::*;
    for event in events.iter() {
        match event {
            Hide => {
                let txt_box = &mut description.single_mut().sections[0];
                txt_box.value.clear();
            }
            UseSeed | EndCheat => {
                let txt_box = &mut description.single_mut().sections[0];
                txt_box.style.color = Color::ANTIQUE_WHITE;
                txt_box.style.font_size = 50.0;
                txt_box.value.clear();
                let text = match event {
                    UseSeed => "Used seed, now is the time to cheat!",
                    EndCheat => "The bird is watching again!",
                    Show(_) | Hide => "BUGBUGBUG D:",
                };
                write!(txt_box.value, "{}", text).unwrap();
            }
            Show(word) => {
                let txt_box = &mut description.single_mut().sections[0];
                txt_box.style.color = word.color();
                txt_box.style.font_size = 60.0;
                txt_box.value.clear();
                write!(txt_box.value, "{}", word.flavor_text()).unwrap();
            }
        }
    }
}

type ScoreComponents = (Entity, &'static mut Number);
fn update_score(
    mut player_score: Query<ScoreComponents, With<PlayerScore>>,
    mut oppo_score: Query<ScoreComponents, (With<OppoScore>, Without<PlayerScore>)>,
    mut events: EventReader<ScoreEvent>,
    mut cmds: Commands,
    stats: CardStats,
) {
    for event in events.iter() {
        match event {
            ScoreEvent::Add(participant, additional) => {
                let ((entity, mut number), score) = match *participant {
                    Participant::Oppo => (oppo_score.single_mut(), stats.oppo_score()),
                    Participant::Player => (player_score.single_mut(), stats.player_score()),
                };
                number.value = score;
                cmds.entity(entity).with_children(|cmds| {
                    cmds.spawn_bundle((
                        Animated::RiseAndFade { duration: 1.2, direction: Vec3::Y * 2.5 },
                        Number::new(*additional, participant.color()),
                        Transform::from_translation(Vec3::Y * 2.),
                        GlobalTransform::default(),
                    ));
                });
            }
            ScoreEvent::Reset => {
                screen_print!("Resetting scores!");
                let (_, mut score) = oppo_score.single_mut();
                score.value = 0;
                let (_, mut score) = player_score.single_mut();
                score.value = 0;
            }
        }
    }
}

fn update_game_ui(
    mut ui_infos: Query<(&mut Text, &UiInfo)>,
    player_seeds: Res<SeedCount>,
    stats: CardStats,
) {
    screen_print!("values left: {}", stats.remaining_score());
    for (mut text, ui_info) in ui_infos.iter_mut() {
        let txt = &mut text.sections[0].value;
        txt.clear();
        match ui_info {
            UiInfo::Seeds => {
                let seeds = player_seeds.count();
                write!(txt, "{seeds}").unwrap();
            }
        }
    }
}
fn reset_scores(mut events: EventWriter<ScoreEvent>) {
    events.send(ScoreEvent::Reset);
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use crate::system_helper::EasySystemSetCtor;
        app.init_resource::<UiAssets>()
            .add_event::<EffectEvent>()
            .add_event::<ScoreEvent>()
            .add_system_set(self.0.on_enter(spawn_game_ui).with_system(reset_scores))
            .add_system(update_score)
            .add_system_set(
                self.0
                    .on_update(update_game_ui)
                    .with_system(handle_effect_events),
            )
            .add_system_set(self.0.on_exit(despawn_game_ui));
    }
}
