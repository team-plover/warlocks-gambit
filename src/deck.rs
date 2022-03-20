use std::str::FromStr;

use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_scene_hook::SceneHook;

use crate::{scene::Scene, state::GameState, war::Card};

#[cfg_attr(feature = "debug", derive(Inspectable))]
struct Deck {
    cards: Vec<Card>,
}
impl Deck {
    fn new(mut cards: Vec<Card>) -> Self {
        cards.reverse();
        Self { cards }
    }
    fn draw(&mut self, count: usize) -> Vec<Card> {
        let len = self.remaining();
        let index = len - count.min(len);
        self.cards.split_off(index)
    }
    fn remaining(&self) -> usize {
        self.cards.len()
    }
    fn score(&self) -> i32 {
        self.cards.iter().map(Card::max_value).sum()
    }
}
impl FromStr for Deck {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deck = s
            .split_ascii_whitespace()
            .map(|s| s.parse())
            .collect::<Result<_, _>>()?;
        Ok(Self::new(deck))
    }
}

macro_rules! impl_deck_methods {
    ($what:ident) => (
        impl $what {
            impl_deck_methods!(@method score((&)) -> i32);
            impl_deck_methods!(@method draw((&mut), count: usize) -> Vec<Card>);
            impl_deck_methods!(@method remaining((&)) -> usize);
        }
    );
    (@method $name:ident (
        ($($self_param:tt)*)
        $(, $param_name:ident : $param_type:ty)*)
        $(-> $ret:ty)?
    ) => (
        pub fn $name($($self_param)* self $(, $param_name : $param_type)*) $(-> $ret)? {
            self.0.$name($($param_name ,)*)
        }
    )
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct PlayerDeck(Deck);
impl_deck_methods!(PlayerDeck);

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Component)]
pub struct OppoDeck(Deck);
impl_deck_methods!(OppoDeck);

impl PlayerDeck {
    pub fn new() -> Self {
        let deck = "7s  0_  1s
                    4_  2_  6d
                    6s  8w  0z
                    2s  3_  4_
                    3_  9z  1d
                    5w  4_  3_";
        Self(deck.parse().unwrap())
    }
}

impl OppoDeck {
    pub fn new() -> Self {
        let deck = "8_  7_  6_ 
                    9z  5d  6_ 
                    8_  9_  6_ 
                    7_  1w  5_ 
                    9d  0w  5_ 
                    1d  6d  5_";
        Self(deck.parse().unwrap())
    }
}

fn update_meshes(
    (player_cards, oppo_cards): (usize, usize),
    (player, oppo): (Entity, Entity),
    meshes: &mut Assets<Mesh>,
    meshes_q: &mut Query<(&Handle<Mesh>, &mut Visibility)>,
) -> Option<()> {
    // 18 -> 0.124 ; -- 0 -> -0.9
    use bevy::render::mesh::VertexAttributeValues::{Float32x2, Float32x3};
    let mut update_deck = |entity, card_count| {
        let (handle, mut visibility) = meshes_q.get_mut(entity).ok()?;
        visibility.is_visible = card_count != 0;
        let mesh = meshes.get_mut(handle.clone())?;
        if let Float32x3(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)? {
            for pos in positions.iter_mut().filter(|v| v[1] > -0.901) {
                pos[1] = card_count as f32 / 18.0 - 0.9;
            }
        }
        if let Float32x2(uvs) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)? {
            // the > 0.218 is to only modify uv points for the sides of the deck,
            // avoiding to modify the uv for the top and bottom face which are
            // not concerned by the resize.
            for point in uvs.iter_mut().filter(|p| p[0] > 0.218 && p[1] > 0.001) {
                point[1] = card_count as f32 / 18. + 0.002;
            }
        }
        Some(())
    };
    update_deck(player, player_cards)?;
    update_deck(oppo, oppo_cards)?;
    Some(())
}

fn resize_decks(
    player_parent: Query<(&Children, &PlayerDeck)>,
    oppo_parent: Query<(&Children, &OppoDeck)>,
    mut meshes_q: Query<(&Handle<Mesh>, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (player, player_deck) = player_parent.single();
    let (oppo, oppo_deck) = oppo_parent.single();
    update_meshes(
        (player_deck.remaining(), oppo_deck.remaining()),
        (player[0], oppo[0]),
        &mut meshes,
        &mut meshes_q,
    );
}

fn reset_decks(mut player: Query<&mut PlayerDeck>, mut oppo: Query<&mut OppoDeck>) {
    *player.single_mut() = PlayerDeck::new();
    *oppo.single_mut() = OppoDeck::new();
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use crate::system_helper::EasySystemSetCtor;
        #[cfg(feature = "debug")]
        app.register_inspectable::<PlayerDeck>()
            .register_inspectable::<OppoDeck>();

        app.add_system(resize_decks.with_run_criteria(Scene::when_spawned))
            .add_system_set(self.0.on_exit(reset_decks));
    }
}
