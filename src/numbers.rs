//! Display numbers in the 3d game world.
use std::iter;

use bevy::{
    prelude::{Plugin as BevyPlugin, *},
    utils::HashMap,
};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct Number {
    pub value: i32,
    pub color: Color,
}
impl Number {
    pub fn new(value: i32, color: Color) -> Self {
        Self { value, color }
    }
}

#[derive(Component)]
struct NumberSprite;

#[rustfmt::skip]
const NUMBER_NAMES: [&str; 10] = 
    [ "Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine"];

fn add_number(
    new_numbers: Query<Entity, Added<Number>>,
    mut cmds: Commands,
    assets: Res<NumberAssets>,
) {
    for entity in new_numbers.iter() {
        cmds.entity(entity).with_children(|cmds| {
            for _ in 0..5 {
                cmds.spawn_bundle((NumberSprite, Name::new("NumberSprite")))
                    .insert_bundle(PbrBundle {
                        mesh: assets.quad.clone(),
                        visibility: Visibility { is_visible: false },
                        ..Default::default()
                    });
            }
        });
    }
}

type SpriteComponents = (
    &'static Parent,
    &'static mut Transform,
    &'static mut Visibility,
    &'static mut Handle<StandardMaterial>,
);
fn display_number(
    numbers: Query<&Number, Changed<Number>>,
    mut sprites: Query<SpriteComponents, With<NumberSprite>>,
    assets: Res<NumberAssets>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mut decimal_streams: HashMap<Entity, _> = HashMap::default();
    for (Parent(parent), mut transform, mut vis, mut material) in sprites.iter_mut() {
        // We only do things for numbers which value changed
        if let Ok(Number { value, color }) = numbers.get(*parent) {
            let initial_iter = || decimals(*value).enumerate();
            let current_decimal = decimal_streams.entry(*parent).or_insert_with(initial_iter);
            if let Some((i, current)) = current_decimal.next() {
                vis.is_visible = true;
                transform.translation.x = i as f32 * -0.9;
                *material = mats.add(StandardMaterial {
                    base_color_texture: Some(assets.images[current].clone()),
                    alpha_mode: AlphaMode::Mask(0.5),
                    emissive: *color,
                    ..Default::default()
                });
            } else {
                vis.is_visible = false;
            }
        }
    }
}

/// The right-to-left decimal values of number.
fn decimals(mut number: i32) -> impl Iterator<Item = usize> {
    iter::from_fn(move || {
        let current = number % 10;
        let is_nonzero = number != 0;
        number = (number - current) / 10;
        is_nonzero.then(|| current as usize)
    })
}

struct NumberAssets {
    images: [Handle<Image>; 10],
    quad: Handle<Mesh>,
}
impl FromWorld for NumberAssets {
    fn from_world(world: &mut World) -> Self {
        let images = world.get_resource::<Assets<Image>>().unwrap();
        let images = NUMBER_NAMES.map(|name| images.get_handle(format!("cards/Value{name}.png")));
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let quad = meshes.add(shape::Quad::new(Vec2::new(1., 2.)).into());
        Self { images, quad }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<Number>();

        app.init_resource::<NumberAssets>()
            .add_system(display_number)
            .add_system(add_number);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimals() {
        macro_rules! number_assert {
            ($initial:expr, $( $result:literal)*) => (
                let mut result: Vec<usize> = decimals($initial).collect();
                let expected: Vec<usize> = vec![$($result,)*];
                result.reverse();
                assert_eq!(result, expected);
            )
        }
        number_assert!(10000, 1 0 0 0 0);
        number_assert!(10203, 1 0 2 0 3);
        number_assert!(12, 1 2);
        number_assert!(10, 1 0);
        number_assert!(93841345, 9 3 8 4 1 3 4 5);
        number_assert!(1, 1);
        number_assert!(0,);
    }
}
