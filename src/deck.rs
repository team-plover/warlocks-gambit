//! The game deck, deck loading from files and drawing cards from deck.
//!
//! We define a custom [`Deck`] asset with a custom loader [`DeckLoader`], this
//! way it is possible for the player to change the decks defined in
//! `assets/decks/*.deck`, and it is also possible to hot-reload the decks for
//! quicker iteration time.
use std::str::FromStr;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::{Plugin as BevyPlugin, *},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_scene_hook::is_scene_hooked;

use crate::{
    scene::Graveyard,
    state::GameState,
    war::{Card, ParseError},
};

pub struct DeckAssets {
    pub player: Handle<Deck>,
    pub oppo: Handle<Deck>,
}
impl Clone for DeckAssets {
    fn clone(&self) -> Self {
        Self {
            player: self.player.clone_weak(),
            oppo: self.oppo.clone_weak(),
        }
    }
}
impl FromWorld for DeckAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();
        Self {
            player: assets.load("decks/player.deck"),
            oppo: assets.load("decks/oppo.deck"),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Inspectable))]
#[derive(Debug, TypeUuid, Clone)]
#[uuid = "010293ef-dc29-4d94-aae1-39da45947644"]
pub struct Deck {
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
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deck = s
            .split_ascii_whitespace()
            .map(|s| s.parse())
            .collect::<Result<_, _>>()?;
        Ok(Self::new(deck))
    }
}
#[derive(Default)]
pub struct DeckLoader;
impl AssetLoader for DeckLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let deck: Deck = std::str::from_utf8(bytes)?.parse()?;
            load_context.set_default_asset(LoadedAsset::new(deck));
            Ok(())
        })
    }
    fn extensions(&self) -> &[&str] {
        &["deck"]
    }
}

macro_rules! impl_deck_methods {
    ($what:ident) => (
        impl $what {
            impl_deck_methods!(@method score((&)) -> i32);
            impl_deck_methods!(@method draw((&mut), count: usize) -> Vec<Card>);
            impl_deck_methods!(@method remaining((&)) -> usize);
            pub fn new(deck: Deck) -> Self {
                Self(deck)
            }
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
        let mesh = meshes.get_mut(handle)?;
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
    player_parent: Query<(&Children, &PlayerDeck), Changed<PlayerDeck>>,
    oppo_parent: Query<(&Children, &OppoDeck), Changed<OppoDeck>>,
    mut meshes_q: Query<(&Handle<Mesh>, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let decks = (player_parent.get_single(), oppo_parent.get_single());
    if let (Ok((player, player_deck)), Ok((oppo, oppo_deck))) = decks {
        update_meshes(
            (player_deck.remaining(), oppo_deck.remaining()),
            (player[0], oppo[0]),
            &mut meshes,
            &mut meshes_q,
        );
    }
}

fn load_decks(
    unloaded_decks: Query<(Entity, &Handle<Deck>, &Name), (Without<PlayerDeck>, Without<OppoDeck>)>,
    mut cmds: Commands,
    decks: Res<Assets<Deck>>,
) {
    for (to_load, handle, name) in unloaded_decks.iter() {
        if let Some(deck) = decks.get(handle) {
            let mut cmds = cmds.entity(to_load);
            match name.as_str() {
                "PlayerDeck" => cmds.insert(PlayerDeck::new(deck.clone())),
                "OppoDeck" => cmds.insert(OppoDeck::new(deck.clone())),
                _ => &mut cmds,
            };
        }
    }
}

fn reset_decks(decks: Query<Entity, Or<(With<PlayerDeck>, With<OppoDeck>)>>, mut cmds: Commands) {
    for to_unload in decks.iter() {
        cmds.entity(to_unload)
            .remove::<PlayerDeck>()
            .remove::<OppoDeck>();
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        use crate::system_helper::EasySystemSetCtor;
        #[cfg(feature = "debug")]
        app.register_inspectable::<PlayerDeck>()
            .register_inspectable::<OppoDeck>();

        app.add_asset::<Deck>()
            .init_asset_loader::<DeckLoader>()
            .init_resource::<DeckAssets>()
            .add_system(resize_decks.with_run_criteria(is_scene_hooked::<Graveyard>))
            .add_system(load_decks)
            .add_system_set(self.0.on_exit(reset_decks.after(load_decks)));
    }
}
