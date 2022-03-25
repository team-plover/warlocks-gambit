//! Ui showing game state to player during gameplay
use std::fmt::Write;

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_debug_text_overlay::screen_print;
use bevy_ui_build_macros::{build_ui, size, style, unit};
use enum_map::{enum_map, EnumMap};

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

#[derive(Component, Clone)]
struct CardEffectImage;

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
    TutoGetSeed,
    TutoUseSeed,
    TutoSleeve,
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
    use EffectEvent::*;
    for event in events.iter() {
        match event {
            TutoGetSeed | TutoUseSeed | TutoSleeve => {
                display.showing = true;
                display.timeout = time.seconds_since_startup() + 5.0;
                let txt_box = &mut description.single_mut().sections[0];
                txt_box.style.color = Color::ORANGE_RED;
                txt_box.style.font_size = 70.0;
                txt_box.value.clear();
                let text = match event {
                    TutoUseSeed => "A seed! Perfect to distract the bird\nPress space bar to use your seed",
                    TutoGetSeed => "This is unfair! The deck is stacked!\nOnly way out is cheating\nBut how? The bird is watching...",
                    TutoSleeve => "Now that the bird can't see you,\ngrab a card and slip it into your sleeve!",
                    Show(_) | UseSeed | EndCheat => "BUGBUGBUG D:",
                };
                write!(txt_box.value, "{}", text).unwrap();
            }
            UseSeed | EndCheat => {
                display.showing = true;
                display.timeout = time.seconds_since_startup() + 3.0;
                let txt_box = &mut description.single_mut().sections[0];
                txt_box.style.color = Color::ANTIQUE_WHITE;
                txt_box.style.font_size = 50.0;
                txt_box.value.clear();
                let text = match event {
                    UseSeed => "Used seed, now is the time to cheat!",
                    EndCheat => "The bird is watching again!",
                    Show(_) | TutoUseSeed | TutoSleeve | TutoGetSeed => "BUGBUGBUG D:",
                };
                write!(txt_box.value, "{}", text).unwrap();
            }
            Show(word) => {
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
            .init_resource::<EffectDisplay>()
            .add_system_set(self.0.on_enter(spawn_game_ui).with_system(reset_scores))
            .add_system(hide_effects)
            .add_system(update_score)
            .add_system_set(
                self.0
                    .on_update(update_game_ui)
                    .with_system(handle_effect_events),
            )
            .add_system_set(self.0.on_exit(despawn_game_ui));
    }
}
