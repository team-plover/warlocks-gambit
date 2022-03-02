use bevy::math::Vec3Swizzles;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_ui_build_macros::{rect, size, unit};
use bevy_ui_navigation::{systems as nav, Focused, NavigationPlugin};

use std::fs::DirBuilder;

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
}

/// The root node of the main menu, to remove the menu when exiting it
#[derive(Component, Clone)]
pub struct MenuRoot;

/// Add this system in menu-specific plugin with SystemSet::on_exit
pub fn exit_menu(mut cmds: Commands, root: Query<Entity, With<MenuRoot>>) {
    root.iter()
        .for_each(|entity| cmds.entity(entity).despawn_recursive())
}

pub struct UiAssets {
    pub font: Handle<Font>,
}
impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self { font: assets.load("Boogaloo-Regular.otf") }
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
            .add_system(nav::default_gamepad_input)
            .add_system(update_highlight);

        app.add_startup_system(|mut cmds: Commands| {
            cmds.spawn_bundle(UiCameraBundle::default());
        });
    }
}
