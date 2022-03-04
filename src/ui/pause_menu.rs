use super::common::*;
use super::main_menu::MenuAssets;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::{app::AppExit, input::mouse::MouseMotion, window::WindowMode};
use bevy_ui_build_macros::{build_ui, rect, size, style, unit};
use bevy_ui_navigation::{Focusable, Focused, NavEvent, NavRequest};

// TODO: this is mostly a copy of main menu

use crate::{
    audio::{AudioChannel, AudioRequest, SfxParam},
    state::GameState,
};

#[derive(Component)]
struct MovingSlider;

#[derive(Component, Clone, PartialEq)]
enum MainMenuElem {
    Continue,
    Exit,
    LockMouse,
    ToggleFullScreen,
    Set16_9,
    AudioSlider(AudioChannel, f32),
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
    mut windows: ResMut<Windows>,
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
            NavEvent::NoChanges { from, request: NavRequest::Action } => {
                match elems.get(*from.first()).map(|t| t.2) {
                    Ok(MainMenuElem::Continue) => game_state.pop().unwrap(),
                    Ok(MainMenuElem::Exit) => exit.send(AppExit),
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
                Name::new("End spacer"),
                style! { size: size!(auto, 10 pct), }
            ],
            node{ flex_direction: FD::Row }[; Name::new("Menu columns")](
                node[; Name::new("Continue/exit column")](
                    node[large_text("Continue"); focusable, Name::new("Continue button"), Continue],
                    node[large_text("Exit to desktop"); focusable, Name::new("Exit button"), Exit]
                ),
                node{ align_items: AlignItems::FlexEnd, margin: rect!(50 px) }[; Name::new("Audio settings")](
                    id(master_slider),
                    id(music_slider),
                    id(sfx_slider)
                ),
                node[; Name::new("Graphics column")](
                    node[large_text("Lock mouse cursor"); focusable, LockMouse],
                    node[large_text("Toggle Full screen"); focusable, ToggleFullScreen],
                    node[large_text("Make exactly 16:9"); focusable, Set16_9]
                )
            )
        )
    };
}

fn toggle_pause_menu(mut keys: ResMut<Input<KeyCode>>, mut game_state: ResMut<State<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        keys.reset(KeyCode::Escape);
        match game_state.current() {
            GameState::PauseMenu => game_state.pop().unwrap(),
            _ => game_state.push(GameState::PauseMenu).unwrap(),
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::PauseMenu).with_system(setup_main_menu))
            .add_system_set(SystemSet::on_exit(GameState::PauseMenu).with_system(exit_menu))
            .add_system_set(
                SystemSet::on_update(GameState::PauseMenu)
                    .with_system(update_sliders)
                    .with_system(update_menu),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing).with_system(toggle_pause_menu),
            );
    }
}
