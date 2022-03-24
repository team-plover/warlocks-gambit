//! Handle the visual elements of cards.
//!
//! Exposes the [`SpawnCard`] system parameter to spawn cards with all
//! proper graphical objects attached to it. [`SpawnCard`] uses assets defined
//! in [`CardAssets`].
//!
//! The only system here is [`update_card_graphics`].
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy::math::EulerRot::XYZ;
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::render::{
    mesh::{
        Indices,
        VertexAttributeValues::{Float32x2, Float32x3},
    },
    render_resource::PrimitiveTopology,
};
use bevy_debug_text_overlay::screen_print;
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use enum_map::{enum_map, EnumMap};

use crate::{
    war::{Card, Value, WordOfPower},
    CardOrigin, Participant,
};

/// Component attached to where the opponent draws cards from.
#[derive(Component)]
pub struct OppoCardSpawner;

/// Component attached to where the player draws cards from.
#[derive(Component)]
pub struct PlayerCardSpawner;

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Clone, Component, Copy, PartialEq, Debug)]
pub enum CardStatus {
    Normal,
    Hovered,
}

#[derive(Component)]
struct CardGraphics {
    value: Entity,
    glow: Entity,
    word: Entity,
}

// TODO: make corner more bevelled
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

/// Spawn a card with all the proper associated graphics.
#[derive(SystemParam)]
pub struct SpawnCard<'w, 's> {
    pub cmds: Commands<'w, 's>,
    assets: Res<'w, CardAssets>,
    player_deck: Query<'w, 's, &'static GlobalTransform, With<PlayerCardSpawner>>,
    oppo_deck: Query<'w, 's, &'static GlobalTransform, With<OppoCardSpawner>>,
}
impl<'w, 's> SpawnCard<'w, 's> {
    pub fn spawn_card<'a>(
        &'a mut self,
        card: Card,
        from: Participant,
    ) -> EntityCommands<'w, 's, 'a> {
        use WordOfPower::Egeq;

        let Card { value, word, .. } = card;
        let spawner_transform = match from {
            Participant::Oppo => self.oppo_deck.single(),
            Participant::Player => self.player_deck.single(),
        };
        let cmds = &mut self.cmds;
        let entity = cmds
            .spawn_bundle((
                card,
                CardOrigin(from),
                Name::new("Card"),
                GlobalTransform::default(),
                Transform {
                    scale: Vec3::splat(0.5),
                    translation: spawner_transform.translation,
                    rotation: spawner_transform.rotation
                        * Quat::from_euler(XYZ, FRAC_PI_2, 0.0, 0.0),
                },
            ))
            .id();
        cmds.spawn_bundle(PbrBundle {
            mesh: self.assets.card.clone(),
            material: self.assets.frontface.clone(),
            ..Default::default()
        })
        .insert_bundle((Parent(entity), Name::new("Front face")));
        cmds.spawn_bundle(PbrBundle {
            mesh: self.assets.card.clone(),
            material: self.assets.backface.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_y(PI)),
            ..Default::default()
        })
        .insert_bundle((Parent(entity), Name::new("Back face")));

        let mut spawn_pbr = |name, pbr| {
            cmds.spawn_bundle(pbr)
                .insert_bundle((Parent(entity), Name::new(name)))
                .id()
        };
        let default_card_pbr = |material: &Handle<StandardMaterial>| PbrBundle {
            mesh: self.assets.quad.clone(),
            material: material.clone(),
            ..Default::default()
        };
        #[rustfmt::skip]
        let graphics = CardGraphics {
            word: spawn_pbr("Word", PbrBundle {
                transform: Transform::from_xyz(0.0, -0.8, 0.01)
                    .with_scale(Vec3::new(1.5, 1.0, 1.0)),
                visibility: Visibility { is_visible: word.is_some() },
                ..default_card_pbr(&self.assets.words[word.unwrap_or(Egeq)])
            }),
            value: spawn_pbr("Value", PbrBundle {
                transform: Transform::from_xyz(0.0, 0.5, 0.01)
                    .with_scale(Vec3::new(1.0, 1.5, 1.0)),
                ..default_card_pbr(&self.assets.values[value])
            }),
            glow: spawn_pbr("Glow", PbrBundle {
                transform: Transform::from_xyz(0.0, -0.8, 0.009)
                    .with_scale(Vec3::new(4.2, 2.2, 0.0)),
                visibility: Visibility { is_visible: false },
                ..default_card_pbr(&self.assets.glow)
            }),
        };
        let mut ent = cmds.entity(entity);
        ent.insert_bundle((CardStatus::Normal, graphics));
        ent
    }
}

#[allow(clippy::type_complexity)]
fn update_card_graphics(
    cards: Query<(&Card, &CardStatus, &CardGraphics), Or<(Changed<Card>, Changed<CardStatus>)>>,
    assets: Res<CardAssets>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
    mut mats: Query<(&mut Visibility, &mut Handle<StandardMaterial>)>,
) {
    for (card, status, graphics) in cards.iter() {
        if let Ok((_, mut mat)) = mats.get_mut(graphics.value) {
            *mat = assets.values[card.value].clone();
        }
        if let Ok((mut vis, mut mat)) = mats.get_mut(graphics.word) {
            vis.is_visible = card.word.is_some();
            if let Some(word) = card.word {
                *mat = assets.words[word].clone();
            }
        }
        if let (Ok((mut vis, mat)), Some(word)) = (mats.get_mut(graphics.glow), card.word) {
            vis.is_visible = *status == CardStatus::Hovered;
            if vis.is_visible {
                let col = word.color();
                screen_print!(sec: 1, col: col, "Swapping color of card with {word:?}");
                let mut mat = mat_assets.get_mut(mat.clone()).unwrap();
                mat.emissive = col;
            }
        }
    }
}

pub struct CardAssets {
    card: Handle<Mesh>,
    values: EnumMap<Value, Handle<StandardMaterial>>,
    backface: Handle<StandardMaterial>,
    frontface: Handle<StandardMaterial>,
    quad: Handle<Mesh>,
    words: EnumMap<WordOfPower, Handle<StandardMaterial>>,
    glow: Handle<StandardMaterial>,
}
impl FromWorld for CardAssets {
    fn from_world(world: &mut World) -> Self {
        use AlphaMode::*;
        macro_rules! add_texture_material {
            ($texture_path:expr $(, alpha: $alpha_mask:expr)? $(, emissive: $emissive:expr)?) => {{
                let asset_server = world.get_resource::<AssetServer>().unwrap();
                let image = asset_server.load::<Image, _>($texture_path);
                let mut mats = world.get_resource_mut::<Assets<_>>().unwrap();
                mats.add(StandardMaterial {
                    base_color_texture: Some(image),
                    $(alpha_mode: $alpha_mask,)?
                    $(emissive: $emissive,)?
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
            quad: meshes.add(shape::Quad::new(Vec2::splat(1.0)).into()),
            backface: add_texture_material!("cards/BackFace.png"),
            frontface: add_texture_material!("cards/FrontFace.png"),
            values: enum_map! {
                value => add_texture_material!(&format!("cards/Value{value:?}.png"), alpha: Mask(0.5)),
            },
            glow: add_texture_material!("glow.png", alpha: Blend),
            words: enum_map! {
                word => add_texture_material!(
                    &format!("cards/Word{word:?}.png"),
                    alpha: Mask(0.5),
                    emissive: word.color()
                ),
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
            .register_inspectable::<WordOfPower>();

        app.init_resource::<CardAssets>()
            .add_system(update_card_graphics);
    }
}
