use bevy::prelude::{*, Plugin as BevyPlugin};
use bevy::{app::AppExit, input::mouse::MouseMotion, math::Vec3Swizzles};
use bevy_ui_navigation::{systems as nav, NavigationPlugin, Focused, Focusable, NavEvent, NavRequest};
use bevy_ui_build_macros::{build_ui, rect, size, style, unit};

use crate::audio::{AudioRequest, SfxParam, AudioChannel};

#[derive(Clone, Component, Default)]
struct MenuCursor {
    size: Vec2,
    position: Vec2,
}
impl MenuCursor {
    fn set_target(&mut self, node: &Node, transform: &GlobalTransform) {
        self.size = node.size * 1.05;
        self.position = transform.translation.xy() - self.size / 2.0;
    }
}

#[derive(Component)]
struct MovingSlider;

#[derive(Component, Clone, PartialEq)]
enum MainMenuElem {
    Start,
    Exit,
    Credits,
    AudioSlider(AudioChannel, f32),
}

struct MenuAssets {
    title_image: Handle<Image>,
    slider_handle: Handle<Image>,
    slider_bg: Handle<Image>,
    font: Handle<Font>,
}
impl FromWorld for MenuAssets {
    fn from_world(world: &mut World) -> Self {      
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            font: assets.load("Boogaloo-Regular.otf"),
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
         if let (Val::Percent(left), AudioSlider(channel, strength)) = (style.position.left, elem.as_mut()) {
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
    mut cursor: Query<&mut MenuCursor>,
    mut exit: EventWriter<AppExit>,
    mut cmds: Commands,
    elems: Query<(&Node, &GlobalTransform, &MainMenuElem)>,
) {
    for nav_event in events.iter() {
        match nav_event {
            NavEvent::FocusChanged { to, from } => {
                let from = *from.first();
                let to = *to.first();
                let (node, transform, _) = elems.get(to).unwrap();
                let (_, _, from_elem) = elems.get(from).unwrap();
                let mut cursor = cursor.get_single_mut().unwrap();
                cursor.set_target(node, transform);
                if matches!(from_elem, MainMenuElem::AudioSlider(..)) {
                    cmds.entity(from).remove::<MovingSlider>();
                }
            }
            NavEvent::NoChanges { from, request: NavRequest::Action } => {
                match elems.get(*from.first()).map(|t| t.2) {
                    Ok(MainMenuElem::Exit) => exit.send(AppExit),
                    _ => {}
                }
            }
            _ => {
                println!("unhandled nav event: {nav_event:?}");
            }
        }
    }
}

fn update_highlight(mut highlight: Query<(&mut Style, &mut Node, &MenuCursor)>) {
    use Val::Px;
    if let Ok((mut style, mut node, target)) = highlight.get_single_mut() {
        if let (Px(left), Px(bot), Px(width), Px(height)) = (
            style.position.left,
            style.position.bottom,
            style.size.width,
            style.size.height,
        ) {
            let size = node.size;
            node.size += (target.size - size) * 0.4;
            style.size.width += (target.size.x - width) * 0.4;
            style.size.height += (target.size.y - height) * 0.4;
            style.position.left += (target.position.x - left) * 0.4;
            style.position.bottom += (target.position.y - bot) * 0.4;
        } else {
            style.position = rect!(1 px);
            style.size = size!(1 px, 1 px);
        }
    }
}

/// Spawns the UI tree
fn setup_main_menu(
    mut cmds: Commands,
    menu_assets: Res<MenuAssets>,
) {
    use PositionType as PT;
    use FlexDirection as FD;
    use MainMenuElem::{Credits, Start, Exit};

    let text_bundle = |content: &str| {
        let color = Color::ANTIQUE_WHITE;
        let horizontal = HorizontalAlign::Left;
        let style = TextStyle { color, font: menu_assets.font.clone(), font_size: 60.0 };
        let align = TextAlignment { horizontal, ..Default::default() };
        let text = Text::with_section(content, style, align);
        TextBundle { text, ..Default::default() }
    };
    let focusable = Focusable::default();
    let image = |image: &Handle<Image>| ImageBundle { image: image.clone().into(), ..Default::default() }; 
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
                node[text_bundle(&volume_name);],
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
        }.id()
    };
    let master_slider = slider("Master", AudioChannel::Master, 100.0);
    let sfx_slider = slider("Sfx", AudioChannel::Sfx, 50.0);
    let music_slider = slider("Music", AudioChannel::Music, 50.0);
    cmds.spawn_bundle(UiCameraBundle::default());
    build_ui! {
        #[cmd(cmds)]
        node{ min_size: size!(100 pct, 100 pct) }[;Name::new("root node")](
            node{ position_type: PT::Absolute }[;
                UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
                MenuCursor::default(),
                Name::new("Cursor")
            ],
            entity[
                image(&menu_assets.title_image);
                Name::new("Title Image"),
                style! { size: size!(auto, 30 pct), }
            ],
            node[; Name::new("Menu node")](
                node[text_bundle("Start");   focusable, Name::new("Start button"), Start],
                node[text_bundle("Credits"); focusable, Name::new("Credits button"), Credits],
                node[text_bundle("Exit");    focusable, Name::new("Exit button"), Exit],
                id(master_slider),
                id(music_slider),
                id(sfx_slider)
            )
        )
    };
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {      
        app.add_plugin(NavigationPlugin)
            .init_resource::<MenuAssets>()
            .add_system(nav::default_mouse_input)
            .add_system(nav::default_gamepad_input)
            .init_resource::<nav::InputMapping>()
            .add_startup_system(setup_main_menu)
            .add_system(update_highlight)
            .add_system(update_sliders)
            .add_system(update_menu)
            ;
    }
}
