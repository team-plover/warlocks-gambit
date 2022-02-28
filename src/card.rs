use std::f32::consts::PI;

use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::render::{
    mesh::{
        Indices,
        VertexAttributeValues::{Float32x2, Float32x3},
    },
    render_resource::PrimitiveTopology,
};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use enum_map::{enum_map, Enum, EnumMap};

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Enum, Clone, Copy, Debug)]
pub enum WordOfMagic {
    Wealth,
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Enum, Debug, Clone, Copy)]
pub enum Value {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component, Debug)]
pub struct Card {
    word: WordOfMagic,
    value: Value,
}
impl Card {
    pub fn new(word: WordOfMagic, value: Value) -> Self {
        Self { word, value }
    }
}

#[derive(Component)]
struct CardFace;
#[derive(Component)]
struct CardBack;
#[derive(Component)]
struct CardWord;

#[rustfmt::skip]
const CARD_VERTICES: [[f32; 2]; 12] = [
    [-1.0, 1.46],  [-0.988, 1.488],  [-0.95, 1.5],
    [0.95, 1.5],   [0.988, 1.488],   [1.0, 1.46],
    [1.0, -1.46],  [0.988, -1.488],  [0.95, -1.5],
    [-0.95, -1.5], [-0.988, -1.488], [-1.0, -1.46],
];

#[rustfmt::skip]
const CARD_EDGES: [u16; 30] = [
    0, 2, 1,    0, 3, 2,    3, 5, 4,
    3, 6, 5,    6, 8, 7,    6, 3, 8,
    8, 3, 0,    8, 0, 9,    9, 11, 10,
    9, 0, 11,
];

#[derive(SystemParam)]
pub struct SpawnCard<'w, 's> {
    cmds: Commands<'w, 's>,
    assets: Res<'w, CardAssets>,
}
impl<'w, 's> SpawnCard<'w, 's> {
    pub fn spawn_card<'a>(&'a mut self, card: Card) -> EntityCommands<'w, 's, 'a> {
        let Card { value, word } = card;
        let mut entity = self.cmds.spawn();
        entity
            .insert_bundle((card, Name::new("Card")))
            .with_children(|cmds| {
                cmds.spawn_bundle(PbrBundle {
                    mesh: self.assets.word_mesh.clone(),
                    material: self.assets.words[word].clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.01),
                    ..Default::default()
                })
                .insert(CardWord);
                cmds.spawn_bundle(PbrBundle {
                    mesh: self.assets.card.clone(),
                    material: self.assets.values[value].clone(),
                    ..Default::default()
                })
                .insert(CardFace);
                cmds.spawn_bundle(PbrBundle {
                    mesh: self.assets.card.clone(),
                    material: self.assets.backface.clone(),
                    transform: Transform::from_rotation(Quat::from_rotation_y(PI)),
                    ..Default::default()
                })
                .insert(CardBack);
            });
        entity
    }
}
fn update_card(
    cards: Query<(&Card, &Children), Changed<Card>>,
    assets: Res<CardAssets>,
    mut face_mats: Query<&mut Handle<StandardMaterial>, (Without<CardWord>, With<CardFace>)>,
    mut word_mats: Query<&mut Handle<StandardMaterial>, With<CardWord>>,
) {
    for (card, children) in cards.iter() {
        for child in children.iter() {
            if let Ok(mut mat) = face_mats.get_mut(*child) {
                *mat = assets.values[card.value].clone();
            }
            if let Ok(mut mat) = word_mats.get_mut(*child) {
                *mat = assets.words[card.word].clone();
            }
        }
    }
}

pub struct CardAssets {
    card: Handle<Mesh>,
    values: EnumMap<Value, Handle<StandardMaterial>>,
    backface: Handle<StandardMaterial>,
    word_mesh: Handle<Mesh>,
    words: EnumMap<WordOfMagic, Handle<StandardMaterial>>,
}
impl FromWorld for CardAssets {
    fn from_world(world: &mut World) -> Self {
        macro_rules! add_texture_material {
            ($texture_path:expr $(, alpha: $alpha_mask:expr)?) => {{
                let asset_server = world.get_resource::<AssetServer>().unwrap();
                let image = asset_server.load::<Image, _>($texture_path);
                let mut mats = world.get_resource_mut::<Assets<_>>().unwrap();
                mats.add(StandardMaterial {
                    base_color_texture: Some(image),
                    $(alpha_mode: AlphaMode::Mask($alpha_mask),)?
                    ..Default::default()
                })
            }};
        }
        let uv_map = |&[x, y]: &[f32; 2]| [x / 2.0 + 0.5, -y / 3.0 + 0.5];
        let mut card_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let card_pos: Vec<[f32; 3]> = CARD_VERTICES.iter().map(|&[x, y]| [x, y, 0.0]).collect();
        let card_uvs: Vec<[f32; 2]> = CARD_VERTICES.iter().map(uv_map).collect();
        card_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(card_pos));
        card_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, Float32x2(card_uvs));
        card_mesh.set_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            Float32x3([[0.0, 0.0, 1.0]; 12].into()),
        );
        card_mesh.set_indices(Some(Indices::U16(CARD_EDGES.into())));

        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        Self {
            card: meshes.add(card_mesh),
            word_mesh: meshes.add(shape::Quad::new(Vec2::splat(2.0)).into()),
            backface: add_texture_material!("cards/Backface.jpg"),
            values: enum_map! {
                value => add_texture_material!(&format!("cards/Value{value:?}.jpg")),
            },
            words: enum_map! {
                word => add_texture_material!(&format!("cards/Word{word:?}.png"), alpha: 0.5),
            },
        }
    }
}

pub struct Plugin;
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.register_inspectable::<Card>()
            .register_inspectable::<Value>()
            .register_inspectable::<WordOfMagic>();

        app.init_resource::<CardAssets>().add_system(update_card);
    }
}
