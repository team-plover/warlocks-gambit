use bevy::prelude::*;

mod animate;
mod audio;
mod card;
mod card_effect;
mod cheat;
#[cfg(feature = "debug")] // only include if compiling in debug mode
mod debug_overlay;
mod deck;
mod game_ui;
mod gltf_hook;
mod oppo_hand;
mod pile;
mod player_hand;
mod scene;
mod state;
mod ui;
mod war;

#[cfg(not(feature = "debug"))] // add a dummy to make sure code doesn't break
#[macro_export]
macro_rules! add_dbg_text {
    ($($whatever:tt)*) => {};
}

mod camera {
    use bevy::prelude::Component;

    #[derive(Component)]
    pub struct PlayerCam;
}
// TODO: rename this to reflect content
mod card_spawner {
    use super::Participant;
    use bevy::prelude::Component;

    #[derive(Default)]
    pub struct GameStarts(pub u32);

    #[derive(Component)]
    pub struct GrabbedCard;

    #[derive(Component)]
    pub struct BirdPupilRoot;

    #[derive(Component)]
    pub struct BirdPupil;

    /// Card in the War pile played by the player
    #[derive(Component)]
    pub struct PlayedCard;

    #[derive(Component)]
    pub struct CardOrigin(pub Participant);

    /// Component attached to where the opponent draws cards from.
    #[derive(Component)]
    pub struct OppoCardSpawner;

    /// Component attached to where the player draws cards from.
    #[derive(Component)]
    pub struct PlayerCardSpawner;

    /// Where to stash cards added to sleeve
    #[derive(Component)]
    pub struct PlayerSleeve;

    /// Position of the hand of the player
    #[derive(Component)]
    pub struct PlayerHand;

    /// Position of the hand of the opposition
    #[derive(Component)]
    pub struct OppoHand;
}

#[derive(Clone, Copy, PartialEq)]
pub enum Participant {
    Player,
    Oppo,
}

use state::{GameState, TurnState};

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            #[cfg(target_os = "linux")]
            vsync: false, // workaround for https://github.com/bevyengine/bevy/issues/1908 (seems to be Mesa bug with X11 + Vulkan)
            ..Default::default()
        })
        .add_state(GameState::MainMenu)
        .add_state(TurnState::Starting)
        .add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new())
        .add_plugin(debug_overlay::Plugin);

    app.insert_resource(ClearColor(Color::rgb(0.293, 0.3828, 0.4023)))
        .init_resource::<card_spawner::GameStarts>()
        .add_plugin(player_hand::Plugin(GameState::Playing))
        .add_plugin(oppo_hand::Plugin(GameState::Playing))
        .add_plugin(scene::Plugin(GameState::LoadScene))
        .add_plugin(deck::Plugin(GameState::Playing))
        .add_plugin(animate::Plugin)
        .add_plugin(cheat::Plugin(GameState::Playing))
        .add_plugin(audio::Plugin)
        .add_plugin(card::Plugin)
        .add_plugin(ui::common::Plugin)
        .add_plugin(ui::main_menu::Plugin(GameState::MainMenu))
        .add_plugin(ui::gameover::Plugin)
        .add_plugin(ui::restart_menu::Plugin)
        //.add_plugin(ui::pause_menu::Plugin) // TODO: disable player control for this state
        .add_plugin(pile::Plugin(GameState::Playing))
        .add_plugin(card_effect::Plugin(GameState::Playing))
        .add_plugin(game_ui::Plugin(GameState::Playing))
        .add_system(first_draw.with_run_criteria(State::on_enter(GameState::Playing)))
        .add_startup_system(setup);

    app.run();
}

fn setup(
    mut ambiant_light: ResMut<AmbientLight>,
    mut audio_events: EventWriter<audio::AudioRequest>,
) {
    *ambiant_light = AmbientLight { color: Color::WHITE, brightness: 1.0 };
    audio_events.send(audio::AudioRequest::StartMusic);
}

fn first_draw(
    mut starts: ResMut<card_spawner::GameStarts>,
    mut game_msgs: EventWriter<game_ui::EffectEvent>,
    mut state: ResMut<State<TurnState>>,
) {
    if starts.0 == 1 {
        game_msgs.send(game_ui::EffectEvent::TutoGetSeed);
    }
    starts.0 += 1;
    state.set(TurnState::Draw).unwrap();
}
