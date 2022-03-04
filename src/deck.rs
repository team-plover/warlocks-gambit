use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::add_dbg_text;
use crate::{
    card::{Card, WordOfPower},
    card_spawner,
    gltf_hook::GltfHook,
    scene::Scene,
    state::GameState,
    war::Value,
};

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

macro_rules! cards {
    ($($value:tt $word:tt |)*) => (
        Deck::new(vec![ $( Card::new(cards!(@word $word), cards!(@val $value)) ,)* ])
    );
    (@val 0) => (Value::Zero);
    (@val 1) => (Value::One);
    (@val 2) => (Value::Two);
    (@val 3) => (Value::Three);
    (@val 4) => (Value::Four);
    (@val 5) => (Value::Five);
    (@val 6) => (Value::Six);
    (@val 7) => (Value::Seven);
    (@val 8) => (Value::Eight);
    (@val 9) => (Value::Nine);
    (@word _) => (None);
    (@word s) => (Some(WordOfPower::Egeq)); // Seed
    (@word d) => (Some(WordOfPower::Qube)); // Double
    (@word w) => (Some(WordOfPower::Zihbm)); // Swap
    (@word z) => (Some(WordOfPower::Geh)); // 0 -> 12
}

pub struct PlayerDeck(Deck);
impl_deck_methods!(PlayerDeck);
impl PlayerDeck {
    #[rustfmt::skip]
    fn new() -> Self {
        Self(cards![
            7 s | 0 _ | 1 s |
            4 _ | 2 _ | 6 d |
            6 s | 8 w | 0 z |
            2 s | 3 _ | 4 _ |
            3 _ | 9 z | 1 d |
            5 w | 4 _ | 3 _ |
        ])
    }
}

pub struct OppoDeck(Deck);
impl_deck_methods!(OppoDeck);
impl OppoDeck {
    #[rustfmt::skip]
    fn new() -> Self {
        Self(cards![
            8 _ | 7 _ | 6 _ |
            9 z | 5 d | 6 _ |
            8 _ | 9 _ | 6 _ |
            7 _ | 1 w | 5 _ |
            9 d | 0 w | 5 _ |
            1 d | 6 d | 5 _ |
        ])
    }
}

fn resize_decks(
    player_parent: Query<&Children, With<card_spawner::PlayerDeck>>,
    oppo_parent: Query<&Children, With<card_spawner::OppoDeck>>,
    meshes_q: Query<&Handle<Mesh>>,
    player_deck: Res<PlayerDeck>,
    oppo_deck: Res<OppoDeck>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    use bevy::render::mesh::VertexAttributeValues::Float32x3;
    let (player, oppo) = (player_parent.single(), oppo_parent.single());
    if let (Ok(player), Ok(oppo)) = (meshes_q.get(player[0]), meshes_q.get(oppo[0])) {
        add_dbg_text!("changing deck sizes", 0.1);
        if let Some(player) = meshes.get_mut(player.clone()) {
            add_dbg_text!("got the player mesh", 0.1);
            // 18 -> 0.124
            // 0 -> -0.9
            let player_cards = player_deck.remaining() as f32;
            if let Some(Float32x3(positions)) = player.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                for pos in positions.iter_mut().filter(|v| v[1] > -0.9) {
                    pos[1] = player_cards / 18.0 - 0.9;
                }
            }
        }
        if let Some(oppo) = meshes.get_mut(oppo.clone()) {
            add_dbg_text!("got the oppo mesh", 0.1);
            // 18 -> 0.124
            // 0 -> -0.9
            let oppo_cards = oppo_deck.remaining() as f32;
            if let Some(Float32x3(positions)) = oppo.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
                for pos in positions.iter_mut().filter(|v| v[1] > -0.9) {
                    pos[1] = oppo_cards / 18.0 - 0.9;
                }
            }
        }
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OppoDeck::new())
            .insert_resource(PlayerDeck::new())
            .add_system(resize_decks.with_run_criteria(Scene::when_spawned))
            .add_system_set(
                SystemSet::on_exit(self.0).with_system(|mut cmds: Commands| {
                    cmds.insert_resource(OppoDeck::new());
                    cmds.insert_resource(PlayerDeck::new());
                }),
            );
    }
}
