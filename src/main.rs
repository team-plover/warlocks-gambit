//! Warlock's gambit.
//!
//! # Architecture
//!
//! The most important module is probably [`game_flow`] where the game logic is
//! defined. Other modules are mostly helpers for input and ai. [See module
//! section](#Modules).
use bevy::prelude::*;

mod animate;
mod audio;
mod card;
mod cheat;
mod deck;
mod game_flow;
mod game_ui;
mod numbers;
mod oppo_hand;
mod pile;
mod player_hand;
mod scene;
mod state;
mod system_helper;
mod ui;
mod war;

use state::{GameState, TurnState};
use bevy_scene_hook::HookedSceneState;

#[derive(Clone, Copy, PartialEq)]
pub enum Participant {
    Player,
    Oppo,
}
impl Participant {
    pub fn color(&self) -> Color {
        match self {
            Participant::Player => Color::rgb_u8(77, 77, 208),
            Participant::Oppo => Color::RED,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::Player => "Player",
            Self::Oppo => "Oppo",
        }
    }
}

/// Event to trigger a game over.
#[derive(Debug)]
pub struct GameOver(pub EndReason);

/// What triggered the game over.
#[derive(Debug)]
pub enum EndReason {
    Victory,
    Loss,
    CaughtCheating,
}

#[derive(Component)]
pub struct CardOrigin(pub Participant);

#[derive(Component, Clone)]
struct WaitRoot;

fn main() {
    use system_helper::EasySystemSetCtor;

    let mut app = App::new();

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            #[cfg(target_os = "linux")]
            // workaround for https://github.com/bevyengine/bevy/issues/1908 (seems to be Mesa bug with X11 + Vulkan)
            present_mode: bevy::window::PresentMode::Immediate, 
            ..default()
        })
        .add_state(GameState::MainMenu)
        .add_state(TurnState::Starting)
        .add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new())
        .add_plugin(bevy::pbr::wireframe::WireframePlugin)
        .insert_resource(bevy::render::settings::WgpuSettings {
            features: bevy::render::render_resource::WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        });

    app.insert_resource(ClearColor(Color::rgb(0.293, 0.3828, 0.4023)))
        .add_plugin(numbers::Plugin)
        .add_plugin(bevy_scene_hook::HookPlugin)
        .add_plugin(bevy_debug_text_overlay::OverlayPlugin::default())
        .add_plugin(player_hand::Plugin(GameState::Playing))
        .add_plugin(oppo_hand::Plugin(GameState::Playing))
        .add_plugin(scene::Plugin)
        .add_plugin(deck::Plugin(GameState::Playing))
        .add_plugin(animate::Plugin)
        .add_plugin(cheat::Plugin(GameState::Playing))
        .add_plugin(audio::Plugin)
        .add_plugin(card::Plugin)
        .add_plugin(ui::Plugin)
        .add_plugin(pile::Plugin(GameState::Playing))
        .add_plugin(game_flow::Plugin(GameState::Playing))
        .add_plugin(game_ui::Plugin(GameState::Playing))
        .add_system_set(GameState::Playing.on_enter(first_draw))
        .add_system_set(GameState::WaitLoaded.on_enter(setup_load_screen))
        .add_system_set(GameState::WaitLoaded.on_update(complete_load_screen))
        .add_system_set(GameState::WaitLoaded.on_exit(cleanup_marked::<WaitRoot>))
        .add_startup_system(setup);

    app.run();
}

pub fn cleanup_marked<T: Component>(mut cmds: Commands, query: Query<Entity, With<T>>) {
    use bevy_debug_text_overlay::screen_print;
    screen_print!(sec: 3.0, "Cleaned up Something (can't show)");
    for entity in query.iter() {
        cmds.entity(entity).despawn_recursive();
    }
}

fn setup(
    mut ambiant_light: ResMut<AmbientLight>,
    mut audio_events: EventWriter<audio::AudioRequest>,
) {
    *ambiant_light = AmbientLight { color: Color::WHITE, brightness: 1.0 };
    audio_events.send(audio::AudioRequest::StartMusic);
}

fn complete_load_screen(
    mut state: ResMut<State<GameState>>,
    scene: HookedSceneState<scene::Graveyard>,
) {
    if scene.is_loaded() {
        state.set(GameState::Playing).expect("no state issues");
    }
}
fn setup_load_screen(
    mut cmds: Commands,
    assets: Res<ui::Assets>,
    scene: HookedSceneState<scene::Graveyard>,
) {
    use bevy_ui_build_macros::{build_ui, size, style, unit};
    if !scene.is_loaded() {
        let node = NodeBundle::default();
        build_ui! {
            #[cmd(cmds)]
            node {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                size: size!(100 pct, 100 pct)
            }[; Name::new("Root loading screen node"), WaitRoot] (
                entity[ assets.background(); Name::new("Background") ],
                entity[assets.large_text("Loading..."); ]
            )
        };
    }
}

fn first_draw(mut state: ResMut<State<TurnState>>) {
    state.set(TurnState::Draw).unwrap();
}
