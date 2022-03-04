use bevy::prelude::{Plugin as BevyPlugin, *};

use crate::{
    card::{Card, WordOfPower},
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
    ($($value:ident $word:ident |)*) => (
        Deck::new(vec![ $( Card::new(cards!(@word $word), Value::$value) ,)* ])
    );
    (@word None) => (None);
    (@word $word:ident) => (Some(WordOfPower::$word));
}

pub struct PlayerDeck(Deck);
impl_deck_methods!(PlayerDeck);
impl PlayerDeck {
    #[rustfmt::skip]
    fn new() -> Self {
        Self(cards![
            Zero Egeq  | One Qube   | Two Zihbm |
            Zero Geh   | Four Egeq  | Five None |
            Zero None  | Three None | Two Geh   |
        ])
    }
}

pub struct OppoDeck(Deck);
impl_deck_methods!(OppoDeck);
impl OppoDeck {
    #[rustfmt::skip]
    fn new() -> Self {
        Self(cards![
            Nine None | Eight None | Seven None |
            Six None  | Five None  | Nine None  |
            Five None | Eight None | Four None  |
        ])
    }
}

pub struct Plugin(pub GameState);
impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OppoDeck::new())
            .insert_resource(PlayerDeck::new())
            .add_system_set(
                SystemSet::on_exit(self.0).with_system(|mut cmds: Commands| {
                    cmds.insert_resource(OppoDeck::new());
                    cmds.insert_resource(PlayerDeck::new());
                }),
            );
    }
}
