use bevy::math::Vec3Swizzles;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_build_macros::{rect, size, style, unit};
use bevy_ui_navigation::{systems as nav, Focused, NavigationPlugin};

#[derive(Clone, Component, Default)]
pub struct MenuCursor {
    size: Vec2,
    position: Vec2,
}
impl MenuCursor {
    fn set_target(&mut self, node: &Node, transform: &GlobalTransform) {
        self.size = node.size * 1.05;
        self.position = transform.translation.xy() - self.size / 2.0;
    }
    pub fn spawn_ui_element(cmds: &mut Commands) -> Entity {
        cmds.spawn_bundle(NodeBundle {
            style: style! { position_type: PositionType::Absolute, size: size!(0 pct, 0 pct), },
            color: UiColor(Color::rgba(1.0, 1.0, 1.0, 0.1)),
            ..Default::default()
        })
        .insert_bundle((Self::default(), Name::new("Cursor")))
        .id()
    }
}

pub struct UiAssets {
    pub font: Handle<Font>,
    pub background_image: Handle<Image>,
}
impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            font: assets.load("Boogaloo-Regular.otf"),
            background_image: assets.load("main_menu_bg.jpg"),
        }
    }
}

impl UiAssets {
    pub fn text_bundle(&self, content: &str, font_size: f32) -> TextBundle {
        let color = Color::ANTIQUE_WHITE;
        let horizontal = HorizontalAlign::Left;
        let style = TextStyle { color, font: self.font.clone(), font_size };
        let align = TextAlignment { horizontal, ..Default::default() };
        let text = Text::with_section(content, style, align);
        TextBundle { text, ..Default::default() }
    }
    pub fn large_text(&self, content: &str) -> TextBundle {
        self.text_bundle(content, 60.)
    }
    pub fn background(&self) -> ImageBundle {
        use PositionType::Absolute;
        ImageBundle {
            image: self.background_image.clone().into(),
            style: style! { position_type: Absolute, size: size!(auto, 100 pct), },
            ..Default::default()
        }
    }
}

fn update_highlight(
    mut highlight: Query<(&mut Style, &mut Node, &mut MenuCursor), Without<Focused>>,
    focused: Query<(&Node, &GlobalTransform), With<Focused>>,
) {
    use Val::Px;
    let query = (highlight.get_single_mut(), focused.get_single());
    if let (Ok((mut style, mut cursor_node, mut target)), Ok((node, transform))) = query {
        target.set_target(node, transform);
        if let (Px(left), Px(bot), Px(width), Px(height)) = (
            style.position.left,
            style.position.bottom,
            style.size.width,
            style.size.height,
        ) {
            let size = cursor_node.size;
            cursor_node.size += (target.size - size) * 0.4;
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

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NavigationPlugin)
            .init_resource::<UiAssets>()
            .init_resource::<nav::InputMapping>()
            .add_system(nav::default_mouse_input)
            .add_system(update_highlight);

        app.add_startup_system(|mut cmds: Commands| {
            cmds.spawn_bundle(UiCameraBundle::default());
        });
    }
}
