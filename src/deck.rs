use bevy::prelude::{Plugin as BevyPlugin, *};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_scene_hook::SceneHook;

use crate::{
    card::{Card, WordOfPower},
    scene::Scene,
    state::GameState,
    war::Value,
};

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

impl PlayerDeck {
    #[rustfmt::skip]
    pub fn new() -> Self {
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

impl OppoDeck {
    #[rustfmt::skip]
    pub fn new() -> Self {
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

fn update_meshes(
    (player_cards, oppo_cards): (usize, usize),
    (player, oppo): (Entity, Entity),
    meshes: &mut Assets<Mesh>,
    meshes_q: &mut Query<(&Handle<Mesh>, &mut Visibility)>,
) -> Option<()> {
    // 18 -> 0.124 ; -- 0 -> -0.9
    use bevy::render::mesh::VertexAttributeValues::Float32x3;
    let (player, _) = meshes_q.get_mut(player).ok()?;
    let player = meshes.get_mut(player.clone())?;
    if let Float32x3(positions) = player.attribute_mut(Mesh::ATTRIBUTE_POSITION)? {
        for pos in positions.iter_mut().filter(|v| v[1] > -0.9) {
            pos[1] = player_cards as f32 / 18.0 - 0.9;
        }
    }
    let (oppo, _) = meshes_q.get_mut(oppo).ok()?;
    let oppo = meshes.get_mut(oppo.clone())?;
    if let Float32x3(positions) = oppo.attribute_mut(Mesh::ATTRIBUTE_POSITION)? {
        for pos in positions.iter_mut().filter(|v| v[1] > -0.9) {
            pos[1] = oppo_cards as f32 / 18.0 - 0.9;
        }
    }
    Some(())
}

// TODO: also change UV
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
