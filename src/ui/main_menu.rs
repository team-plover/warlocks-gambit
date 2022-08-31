use super::common::{MenuCursor, UiAssets};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::{app::AppExit, input::mouse::MouseMotion, window::WindowMode};
use bevy_debug_text_overlay::screen_print;
use bevy_ui_build_macros::{build_ui, rect, size, style, unit};
use bevy_ui_navigation::prelude::*;

use crate::{
    audio::{AudioChannel, AudioRequest, AudioRequestSystem, SfxParam},
    cleanup_marked,
    state::GameState,
};

#[derive(Component)]
struct MovingSlider;

#[derive(Component, Clone)]
struct RulesOverlay;

#[derive(Component, Clone)]
struct CreditOverlay;

#[derive(Clone, Component)]
struct MainMenuRoot;

#[derive(Component, Clone, PartialEq)]
enum MainMenuElem {
    Start,
    Exit,
    Credits,
    Rules,
    LockMouse,
    ToggleFullScreen,
    Set16_9,
    AudioSlider(AudioChannel, f64),
}

pub struct MenuAssets {
    team_name: Handle<Image>,
    title_image: Handle<Image>,
    slider_handle: Handle<Image>,
    slider_bg: Handle<Image>,
}
impl FromWorld for MenuAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            team_name: assets.load("team_name.png"),
            title_image: assets.load("title_image.png"),
            slider_bg: assets.load("slider_bg.png"),
            slider_handle: assets.load("slider_handle.png"),
        }
    }
}

fn update_sliders(
    mut styles: Query<(Entity, &mut Style, &mut MainMenuElem), With<MovingSlider>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cmds: Commands,
    mut audio_requests: EventWriter<AudioRequest>,
    mut nav_requests: EventWriter<NavRequest>,
    focused: Query<Entity, With<Focused>>,
    elems: Query<&MainMenuElem, Without<MovingSlider>>,
    mut mouse_buttons: ResMut<Input<MouseButton>>,
) {
    use MainMenuElem::AudioSlider;
    if let Ok((entity, mut style, mut elem)) = styles.get_single_mut() {
        if let (Val::Percent(left), AudioSlider(channel, strength)) =
            (style.position.left, elem.as_mut())
        {
            let horizontal_delta: f64 = mouse_motion.iter().map(|m| m.delta.x as f64).sum();
            let new_left = (left as f64 / 0.9 + horizontal_delta * 0.40)
                .min(100.0)
                .max(0.0);
            *strength = new_left;
            audio_requests.send(AudioRequest::SetVolume(*channel, new_left / 100.0));
            style.position.left = Val::Percent(new_left as f32 * 0.9)
        };
        if mouse_buttons.just_released(MouseButton::Left) {
            mouse_buttons.clear_just_released(MouseButton::Left);
            nav_requests.send(NavRequest::Unlock);
            audio_requests.send(AudioRequest::StopSfxLoop);
            cmds.entity(entity).remove::<MovingSlider>();
        }
    }
    if let Ok(entity) = focused.get_single() {
        let is_volume_slider = matches!(elems.get(entity), Ok(AudioSlider(..)));
        if mouse_buttons.just_pressed(MouseButton::Left) && is_volume_slider {
            nav_requests.send(NavRequest::Action);
            audio_requests.send(AudioRequest::PlayWoodClink(SfxParam::StartLoop));
            cmds.entity(entity).insert(MovingSlider);
        }
    }
}

fn update_menu(
    mut events: EventReader<NavEvent>,
    mut exit: EventWriter<AppExit>,
    mut cmds: Commands,
    mut audio_requests: EventWriter<AudioRequest>,
    mut windows: ResMut<Windows>,
    mut credit_overlay: Query<&mut Style, With<CreditOverlay>>,
    mut rules_overlay: Query<&mut Style, (Without<CreditOverlay>, With<RulesOverlay>)>,
    mut game_state: ResMut<State<GameState>>,
    elems: Query<&MainMenuElem>,
) {
    use NavEvent::{FocusChanged, Locked, NoChanges};
    use NavRequest::Action;
    let window_msg = "There is at least one game window open";
    for (event_type, from) in events.nav_iter().types() {
        match (event_type, elems.get(from)) {
            (FocusChanged { .. }, Ok(MainMenuElem::AudioSlider(..))) => {
                cmds.entity(from).remove::<MovingSlider>();
            }
            (Locked(..), Ok(MainMenuElem::Credits)) => {
                let mut style = credit_overlay.single_mut();
                style.display = Display::Flex;
            }
            (Locked(..), Ok(MainMenuElem::Rules)) => {
                let mut style = rules_overlay.single_mut();
                style.display = Display::Flex;
            }
            (NoChanges { request: Action, .. }, Ok(MainMenuElem::Exit)) => exit.send(AppExit),
            (NoChanges { request: Action, .. }, Ok(MainMenuElem::Start)) => {
                screen_print!("Player pressed the start button");
                audio_requests.send(AudioRequest::PlayWoodClink(SfxParam::PlayOnce));
                game_state.set(GameState::WaitLoaded).unwrap();
            }
            (NoChanges { request: Action, .. }, Ok(MainMenuElem::LockMouse)) => {
                let window = windows.get_primary_mut().expect(window_msg);
                let prev_lock_mode = window.cursor_locked();
                window.set_cursor_lock_mode(!prev_lock_mode);
            }
            (NoChanges { request: Action, .. }, Ok(MainMenuElem::ToggleFullScreen)) => {
                use WindowMode::*;
                let window = windows.get_primary_mut().expect(window_msg);
                let new_mode = if window.mode() == BorderlessFullscreen {
                    Windowed
                } else {
                    BorderlessFullscreen
                };
                window.set_mode(new_mode);
            }
            (NoChanges { request: Action, .. }, Ok(MainMenuElem::Set16_9)) => {
                let window = windows.get_primary_mut().expect(window_msg);
                if window.mode() == WindowMode::Windowed {
                    let height = window.height();
                    window.set_resolution(height * 16.0 / 9.0, height);
                }
            }
            (NavEvent::Unlocked(..), _) => {}
            (_, Err(err)) => {
                println!("error in main_menu update: {err:?}");
            }
            _ => {}
        }
    }
}

#[allow(clippy::type_complexity)]
fn leave_overlay(
    mut overlay: Query<&mut Style, Or<(With<CreditOverlay>, With<RulesOverlay>)>>,
    mut nav_requests: EventWriter<NavRequest>,
    gamepad: Res<Input<GamepadButton>>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if gamepad.get_just_pressed().len() != 0
        || mouse.get_just_pressed().len() != 0
        || keyboard.get_just_pressed().len() != 0
    {
        for mut style in overlay.iter_mut() {
            if style.display == Display::Flex {
                style.display = Display::None;
                nav_requests.send(NavRequest::Unlock)
            }
        }
    }
}

/// Spawns the UI tree
fn setup_main_menu(mut cmds: Commands, menu_assets: Res<MenuAssets>, ui_assets: Res<UiAssets>) {
    use FlexDirection as FD;
    use MainMenuElem::*;
    use PositionType as PT;

    let text_bundle = |content: &str, font_size: f32| ui_assets.text_bundle(content, font_size);
    let large_text = |content| ui_assets.large_text(content);
    let focusable = Focusable::default();
    let image =
        |image: &Handle<Image>| ImageBundle { image: image.clone().into(), ..Default::default() };
    let node = NodeBundle {
        color: Color::NONE.into(),
        style: style! {
            display: Display::Flex,
            flex_direction: FD::ColumnReverse,
            align_items: AlignItems::Center,
        },
        ..Default::default()
    };
    let mut slider = |name: &str, channel: AudioChannel, strength: f64| {
        let volume_name = name.to_string() + " volume";
        let handle_name = Name::new(name.to_string() + " volume slider handle");
        let slider_name = Name::new(name.to_string() + " volume slider");
        let position = UiRect {
            bottom: Val::Px(-10.0),
            left: Val::Percent(strength as f32 * 0.9),
            ..Default::default()
        };
        build_ui! {
            #[cmd(cmds)]
            node { flex_direction: FD::Row }[; slider_name](
                node[text_bundle(&volume_name, 30.0); style! { margin: rect!(10 px), }],
                node(
                    entity[
                        image(&menu_assets.slider_bg);
                        style! { size: size!( 200 px, 20 px), }
                    ],
                    entity[
                        image(&menu_assets.slider_handle);
                        Focusable::lock(),
                        MainMenuElem::AudioSlider(channel, strength),
                        handle_name,
                        style! {
                            size: size!( 40 px, 40 px),
                            position_type: PT::Absolute,
                            position: position,
                        }
                    ]
                )
            )
        }
        .id()
    };
    let master_slider = slider("Master", AudioChannel::Master, 100.0);
    let sfx_slider = slider("Sfx", AudioChannel::Sfx, 50.0);
    let music_slider = slider("Music", AudioChannel::Music, 50.0);
    let cursor = MenuCursor::spawn_ui_element(&mut cmds);

    build_ui! {
        #[cmd(cmds)]
        node{
            min_size: size!(100 pct, 100 pct),
            flex_direction: FD::ColumnReverse,
            justify_content: JustifyContent::Center
        }[; Name::new("Main menu root node"), MainMenuRoot](
            entity[ui_assets.background() ;],
            id(cursor),
            entity[
                image(&menu_assets.title_image);
                Name::new("Title card"),
                style! { size: size!(auto, 45 pct), }
            ],
            node{ flex_direction: FD::Row }[; Name::new("Menu columns")](
                node[; Name::new("Menu node")](
                    node[large_text("Start"); Focusable::new().prioritized(), Name::new("Start"), Start],
                    node[large_text("Credits"); Focusable::lock(), Name::new("Credits"), Credits],
                    node[large_text("How to play"); Focusable::lock(), Name::new("Rules"), Rules],
                    if (!cfg!(target_arch = "wasm32")) {
                        node[large_text("Exit"); focusable, Name::new("Exit"), Exit]
                    },
                ),
                node{ align_items: AlignItems::FlexEnd, margin: rect!(50 px) }[; Name::new("Audio settings")](
                    id(master_slider),
                    id(music_slider),
                    id(sfx_slider),
                ),
                node[; Name::new("Graphics column")](
                    if (!cfg!(target_arch = "wasm32")) {
                        node[large_text("Lock mouse cursor"); focusable, LockMouse],
                        node[large_text("Fit window to 16:9"); focusable, Set16_9],
                    },
                    node[large_text("Toggle Full screen"); focusable, ToggleFullScreen],
                )
            ),
            node{
                position_type: PT::Absolute,
                position: rect!(10 pct),
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center
            }[; UiColor(Color::rgb(0.1, 0.1, 0.1)), Name::new("Rules overlay"), RulesOverlay](
                node[large_text("Turns");],
                node[text_bundle("The game is like War, but in turns, each player plays the", 30.0);],
                node[text_bundle("first card. The one with the most point at the end wins.", 30.0);],
                node[large_text("Effects");],
                node[text_bundle("Cards may have special effects, hover over it to see", 30.0);],
                node[text_bundle("what they do.", 30.0);],
                node[large_text("Cheating");],
                node[text_bundle("Drag a card toward your sleeve to store it.", 30.0);],
                node[text_bundle("Cards stored in your sleeve return to your", 30.0);],
                node[text_bundle("hand next time players draw cards, this replaces", 30.0);],
                node[text_bundle("the card you would have otherwise drawn", 30.0);],
                node[text_bundle("from the deck.", 30.0);],
            ),
            node{
                position_type: PT::Absolute,
                position: rect!(10 pct),
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center
            }[; UiColor(Color::rgb(0.1, 0.1, 0.1)), Name::new("Credits overlay"), CreditOverlay](
                node[
                    image(&menu_assets.team_name);
                    Name::new("Team name"),
                    style! { size: size!(auto, 30 pct), }
                ],
                node[large_text("music, sfx: Samuel_sound");],
                node[large_text("graphics: Xolotl");],
                node[large_text("code, voices, design: Gibonus");],
                node[large_text("more code: vasukas");],
                node[large_text("thanks: BLucky (devops), Lorithan (game idea)");],
                node[large_text("Also the BEVY community <3 <3 <3");],
                node[text_bundle("(Click anywhere to exit)", 30.0);]
            )
        )
    };
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use crate::system_helper::EasySystemSetCtor;
        app.init_resource::<MenuAssets>()
            .add_system_set(self.0.on_enter(setup_main_menu))
            .add_system_set(self.0.on_exit(cleanup_marked::<MainMenuRoot>))
            .add_system_set(
                SystemSet::on_update(self.0)
                    .with_system(
                        update_sliders
                            .before(NavRequestSystem)
                            .before(AudioRequestSystem),
                    )
                    .with_system(leave_overlay.before(NavRequestSystem))
                    .with_system(update_menu.after(NavRequestSystem)),
            );
    }
}
