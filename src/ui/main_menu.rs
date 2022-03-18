use super::common::*;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::{app::AppExit, input::mouse::MouseMotion, window::WindowMode};
use bevy_debug_text_overlay::screen_print;
use bevy_ui_build_macros::{build_ui, rect, size, style, unit};
use bevy_ui_navigation::{Focusable, Focused, NavEvent, NavRequest};

// TODO: wait until background scene is loaded (it should take less than second)

use crate::scene::ScenePreload;
use crate::{
    audio::{AudioChannel, AudioRequest, SfxParam},
    state::GameState,
};

#[derive(Component)]
struct MovingSlider;

#[derive(Component, Clone)]
struct CreditOverlay;

#[derive(Component, Clone, PartialEq)]
enum MainMenuElem {
    Start,
    Exit,
    Credits,
    LockMouse,
    ToggleFullScreen,
    Set16_9,
    AudioSlider(AudioChannel, f32),
}

pub struct MenuAssets {
    title_image: Handle<Image>,
    pub slider_handle: Handle<Image>,
    pub slider_bg: Handle<Image>,
}
impl FromWorld for MenuAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
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
    focused: Query<Entity, With<Focused>>,
    elems: Query<&MainMenuElem, Without<MovingSlider>>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    use MainMenuElem::AudioSlider;
    if let Ok((entity, mut style, mut elem)) = styles.get_single_mut() {
        if let (Val::Percent(left), AudioSlider(channel, strength)) =
            (style.position.left, elem.as_mut())
        {
            let horizontal_delta: f32 = mouse_motion.iter().map(|m| m.delta.x).sum();
            let new_left = (left / 0.9 + horizontal_delta * 0.40).min(100.0).max(0.0);
            *strength = new_left;
            audio_requests.send(AudioRequest::SetChannelVolume(*channel, new_left / 100.0));
            style.position.left = Val::Percent(new_left * 0.9)
        };
        if mouse_buttons.just_released(MouseButton::Left) {
            audio_requests.send(AudioRequest::StopSfxLoop);
            cmds.entity(entity).remove::<MovingSlider>();
        }
    }
    if let Ok(entity) = focused.get_single() {
        let is_volume_slider = matches!(elems.get(entity), Ok(AudioSlider(..)));
        if mouse_buttons.just_pressed(MouseButton::Left) && is_volume_slider {
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
    mut game_state: ResMut<State<GameState>>,
    elems: Query<(&Node, &GlobalTransform, &MainMenuElem)>,
) {
    let window_msg = "There is at least one game window open";
    for nav_event in events.iter() {
        match nav_event {
            NavEvent::FocusChanged { from, .. } => {
                let from = *from.first();
                let (_, _, from_elem) = elems.get(from).unwrap();
                if matches!(from_elem, MainMenuElem::AudioSlider(..)) {
                    cmds.entity(from).remove::<MovingSlider>();
                }
            }
            NavEvent::Locked(from) => {
                if let Ok(MainMenuElem::Credits) = elems.get(*from).map(|t| t.2) {
                    let mut style = credit_overlay.single_mut();
                    style.display = Display::Flex;
                }
            }
            NavEvent::NoChanges { from, request: NavRequest::Action } => {
                match elems.get(*from.first()).map(|t| t.2) {
                    Ok(MainMenuElem::Exit) => exit.send(AppExit),
                    Ok(MainMenuElem::Start) => {
                        screen_print!("Player pressed the start button");
                        audio_requests.send(AudioRequest::PlayWoodClink(SfxParam::PlayOnce));
                        game_state.set(GameState::LoadScene).unwrap();
                    }
                    Ok(MainMenuElem::LockMouse) => {
                        let window = windows.get_primary_mut().expect(window_msg);
                        let prev_lock_mode = window.cursor_locked();
                        window.set_cursor_lock_mode(!prev_lock_mode);
                    }
                    Ok(MainMenuElem::ToggleFullScreen) => {
                        use WindowMode::*;
                        let window = windows.get_primary_mut().expect(window_msg);
                        let prev_mode = window.mode();
                        let new_mode = if prev_mode == BorderlessFullscreen {
                            Windowed
                        } else {
                            BorderlessFullscreen
                        };
                        window.set_mode(new_mode);
                    }
                    Ok(MainMenuElem::Set16_9) => {
                        let window = windows.get_primary_mut().expect(window_msg);
                        if window.mode() == WindowMode::Windowed {
                            let height = window.height();
                            window.set_resolution(height * 16.0 / 9.0, height);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                println!("unhandled nav event: {nav_event:?}");
            }
        }
    }
}

fn leave_credits(
    mut credit_overlay: Query<&mut Style, With<CreditOverlay>>,
    mut nav_requests: EventWriter<NavRequest>,
    gamepad: Res<Input<GamepadButton>>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
) {
    if gamepad.get_just_pressed().len() != 0
        || mouse.get_just_pressed().len() != 0
        || keyboard.get_just_pressed().len() != 0
    {
        let mut style = credit_overlay.single_mut();
        if style.display == Display::Flex {
            style.display = Display::None;
            nav_requests.send(NavRequest::Free)
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
    let mut slider = |name: &str, channel: AudioChannel, strength: f32| {
        let volume_name = name.to_string() + " volume";
        let handle_name = Name::new(name.to_string() + " volume slider handle");
        let slider_name = Name::new(name.to_string() + " volume slider");
        build_ui! {
            #[cmd(cmds)]
            node { flex_direction: FD::Row }[; slider_name](
                node[text_bundle(&volume_name, 30.0); style! { margin: rect!(10 px), }],
                node(
                    entity[image(&menu_assets.slider_bg); style! { size: size!( 200 px, 20 px), }],
                    entity[
                        image(&menu_assets.slider_handle);
                        focusable,
                        MainMenuElem::AudioSlider(channel, strength),
                        handle_name,
                        style! {
                            size: size!( 40 px, 40 px),
                            position_type: PT::Absolute,
                            position: Rect {
                                bottom: Val::Px(-10.0),
                                left: Val::Percent(strength * 0.9),
                                ..Default::default()
                            },
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

    #[cfg(not(target_arch = "wasm32"))]
    build_ui! {
        #[cmd(cmds)]
        node{ min_size: size!(100 pct, 100 pct), flex_direction: FD::Column }[;Name::new("root node"), MenuRoot](
            node{ position_type: PT::Absolute, size: Size::new(Val::Percent(0.), Val::Percent(0.)) }[;
                UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
                MenuCursor::default(),
                Name::new("Cursor")
            ],
            entity[
                large_text(""); // I have no idea what I am doing, but it works
                Name::new("End pacer"),
                style! { size: size!(auto, 10 pct), }
            ],
            node{ flex_direction: FD::Row }[; Name::new("Menu columns")](
                node[; Name::new("Menu node")](
                    node[large_text("Start");   focusable, Name::new("Start button"), Start],
                    node[large_text("Credits"); Focusable::lock(), Name::new("Credits button"), Credits],
                    node[large_text("Exit");    focusable, Name::new("Exit button"), Exit]
                ),
                node{ align_items: AlignItems::FlexEnd, margin: rect!(50 px) }[; Name::new("Audio settings")](
                    id(master_slider),
                    id(music_slider),
                    id(sfx_slider)
                ),
                node[; Name::new("Graphics column")](
                    node[large_text("Lock mouse cursor"); focusable, LockMouse],
                    node[large_text("Toggle Full screen"); focusable, ToggleFullScreen],
                    node[large_text("Fit window to 16:9"); focusable, Set16_9]
                )
            ),
            node{
                position_type: PT::Absolute,
                position: rect!(10 pct),
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center
            }[; UiColor(Color::rgb(0.1, 0.1, 0.1)), Name::new("Credits overlay"), CreditOverlay](
                node[
                    image(&menu_assets.title_image);
                    Name::new("Title Image"),
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

    // TODO: this is copied from code above with few minor changes
    #[cfg(target_arch = "wasm32")]
    build_ui! {
        #[cmd(cmds)]
        node{ min_size: size!(100 pct, 100 pct), flex_direction: FD::Column }[;Name::new("root node"), MenuRoot](
            node{ position_type: PT::Absolute, size: Size::new(Val::Percent(0.), Val::Percent(0.)) }[;
                UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
                MenuCursor::default(),
                Name::new("Cursor")
            ],
            entity[
                large_text(""); // I have no idea what I am doing, but it works
                Name::new("End pacer"),
                style! { size: size!(auto, 10 pct), }
            ],
            node{ flex_direction: FD::Row }[; Name::new("Menu columns")](
                node[; Name::new("Menu node")](
                    node[large_text("Start");   focusable, Name::new("Start button"), Start],
                    node[large_text("Credits"); Focusable::lock(), Name::new("Credits button"), Credits]
                ),
                node{ align_items: AlignItems::FlexEnd, margin: rect!(50 px) }[; Name::new("Audio settings")](
                    id(master_slider),
                    id(music_slider),
                    id(sfx_slider)
                ),
                node[; Name::new("Graphics column")](
                    node[large_text("Toggle Full screen"); focusable, ToggleFullScreen]
                )
            ),
            node{
                position_type: PT::Absolute,
                position: rect!(10 pct),
                display: Display::None,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center
            }[; UiColor(Color::rgb(0.1, 0.1, 0.1)), Name::new("Credits overlay"), CreditOverlay](
                node[
                    image(&menu_assets.title_image);
                    Name::new("Title Image"),
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

fn setup_scene(
    mut cmds: Commands,
    mut scene_spawner: ResMut<SceneSpawner>,
    preload: Res<ScenePreload>,
) {
    let scene = preload.main_menu.clone();
    let parent = cmds
        .spawn()
        .insert(MenuRoot)
        .insert(Name::new("Menu background"))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .id();
    scene_spawner.spawn_as_child(scene, parent);
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuAssets>()
            .insert_resource(ScenePreload::default())
            .add_system_set(
                SystemSet::on_enter(self.0)
                    .with_system(setup_main_menu)
                    .with_system(setup_scene),
            )
            .add_system_set(SystemSet::on_exit(self.0).with_system(exit_menu))
            .add_system_set(
                SystemSet::on_update(self.0)
                    .with_system(update_sliders)
                    .with_system(leave_credits)
                    .with_system(update_menu),
            );
    }
}
